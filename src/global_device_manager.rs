use crate::adb_service::ADBService;
use crate::atv_controller::ATVController;
use crate::device::{ADBDevice, DiscoveredDevice};
use crate::device_manager::DeviceManager;
use adb_client::usb::find_all_connected_adb_devices;
use std::collections::HashMap;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};

pub struct GlobalDeviceManager {
    devices: Arc<Mutex<HashMap<String, Arc<Mutex<ATVController>>>>>,
}

impl GlobalDeviceManager {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_device(&self, ip: &str, port: u16) -> Result<(), Box<dyn Error>> {
        let ip_addr: Ipv4Addr = ip.parse()?;
        let address = SocketAddr::new(IpAddr::V4(ip_addr), port);
        let device = ADBDevice::tcp(address)?;
        self.insert_device(ip.to_string(), device)
    }

    pub fn add_usb_device(
        &self,
        vid: Option<u16>,
        pid: Option<u16>,
    ) -> Result<String, Box<dyn Error>> {
        let device_id = match (vid, pid) {
            (Some(vendor_id), Some(product_id)) => {
                let device = ADBDevice::usb(Some(vendor_id), Some(product_id))?;
                let device_id = format!("usb:{vendor_id:04x}:{product_id:04x}");
                self.insert_device(device_id.clone(), device)?;
                device_id
            }
            (None, None) => {
                let devices = find_all_connected_adb_devices()?;
                let info = devices
                    .first()
                    .ok_or_else(|| "No USB ADB devices found".to_string())?;
                let device = ADBDevice::usb(None, None)?;
                let device_id = format!("usb:{:04x}:{:04x}", info.vendor_id, info.product_id);
                self.insert_device(device_id.clone(), device)?;
                device_id
            }
            _ => {
                return Err("Both vid and pid must be provided together for USB connection".into());
            }
        };

        Ok(device_id)
    }

    pub fn add_server_device(
        &self,
        server_addr: Option<SocketAddrV4>,
        serial: &str,
    ) -> Result<String, Box<dyn Error>> {
        let device_id = format!("server:{serial}");
        let device = ADBDevice::server(serial.to_string(), server_addr);
        self.insert_device(device_id.clone(), device)?;
        Ok(device_id)
    }

    pub fn discover_devices(&self) -> Vec<DiscoveredDevice> {
        let mut discovered_devices = Vec::new();
        let mut adb_service = ADBService::default();

        if let Ok(devices) = adb_service.get_devices() {
            for device in devices {
                let connection_type = if DeviceManager::is_usb_device(&device) {
                    "server-usb"
                } else {
                    "server"
                };

                discovered_devices.push(DiscoveredDevice {
                    id: format!("server:{}", device.identifier),
                    identifier: device.identifier,
                    connection_type: connection_type.to_string(),
                    state: device.state.to_string(),
                });
            }
        }

        if let Ok(devices) = find_all_connected_adb_devices() {
            for device in devices {
                let identifier = format!("{:04x}:{:04x}", device.vendor_id, device.product_id);
                discovered_devices.push(DiscoveredDevice {
                    id: format!("usb:{identifier}"),
                    identifier,
                    connection_type: "usb".to_string(),
                    state: "available".to_string(),
                });
            }
        }

        discovered_devices
    }

    pub fn remove_device(&self, device_id: &str) -> Result<(), Box<dyn Error>> {
        let mut devices = self
            .devices
            .lock()
            .map_err(|e| format!("Failed to lock devices: {}", e))?;
        if devices.remove(device_id).is_some() {
            Ok(())
        } else {
            Err(format!("Device {} not found", device_id).into())
        }
    }

    pub fn get_controller(
        &self,
        device_id: &str,
    ) -> Result<Arc<Mutex<ATVController>>, Box<dyn Error>> {
        let devices = self
            .devices
            .lock()
            .map_err(|e| format!("Failed to lock devices: {}", e))?;
        devices
            .get(device_id)
            .cloned()
            .ok_or_else(|| format!("Device {} not found", device_id).into())
    }

    pub fn list_devices(&self) -> Vec<String> {
        let devices = self.devices.lock().unwrap_or_else(|e| e.into_inner());
        devices.keys().cloned().collect()
    }

    fn insert_device(&self, id: String, device: ADBDevice) -> Result<(), Box<dyn Error>> {
        let controller = ATVController::new(device);
        let mut devices = self
            .devices
            .lock()
            .map_err(|e| format!("Failed to lock devices: {}", e))?;
        devices.insert(id, Arc::new(Mutex::new(controller)));
        Ok(())
    }
}

impl Default for GlobalDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}
