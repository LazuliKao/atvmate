use adb_client::{tcp::ADBTcpDevice, ADBDeviceExt};
use std::error::Error;

pub struct ATVController {
    device: ADBTcpDevice,
}

impl ATVController {
    pub fn new(device: ADBTcpDevice) -> Self {
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

    pub fn send_keyevent(&mut self, keycode: u32) -> Result<(), Box<dyn Error>> {
        let command = format!("input keyevent {}", keycode);
        let mut output = Vec::new();
        self.device.shell_command(&command, &mut output)?;
        Ok(())
    }

    pub fn input_text(&mut self, text: &str) -> Result<(), Box<dyn Error>> {
        let escaped_text = text.replace("'", "\\'");
        let command = format!("input text '{}'", escaped_text);
        let mut output = Vec::new();
        self.device.shell_command(&command, &mut output)?;
        Ok(())
    }

    pub fn tap(&mut self, x: u32, y: u32) -> Result<(), Box<dyn Error>> {
        let command = format!("input tap {} {}", x, y);
        let mut output = Vec::new();
        self.device.shell_command(&command, &mut output)?;
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
