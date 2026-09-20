//! Mock implementations for testing without real ADB devices.
//!
//! Since `ADBDevice` is an enum (not a trait), we provide parallel mock structs
//! that replicate the same public interface. These are useful for unit testing
//! code patterns that interact with device APIs without requiring real hardware.

use crate::atv_controller::DeviceInfo;
use adb_client::RebootType;
use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// MockADBDevice – simulates `crate::device::ADBDevice`
// ---------------------------------------------------------------------------

/// A configurable mock that mirrors the public methods of [`crate::device::ADBDevice`].
///
/// Pre-load shell-command responses (or errors) and then call methods to
/// retrieve them, exactly as you would with the real device.
///
/// # Example
/// ```
/// use atvmate::tests::mocks::MockADBDevice;
///
/// let mut dev = MockADBDevice::new();
/// dev.set_shell_response("getprop ro.build.display.id", "12345");
/// let mut out = Vec::new();
/// dev.shell_command("getprop ro.build.display.id".to_string(), &mut out).unwrap();
/// assert_eq!(String::from_utf8(out).unwrap(), "12345");
/// ```
pub struct MockADBDevice {
    /// Maps command string → simulated stdout output.
    shell_responses: HashMap<String, String>,
    /// When `true`, every `shell_command` call returns `Err`.
    should_fail: bool,
    /// Optional error message returned when `should_fail` is true.
    fail_message: String,
    /// Stores every command that was executed (for assertions).
    command_log: Vec<String>,
}

impl MockADBDevice {
    /// Create a new mock with no pre-loaded responses.
    pub fn new() -> Self {
        Self {
            shell_responses: HashMap::new(),
            should_fail: false,
            fail_message: "Mock ADB error".to_string(),
            command_log: Vec::new(),
        }
    }

    /// Create a mock that immediately returns errors on every command.
    pub fn failing(message: impl Into<String>) -> Self {
        Self {
            should_fail: true,
            fail_message: message.into(),
            ..Self::new()
        }
    }

    // -- configuration -------------------------------------------------------

    /// Register a response for an exact command string.
    pub fn set_shell_response(&mut self, command: impl Into<String>, output: impl Into<String>) {
        self.shell_responses.insert(command.into(), output.into());
    }

    /// Register multiple responses at once.
    pub fn set_shell_responses(&mut self, responses: impl IntoIterator<Item = (String, String)>) {
        self.shell_responses.extend(responses);
    }

    /// Make every subsequent command fail with the given message.
    pub fn set_failing(&mut self, message: impl Into<String>) {
        self.should_fail = true;
        self.fail_message = message.into();
    }

    /// Stop failing and restore normal operation.
    pub fn clear_failure(&mut self) {
        self.should_fail = false;
    }

    // -- ADBDevice-like methods ----------------------------------------------

    /// Simulates [`ADBDevice::shell_command`].
    pub fn shell_command(
        &mut self,
        command: String,
        output: &mut dyn Write,
    ) -> Result<(), Box<dyn Error>> {
        self.command_log.push(command.clone());

        if self.should_fail {
            return Err(self.fail_message.clone().into());
        }

        match self.shell_responses.get(&command) {
            Some(resp) => {
                output.write_all(resp.as_bytes())?;
                Ok(())
            }
            // Empty response for unregistered commands (mimics real ADB behaviour
            // where many commands succeed silently).
            None => Ok(()),
        }
    }

    /// Simulates [`ADBDevice::read_property`].
    pub fn read_property(&mut self, name: &str) -> Result<String, Box<dyn Error>> {
        let command = format!("getprop {}", name);
        let mut buf = Vec::new();
        self.shell_command(command, &mut buf)?;
        Ok(String::from_utf8_lossy(&buf).trim().to_string())
    }

    /// Simulates `adb pull`.
    pub fn pull(&mut self, remote_path: &str, local_path: &str) -> Result<(), Box<dyn Error>> {
        let command = format!("pull {} {}", remote_path, local_path);
        let mut output = Vec::new();
        self.shell_command(command, &mut output)
    }

