use crate::atv_controller::DeviceInfo;
use crate::global_device_manager::GlobalDeviceManager;
use crate::operation_history::{HistoryResponse, OperationHistory, OperationRecord, ReplayRequest};
use futures_util::sink::SinkExt;
use poem::{
    EndpointExt, IntoResponse, Response, Route, get, handler,
    http::StatusCode,
    web::{
        Data, Multipart, Path as PoemPath,
        websocket::{Message, WebSocket},
    },
};
use poem_openapi::{
    ApiResponse as OpenApiResponse, Object, OpenApi, OpenApiService, ResponseContent,
    param::{Path as ApiPath, Query},
    payload::{Binary, Json},
};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::net::SocketAddrV4;
use std::sync::Arc;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Serialize, Object)]
struct DeviceList {
    devices: Vec<String>,
}

#[derive(Serialize, Object)]
struct DeviceDiscovery {
    devices: Vec<DiscoveredDeviceResponse>,
}

#[derive(Serialize, Object)]
struct DiscoveredDeviceResponse {
    id: String,
    identifier: String,
    connection_type: String,
    state: String,
}

#[derive(Deserialize, Object)]
struct AddDeviceRequest {
    ip: String,
    port: u16,
}

#[derive(Deserialize, Object)]
struct AddUsbDeviceRequest {
    vid: Option<u16>,
    pid: Option<u16>,
}

#[derive(Deserialize, Object)]
struct AddServerDeviceRequest {
    serial: String,
    server_addr: Option<String>,
}

#[derive(Deserialize, Object)]
struct TextInputRequest {
    text: String,
}

#[derive(Deserialize, Object)]
struct ComboKeyRequest {
    keys: Vec<String>,
    duration_ms: Option<u32>,
}

#[derive(Deserialize)]
struct ReplayKeyParams {
    #[serde(alias = "key")]
    key_name: String,
}

#[derive(Deserialize)]
struct ReplayComboParams {
    keys: Vec<String>,
    duration_ms: Option<u32>,
}

#[derive(Deserialize, Object)]
struct MouseScrollRequest {
    direction: String,
    amount: u32,
}

#[derive(Deserialize, Object)]
struct GesturePoint {
    x: u32,
    y: u32,
}

#[derive(Deserialize, Object)]
struct GestureRequest {
    points: Vec<GesturePoint>,
    duration_ms: u32,
}

#[derive(Deserialize, Object)]
struct KeymapRequest {
    key: String,
    mapping: std::collections::HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Object)]
struct BatchKeyRequest {
    devices: Vec<String>,
    key: String,
}

#[derive(Serialize, Deserialize, Object)]
struct BatchCommandRequest {
    devices: Vec<String>,
    command: String,
}

#[derive(Serialize, Deserialize, Object)]
struct BatchResult {
    device_id: String,
    success: bool,
    message: String,
}

#[derive(Serialize, Deserialize, Object)]
struct BatchResponse {
    results: Vec<BatchResult>,
}

#[derive(Serialize, Deserialize, Object)]
struct DeviceStatus {
    device_id: String,
    device_type: String,
    connected: bool,
}

#[derive(Serialize, Deserialize, Object)]
struct ApiResponse {
    success: bool,
    message: String,
}

#[derive(Serialize, Object)]
struct DeviceInfoResponse {
    success: bool,
    message: String,
    device_info: Option<DeviceInfo>,
}

#[derive(OpenApiResponse)]
enum ScreenshotResponse {
    #[oai(status = 200)]
    Ok(ScreenshotContent),
    #[oai(status = 500)]
    Error(Json<ApiResponse>),
}

#[derive(ResponseContent)]
enum ScreenshotContent {
    #[oai(content_type = "image/png")]
    Png(Binary<Vec<u8>>),
    #[oai(content_type = "application/octet-stream")]
    Binary(Binary<Vec<u8>>),
}

#[derive(Serialize, Deserialize, Object)]
struct AppListResponse {
    success: bool,
    message: String,
    apps: Vec<String>,
}

#[derive(Serialize)]
struct LogEvent {
    r#type: &'static str,
    message: String,
}

pub struct ApiService {
    device_manager: Arc<GlobalDeviceManager>,
    operation_history: OperationHistory,
}

impl ApiService {
    pub fn new(device_manager: Arc<GlobalDeviceManager>) -> Self {
        Self::with_history(device_manager, OperationHistory::new())
    }

    pub fn with_history(
        device_manager: Arc<GlobalDeviceManager>,
        operation_history: OperationHistory,
    ) -> Self {
        Self {
            device_manager,
            operation_history,
        }
    }

    fn batch_key_result(&self, device_id: String, key: &str) -> BatchResult {
        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.send_key(key) {
                    Ok(_) => BatchResult {
                        device_id,
                        success: true,
                        message: format!("Key '{}' sent successfully", key),
                    },
                    Err(error) => BatchResult {
                        device_id,
                        success: false,
                        message: format!("Failed to send key: {}", error),
                    },
                }
            }
            Err(_) => BatchResult {
                device_id,
                success: false,
                message: "Device not found".to_string(),
            },
        }
    }

    fn batch_command_result(&self, device_id: String, command: &str) -> BatchResult {
        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.execute_shell_command(command) {
                    Ok(output) => BatchResult {
                        device_id,
                        success: true,
                        message: if output.is_empty() {
                            "Command executed successfully".to_string()
                        } else {
                            output
                        },
                    },
                    Err(error) => BatchResult {
                        device_id,
                        success: false,
                        message: format!("Failed to run command: {}", error),
                    },
                }
            }
            Err(_) => BatchResult {
                device_id,
                success: false,
                message: "Device not found".to_string(),
            },
        }
    }

    fn record_operation(&self, device_id: &str, operation: &str, params: String, success: bool) {
        self.operation_history
            .add_record(device_id, OperationRecord::new(operation, params, success));
    }
}

trait ReplayController {
    fn send_named_key(&mut self, key_name: &str) -> Result<(), String>;
    fn send_combo_keys(&mut self, keycodes: &[u32]) -> Result<(), String>;
    fn long_press(&mut self, keycode: u32, duration_ms: u32) -> Result<(), String>;
}

impl ReplayController for crate::atv_controller::ATVController {
    fn send_named_key(&mut self, key_name: &str) -> Result<(), String> {
        match key_name {
            "recent_apps" => self.recent_apps(),
            "sleep" => self.sleep(),
            "wake_up" => self.wake_up(),
            _ => self.send_key(key_name),
        }
        .map_err(|error| error.to_string())
    }

