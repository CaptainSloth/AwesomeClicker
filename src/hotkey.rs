use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use rdev::{EventType, Key};
use serde::{Deserialize, Serialize};

pub enum HotkeyEvent {
    Toggle,
    Capture { x: i32, y: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HotkeyKey {
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
}

impl HotkeyKey {
    pub const ALL: &'static [HotkeyKey] = &[
        HotkeyKey::F1, HotkeyKey::F2, HotkeyKey::F3, HotkeyKey::F4,
        HotkeyKey::F5, HotkeyKey::F6, HotkeyKey::F7, HotkeyKey::F8,
        HotkeyKey::F9, HotkeyKey::F10, HotkeyKey::F11, HotkeyKey::F12,
    ];

    pub fn name(self) -> &'static str {
        match self {
            HotkeyKey::F1 => "F1",   HotkeyKey::F2 => "F2",
            HotkeyKey::F3 => "F3",   HotkeyKey::F4 => "F4",
            HotkeyKey::F5 => "F5",   HotkeyKey::F6 => "F6",
            HotkeyKey::F7 => "F7",   HotkeyKey::F8 => "F8",
            HotkeyKey::F9 => "F9",   HotkeyKey::F10 => "F10",
            HotkeyKey::F11 => "F11", HotkeyKey::F12 => "F12",
        }
    }

    pub fn to_rdev(self) -> Key {
        match self {
            HotkeyKey::F1 => Key::F1,   HotkeyKey::F2 => Key::F2,
            HotkeyKey::F3 => Key::F3,   HotkeyKey::F4 => Key::F4,
            HotkeyKey::F5 => Key::F5,   HotkeyKey::F6 => Key::F6,
            HotkeyKey::F7 => Key::F7,   HotkeyKey::F8 => Key::F8,
            HotkeyKey::F9 => Key::F9,   HotkeyKey::F10 => Key::F10,
            HotkeyKey::F11 => Key::F11, HotkeyKey::F12 => Key::F12,
        }
    }
}

#[derive(Clone)]
pub struct HotkeyConfig {
    pub toggle_key: HotkeyKey,
    pub capture_key: HotkeyKey,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self { toggle_key: HotkeyKey::F8, capture_key: HotkeyKey::F6 }
    }
}

pub fn start_listener(tx: mpsc::Sender<HotkeyEvent>, config: Arc<Mutex<HotkeyConfig>>) {
    thread::spawn(move || {
        let mut last_x = 0i32;
        let mut last_y = 0i32;
        let callback = move |event: rdev::Event| {
            match event.event_type {
                EventType::MouseMove { x, y } => {
                    last_x = x as i32;
                    last_y = y as i32;
                }
                EventType::KeyPress(key) => {
                    let cfg = config.lock().unwrap();
                    if key == cfg.toggle_key.to_rdev() {
                        let _ = tx.send(HotkeyEvent::Toggle);
                    } else if key == cfg.capture_key.to_rdev() {
                        let _ = tx.send(HotkeyEvent::Capture { x: last_x, y: last_y });
                    }
                }
                _ => {}
            }
        };
        if let Err(e) = rdev::listen(callback) {
            eprintln!("Hotkey listener error: {:?}", e);
        }
    });
}
