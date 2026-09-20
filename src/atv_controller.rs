use crate::device::ADBDevice;
use adb_client::RebootType;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize, Object, Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub model: String,
    pub android_version: String,
    pub screen_resolution: String,
    pub serial: String,
}

pub struct ATVController {
    device: ADBDevice,
}

fn combo_keyevent_command(keys: &[u32]) -> Result<String, Box<dyn Error>> {
    if keys.is_empty() {
        return Err("At least one keycode is required".into());
    }

    let key_str = keys
        .iter()
        .map(|keycode| keycode.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    Ok(format!("input keyevent {}", key_str))
}

fn long_press_command(keycode: u32, duration_ms: u32) -> String {
    let _ = duration_ms;
    format!("input keyevent --longpress {}", keycode)
}

fn mouse_scroll_command(direction: &str, amount: u32) -> Result<String, Box<dyn Error>> {
    let x = 540;
    let start_y = 800;
    let delta = amount as i32 * 100;
    let end_y = match direction {
        "up" => start_y - delta,
        "down" => start_y + delta,
        _ => return Err("Invalid scroll direction".into()),
    };

    Ok(format!("input swipe {} {} {} {} 300", x, start_y, x, end_y))
}

fn gesture_commands(
    points: &[(u32, u32)],
    duration_ms: u32,
) -> Result<Vec<String>, Box<dyn Error>> {
    if points.len() < 2 {
        return Err("At least 2 points required".into());
    }

    let segment_duration = if points.len() == 2 {
        duration_ms
    } else {
        duration_ms / (points.len() as u32 - 1)
    };

    Ok(points
        .windows(2)
        .map(|segment| {
            let (from_x, from_y) = segment[0];
            let (to_x, to_y) = segment[1];
            format!(
                "input swipe {} {} {} {} {}",
                from_x, from_y, to_x, to_y, segment_duration
            )
        })
        .collect())
}

fn mapped_keycode(key: &str, mapping: &HashMap<String, u32>) -> Result<u32, Box<dyn Error>> {
    mapping
        .get(key)
        .copied()
        .ok_or_else(|| "Unknown mapped key".into())
}

impl ATVController {
    pub fn new(device: ADBDevice) -> Self {
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

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<(), Box<dyn Error>> {
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

    pub fn push_file(
        &mut self,
        local_data: &[u8],
        remote_path: &str,
    ) -> Result<(), Box<dyn Error>> {
        let mut cursor = std::io::Cursor::new(local_data);
        self.device.push(&mut cursor, &remote_path)?;
        Ok(())
    }

    pub fn pull_file(&mut self, remote_path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut output = Vec::new();
        self.device.pull(&remote_path, &mut output)?;
        Ok(output)
    }

    pub fn install_apk(&mut self, apk_path: &Path) -> Result<(), Box<dyn Error>> {
        self.device.install(&apk_path)?;
        Ok(())
    }

    pub fn send_combo_keys(&mut self, keys: &[u32]) -> Result<(), Box<dyn Error>> {
        let command = combo_keyevent_command(keys)?;
        let mut output = Vec::new();
        self.device.shell_command(&command, &mut output)?;
        Ok(())
    }

    pub fn long_press(&mut self, keycode: u32, duration_ms: u32) -> Result<(), Box<dyn Error>> {
        let command = long_press_command(keycode, duration_ms);
        let mut output = Vec::new();
        self.device.shell_command(&command, &mut output)?;
        Ok(())
    }

    pub fn send_keyevent(&mut self, keycode: u32) -> Result<(), Box<dyn Error>> {
        let command = format!("input keyevent {}", keycode);
        let mut output = Vec::new();
        self.device.shell_command(&command, &mut output)?;
        Ok(())
    }

    pub fn send_key(&mut self, key: &str) -> Result<(), Box<dyn Error>> {
        let keycode = match key.to_lowercase().as_str() {
            "home" => 3,
            "back" => 4,
            "menu" => 82,
            "recent_apps" => 187,
            "power" => 26,
            "volume_up" | "vol_up" => 24,
            "volume_down" | "vol_down" => 25,
            "mute" | "volume_mute" => 164,
            "enter" => 66,
            "space" => 62,
            "tab" => 61,
            "delete" | "del" | "backspace" => 67,
            "up" | "dpad_up" => 19,
            "down" | "dpad_down" => 20,
            "left" | "dpad_left" => 21,
            "right" | "dpad_right" => 22,
            "center" | "ok" | "select" | "dpad_center" => 23,
            "play_pause" | "play" | "pause" | "media_play_pause" => 85,
            "stop" => 86,
            "next" | "fast_forward" => 87,
            "previous" | "rewind" | "media_previous" => 88,
            "media_play" => 126,
            "media_pause" => 127,
            "media_stop" => 86,
            "media_next" => 87,
            "media_prev" => 88,
            _ => return Err(format!("Unknown key: {}", key).into()),
        };

        self.send_keyevent(keycode)
    }

    pub fn execute_shell_command(&mut self, command: &str) -> Result<String, Box<dyn Error>> {
        let mut output = Vec::new();
        self.device.shell_command(&command, &mut output)?;
        Ok(String::from_utf8_lossy(&output).trim().to_string())
    }

    pub fn input_text(&mut self, text: &str) -> Result<(), Box<dyn Error>> {
        let escaped_text = text.replace("'", "\\'");
        let command = format!("input text '{}'", escaped_text);
        let mut output = Vec::new();
        self.device.shell_command(&command, &mut output)?;
        Ok(())
    }

    pub fn mouse_scroll(&mut self, direction: &str, amount: u32) -> Result<(), Box<dyn Error>> {
        let command = mouse_scroll_command(direction, amount)?;
        let mut output = Vec::new();
        self.device.shell_command(&command, &mut output)?;
        Ok(())
    }

    pub fn execute_gesture(
        &mut self,
        points: &[(u32, u32)],
        duration_ms: u32,
    ) -> Result<(), Box<dyn Error>> {
        for command in gesture_commands(points, duration_ms)? {
            let mut output = Vec::new();
            self.device.shell_command(&command, &mut output)?;
        }
        Ok(())
    }

    pub fn send_mapped_key(
        &mut self,
        key: &str,
        mapping: &HashMap<String, u32>,
    ) -> Result<(), Box<dyn Error>> {
        let keycode = mapped_keycode(key, mapping)?;
        self.send_keyevent(keycode)
    }

    pub fn list_packages(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
        let mut output = Vec::new();
        self.device
            .shell_command(&"pm list packages".to_string(), &mut output)?;

        Ok(parse_package_list(&String::from_utf8_lossy(&output)))
    }

    pub fn launch_app(&mut self, package: &str) -> Result<(), Box<dyn Error>> {
        let command = format!(
            "monkey -p {} -c android.intent.category.LAUNCHER 1",
            package
        );
        let mut output = Vec::new();
        self.device.shell_command(&command, &mut output)?;
        Ok(())
    }

    pub fn uninstall_app(&mut self, package: &str) -> Result<(), Box<dyn Error>> {
        self.device.uninstall(&package)?;
        Ok(())
    }

    pub fn reboot(&mut self) -> Result<(), Box<dyn Error>> {
        self.device.reboot(RebootType::System)?;
        Ok(())
    }

    pub fn reboot_recovery(&mut self) -> Result<(), Box<dyn Error>> {
        self.device.reboot(RebootType::Recovery)?;
        Ok(())
    }

    pub fn reboot_bootloader(&mut self) -> Result<(), Box<dyn Error>> {
        self.device.reboot(RebootType::Bootloader)?;
        Ok(())
    }

    pub fn clear_app_data(&mut self, package: &str) -> Result<(), Box<dyn Error>> {
        let command = format!("pm clear {}", package);
        let mut output = Vec::new();
        self.device.shell_command(&command, &mut output)?;
        Ok(())
    }

    fn get_screen_resolution(&mut self) -> Result<String, Box<dyn Error>> {
        let mut output = Vec::new();
        self.device
            .shell_command(&"wm size".to_string(), &mut output)?;
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

    pub fn tap(&mut self, x: u32, y: u32) -> Result<(), Box<dyn Error>> {
        let command = format!("input tap {} {}", x, y);
        let mut output = Vec::new();
        self.device.shell_command(&command, &mut output)?;
        Ok(())
    }

    pub fn take_screenshot(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut output = Vec::new();
        self.device
            .shell_command(&"screencap -p".to_string(), &mut output)?;
        Ok(output)
    }

    pub fn stream_logcat(&mut self, output: &mut dyn Write) -> Result<(), Box<dyn Error>> {
        self.device
            .shell_command(&"logcat -v time".to_string(), output)?;
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
            Some(duration) => format!(
                "input swipe {} {} {} {} {}",
                from_x, from_y, to_x, to_y, duration
            ),
            None => format!("input swipe {} {} {} {}", from_x, from_y, to_x, to_y),
        };
        let mut output = Vec::new();
        self.device.shell_command(&command, &mut output)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        combo_keyevent_command, gesture_commands, long_press_command, mapped_keycode,
        mouse_scroll_command,
    };
    use std::collections::HashMap;

    #[test]
    fn combo_keyevent_command_joins_keycodes() {
        let command = combo_keyevent_command(&[3, 82, 66]).unwrap();
        assert_eq!(command, "input keyevent 3 82 66");
    }

    #[test]
    fn combo_keyevent_command_rejects_empty_input() {
        let error = combo_keyevent_command(&[]).unwrap_err();
        assert_eq!(error.to_string(), "At least one keycode is required");
    }

    #[test]
    fn long_press_command_uses_longpress_flag() {
        let command = long_press_command(23, 1500);
        assert_eq!(command, "input keyevent --longpress 23");
    }

    #[test]
    fn mouse_scroll_command_builds_upward_swipe() {
        let command = mouse_scroll_command("up", 2).unwrap();
        assert_eq!(command, "input swipe 540 800 540 600 300");
    }

    #[test]
    fn mouse_scroll_command_rejects_invalid_direction() {
        let error = mouse_scroll_command("left", 1).unwrap_err();
        assert_eq!(error.to_string(), "Invalid scroll direction");
    }

    #[test]
    fn gesture_commands_build_segmented_swipes() {
        let commands = gesture_commands(&[(10, 20), (30, 40), (50, 60)], 900).unwrap();
        assert_eq!(
            commands,
            vec![
                "input swipe 10 20 30 40 450".to_string(),
                "input swipe 30 40 50 60 450".to_string(),
            ]
        );
    }

    #[test]
    fn gesture_commands_require_two_points() {
        let error = gesture_commands(&[(10, 20)], 300).unwrap_err();
        assert_eq!(error.to_string(), "At least 2 points required");
    }

    #[test]
    fn mapped_keycode_returns_matching_keycode() {
        let mapping = HashMap::from([("play".to_string(), 85)]);
        let keycode = mapped_keycode("play", &mapping).unwrap();
        assert_eq!(keycode, 85);
    }

    #[test]
    fn mapped_keycode_rejects_unknown_key() {
        let mapping = HashMap::new();
        let error = mapped_keycode("play", &mapping).unwrap_err();
        assert_eq!(error.to_string(), "Unknown mapped key");
    }

    #[test]
    fn mouse_scroll_command_supports_downward_swipe() {
        let command = mouse_scroll_command("down", 2).unwrap();
        assert_eq!(command, "input swipe 540 800 540 1000 300");
    }
}

fn parse_package_list(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| line.strip_prefix("package:"))
        .map(|package| package.trim().to_string())
        .collect()
}

#[cfg(test)]
mod package_tests {
    use super::parse_package_list;

    #[test]
    fn parse_package_list_extracts_package_names() {
        let output = "package:com.example.one\npackage: com.example.two \nignored\n";

        let packages = parse_package_list(output);

        assert_eq!(packages, vec!["com.example.one", "com.example.two"]);
    }

    #[test]
    fn parse_package_list_returns_empty_when_no_packages_present() {
        let packages = parse_package_list("warning\nerror\n");

        assert!(packages.is_empty());
    }
}

#[cfg(test)]
mod screenshot_tests {
    use crate::tests::mocks::{MockADBDevice, MockATVController};
    use std::io::Cursor;

    #[test]
    fn take_screenshot_returns_png_bytes() {
        let mut device = MockADBDevice::new();
        let png_bytes = "\u{0089}PNG\r\n\u{001a}\nmock";
        device.set_shell_response("screencap -p", png_bytes);

        let mut controller = MockATVController::new(device);
        let screenshot = controller.take_screenshot().unwrap();

        assert_eq!(screenshot, png_bytes.as_bytes());
        assert_eq!(controller.device().command_log(), &["screencap -p"]);
    }

    #[test]
    fn take_screenshot_propagates_shell_errors() {
        let device = MockADBDevice::failing("screenshot failed");
        let mut controller = MockATVController::new(device);

        let error = controller.take_screenshot().unwrap_err();

        assert_eq!(error.to_string(), "screenshot failed");
    }

    #[test]
    fn stream_logcat_uses_timestamped_command() {
        let mut device = MockADBDevice::new();
        device.set_shell_response("logcat -v time", "06-01 10:00:00.000 I/Test: hello\n");

        let mut controller = MockATVController::new(device);
        let mut output = Cursor::new(Vec::new());

        controller.stream_logcat(&mut output).unwrap();

        assert_eq!(
            String::from_utf8(output.into_inner()).unwrap(),
            "06-01 10:00:00.000 I/Test: hello\n"
        );
        assert_eq!(controller.device().command_log(), &["logcat -v time"]);
    }
}

#[cfg(test)]
mod app_management_tests {
    use crate::tests::mocks::{MockADBDevice, MockATVController};

    #[test]
    fn clear_app_data_sends_pm_clear_command() {
        let mut device = MockADBDevice::new();
        device.set_shell_response("pm clear com.example.app", "Success\n");

        let mut controller = MockATVController::new(device);
        controller.clear_app_data("com.example.app").unwrap();

        assert_eq!(
            controller.device().command_log(),
            &["pm clear com.example.app"]
        );
    }
}

#[cfg(test)]
mod device_info_tests {
    use super::DeviceInfo;
    use crate::tests::mocks::{MockADBDevice, MockATVController};

    #[test]
    fn get_device_info_reads_expected_properties() {
        let mut device = MockADBDevice::new();
        device.set_shell_responses([
            (
                "getprop ro.product.model".to_string(),
                "Chromecast with Google TV\n".to_string(),
            ),
            (
                "getprop ro.build.version.release".to_string(),
                "14\n".to_string(),
            ),
            (
                "wm size".to_string(),
                "Physical size: 1920x1080\n".to_string(),
            ),
            ("getprop ro.serialno".to_string(), "ABC123\n".to_string()),
        ]);

        let mut controller = MockATVController::new(device);

        let info = controller.get_device_info().unwrap();

        assert_eq!(
            info,
            DeviceInfo {
                model: "Chromecast with Google TV".to_string(),
                android_version: "14".to_string(),
                screen_resolution: "1920x1080".to_string(),
                serial: "ABC123".to_string(),
            }
        );
        assert_eq!(
            controller.device().command_log(),
            &[
                "getprop ro.product.model",
                "getprop ro.build.version.release",
                "wm size",
                "getprop ro.serialno",
            ]
        );
    }

    #[test]
    fn get_device_info_propagates_property_errors() {
        let device = MockADBDevice::failing("device offline");
        let mut controller = MockATVController::new(device);

        let error = controller.get_device_info().unwrap_err();

        assert_eq!(error.to_string(), "device offline");
    }
}