    fn send_combo_keys(&mut self, keycodes: &[u32]) -> Result<(), String> {
        crate::atv_controller::ATVController::send_combo_keys(self, keycodes)
            .map_err(|error| error.to_string())
    }

    fn long_press(&mut self, keycode: u32, duration_ms: u32) -> Result<(), String> {
        crate::atv_controller::ATVController::long_press(self, keycode, duration_ms)
            .map_err(|error| error.to_string())
    }
}

#[cfg(test)]
impl ReplayController for crate::tests::mocks::MockATVController {
    fn send_named_key(&mut self, key_name: &str) -> Result<(), String> {
        match key_name {
            "recent_apps" => self.recent_apps(),
            "sleep" => self.sleep(),
            "wake_up" => self.wake_up(),
            _ => {
                let keycode = key_name_to_code(key_name);
                if keycode == 0 {
                    return Err(format!("Unknown key: {}", key_name));
                }

                self.send_keyevent(keycode)
            }
        }
        .map_err(|error| error.to_string())
    }

    fn send_combo_keys(&mut self, keycodes: &[u32]) -> Result<(), String> {
        if keycodes.is_empty() {
            return Err("At least one keycode is required".to_string());
        }

        let command = format!(
            "input keyevent {}",
            keycodes
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        );
        self.device_mut()
            .shell_command(command, &mut Vec::new())
            .map_err(|error| error.to_string())
    }

    fn long_press(&mut self, keycode: u32, _duration_ms: u32) -> Result<(), String> {
        let command = format!("input keyevent --longpress {}", keycode);
        self.device_mut()
            .shell_command(command, &mut Vec::new())
            .map_err(|error| error.to_string())
    }
}

struct LogcatLineWriter {
    sender: tokio::sync::mpsc::UnboundedSender<String>,
    buffer: Vec<u8>,
}

impl LogcatLineWriter {
    fn new(sender: tokio::sync::mpsc::UnboundedSender<String>) -> Self {
        Self {
            sender,
            buffer: Vec::new(),
        }
    }

    fn send_line(&self, line: String) -> io::Result<()> {
        self.sender
            .send(line)
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "websocket receiver closed"))
    }

    fn flush_complete_lines(&mut self) -> io::Result<()> {
        while let Some(position) = self.buffer.iter().position(|byte| *byte == b'\n') {
            let line = self.buffer.drain(..=position).collect::<Vec<_>>();
            let line = String::from_utf8_lossy(&line).trim_end().to_string();
            if !line.is_empty() {
                self.send_line(line)?;
            }
        }

        Ok(())
    }
}

impl Write for LogcatLineWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        self.flush_complete_lines()?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let line = String::from_utf8_lossy(&self.buffer).trim_end().to_string();
        self.buffer.clear();
        if line.is_empty() {
            return Ok(());
        }

        self.send_line(line)
    }
}

fn log_event_payload(message: impl Into<String>) -> String {
    serde_json::to_string(&LogEvent {
        r#type: "log",
        message: message.into(),
    })
    .expect("log event serialization should succeed")
}

fn api_error_response(status: StatusCode, message: String) -> Response {
    (
        status,
        Json(ApiResponse {
            success: false,
            message,
        }),
    )
        .into_response()
}

#[handler]
async fn device_ws(
    PoemPath(device_id): PoemPath<String>,
    ws: WebSocket,
    Data(device_manager): Data<&Arc<GlobalDeviceManager>>,
) -> Response {
    let device_manager = Arc::clone(device_manager);
    if let Err(error) = device_manager.get_controller(&device_id) {
        return api_error_response(
            StatusCode::NOT_FOUND,
            format!("Failed to get controller: {error}"),
        );
    }

    ws.on_upgrade(move |socket| async move {
        let (join_handle, mut rx) = {
            let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<String>();
            let stream_manager = device_manager.clone();
            let stream_device_id = device_id.clone();

            let handle = tokio::task::spawn_blocking(move || {
                let error_sender = sender.clone();
                let mut writer = LogcatLineWriter::new(sender);
                let stream_result = stream_manager
                    .get_stream_controller(&stream_device_id)
                    .and_then(|mut controller| controller.stream_logcat(&mut writer));

                let _ = writer.flush();

                if let Err(error) = stream_result {
                    let _ = error_sender.send(format!("logcat stream ended: {}", error));
                }
            });

            (handle, receiver)
        };

        let mut socket = socket;
        while let Some(line) = rx.recv().await {
            if socket
                .send(Message::Text(log_event_payload(line)))
                .await
                .is_err()
            {
                break;
            }
        }
        join_handle.abort();
    })
    .into_response()
}

pub fn openapi_service(
    device_manager: Arc<GlobalDeviceManager>,
    operation_history: OperationHistory,
) -> OpenApiService<ApiService, ()> {
    OpenApiService::new(
        ApiService::with_history(device_manager, operation_history),
        "ATV Remote Control",
        "1.0",
    )
    .server("/api")
}

pub fn app(device_manager: Arc<GlobalDeviceManager>, operation_history: OperationHistory) -> Route {
    let api_service = openapi_service(device_manager.clone(), operation_history);
    let ui = api_service.swagger_ui();
    let spec = api_service.spec_endpoint();

    Route::new()
        .at(
            "/api/devices/:device_id/ws",
            get(device_ws).data(device_manager),
        )
        .nest("/api", api_service)
        .nest("/docs", ui)
        .at("/api-docs/openapi.json", spec)
}

fn parse_server_addr(server_addr: &Option<String>) -> Result<Option<SocketAddrV4>, String> {
    server_addr
        .as_deref()
        .map(|addr| {
            addr.parse::<SocketAddrV4>()
                .map_err(|e| format!("Invalid server_addr '{}': {}", addr, e))
        })
        .transpose()
}

fn key_name_to_code(name: &str) -> u32 {
    match name.to_lowercase().as_str() {
        "home" => 3,
        "back" => 4,
        "menu" => 82,
        "power" => 26,
        "volume_up" | "vol_up" => 24,
        "volume_down" | "vol_down" => 25,
        "mute" => 164,
        "enter" => 66,
        "space" => 62,
        "tab" => 61,
        "delete" | "del" => 67,
        "up" => 19,
        "down" => 20,
        "left" => 21,
        "right" => 22,
        "center" | "ok" | "select" => 23,
        "play_pause" | "play" | "pause" | "media_play_pause" => 85,
        "stop" => 86,
        "next" | "fast_forward" => 87,
        "previous" | "rewind" | "media_previous" => 88,
        "media_play" => 126,
        "media_pause" => 127,
        "media_stop" => 86,
        "media_next" => 87,
        "media_prev" => 88,
        _ => 0,
    }
}

