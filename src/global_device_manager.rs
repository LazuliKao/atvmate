use crate::adb_service::ADBService;
use crate::atv_controller::ATVController;
use crate::device::{ADBDevice, DiscoveredDevice};
use crate::device_manager::DeviceManager;
#[cfg(feature = "usb")]
use adb_client::usb::find_all_connected_adb_devices;
use std::collections::HashMap;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};

pub struct GlobalDeviceManager {
    devices: Arc<Mutex<HashMap<String, ManagedDevice>>>,
    default_server_addr: Option<SocketAddrV4>,
}

struct ManagedDevice {
    controller: Arc<Mutex<ATVController>>,
    factory: DeviceFactory,
}

#[derive(Clone)]
enum DeviceFactory {
    Server {
        serial: String,
        server_addr: Option<SocketAddrV4>,
    },
    Tcp {
        address: SocketAddr,
    },
    #[cfg(feature = "usb")]
    Usb {
        vendor_id: Option<u16>,
        product_id: Option<u16>,
    },
}

impl DeviceFactory {
    fn create_controller(&self) -> Result<ATVController, Box<dyn Error>> {
        let device = match self {
            Self::Server {
                serial,
                server_addr,
            } => ADBDevice::server(serial.clone(), *server_addr),
            Self::Tcp { address } => ADBDevice::tcp(*address)?,
            #[cfg(feature = "usb")]
            Self::Usb {
                vendor_id,
                product_id,
            } => ADBDevice::usb(*vendor_id, *product_id)?,
        };
        Ok(ATVController::new(device))
    }
}

impl GlobalDeviceManager {
    pub fn new() -> Self {
        Self::with_server_addr(None)
    }

