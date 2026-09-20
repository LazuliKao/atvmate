#[cfg(feature = "usb")]
use adb_client::usb::ADBUSBDevice;
use adb_client::{
    ADBDeviceExt, ADBListItemType, AdbStatResponse, RebootType, Result,
    server_device::ADBServerDevice, tcp::ADBTcpDevice,
};
use std::io::Write;
use std::net::{SocketAddr, SocketAddrV4};
use std::path::Path;

pub enum ADBDevice {
    Server(ADBServerDevice),
    Tcp(ADBTcpDevice),
    #[cfg(feature = "usb")]
    Usb(ADBUSBDevice),
}

impl ADBDevice {
    pub fn tcp(address: SocketAddr) -> Result<Self> {
        Ok(Self::Tcp(ADBTcpDevice::new(address)?))
    }

    #[cfg(feature = "usb")]
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
            #[cfg(feature = "usb")]
            Self::Usb(device) => ADBDeviceExt::shell_command(device, command, output),
        }
    }

    pub fn pull(&mut self, source: &dyn AsRef<str>, output: &mut dyn Write) -> Result<()> {
        match self {
            Self::Server(device) => ADBDeviceExt::pull(device, source, output),
            Self::Tcp(device) => ADBDeviceExt::pull(device, source, output),
            #[cfg(feature = "usb")]
            Self::Usb(device) => ADBDeviceExt::pull(device, source, output),
        }
    }

    pub fn push(&mut self, stream: &mut dyn std::io::Read, path: &dyn AsRef<str>) -> Result<()> {
        match self {
            Self::Server(device) => ADBDeviceExt::push(device, stream, path),
            Self::Tcp(device) => ADBDeviceExt::push(device, stream, path),
            #[cfg(feature = "usb")]
            Self::Usb(device) => ADBDeviceExt::push(device, stream, path),
        }
    }

    pub fn install(&mut self, apk_path: &dyn AsRef<Path>) -> Result<()> {
        match self {
            Self::Server(device) => ADBDeviceExt::install(device, apk_path),
            Self::Tcp(device) => ADBDeviceExt::install(device, apk_path),
            #[cfg(feature = "usb")]
            Self::Usb(device) => ADBDeviceExt::install(device, apk_path),
        }
    }

    pub fn uninstall(&mut self, package: &dyn AsRef<str>) -> Result<()> {
        match self {
            Self::Server(device) => ADBDeviceExt::uninstall(device, package),
            Self::Tcp(device) => ADBDeviceExt::uninstall(device, package),
            #[cfg(feature = "usb")]
            Self::Usb(device) => ADBDeviceExt::uninstall(device, package),
        }
    }

    pub fn reboot(&mut self, reboot_type: RebootType) -> Result<()> {
        match self {
            Self::Server(device) => ADBDeviceExt::reboot(device, reboot_type),
            Self::Tcp(device) => ADBDeviceExt::reboot(device, reboot_type),
            #[cfg(feature = "usb")]
            Self::Usb(device) => ADBDeviceExt::reboot(device, reboot_type),
        }
    }

    pub fn list(&mut self, path: &dyn AsRef<str>) -> Result<Vec<ADBListItemType>> {
        match self {
            Self::Server(device) => ADBDeviceExt::list(device, path),
            Self::Tcp(device) => ADBDeviceExt::list(device, path),
            #[cfg(feature = "usb")]
            Self::Usb(device) => ADBDeviceExt::list(device, path),
        }
    }

    pub fn stat(&mut self, remote_path: &dyn AsRef<str>) -> Result<AdbStatResponse> {
        match self {
            Self::Server(device) => ADBDeviceExt::stat(device, remote_path),
            Self::Tcp(device) => ADBDeviceExt::stat(device, remote_path),
            #[cfg(feature = "usb")]
            Self::Usb(device) => ADBDeviceExt::stat(device, remote_path),
        }
    }

    pub fn run_activity(
        &mut self,
        package: &dyn AsRef<str>,
        activity: &dyn AsRef<str>,
    ) -> Result<Vec<u8>> {
        match self {
            Self::Server(device) => ADBDeviceExt::run_activity(device, package, activity),
            Self::Tcp(device) => ADBDeviceExt::run_activity(device, package, activity),
            #[cfg(feature = "usb")]
            Self::Usb(device) => ADBDeviceExt::run_activity(device, package, activity),
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