fn key_params_json(key_name: &str) -> String {
    serde_json::json!({ "key_name": key_name }).to_string()
}

fn combo_params_json(keys: &[String], duration_ms: Option<u32>) -> String {
    serde_json::json!({ "keys": keys, "duration_ms": duration_ms }).to_string()
}

fn replay_operation<C: ReplayController>(
    controller: &mut C,
    operation: &OperationRecord,
) -> Result<(), String> {
    match operation.operation.as_str() {
        "key" | "send" => {
            let params: ReplayKeyParams = serde_json::from_str(&operation.params)
                .map_err(|error| format!("Invalid key params: {}", error))?;
            controller.send_named_key(&params.key_name)
        }
        "combo" => {
            let params: ReplayComboParams = serde_json::from_str(&operation.params)
                .map_err(|error| format!("Invalid combo params: {}", error))?;
            let mut keycodes = Vec::with_capacity(params.keys.len());

            for key_name in &params.keys {
                let keycode = key_name_to_code(key_name);
                if keycode == 0 {
                    return Err(format!("Unknown key: {}", key_name));
                }
                keycodes.push(keycode);
            }

            if let Some(duration_ms) = params.duration_ms {
                if keycodes.len() == 1 {
                    controller.long_press(keycodes[0], duration_ms)
                } else {
                    controller.send_combo_keys(&keycodes)
                }
            } else {
                controller.send_combo_keys(&keycodes)
            }
        }
        other => Err(format!("Unsupported operation: {}", other)),
    }
}

#[OpenApi]
impl ApiService {
    #[oai(path = "/devices", method = "get")]
    async fn list_devices(&self) -> Json<DeviceList> {
        let devices = self.device_manager.list_devices();
        Json(DeviceList { devices })
    }

    #[oai(path = "/devices/discover", method = "get")]
    async fn discover_devices(&self) -> Json<DeviceDiscovery> {
        let devices = self
            .device_manager
            .discover_devices()
            .into_iter()
            .map(|device| DiscoveredDeviceResponse {
                id: device.id,
                identifier: device.identifier,
                connection_type: device.connection_type,
                state: device.state,
            })
            .collect();

        Json(DeviceDiscovery { devices })
    }