    /// Simulates `adb push`.
    pub fn push(&mut self, local_path: &str, remote_path: &str) -> Result<(), Box<dyn Error>> {
        let command = format!("push {} {}", local_path, remote_path);
        let mut output = Vec::new();
        self.shell_command(command, &mut output)
    }

    /// Simulates `adb install`.
    pub fn install(&mut self, apk_path: &str) -> Result<(), Box<dyn Error>> {
        let command = format!("install {}", apk_path);
        let mut output = Vec::new();
        self.shell_command(command, &mut output)
    }

    /// Simulates `adb uninstall`.
    pub fn uninstall(&mut self, package: &str) -> Result<(), Box<dyn Error>> {
        let command = format!("uninstall {}", package);
        let mut output = Vec::new();
        self.shell_command(command, &mut output)
    }

    /// Simulates `adb reboot`.
    pub fn reboot(&mut self, reboot_type: RebootType) -> Result<(), Box<dyn Error>> {
        let command = match reboot_type {
            RebootType::System => "reboot".to_string(),
            RebootType::Bootloader => "reboot bootloader".to_string(),
            RebootType::Recovery => "reboot recovery".to_string(),
            RebootType::Sideload => "reboot sideload".to_string(),
            RebootType::SideloadAutoReboot => "reboot sideload-auto-reboot".to_string(),
            RebootType::Fastboot => "reboot fastboot".to_string(),
        };
        let mut output = Vec::new();
        self.shell_command(command, &mut output)
    }

    /// Simulates `adb shell ls`.
    pub fn list(&mut self, path: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let mut output = Vec::new();
        self.shell_command(format!("ls {}", path), &mut output)?;
        let text = String::from_utf8_lossy(&output);
        Ok(text.lines().map(|s| s.to_string()).collect())
    }

    /// Simulates `stat` on a remote path.
    pub fn stat(&mut self, remote_path: &str) -> Result<String, Box<dyn Error>> {
        let mut output = Vec::new();
        self.shell_command(format!("stat {}", remote_path), &mut output)?;
        Ok(String::from_utf8_lossy(&output).to_string())
    }

    /// Simulates `am start`.
    pub fn run_activity(&mut self, package: &str, activity: &str) -> Result<(), Box<dyn Error>> {
        let command = format!("am start -n {}/{}", package, activity);
        let mut output = Vec::new();
        self.shell_command(command, &mut output)
    }

    // -- introspection -------------------------------------------------------

    /// Return the list of commands that have been executed.
    pub fn command_log(&self) -> &[String] {
        &self.command_log
    }

    /// Clear the command log.
    pub fn clear_log(&mut self) {
        self.command_log.clear();
    }
}

impl Default for MockADBDevice {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// MockATVController – simulates `crate::atv_controller::ATVController`
// ---------------------------------------------------------------------------

/// A mock controller that mirrors [`crate::atv_controller::ATVController`]'s
/// public API, backed by a [`MockADBDevice`] instead of a real ADB connection.
pub struct MockATVController {
    device: MockADBDevice,
}

impl MockATVController {
    pub fn new(device: MockADBDevice) -> Self {
        Self { device }
    }

    pub fn power(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(26)
    }

    pub fn sleep(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(223)
    }

    pub fn wake_up(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(224)
    }

    pub fn volume_up(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(24)
    }

    pub fn volume_down(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(25)
    }

    pub fn volume_mute(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(164)
    }

    pub fn home(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(3)
    }

    pub fn back(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(4)
    }

    pub fn menu(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(82)
    }

    pub fn recent_apps(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(187)
    }

    pub fn dpad_up(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(19)
    }

    pub fn dpad_down(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(20)
    }

    pub fn dpad_left(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(21)
    }

    pub fn dpad_right(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(22)
    }

    pub fn dpad_center(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(23)
    }

    pub fn play_pause(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(85)
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(86)
    }

    pub fn next_track(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(87)
    }

    pub fn previous(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(88)
    }

    pub fn enter(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(66)
    }

    pub fn space(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(62)
    }

    pub fn backspace(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(67)
    }

    pub fn tab(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_keyevent(61)
    }

