use crate::atv_controller::ATVController;
use adb_client::tcp::ADBTcpDevice;
use std::collections::HashMap;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
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
        let device = ADBTcpDevice::new(address)?;
        let controller = ATVController::new(device);

        let mut devices = self
            .devices
            .lock()
            .map_err(|e| format!("Failed to lock devices: {}", e))?;
        devices.insert(ip.to_string(), Arc::new(Mutex::new(controller)));

        Ok(())
    }

    pub fn remove_device(&self, ip: &str) -> Result<(), Box<dyn Error>> {
        let mut devices = self
            .devices
            .lock()
            .map_err(|e| format!("Failed to lock devices: {}", e))?;
        if devices.remove(ip).is_some() {
            Ok(())
        } else {
            Err(format!("Device {} not found", ip).into())
        }
    }

    pub fn get_controller(&self, ip: &str) -> Result<Arc<Mutex<ATVController>>, Box<dyn Error>> {
        let devices = self
            .devices
            .lock()
            .map_err(|e| format!("Failed to lock devices: {}", e))?;
        devices
            .get(ip)
            .cloned()
            .ok_or_else(|| format!("Device {} not found", ip).into())
    }

    pub fn list_devices(&self) -> Vec<String> {
        let devices = self.devices.lock().unwrap_or_else(|e| e.into_inner());
        devices.keys().cloned().collect()
    }
}

impl Default for GlobalDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}
