use adb_client::server::{ADBServer, DeviceShort};
use std::error::Error;
use std::net::{Ipv4Addr, SocketAddrV4};

pub struct ADBService {
    server: ADBServer,
}

impl ADBService {
    pub fn new(server_ip: Ipv4Addr, server_port: u16) -> Self {
        let server = ADBServer::new(SocketAddrV4::new(server_ip, server_port));
        Self { server }
    }

    pub fn get_devices(&mut self) -> Result<Vec<DeviceShort>, Box<dyn Error>> {
        Ok(self.server.devices()?)
    }
}