    pub fn get_device_info(&mut self) -> Result<DeviceInfo, Box<dyn Error>> {
        Ok(DeviceInfo {
            model: self.device.read_property("ro.product.model")?,
            android_version: self.device.read_property("ro.build.version.release")?,
            screen_resolution: self.get_screen_resolution()?,
            serial: self.device.read_property("ro.serialno")?,
        })
    }

    pub fn send_keyevent(&mut self, keycode: u32) -> Result<(), Box<dyn Error>> {
        let command = format!("input keyevent {}", keycode);
        let mut output = Vec::new();
        self.device.shell_command(command, &mut output)?;
        Ok(())
    }

    pub fn input_text(&mut self, text: &str) -> Result<(), Box<dyn Error>> {
        let escaped_text = text.replace('\'', "\\'");
        let command = format!("input text '{}'", escaped_text);
        let mut output = Vec::new();
        self.device.shell_command(command, &mut output)?;
        Ok(())
    }

    pub fn tap(&mut self, x: u32, y: u32) -> Result<(), Box<dyn Error>> {
        let command = format!("input tap {} {}", x, y);
        let mut output = Vec::new();
        self.device.shell_command(command, &mut output)?;
        Ok(())
    }

    pub fn take_screenshot(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut output = Vec::new();
        self.device
            .shell_command("screencap -p".to_string(), &mut output)?;
        Ok(output)
    }

    pub fn stream_logcat(&mut self, output: &mut dyn Write) -> Result<(), Box<dyn Error>> {
        self.device
            .shell_command("logcat -v time".to_string(), output)?;
        Ok(())
    }

    fn get_screen_resolution(&mut self) -> Result<String, Box<dyn Error>> {
        let mut output = Vec::new();
        self.device
            .shell_command("wm size".to_string(), &mut output)?;
        let text = String::from_utf8_lossy(&output);
        let trimmed = text.trim();

        Ok(trimmed
            .lines()
            .find_map(|line| {
                line.trim()
                    .strip_prefix("Physical size:")
                    .map(|value| value.trim().to_string())
            })
            .unwrap_or_else(|| trimmed.to_string()))
    }

    pub fn list_packages(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
        let mut output = Vec::new();
        self.device
            .shell_command("pm list packages".to_string(), &mut output)?;

        let text = String::from_utf8_lossy(&output);
        Ok(text
            .lines()
            .filter_map(|line| line.strip_prefix("package:"))
            .map(|package| package.trim().to_string())
            .collect())
    }

    pub fn launch_app(&mut self, package: &str) -> Result<(), Box<dyn Error>> {
        let command = format!(
            "monkey -p {} -c android.intent.category.LAUNCHER 1",
            package
        );
        let mut output = Vec::new();
        self.device.shell_command(command, &mut output)?;
        Ok(())
    }

    pub fn uninstall_app(&mut self, package: &str) -> Result<(), Box<dyn Error>> {
        self.device.uninstall(package)
    }

    pub fn reboot(&mut self) -> Result<(), Box<dyn Error>> {
        self.device.reboot(RebootType::System)
    }

    pub fn reboot_recovery(&mut self) -> Result<(), Box<dyn Error>> {
        self.device.reboot(RebootType::Recovery)
    }

    pub fn reboot_bootloader(&mut self) -> Result<(), Box<dyn Error>> {
        self.device.reboot(RebootType::Bootloader)
    }

    pub fn clear_app_data(&mut self, package: &str) -> Result<(), Box<dyn Error>> {
        let command = format!("pm clear {}", package);
        let mut output = Vec::new();
        self.device.shell_command(command, &mut output)?;
        Ok(())
    }

    pub fn swipe(
        &mut self,
        from_x: u32,
        from_y: u32,
        to_x: u32,
        to_y: u32,
        duration_ms: Option<u32>,
    ) -> Result<(), Box<dyn Error>> {
        let command = match duration_ms {
            Some(d) => format!("input swipe {} {} {} {} {}", from_x, from_y, to_x, to_y, d),
            None => format!("input swipe {} {} {} {}", from_x, from_y, to_x, to_y),
        };
        let mut output = Vec::new();
        self.device.shell_command(command, &mut output)?;
        Ok(())
    }