    #[oai(path = "/devices", method = "post")]
    async fn add_device(&self, body: Json<AddDeviceRequest>) -> Json<ApiResponse> {
        match self.device_manager.add_device(&body.ip, body.port) {
            Ok(_) => Json(ApiResponse {
                success: true,
                message: format!("Device {}:{} added successfully", body.ip, body.port),
            }),
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to add device: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/usb", method = "post")]
    async fn add_usb_device(&self, body: Json<AddUsbDeviceRequest>) -> Json<ApiResponse> {
        match self.device_manager.add_usb_device(body.vid, body.pid) {
            Ok(device_id) => Json(ApiResponse {
                success: true,
                message: format!("USB device {} added successfully", device_id),
            }),
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to add USB device: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/server", method = "post")]
    async fn add_server_device(&self, body: Json<AddServerDeviceRequest>) -> Json<ApiResponse> {
        let server_addr = match parse_server_addr(&body.server_addr) {
            Ok(server_addr) => server_addr,
            Err(e) => {
                return Json(ApiResponse {
                    success: false,
                    message: e,
                });
            }
        };

        match self
            .device_manager
            .add_server_device(server_addr, &body.serial)
        {
            Ok(device_id) => Json(ApiResponse {
                success: true,
                message: format!("ADB server device {} added successfully", device_id),
            }),
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to add ADB server device: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/:device_id", method = "delete")]
    async fn remove_device(&self, device_id: ApiPath<String>) -> Json<ApiResponse> {
        let device_id = device_id.0;
        match self.device_manager.remove_device(&device_id) {
            Ok(_) => Json(ApiResponse {
                success: true,
                message: format!("Device {} removed successfully", device_id),
            }),
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to remove device: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/:device_id/info", method = "get")]
    async fn get_device_info(&self, device_id: ApiPath<String>) -> Json<DeviceInfoResponse> {
        let device_id = device_id.0;

        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.get_device_info() {
                    Ok(device_info) => Json(DeviceInfoResponse {
                        success: true,
                        message: format!("Device info retrieved for {}", device_id),
                        device_info: Some(device_info),
                    }),
                    Err(e) => Json(DeviceInfoResponse {
                        success: false,
                        message: format!("Failed to get device info: {}", e),
                        device_info: None,
                    }),
                }
            }
            Err(e) => Json(DeviceInfoResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
                device_info: None,
            }),
        }
    }

    #[oai(path = "/devices/:device_id/key/:key_name", method = "post")]
    async fn send_key(
        &self,
        device_id: ApiPath<String>,
        key_name: ApiPath<String>,
    ) -> Json<ApiResponse> {
        let device_id = device_id.0;
        let key_name = key_name.0;

        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                let result = match key_name.as_str() {
                    "recent_apps" => ctrl.recent_apps(),
                    "sleep" => ctrl.sleep(),
                    "wake_up" => ctrl.wake_up(),
                    _ => ctrl.send_key(&key_name),
                };
                self.record_operation(
                    &device_id,
                    "key",
                    key_params_json(&key_name),
                    result.is_ok(),
                );

                match result {
                    Ok(_) => Json(ApiResponse {
                        success: true,
                        message: format!("Key '{}' sent to device {}", key_name, device_id),
                    }),
                    Err(e) => Json(ApiResponse {
                        success: false,
                        message: format!("Failed to send key: {}", e),
                    }),
                }
            }
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/:device_id/key/combo", method = "post")]
    async fn send_combo_key(
        &self,
        device_id: ApiPath<String>,
        body: Json<ComboKeyRequest>,
    ) -> Json<ApiResponse> {
        let device_id = device_id.0;
        let ComboKeyRequest { keys, duration_ms } = body.0;

        if keys.is_empty() {
            return Json(ApiResponse {
                success: false,
                message: "At least one key is required".to_string(),
            });
        }

        let mut keycodes = Vec::with_capacity(keys.len());
        for key_name in &keys {
            let keycode = key_name_to_code(key_name);
            if keycode == 0 {
                return Json(ApiResponse {
                    success: false,
                    message: format!("Unknown key: {}", key_name),
                });
            }
            keycodes.push(keycode);
        }

        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                let result = if let Some(duration_ms) = duration_ms {
                    if keycodes.len() == 1 {
                        ctrl.long_press(keycodes[0], duration_ms)
                    } else {
                        ctrl.send_combo_keys(&keycodes)
                    }
                } else {
                    ctrl.send_combo_keys(&keycodes)
                };
                self.record_operation(
                    &device_id,
                    "combo",
                    combo_params_json(&keys, duration_ms),
                    result.is_ok(),
                );

                match result {
                    Ok(_) => Json(ApiResponse {
                        success: true,
                        message: format!("Combo keys sent to device {}", device_id),
                    }),
                    Err(e) => Json(ApiResponse {
                        success: false,
                        message: format!("Failed to send combo keys: {}", e),
                    }),
                }
            }
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/:device_id/history", method = "get")]
    async fn get_history(&self, device_id: ApiPath<String>) -> Json<HistoryResponse> {
        Json(HistoryResponse {
            operations: self.operation_history.get_history(&device_id.0),
        })
    }

    #[oai(path = "/devices/:device_id/history/replay", method = "post")]
    async fn replay_history(
        &self,
        device_id: ApiPath<String>,
        body: Json<ReplayRequest>,
    ) -> Json<ApiResponse> {
        let device_id = device_id.0;
        let ReplayRequest { operations } = body.0;

        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                let total = operations.len();
                let mut success_count = 0usize;

                for operation in operations {
                    let result = replay_operation(&mut *ctrl, &operation);
                    if result.is_ok() {
                        success_count += 1;
                    }

                    self.record_operation(
                        &device_id,
                        &operation.operation,
                        operation.params.clone(),
                        result.is_ok(),
                    );
                }

                Json(ApiResponse {
                    success: success_count == total,
                    message: format!("Replayed {success_count} of {total} operations"),
                })
            }
            Err(error) => Json(ApiResponse {
                success: false,
                message: format!("Failed to get controller: {}", error),
            }),
        }
    }

    #[oai(path = "/devices/:device_id/input/text", method = "post")]
    async fn send_text(
        &self,
        device_id: ApiPath<String>,
        body: Json<TextInputRequest>,
    ) -> Json<ApiResponse> {
        let device_id = device_id.0;
        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.input_text(&body.text) {
                    Ok(_) => Json(ApiResponse {
                        success: true,
                        message: format!("Text sent to device {}", device_id),
                    }),
                    Err(e) => Json(ApiResponse {
                        success: false,
                        message: format!("Failed to send text: {}", e),
                    }),
                }
            }
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/:device_id/input/mouse", method = "post")]
    async fn mouse_scroll(
        &self,
        device_id: ApiPath<String>,
        body: Json<MouseScrollRequest>,
    ) -> Json<ApiResponse> {
        let device_id = device_id.0;

        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.mouse_scroll(&body.direction, body.amount) {
                    Ok(_) => Json(ApiResponse {
                        success: true,
                        message: format!("Mouse scroll sent to device {}", device_id),
                    }),
                    Err(e) => Json(ApiResponse {
                        success: false,
                        message: format!("Failed to send mouse scroll: {}", e),
                    }),
                }
            }
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/:device_id/input/gesture", method = "post")]
    async fn execute_gesture(
        &self,
        device_id: ApiPath<String>,
        body: Json<GestureRequest>,
    ) -> Json<ApiResponse> {
        let device_id = device_id.0;
        let points = body
            .points
            .iter()
            .map(|point| (point.x, point.y))
            .collect::<Vec<_>>();

        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.execute_gesture(&points, body.duration_ms) {
                    Ok(_) => Json(ApiResponse {
                        success: true,
                        message: format!("Gesture sent to device {}", device_id),
                    }),
                    Err(e) => Json(ApiResponse {
                        success: false,
                        message: format!("Failed to send gesture: {}", e),
                    }),
                }
            }
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/:device_id/input/keymap", method = "post")]
    async fn send_mapped_key(
        &self,
        device_id: ApiPath<String>,
        body: Json<KeymapRequest>,
    ) -> Json<ApiResponse> {
        let device_id = device_id.0;

        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.send_mapped_key(&body.key, &body.mapping) {
                    Ok(_) => Json(ApiResponse {
                        success: true,
                        message: format!("Mapped key sent to device {}", device_id),
                    }),
                    Err(e) => Json(ApiResponse {
                        success: false,
                        message: format!("Failed to send mapped key: {}", e),
                    }),
                }
            }
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/batch/key", method = "post")]
    async fn send_batch_key(&self, body: Json<BatchKeyRequest>) -> Json<BatchResponse> {
        let BatchKeyRequest { devices, key } = body.0;
        let results = devices
            .into_iter()
            .map(|device_id| self.batch_key_result(device_id, &key))
            .collect();

        Json(BatchResponse { results })
    }

    #[oai(path = "/devices/batch/command", method = "post")]
    async fn send_batch_command(&self, body: Json<BatchCommandRequest>) -> Json<BatchResponse> {
        let BatchCommandRequest { devices, command } = body.0;
        let results = devices
            .into_iter()
            .map(|device_id| self.batch_command_result(device_id, &command))
            .collect();

        Json(BatchResponse { results })
    }

    #[oai(path = "/devices/status", method = "get")]
    async fn get_device_status(&self) -> Json<Vec<DeviceStatus>> {
        let statuses = self
            .device_manager
            .get_all_devices()
            .into_iter()
            .map(|(device_id, device_type)| {
                let connected = self
                    .device_manager
                    .get_controller(&device_id)
                    .and_then(|controller| {
                        controller
                            .lock()
                            .map_err(|error| format!("Failed to lock controller: {error}").into())
                            .and_then(|mut controller| {
                                controller.execute_shell_command("echo connected")
                            })
                    })
                    .is_ok();
                DeviceStatus {
                    device_id,
                    device_type,
                    connected,
                }
            })
            .collect();

        Json(statuses)
    }

    #[oai(path = "/devices/:device_id/apps", method = "get")]
    async fn list_apps(&self, device_id: ApiPath<String>) -> Json<AppListResponse> {
        let device_id = device_id.0;
        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.list_packages() {
                    Ok(apps) => Json(AppListResponse {
                        success: true,
                        message: format!("Apps listed for device {}", device_id),
                        apps,
                    }),
                    Err(e) => Json(AppListResponse {
                        success: false,
                        message: format!("Failed to list apps: {}", e),
                        apps: Vec::new(),
                    }),
                }
            }
            Err(e) => Json(AppListResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
                apps: Vec::new(),
            }),
        }
    }