    pub fn with_server_addr(default_server_addr: Option<SocketAddrV4>) -> Self {
        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
            default_server_addr,
        }
    }

    pub fn add_device(&self, ip: &str, port: u16) -> Result<(), Box<dyn Error>> {
        let ip_addr: Ipv4Addr = ip.parse()?;
        let address = SocketAddr::new(IpAddr::V4(ip_addr), port);
        let device = ADBDevice::tcp(address)?;
        self.insert_device(ip.to_string(), device, DeviceFactory::Tcp { address })
    }

    pub fn add_usb_device(
        &self,
        vid: Option<u16>,
        pid: Option<u16>,
    ) -> Result<String, Box<dyn Error>> {
        #[cfg(not(feature = "usb"))]
        {
            let _ = (vid, pid);
            Err("USB support is disabled at compile time".into())
        }

        #[cfg(feature = "usb")]
        {
            let device_id = match (vid, pid) {
                (Some(vendor_id), Some(product_id)) => {
                    let device = ADBDevice::usb(Some(vendor_id), Some(product_id))?;
                    let device_id = format!("usb:{vendor_id:04x}:{product_id:04x}");
                    self.insert_device(
                        device_id.clone(),
                        device,
                        DeviceFactory::Usb {
                            vendor_id: Some(vendor_id),
                            product_id: Some(product_id),
                        },
                    )?;
                    device_id
                }
                (None, None) => {
                    let devices = find_all_connected_adb_devices()?;
                    let info = devices
                        .first()
                        .ok_or_else(|| "No USB ADB devices found".to_string())?;
                    let device = ADBDevice::usb(None, None)?;
                    let device_id = format!("usb:{:04x}:{:04x}", info.vendor_id, info.product_id);
                    self.insert_device(
                        device_id.clone(),
                        device,
                        DeviceFactory::Usb {
                            vendor_id: Some(info.vendor_id),
                            product_id: Some(info.product_id),
                        },
                    )?;
                    device_id
                }
                _ => {
                    return Err(
                        "Both vid and pid must be provided together for USB connection".into(),
                    );
                }
            };

            Ok(device_id)
        }
    }

    pub fn add_server_device(
        &self,
        server_addr: Option<SocketAddrV4>,
        serial: &str,
    ) -> Result<String, Box<dyn Error>> {
        let device_id = format!("server:{serial}");
        let server_addr = server_addr.or(self.default_server_addr);
        let device = ADBDevice::server(serial.to_string(), server_addr);
        self.insert_device(
            device_id.clone(),
            device,
            DeviceFactory::Server {
                serial: serial.to_string(),
                server_addr,
            },
        )?;
        Ok(device_id)
    }

    pub fn discover_devices(&self) -> Vec<DiscoveredDevice> {
        let mut discovered_devices = Vec::new();
        let mut adb_service = self
            .default_server_addr
            .map(ADBService::from_addr)
            .unwrap_or_default();

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

        #[cfg(feature = "usb")]
        {
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
            .map(|device| device.controller.clone())
            .ok_or_else(|| format!("Device {} not found", device_id).into())
    }

    /// Creates an independent ADB connection for long-running streams such as logcat.
    pub fn get_stream_controller(&self, device_id: &str) -> Result<ATVController, Box<dyn Error>> {
        let factory = self
            .devices
            .lock()
            .map_err(|e| format!("Failed to lock devices: {e}"))?
            .get(device_id)
            .map(|device| device.factory.clone())
            .ok_or_else(|| format!("Device {} not found", device_id))?;
        factory.create_controller()
    }

    pub fn list_devices(&self) -> Vec<String> {
        let devices = self.devices.lock().unwrap_or_else(|e| e.into_inner());
        devices.keys().cloned().collect()
    }

    pub fn get_all_devices(&self) -> Vec<(String, String)> {
        let devices = self.devices.lock().unwrap_or_else(|e| e.into_inner());
        let mut all_devices = devices
            .keys()
            .cloned()
            .map(|device_id| {
                let device_type = Self::device_type_from_id(&device_id).to_string();
                (device_id, device_type)
            })
            .collect::<Vec<_>>();
        all_devices.sort_by(|left, right| left.0.cmp(&right.0));
        all_devices
    }

    fn insert_device(
        &self,
        id: String,
        device: ADBDevice,
        factory: DeviceFactory,
    ) -> Result<(), Box<dyn Error>> {
        let controller = ATVController::new(device);
        let mut devices = self
            .devices
            .lock()
            .map_err(|e| format!("Failed to lock devices: {}", e))?;
        devices.insert(
            id,
            ManagedDevice {
                controller: Arc::new(Mutex::new(controller)),
                factory,
            },
        );
        Ok(())
    }

    fn device_type_from_id(device_id: &str) -> &'static str {
        if device_id.starts_with("server:") {
            "Server"
        } else if device_id.starts_with("usb:") {
            "Usb"
        } else {
            "Tcp"
        }
    }
}

impl Default for GlobalDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::GlobalDeviceManager;

    #[test]
    fn get_all_devices_returns_sorted_registered_devices() {
        let manager = GlobalDeviceManager::new();

        manager.add_server_device(None, "living-room").unwrap();
        manager.add_server_device(None, "bedroom").unwrap();

        let devices = manager.get_all_devices();

        assert_eq!(
            devices,
            vec![
                ("server:bedroom".to_string(), "Server".to_string()),
                ("server:living-room".to_string(), "Server".to_string()),
            ]
        );
    }

    #[test]
    fn device_type_from_id_uses_id_prefixes() {
        assert_eq!(
            GlobalDeviceManager::device_type_from_id("server:living-room"),
            "Server"
        );
        assert_eq!(
            GlobalDeviceManager::device_type_from_id("usb:18d1:4ee7"),
            "Usb"
        );
        assert_eq!(
            GlobalDeviceManager::device_type_from_id("192.168.1.10"),
            "Tcp"
        );
    }

    #[test]
    fn stream_controller_uses_a_separate_server_connection() {
        let manager = GlobalDeviceManager::new();
        manager.add_server_device(None, "living-room").unwrap();

        let stream_controller = manager.get_stream_controller("server:living-room");

        assert!(stream_controller.is_ok());
    }
}