    /// Access the underlying mock device for assertions.
    pub fn device(&self) -> &MockADBDevice {
        &self.device
    }

    /// Access the underlying mock device mutably.
    pub fn device_mut(&mut self) -> &mut MockADBDevice {
        &mut self.device
    }
}

// ---------------------------------------------------------------------------
// MockGlobalDeviceManager – simulates `crate::global_device_manager::GlobalDeviceManager`
// ---------------------------------------------------------------------------

/// A mock device manager that mirrors the public API of
/// [`crate::global_device_manager::GlobalDeviceManager`] but operates entirely
/// with [`MockATVController`] instances – no real ADB required.
pub struct MockGlobalDeviceManager {
    devices: Arc<Mutex<HashMap<String, Arc<Mutex<MockATVController>>>>>,
}

impl MockGlobalDeviceManager {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add a device backed by a pre-configured [`MockADBDevice`].
    pub fn add_mock_device(
        &self,
        id: String,
        mock_device: MockADBDevice,
    ) -> Result<(), Box<dyn Error>> {
        let controller = MockATVController::new(mock_device);
        let mut devices = self
            .devices
            .lock()
            .map_err(|e| format!("Failed to lock devices: {}", e))?;
        devices.insert(id, Arc::new(Mutex::new(controller)));
        Ok(())
    }

    /// Get a handle to a controller by device id.
    pub fn get_controller(
        &self,
        device_id: &str,
    ) -> Result<Arc<Mutex<MockATVController>>, Box<dyn Error>> {
        let devices = self
            .devices
            .lock()
            .map_err(|e| format!("Failed to lock devices: {}", e))?;
        devices
            .get(device_id)
            .cloned()
            .ok_or_else(|| format!("Device {} not found", device_id).into())
    }

    /// List all registered device ids.
    pub fn list_devices(&self) -> Vec<String> {
        let devices = self.devices.lock().unwrap_or_else(|e| e.into_inner());
        devices.keys().cloned().collect()
    }

    /// Remove a device by id.
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

