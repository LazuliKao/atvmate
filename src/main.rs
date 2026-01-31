use std::net::{SocketAddrV4, Ipv4Addr};
use atvmate::{adb_service::ADBService, device_manager::DeviceManager, tcpip_config::TCPIPConfig, remote_device::RemoteDevice};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    enable_tcpip_for_all_usb_devices().await?;
    demonstrate_atv_controller().await?;
    Ok(())
}

async fn enable_tcpip_for_all_usb_devices() -> Result<(), Box<dyn Error>> {
    println!("Connecting to ADB server...");
    let server_ip = Ipv4Addr::new(127, 0, 0, 1);
    let server_port = 5037;
    
    let mut adb_service = ADBService::new(server_ip, server_port);

    println!("Searching for USB devices...");
    let devices = adb_service.get_devices()?;

    if devices.is_empty() {
        println!("No devices found.");
        return Ok(());
    }

    let usb_devices = DeviceManager::filter_usb_devices(devices);
    
    if usb_devices.is_empty() {
        println!("No USB devices found.");
        return Ok(());
    }

    for device_info in usb_devices {
        println!("Found USB device: {}. Enabling tcpip on port 5555...", device_info.identifier);
        
        let server_addr = Some(SocketAddrV4::new(server_ip, server_port));
        match TCPIPConfig::enable_tcpip_for_device(device_info.identifier.clone(), server_addr, 5555) {
            Ok(_) => {
                println!("Successfully enabled tcpip for {}.", device_info.identifier);
            }
            Err(e) => {
                eprintln!("Failed to enable tcpip for {}: {}", device_info.identifier, e);
            }
        }
    }

    Ok(())
}

async fn demonstrate_atv_controller() -> Result<(), Box<dyn Error>> {
    println!("\nConnecting to remote device at 100.108.112.14:5555...");
    let remote_ip = Ipv4Addr::new(100, 108, 112, 14);
    let remote_port = 5555;
    
    let remote_device = RemoteDevice::new(remote_ip, remote_port)?;
    println!("Connected successfully!");
    
    let mut device_for_info = remote_device;
    let device_info = device_for_info.get_device_info()?;
    
    println!("Device Information:");
    println!("  Serial: {}", device_info.serial);
    println!("  Model: {}", device_info.model);
    println!("  Manufacturer: {}", device_info.manufacturer);
    println!("  Android Version: {}", device_info.android_version);
    println!("  Build ID: {}", device_info.build_id);
    println!("  Product Name: {}", device_info.product_name);
    
    let device_ip = device_for_info.get_device_ip()?;
    println!("  Device IP: {}", device_ip);
    
    println!("\nDemonstrating ATV Controller functionality...");
    let mut controller = device_for_info.into_controller();
    
    println!("Sending HOME key event...");
    controller.home()?;
    
    println!("Sending BACK key event...");
    controller.back()?;
    
    println!("Sending VOLUME UP key event...");
    controller.volume_up()?;
    
    println!("Sending VOLUME DOWN key event...");
    controller.volume_down()?;
    
    println!("Sending DPAD navigation sequence...");
    controller.dpad_up()?;
    controller.dpad_down()?;
    controller.dpad_left()?;
    controller.dpad_right()?;
    controller.dpad_center()?;
    
    println!("Sending text input: 'Hello ATV!'...");
    controller.input_text("Hello ATV!")?;
    
    println!("ATV Controller demonstration completed!");
    
    Ok(())
}