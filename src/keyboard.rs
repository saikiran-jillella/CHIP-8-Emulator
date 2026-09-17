use crate::{Chip8, Input};
use evdev::{Device, EventSummary, KeyCode};
use std::io;
use udev::Enumerator;

const KEYMAP: [KeyCode; 16] = [
    KeyCode::KEY_X, // 0
    KeyCode::KEY_1, // 1
    KeyCode::KEY_2, // 2
    KeyCode::KEY_3, // 3
    KeyCode::KEY_Q, // 4
    KeyCode::KEY_W, // 5
    KeyCode::KEY_E, // 6
    KeyCode::KEY_A, // 7
    KeyCode::KEY_S, // 8
    KeyCode::KEY_D, // 9
    KeyCode::KEY_Z, // A
    KeyCode::KEY_C, // B
    KeyCode::KEY_4, // C
    KeyCode::KEY_R, // D
    KeyCode::KEY_F, // E
    KeyCode::KEY_V, // F
];

pub struct Keyboard {
    device: Device,
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Keyboard {
    pub fn new() -> Self {
        let mut enumerator = Enumerator::new().expect("failed to create udev enumerator");

        enumerator.match_subsystem("input").expect("failed to match input subsystem");

        enumerator.match_sysname("event*").expect("failed to match event devices");

        let device = enumerator
            .scan_devices()
            .expect("failed to enumerate input devices")
            .find_map(|udev_device| {
                let keyboard = udev_device.property_value("ID_INPUT_KEYBOARD").and_then(|v| v.to_str()) == Some("1");

                if !keyboard {
                    return None;
                }

                let path = udev_device.devnode()?;

                let device = Device::open(path).ok()?;

                let keys = device.supported_keys()?;

                let supports_chip8_keys = KEYMAP.iter().all(|key| keys.contains(*key));

                supports_chip8_keys.then_some(device)
            })
            .expect("keyboard not found");

        let device = device;

        device.set_nonblocking(true).expect("failed to set keyboard nonblocking");

        println!("Using keyboard: {}", device.name().unwrap_or("unknown"));

        Self { device }
    }
}

impl Input for Keyboard {
    fn update(&mut self, chip8: &mut Chip8) {
        let events = match self.device.fetch_events() {
            Ok(events) => events,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => return,
            Err(e) => panic!("failed to read keyboard events: {e}"),
        };

        for event in events {
            match event.destructure() {
                EventSummary::Key(_, key, 1) => {
                    if let Some(index) = KEYMAP.iter().position(|&mapped| mapped == key) {
                        chip8.key_down(index);
                    }
                }

                EventSummary::Key(_, key, 0) => {
                    if let Some(index) = KEYMAP.iter().position(|&mapped| mapped == key) {
                        chip8.key_up(index);
                    }
                }

                _ => {}
            }
        }
    }
}