    #[oai(path = "/devices/:device_id/apps/install", method = "post")]
    async fn install_app(
        &self,
        device_id: ApiPath<String>,
        mut multipart: Multipart,
    ) -> Json<ApiResponse> {
        let device_id = device_id.0;
        let mut apk_data = None;

        loop {
            let field = match multipart.next_field().await {
                Ok(Some(field)) => field,
                Ok(None) => break,
                Err(error) => {
                    return Json(ApiResponse {
                        success: false,
                        message: format!("Failed to read multipart form: {error}"),
                    });
                }
            };

            if field.name() == Some("file") {
                match field.bytes().await {
                    Ok(bytes) => apk_data = Some(bytes.to_vec()),
                    Err(error) => {
                        return Json(ApiResponse {
                            success: false,
                            message: format!("Failed to read APK file: {error}"),
                        });
                    }
                }
            }
        }

        let apk_data = match apk_data {
            Some(data) if !data.is_empty() => data,
            _ => {
                return Json(ApiResponse {
                    success: false,
                    message: "Missing APK file".to_string(),
                });
            }
        };
        let apk_path = std::env::temp_dir().join(format!(
            "atvmate-install-{}-{}.apk",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));

        if let Err(error) = fs::write(&apk_path, apk_data) {
            return Json(ApiResponse {
                success: false,
                message: format!("Failed to create temporary APK: {error}"),
            });
        }

        let result = match self.device_manager.get_controller(&device_id) {
            Ok(controller) => controller
                .lock()
                .map_err(|error| format!("Failed to lock controller: {error}"))
                .and_then(|mut controller| {
                    controller
                        .install_apk(&apk_path)
                        .map_err(|error| error.to_string())
                }),
            Err(error) => Err(error.to_string()),
        };
        let _ = fs::remove_file(&apk_path);

        match result {
            Ok(()) => Json(ApiResponse {
                success: true,
                message: format!("APK installed on device {device_id}"),
            }),
            Err(error) => Json(ApiResponse {
                success: false,
                message: format!("Failed to install APK: {error}"),
            }),
        }
    }

    #[oai(path = "/devices/:device_id/apps/:package/launch", method = "post")]
    async fn launch_app(
        &self,
        device_id: ApiPath<String>,
        package: ApiPath<String>,
    ) -> Json<ApiResponse> {
        let device_id = device_id.0;
        let package = package.0;

        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.launch_app(&package) {
                    Ok(_) => Json(ApiResponse {
                        success: true,
                        message: format!("App '{}' launched on device {}", package, device_id),
                    }),
                    Err(e) => Json(ApiResponse {
                        success: false,
                        message: format!("Failed to launch app: {}", e),
                    }),
                }
            }
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/:device_id/apps/:package/uninstall", method = "post")]
    async fn uninstall_app(
        &self,
        device_id: ApiPath<String>,
        package: ApiPath<String>,
    ) -> Json<ApiResponse> {
        let device_id = device_id.0;
        let package = package.0;

        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.uninstall_app(&package) {
                    Ok(_) => Json(ApiResponse {
                        success: true,
                        message: format!("App '{}' uninstalled from device {}", package, device_id),
                    }),
                    Err(e) => Json(ApiResponse {
                        success: false,
                        message: format!("Failed to uninstall app: {}", e),
                    }),
                }
            }
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/:device_id/reboot", method = "post")]
    async fn reboot(&self, device_id: ApiPath<String>) -> Json<ApiResponse> {
        let device_id = device_id.0;

        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.reboot() {
                    Ok(_) => Json(ApiResponse {
                        success: true,
                        message: format!("Device {} reboot initiated", device_id),
                    }),
                    Err(e) => Json(ApiResponse {
                        success: false,
                        message: format!("Failed to reboot device: {}", e),
                    }),
                }
            }
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/:device_id/reboot/recovery", method = "post")]
    async fn reboot_recovery(&self, device_id: ApiPath<String>) -> Json<ApiResponse> {
        let device_id = device_id.0;

        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.reboot_recovery() {
                    Ok(_) => Json(ApiResponse {
                        success: true,
                        message: format!("Device {} rebooted into recovery", device_id),
                    }),
                    Err(e) => Json(ApiResponse {
                        success: false,
                        message: format!("Failed to reboot device into recovery: {}", e),
                    }),
                }
            }
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/:device_id/reboot/bootloader", method = "post")]
    async fn reboot_bootloader(&self, device_id: ApiPath<String>) -> Json<ApiResponse> {
        let device_id = device_id.0;

        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.reboot_bootloader() {
                    Ok(_) => Json(ApiResponse {
                        success: true,
                        message: format!("Device {} rebooted into bootloader", device_id),
                    }),
                    Err(e) => Json(ApiResponse {
                        success: false,
                        message: format!("Failed to reboot device into bootloader: {}", e),
                    }),
                }
            }
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/:device_id/apps/:package/clear", method = "post")]
    async fn clear_app_data(
        &self,
        device_id: ApiPath<String>,
        package: ApiPath<String>,
    ) -> Json<ApiResponse> {
        let device_id = device_id.0;
        let package = package.0;

        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.clear_app_data(&package) {
                    Ok(_) => Json(ApiResponse {
                        success: true,
                        message: format!("App '{}' data cleared on device {}", package, device_id),
                    }),
                    Err(e) => Json(ApiResponse {
                        success: false,
                        message: format!("Failed to clear app data: {}", e),
                    }),
                }
            }
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/:device_id/push", method = "post")]
    async fn push_file(
        &self,
        device_id: ApiPath<String>,
        mut multipart: Multipart,
    ) -> Json<ApiResponse> {
        let device_id = device_id.0;
        let mut remote_path: Option<String> = None;
        let mut file_data: Option<Vec<u8>> = None;

        loop {
            let field = match multipart.next_field().await {
                Ok(Some(field)) => field,
                Ok(None) => break,
                Err(e) => {
                    return Json(ApiResponse {
                        success: false,
                        message: format!("Failed to read multipart form: {}", e),
                    });
                }
            };

            let field_name = field.name().map(str::to_owned);
            match field_name.as_deref() {
                Some("remote_path") => match field.text().await {
                    Ok(text) => remote_path = Some(text),
                    Err(e) => {
                        return Json(ApiResponse {
                            success: false,
                            message: format!("Failed to read remote_path field: {}", e),
                        });
                    }
                },
                Some("file") => match field.bytes().await {
                    Ok(bytes) => file_data = Some(bytes.to_vec()),
                    Err(e) => {
                        return Json(ApiResponse {
                            success: false,
                            message: format!("Failed to read file field: {}", e),
                        });
                    }
                },
                _ => {}
            }
        }

        let remote_path = match remote_path {
            Some(remote_path) if !remote_path.is_empty() => remote_path,
            _ => {
                return Json(ApiResponse {
                    success: false,
                    message: "Missing remote_path field".to_string(),
                });
            }
        };

        let file_data = match file_data {
            Some(file_data) => file_data,
            None => {
                return Json(ApiResponse {
                    success: false,
                    message: "Missing file field".to_string(),
                });
            }
        };

        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.push_file(&file_data, &remote_path) {
                    Ok(_) => Json(ApiResponse {
                        success: true,
                        message: format!("File pushed to device {} at {}", device_id, remote_path),
                    }),
                    Err(e) => Json(ApiResponse {
                        success: false,
                        message: format!("Failed to push file: {}", e),
                    }),
                }
            }
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/:device_id/pull", method = "get")]
    async fn pull_file(
        &self,
        device_id: ApiPath<String>,
        path: Query<String>,
    ) -> ScreenshotResponse {
        let device_id = device_id.0;
        let remote_path = path.0;

        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.pull_file(&remote_path) {
                    Ok(data) => ScreenshotResponse::Ok(ScreenshotContent::Binary(Binary(data))),
                    Err(e) => ScreenshotResponse::Error(Json(ApiResponse {
                        success: false,
                        message: format!("Failed to pull file: {}", e),
                    })),
                }
            }
            Err(e) => ScreenshotResponse::Error(Json(ApiResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
            })),
        }
    }

    #[oai(path = "/devices/:device_id/screenshot", method = "get")]
    async fn take_screenshot(&self, device_id: ApiPath<String>) -> ScreenshotResponse {
        let device_id = device_id.0;

        match self.device_manager.get_controller(&device_id) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.take_screenshot() {
                    Ok(data) => ScreenshotResponse::Ok(ScreenshotContent::Png(Binary(data))),
                    Err(e) => ScreenshotResponse::Error(Json(ApiResponse {
                        success: false,
                        message: format!("Failed to take screenshot: {}", e),
                    })),
                }
            }
            Err(e) => ScreenshotResponse::Error(Json(ApiResponse {
                success: false,
                message: format!("Failed to get controller: {}", e),
            })),
        }
    }
}

