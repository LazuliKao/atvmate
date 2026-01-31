use adb_client::server::DeviceShort;

pub struct DeviceManager;

impl DeviceManager {
    pub fn filter_usb_devices(devices: Vec<DeviceShort>) -> Vec<DeviceShort> {
        devices
            .into_iter()
            .filter(|device| !device.identifier.contains(':'))
            .collect()
    }

    pub fn is_usb_device(device: &DeviceShort) -> bool {
        !device.identifier.contains(':')
    }
}
