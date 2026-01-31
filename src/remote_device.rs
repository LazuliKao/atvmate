use crate::atv_controller::ATVController;
use adb_client::{tcp::ADBTcpDevice, ADBDeviceExt};
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub struct RemoteDevice {
    device: ADBTcpDevice,
    address: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub serial: String,
    pub model: String,
    pub manufacturer: String,
    pub android_version: String,
    pub build_id: String,
    pub product_name: String,
}

impl RemoteDevice {
    pub fn new(ip: Ipv4Addr, port: u16) -> Result<Self, Box<dyn Error>> {
        let address = SocketAddr::new(IpAddr::V4(ip), port);
        let device = ADBTcpDevice::new(address)?;

        Ok(Self { device, address })
    }

    pub fn get_device_info(&mut self) -> Result<DeviceInfo, Box<dyn Error>> {
        let props = self.get_all_properties()?;

        let serial = Self::extract_property(&props, "ro.serialno");
        let model = Self::extract_property(&props, "ro.product.model");
        let manufacturer = Self::extract_property(&props, "ro.product.manufacturer");
        let android_version = Self::extract_property(&props, "ro.build.version.release");
        let build_id = Self::extract_property(&props, "ro.build.id");
        let product_name = Self::extract_property(&props, "ro.product.name");

        Ok(DeviceInfo {
            serial,
            model,
            manufacturer,
            android_version,
            build_id,
            product_name,
        })
    }

    pub fn get_all_properties(&mut self) -> Result<String, Box<dyn Error>> {
        let mut output = Vec::new();
        self.device.shell_command(&"getprop", &mut output)?;
        Ok(String::from_utf8_lossy(&output).trim().to_string())
    }

    pub fn execute_shell_command(&mut self, command: &str) -> Result<String, Box<dyn Error>> {
        let mut output = Vec::new();
        self.device.shell_command(&command, &mut output)?;
        Ok(String::from_utf8_lossy(&output).trim().to_string())
    }

    pub fn enable_tcpip(&mut self, port: u16) -> Result<(), Box<dyn Error>> {
        let mut output = Vec::new();
        self.device.shell_command(
            &format!("setprop service.adb.tcp.port {}", port),
            &mut output,
        )?;
        self.device.shell_command(&"stop adbd", &mut output)?;
        self.device.shell_command(&"start adbd", &mut output)?;
        Ok(())
    }

    pub fn get_device_ip(&mut self) -> Result<String, Box<dyn Error>> {
        let mut output = Vec::new();
        self.device
            .shell_command(&"getprop dhcp.wlan0.ipaddress", &mut output)?;
        let ip = String::from_utf8_lossy(&output).trim().to_string();
        if ip.is_empty() {
            self.device
                .shell_command(&"ip addr show wlan0", &mut output)?;
            let ip_output = String::from_utf8_lossy(&output);
            for line in ip_output.lines() {
                if line.contains("inet ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 1 {
                        let ip_with_mask = parts[1];
                        let ip_part = ip_with_mask.split('/').next().unwrap_or("");
                        if !ip_part.is_empty() {
                            return Ok(ip_part.to_string());
                        }
                    }
                }
            }
            Ok("Unknown".to_string())
        } else {
            Ok(ip)
        }
    }

    fn extract_property(props: &str, key: &str) -> String {
        for line in props.lines() {
            if line.starts_with(&format!("[{}]", key)) {
                if let Some(value_part) = line.split_once("]: [") {
                    let value = value_part.1.trim_end_matches(']');
                    return value.to_string();
                }
            }
        }
        "Unknown".to_string()
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn into_controller(self) -> ATVController {
        ATVController::new(self.device)
    }
}
