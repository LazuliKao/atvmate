use crate::global_device_manager::GlobalDeviceManager;
use poem_openapi::{Object, OpenApi, param::Path as ApiPath, payload::Json};
use serde::{Deserialize, Serialize};
use std::net::SocketAddrV4;
use std::sync::Arc;

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

#[derive(Serialize, Object)]
struct ApiResponse {
    success: bool,
    message: String,
}

pub struct ApiService {
    device_manager: Arc<GlobalDeviceManager>,
}

impl ApiService {
    pub fn new(device_manager: Arc<GlobalDeviceManager>) -> Self {
        Self { device_manager }
    }
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
                    "home" => ctrl.home(),
                    "back" => ctrl.back(),
                    "volume_up" => ctrl.volume_up(),
                    "volume_down" => ctrl.volume_down(),
                    "volume_mute" => ctrl.volume_mute(),
                    "menu" => ctrl.menu(),
                    "recent_apps" => ctrl.recent_apps(),
                    "dpad_up" => ctrl.dpad_up(),
                    "dpad_down" => ctrl.dpad_down(),
                    "dpad_left" => ctrl.dpad_left(),
                    "dpad_right" => ctrl.dpad_right(),
                    "dpad_center" => ctrl.dpad_center(),
                    "play_pause" => ctrl.play_pause(),
                    "stop" => ctrl.stop(),
                    "next" => ctrl.next(),
                    "previous" => ctrl.previous(),
                    "enter" => ctrl.enter(),
                    "space" => ctrl.space(),
                    "backspace" => ctrl.backspace(),
                    "tab" => ctrl.tab(),
                    "power" => ctrl.power(),
                    "sleep" => ctrl.sleep(),
                    "wake_up" => ctrl.wake_up(),
                    _ => Err(format!("Unknown key: {}", key_name).into()),
                };

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
}
