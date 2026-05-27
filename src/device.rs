use adb_client::{
    ADBDeviceExt, Result, server_device::ADBServerDevice, tcp::ADBTcpDevice, usb::ADBUSBDevice,
};
use std::io::Write;
use std::net::{SocketAddr, SocketAddrV4};

pub enum ADBDevice {
    Server(ADBServerDevice),
    Tcp(ADBTcpDevice),
    Usb(ADBUSBDevice),
}

impl ADBDevice {
    pub fn tcp(address: SocketAddr) -> Result<Self> {
        Ok(Self::Tcp(ADBTcpDevice::new(address)?))
    }

    pub fn usb(vendor_id: Option<u16>, product_id: Option<u16>) -> Result<Self> {
        match (vendor_id, product_id) {
            (Some(vendor_id), Some(product_id)) => {
                Ok(Self::Usb(ADBUSBDevice::new(vendor_id, product_id)?))
            }
            _ => Ok(Self::Usb(ADBUSBDevice::autodetect()?)),
        }
    }

    pub fn server(identifier: String, server_addr: Option<SocketAddrV4>) -> Self {
        Self::Server(ADBServerDevice::new(identifier, server_addr))
    }

    pub fn read_property(&mut self, name: &str) -> Result<String> {
        let command = format!("getprop {}", name);
        let mut output = Vec::new();
        self.shell_command(&command, &mut output)?;
        Ok(String::from_utf8_lossy(&output).trim().to_string())
    }

    pub fn shell_command(
        &mut self,
        command: &dyn AsRef<str>,
        output: &mut dyn Write,
    ) -> Result<()> {
        match self {
            Self::Server(device) => ADBDeviceExt::shell_command(device, command, output),
            Self::Tcp(device) => ADBDeviceExt::shell_command(device, command, output),
            Self::Usb(device) => ADBDeviceExt::shell_command(device, command, output),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DiscoveredDevice {
    pub id: String,
    pub identifier: String,
    pub connection_type: String,
    pub state: String,
}
