use adb_client::server_device::ADBServerDevice;
use std::error::Error;
use std::net::SocketAddrV4;

pub struct TCPIPConfig;

impl TCPIPConfig {
    pub fn enable_tcpip_for_device(
        device_identifier: String,
        server_addr: Option<SocketAddrV4>,
        port: u16,
    ) -> Result<(), Box<dyn Error>> {
        let mut device = ADBServerDevice::new(device_identifier.clone(), server_addr);
        device.tcpip(port)?;
        Ok(())
    }
}
