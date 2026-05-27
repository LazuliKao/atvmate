use adb_client::server::{ADBServer, DeviceShort};
use std::error::Error;
use std::net::{Ipv4Addr, SocketAddrV4};

pub const DEFAULT_ADB_SERVER_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
pub const DEFAULT_ADB_SERVER_PORT: u16 = 5037;

pub struct ADBService {
    server: ADBServer,
}

impl ADBService {
    pub fn new(server_ip: Ipv4Addr, server_port: u16) -> Self {
        Self::from_addr(SocketAddrV4::new(server_ip, server_port))
    }

    pub fn from_addr(server_addr: SocketAddrV4) -> Self {
        let server = ADBServer::new(server_addr);
        Self { server }
    }

    pub fn default_server() -> Self {
        Self::new(DEFAULT_ADB_SERVER_IP, DEFAULT_ADB_SERVER_PORT)
    }

    pub fn get_devices(&mut self) -> Result<Vec<DeviceShort>, Box<dyn Error>> {
        Ok(self.server.devices()?)
    }
}

impl Default for ADBService {
    fn default() -> Self {
        Self::default_server()
    }
}