    /// Return the number of registered devices.
    pub fn device_count(&self) -> usize {
        let devices = self.devices.lock().unwrap_or_else(|e| e.into_inner());
        devices.len()
    }
}

impl Default for MockGlobalDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_adb_device_shell_returns_stored_response() {
        let mut dev = MockADBDevice::new();
        dev.set_shell_response("echo hello", "hello\n");
        let mut buf = Vec::new();
        dev.shell_command("echo hello".to_string(), &mut buf)
            .unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "hello\n");
    }

    #[test]
    fn mock_adb_device_shell_unregistered_returns_empty() {
        let mut dev = MockADBDevice::new();
        let mut buf = Vec::new();
        dev.shell_command("ls /".to_string(), &mut buf).unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn mock_adb_device_failing_returns_error() {
        let mut dev = MockADBDevice::failing("device offline");
        let mut buf = Vec::new();
        assert!(dev.shell_command("echo hi".to_string(), &mut buf).is_err());
    }

    #[test]
    fn mock_adb_device_read_property() {
        let mut dev = MockADBDevice::new();
        dev.set_shell_response("getprop ro.build.display.id", "ABC123");
        let val = dev.read_property("ro.build.display.id").unwrap();
        assert_eq!(val, "ABC123");
    }

    #[test]
    fn mock_adb_device_command_log() {
        let mut dev = MockADBDevice::new();
        let mut buf = Vec::new();
        dev.shell_command("cmd1".to_string(), &mut buf).unwrap();
        dev.shell_command("cmd2".to_string(), &mut buf).unwrap();
        assert_eq!(dev.command_log(), &["cmd1", "cmd2"]);
    }

    #[test]
    fn mock_atv_controller_power_sends_keyevent() {
        let mut dev = MockADBDevice::new();
        dev.set_shell_response("input keyevent 26", "");
        let mut ctrl = MockATVController::new(dev);
        ctrl.power().unwrap();
        assert_eq!(ctrl.device().command_log(), &["input keyevent 26"]);
    }

    #[test]
    fn mock_atv_controller_take_screenshot_reads_png_output() {
        let mut dev = MockADBDevice::new();
        dev.set_shell_response("screencap -p", "png-bytes");
        let mut ctrl = MockATVController::new(dev);

        let output = ctrl.take_screenshot().unwrap();

        assert_eq!(output, b"png-bytes");
        assert_eq!(ctrl.device().command_log(), &["screencap -p"]);
    }

    #[test]
    fn mock_global_device_manager_add_and_list() {
        let mgr = MockGlobalDeviceManager::new();
        assert!(mgr.list_devices().is_empty());
        mgr.add_mock_device("test-device".into(), MockADBDevice::new())
            .unwrap();
        assert_eq!(mgr.list_devices(), vec!["test-device"]);
    }

    #[test]
    fn mock_global_device_manager_remove() {
        let mgr = MockGlobalDeviceManager::new();
        mgr.add_mock_device("d1".into(), MockADBDevice::new())
            .unwrap();
        mgr.remove_device("d1").unwrap();
        assert!(mgr.list_devices().is_empty());
    }

    #[test]
    fn mock_global_device_manager_remove_nonexistent_is_error() {
        let mgr = MockGlobalDeviceManager::new();
        assert!(mgr.remove_device("nope").is_err());
    }

    #[test]
    fn mock_atv_controller_list_packages_parses_shell_output() {
        let mut dev = MockADBDevice::new();
        dev.set_shell_response(
            "pm list packages",
            "package:com.example.one\npackage: com.example.two \n",
        );
        let mut ctrl = MockATVController::new(dev);

        let packages = ctrl.list_packages().unwrap();

        assert_eq!(packages, vec!["com.example.one", "com.example.two"]);
        assert_eq!(ctrl.device().command_log(), &["pm list packages"]);
    }

    #[test]
    fn mock_atv_controller_launch_app_uses_monkey_command() {
        let mut dev = MockADBDevice::new();
        dev.set_shell_response(
            "monkey -p com.example.app -c android.intent.category.LAUNCHER 1",
            "",
        );
        let mut ctrl = MockATVController::new(dev);

        ctrl.launch_app("com.example.app").unwrap();

        assert_eq!(
            ctrl.device().command_log(),
            &["monkey -p com.example.app -c android.intent.category.LAUNCHER 1"]
        );
    }

    #[test]
    fn mock_atv_controller_uninstall_app_uses_uninstall_command() {
        let mut dev = MockADBDevice::new();
        dev.set_shell_response("uninstall com.example.app", "");
        let mut ctrl = MockATVController::new(dev);

        ctrl.uninstall_app("com.example.app").unwrap();

        assert_eq!(ctrl.device().command_log(), &["uninstall com.example.app"]);
    }

    #[test]
    fn mock_atv_controller_reboot_uses_reboot_command() {
        let mut dev = MockADBDevice::new();
        dev.set_shell_response("reboot", "");
        let mut ctrl = MockATVController::new(dev);

        ctrl.reboot().unwrap();

        assert_eq!(ctrl.device().command_log(), &["reboot"]);
    }

    #[test]
    fn mock_atv_controller_reboot_recovery_uses_recovery_command() {
        let mut dev = MockADBDevice::new();
        dev.set_shell_response("reboot recovery", "");
        let mut ctrl = MockATVController::new(dev);

        ctrl.reboot_recovery().unwrap();

        assert_eq!(ctrl.device().command_log(), &["reboot recovery"]);
    }

    #[test]
    fn mock_atv_controller_reboot_bootloader_uses_bootloader_command() {
        let mut dev = MockADBDevice::new();
        dev.set_shell_response("reboot bootloader", "");
        let mut ctrl = MockATVController::new(dev);

        ctrl.reboot_bootloader().unwrap();

        assert_eq!(ctrl.device().command_log(), &["reboot bootloader"]);
    }

    #[test]
    fn mock_atv_controller_clear_app_data_uses_pm_clear_command() {
        let mut dev = MockADBDevice::new();
        dev.set_shell_response("pm clear com.example.app", "Success\n");
        let mut ctrl = MockATVController::new(dev);

        ctrl.clear_app_data("com.example.app").unwrap();

        assert_eq!(ctrl.device().command_log(), &["pm clear com.example.app"]);
    }
}