#[cfg(test)]
mod combo_tests {
    use super::{ApiService, ComboKeyRequest, key_name_to_code};
    use crate::tests::utils::{api_service_with_one_device, empty_device_manager};
    use poem_openapi::{param::Path as ApiPath, payload::Json};

    #[test]
    fn key_name_to_code_supports_expected_aliases() {
        assert_eq!(key_name_to_code("HOME"), 3);
        assert_eq!(key_name_to_code("vol_up"), 24);
        assert_eq!(key_name_to_code("Select"), 23);
        assert_eq!(key_name_to_code("rewind"), 88);
        assert_eq!(key_name_to_code("media_play_pause"), 85);
        assert_eq!(key_name_to_code("media_previous"), 88);
        assert_eq!(key_name_to_code("unknown"), 0);
    }

    #[tokio::test]
    async fn combo_key_endpoint_rejects_empty_keys() {
        let (_, device_id) = api_service_with_one_device();
        let service = ApiService::new(empty_device_manager());

        let Json(body) = service
            .send_combo_key(
                ApiPath(device_id),
                Json(ComboKeyRequest {
                    keys: vec![],
                    duration_ms: None,
                }),
            )
            .await;

        assert!(!body.success);
        assert_eq!(body.message, "At least one key is required");
    }

    #[tokio::test]
    async fn combo_key_endpoint_rejects_unknown_keys() {
        let (_, device_id) = api_service_with_one_device();
        let service = ApiService::new(empty_device_manager());

        let Json(body) = service
            .send_combo_key(
                ApiPath(device_id),
                Json(ComboKeyRequest {
                    keys: vec!["home".to_string(), "bogus".to_string()],
                    duration_ms: None,
                }),
            )
            .await;

        assert!(!body.success);
        assert_eq!(body.message, "Unknown key: bogus");
    }
}

#[cfg(test)]
mod history_tests {
    use super::{
        ApiService, HistoryResponse, OperationRecord, ReplayRequest, combo_params_json,
        key_params_json, replay_operation,
    };
    use crate::operation_history::OperationHistory;
    use crate::tests::{
        mocks::{MockADBDevice, MockATVController},
        utils::{empty_device_manager, test_device_manager},
    };
    use poem_openapi::{param::Path as ApiPath, payload::Json};

    #[tokio::test]
    async fn send_key_records_failed_operation_in_history() {
        let history = OperationHistory::new();
        let service = ApiService::with_history(test_device_manager(vec!["living-room"]), history);

        let Json(body) = service
            .send_key(
                ApiPath("server:living-room".to_string()),
                ApiPath("bogus".to_string()),
            )
            .await;

        assert!(!body.success);

        let Json(HistoryResponse { operations }) = service
            .get_history(ApiPath("server:living-room".to_string()))
            .await;
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].operation, "key");
        assert_eq!(operations[0].params, key_params_json("bogus"));
        assert!(!operations[0].success);
    }

    #[tokio::test]
    async fn combo_key_records_controller_failure_in_history() {
        let history = OperationHistory::new();
        let service = ApiService::with_history(test_device_manager(vec!["den"]), history);

        let Json(body) = service
            .send_combo_key(
                ApiPath("server:den".to_string()),
                Json(super::ComboKeyRequest {
                    keys: vec!["home".to_string(), "back".to_string()],
                    duration_ms: None,
                }),
            )
            .await;

        assert!(!body.success);

        let Json(HistoryResponse { operations }) =
            service.get_history(ApiPath("server:den".to_string())).await;
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].operation, "combo");
        assert_eq!(
            operations[0].params,
            combo_params_json(&["home".to_string(), "back".to_string()], None)
        );
        assert!(!operations[0].success);
    }

    #[test]
    fn replay_operation_reexecutes_key_and_combo_commands() {
        let mut controller = MockATVController::new(MockADBDevice::new());

        replay_operation(
            &mut controller,
            &OperationRecord::new("key", key_params_json("home"), true),
        )
        .unwrap();
        replay_operation(
            &mut controller,
            &OperationRecord::new(
                "combo",
                combo_params_json(&["home".to_string(), "back".to_string()], None),
                true,
            ),
        )
        .unwrap();
        replay_operation(
            &mut controller,
            &OperationRecord::new(
                "combo",
                combo_params_json(&["center".to_string()], Some(750)),
                true,
            ),
        )
        .unwrap();

        assert_eq!(
            controller.device().command_log(),
            &[
                "input keyevent 3".to_string(),
                "input keyevent 3 4".to_string(),
                "input keyevent --longpress 23".to_string(),
            ]
        );
    }

    #[test]
    fn replay_operation_rejects_invalid_payloads() {
        let mut controller = MockATVController::new(MockADBDevice::new());
        let error = replay_operation(
            &mut controller,
            &OperationRecord::new("combo", r#"{"keys":["bogus"]}"#, true),
        )
        .unwrap_err();

        assert_eq!(error, "Unknown key: bogus");
    }

    #[tokio::test]
    async fn history_endpoint_returns_seeded_operations() {
        let history = OperationHistory::new();
        history.add_record(
            "server:office",
            OperationRecord::new("key", key_params_json("home"), true),
        );
        let service = ApiService::with_history(empty_device_manager(), history);

        let Json(HistoryResponse { operations }) = service
            .get_history(ApiPath("server:office".to_string()))
            .await;

        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].operation, "key");
    }

    #[tokio::test]
    async fn replay_endpoint_returns_missing_device_error() {
        let service = ApiService::with_history(empty_device_manager(), OperationHistory::new());

        let Json(response) = service
            .replay_history(
                ApiPath("missing".to_string()),
                Json(ReplayRequest {
                    operations: vec![OperationRecord::new("key", key_params_json("home"), true)],
                }),
            )
            .await;

        assert!(!response.success);
        assert_eq!(
            response.message,
            "Failed to get controller: Device missing not found"
        );
    }
}

