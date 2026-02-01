use poem_openapi::{payload::Json, OpenApi, param::Path as ApiPath, Object};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::global_device_manager::GlobalDeviceManager;

#[derive(Serialize, Object)]
struct DeviceList {
    devices: Vec<String>,
}

#[derive(Deserialize, Object)]
struct AddDeviceRequest {
    ip: String,
    port: u16,
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

#[OpenApi]
impl ApiService {
    #[oai(path = "/devices", method = "get")]
    async fn list_devices(&self) -> Json<DeviceList> {
        let devices = self.device_manager.list_devices();
        Json(DeviceList { devices })
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

    #[oai(path = "/devices/:ip", method = "delete")]
    async fn remove_device(&self, ip: ApiPath<String>) -> Json<ApiResponse> {
        let ip = ip.0;
        match self.device_manager.remove_device(&ip) {
            Ok(_) => Json(ApiResponse {
                success: true,
                message: format!("Device {} removed successfully", ip),
            }),
            Err(e) => Json(ApiResponse {
                success: false,
                message: format!("Failed to remove device: {}", e),
            }),
        }
    }

    #[oai(path = "/devices/:ip/key/:key_name", method = "post")]
    async fn send_key(
        &self,
        ip: ApiPath<String>,
        key_name: ApiPath<String>,
    ) -> Json<ApiResponse> {
        let ip = ip.0;
        let key_name = key_name.0;
        
        match self.device_manager.get_controller(&ip) {
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
                        message: format!("Key '{}' sent to device {}", key_name, ip),
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

    #[oai(path = "/devices/:ip/input/text", method = "post")]
    async fn send_text(
        &self,
        ip: ApiPath<String>,
        body: Json<TextInputRequest>,
    ) -> Json<ApiResponse> {
        let ip = ip.0;
        match self.device_manager.get_controller(&ip) {
            Ok(controller) => {
                let mut ctrl = controller.lock().unwrap();
                match ctrl.input_text(&body.text) {
                    Ok(_) => Json(ApiResponse {
                        success: true,
                        message: format!("Text sent to device {}", ip),
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