#[cfg(test)]
mod input_endpoint_tests {
    use super::{ApiService, GesturePoint, GestureRequest, KeymapRequest, MouseScrollRequest};
    use crate::tests::utils::empty_device_manager;
    use poem_openapi::{param::Path as ApiPath, payload::Json};
    use std::collections::HashMap;

    #[tokio::test]
    async fn mouse_scroll_returns_error_for_missing_device() {
        let service = ApiService::new(empty_device_manager());

        let Json(body) = service
            .mouse_scroll(
                ApiPath("missing".to_string()),
                Json(MouseScrollRequest {
                    direction: "up".to_string(),
                    amount: 1,
                }),
            )
            .await;

        assert!(!body.success);
        assert_eq!(
            body.message,
            "Failed to get controller: Device missing not found"
        );
    }

    #[tokio::test]
    async fn execute_gesture_returns_error_for_missing_device() {
        let service = ApiService::new(empty_device_manager());

        let Json(body) = service
            .execute_gesture(
                ApiPath("missing".to_string()),
                Json(GestureRequest {
                    points: vec![GesturePoint { x: 10, y: 20 }, GesturePoint { x: 30, y: 40 }],
                    duration_ms: 300,
                }),
            )
            .await;

        assert!(!body.success);
        assert_eq!(
            body.message,
            "Failed to get controller: Device missing not found"
        );
    }

    #[tokio::test]
    async fn send_mapped_key_returns_error_for_missing_device() {
        let service = ApiService::new(empty_device_manager());

        let Json(body) = service
            .send_mapped_key(
                ApiPath("missing".to_string()),
                Json(KeymapRequest {
                    key: "play".to_string(),
                    mapping: HashMap::from([("play".to_string(), 85)]),
                }),
            )
            .await;

        assert!(!body.success);
        assert_eq!(
            body.message,
            "Failed to get controller: Device missing not found"
        );
    }
}

#[cfg(test)]
mod batch_endpoint_tests {
    use super::{ApiService, BatchCommandRequest, BatchKeyRequest, BatchResponse, DeviceStatus};
    use crate::tests::utils::{empty_device_manager, test_device_manager};
    use poem_openapi::payload::Json;

    #[tokio::test]
    async fn batch_key_returns_not_found_for_missing_devices() {
        let service = ApiService::new(empty_device_manager());

        let Json(BatchResponse { results }) = service
            .send_batch_key(Json(BatchKeyRequest {
                devices: vec!["missing-a".to_string(), "missing-b".to_string()],
                key: "home".to_string(),
            }))
            .await;

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].device_id, "missing-a");
        assert!(!results[0].success);
        assert_eq!(results[0].message, "Device not found");
        assert_eq!(results[1].device_id, "missing-b");
        assert!(!results[1].success);
        assert_eq!(results[1].message, "Device not found");
    }

    #[tokio::test]
    async fn batch_command_returns_not_found_for_missing_devices() {
        let service = ApiService::new(empty_device_manager());

        let Json(BatchResponse { results }) = service
            .send_batch_command(Json(BatchCommandRequest {
                devices: vec!["missing".to_string()],
                command: "getprop ro.product.model".to_string(),
            }))
            .await;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].device_id, "missing");
        assert!(!results[0].success);
        assert_eq!(results[0].message, "Device not found");
    }

    #[tokio::test]
    async fn device_status_lists_registered_devices_with_types() {
        let service = ApiService::new(test_device_manager(vec!["alpha", "beta"]));

        let Json(statuses) = service.get_device_status().await;

        assert_eq!(
            statuses
                .into_iter()
                .map(|status: DeviceStatus| (
                    status.device_id,
                    status.device_type,
                    status.connected
                ))
                .collect::<Vec<_>>(),
            vec![
                ("server:alpha".to_string(), "Server".to_string(), false),
                ("server:beta".to_string(), "Server".to_string(), false),
            ]
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ApiService, LogcatLineWriter, ScreenshotContent, ScreenshotResponse, api_error_response,
        log_event_payload,
    };
    use crate::operation_history::OperationHistory;
    use crate::tests::{mocks::MockADBDevice, utils::empty_device_manager};
    use poem::{
        IntoResponse,
        http::{StatusCode, header::CONTENT_TYPE},
        test::TestClient,
    };
    use poem_openapi::{
        param::{Path as ApiPath, Query},
        payload::{Binary, Json},
    };
    use std::io::Write;
    use tokio::sync::mpsc::error::TryRecvError;

    #[tokio::test]
    async fn take_screenshot_returns_json_error_for_missing_device() {
        let service = ApiService::new(empty_device_manager());

        match service
            .take_screenshot(ApiPath("missing-device".to_string()))
            .await
        {
            ScreenshotResponse::Error(Json(body)) => {
                assert!(!body.success);
                assert_eq!(
                    body.message,
                    "Failed to get controller: Device missing-device not found"
                );
            }
            ScreenshotResponse::Ok(_) => panic!("expected error response"),
        }
    }

    #[tokio::test]
    async fn screenshot_response_success_sets_png_content_type() {
        let response = ScreenshotResponse::Ok(ScreenshotContent::Png(Binary(b"png-data".to_vec())))
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), "image/png");
        assert_eq!(response.into_body().into_vec().await.unwrap(), b"png-data");
    }

    #[tokio::test]
    async fn pull_response_success_sets_octet_stream_content_type() {
        let response =
            ScreenshotResponse::Ok(ScreenshotContent::Binary(Binary(b"file-data".to_vec())))
                .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/octet-stream"
        );
        assert_eq!(response.into_body().into_vec().await.unwrap(), b"file-data");
    }

    #[tokio::test]
    async fn pull_file_returns_json_error_for_missing_device() {
        let service = ApiService::new(empty_device_manager());

        match service
            .pull_file(
                ApiPath("missing-device".to_string()),
                Query("/sdcard/test.txt".to_string()),
            )
            .await
        {
            ScreenshotResponse::Error(Json(body)) => {
                assert!(!body.success);
                assert_eq!(
                    body.message,
                    "Failed to get controller: Device missing-device not found"
                );
            }
            ScreenshotResponse::Ok(_) => panic!("expected error response"),
        }
    }

    #[tokio::test]
    async fn push_file_route_returns_json_error_for_missing_device() {
        let client = TestClient::new(super::app(empty_device_manager(), OperationHistory::new()));

        let response = client
            .post("/api/devices/missing-device/push")
            .multipart(
                poem::test::TestForm::new()
                    .text("remote_path", "/sdcard/test.txt")
                    .bytes("file", b"hello".to_vec()),
            )
            .send()
            .await;

        response.assert_status(StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn take_screenshot_returns_json_error_for_shell_failure() {
        let mut device = MockADBDevice::new();
        device.set_failing("shell failed");
        let mut controller = crate::tests::mocks::MockATVController::new(device);

        let result = controller.take_screenshot();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "shell failed");
    }

    #[test]
    fn log_event_payload_uses_expected_json_shape() {
        assert_eq!(
            log_event_payload("06-01 10:00:00.000 I/Test: hello"),
            r#"{"type":"log","message":"06-01 10:00:00.000 I/Test: hello"}"#
        );
    }

    #[test]
    fn logcat_line_writer_emits_complete_lines() {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        let mut writer = LogcatLineWriter::new(sender);

        writer.write_all(b"first line\nsecond").unwrap();
        assert_eq!(receiver.blocking_recv().as_deref(), Some("first line"));
        assert!(matches!(receiver.try_recv(), Err(TryRecvError::Empty)));

        writer.write_all(b" line\n").unwrap();
        writer.flush().unwrap();
        assert_eq!(receiver.blocking_recv().as_deref(), Some("second line"));
    }

    #[test]
    fn logcat_line_writer_returns_broken_pipe_when_receiver_is_gone() {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        drop(receiver);

        let mut writer = LogcatLineWriter::new(sender);
        let error = writer.write_all(b"hello\n").unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
    }

    #[tokio::test]
    async fn api_error_response_returns_json_with_status() {
        let response = api_error_response(StatusCode::NOT_FOUND, "missing".to_string());

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response.into_body().into_string().await.unwrap();
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"success": false, "message": "missing"})
        );
    }

    #[tokio::test]
    async fn get_device_info_returns_error_for_missing_device() {
        let service = ApiService::new(empty_device_manager());

        let Json(json) = service
            .get_device_info(ApiPath("missing-device".to_string()))
            .await;

        assert!(!json.success);
        assert_eq!(
            json.message,
            "Failed to get controller: Device missing-device not found"
        );
        assert!(json.device_info.is_none());
    }

    #[tokio::test]
    async fn list_apps_returns_error_for_missing_device() {
        let service = ApiService::new(empty_device_manager());

        let Json(json) = service.list_apps(ApiPath("missing".to_string())).await;

        assert!(!json.success);
        assert_eq!(
            json.message,
            "Failed to get controller: Device missing not found"
        );
        assert!(json.apps.is_empty());
    }

    #[tokio::test]
    async fn launch_app_returns_error_for_missing_device() {
        let service = ApiService::new(empty_device_manager());

        let Json(json) = service
            .launch_app(
                ApiPath("missing".to_string()),
                ApiPath("com.example.app".to_string()),
            )
            .await;

        assert!(!json.success);
        assert_eq!(
            json.message,
            "Failed to get controller: Device missing not found"
        );
    }

    #[tokio::test]
    async fn uninstall_app_returns_error_for_missing_device() {
        let service = ApiService::new(empty_device_manager());

        let Json(json) = service
            .uninstall_app(
                ApiPath("missing".to_string()),
                ApiPath("com.example.app".to_string()),
            )
            .await;

        assert!(!json.success);
        assert_eq!(
            json.message,
            "Failed to get controller: Device missing not found"
        );
    }

    #[tokio::test]
    async fn reboot_returns_error_for_missing_device() {
        let service = ApiService::new(empty_device_manager());

        let Json(json) = service.reboot(ApiPath("missing".to_string())).await;

        assert!(!json.success);
        assert_eq!(
            json.message,
            "Failed to get controller: Device missing not found"
        );
    }

    #[tokio::test]
    async fn reboot_recovery_returns_error_for_missing_device() {
        let service = ApiService::new(empty_device_manager());

        let Json(json) = service
            .reboot_recovery(ApiPath("missing".to_string()))
            .await;

        assert!(!json.success);
        assert_eq!(
            json.message,
            "Failed to get controller: Device missing not found"
        );
    }

    #[tokio::test]
    async fn reboot_bootloader_returns_error_for_missing_device() {
        let service = ApiService::new(empty_device_manager());

        let Json(json) = service
            .reboot_bootloader(ApiPath("missing".to_string()))
            .await;

        assert!(!json.success);
        assert_eq!(
            json.message,
            "Failed to get controller: Device missing not found"
        );
    }

    #[tokio::test]
    async fn clear_app_data_returns_error_for_missing_device() {
        let service = ApiService::new(empty_device_manager());

        let Json(json) = service
            .clear_app_data(
                ApiPath("missing".to_string()),
                ApiPath("com.example.app".to_string()),
            )
            .await;

        assert!(!json.success);
        assert_eq!(
            json.message,
            "Failed to get controller: Device missing not found"
        );
    }
}
