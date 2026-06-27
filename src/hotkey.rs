use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use rdev::{EventType, Key};
use serde::{Deserialize, Serialize};

pub enum HotkeyEvent {
    Toggle,
    Capture { x: i32, y: i32 },
    Recorded { target: RecordingTarget, key: HotkeyKey },
    RecordingCancelled,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecordingTarget {
    Toggle,
    Capture,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HotkeyKey {
    pub key: Key,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl HotkeyKey {
    pub fn new(key: Key, shift: bool, ctrl: bool, alt: bool) -> Self {
        Self { key, shift, ctrl, alt }
    }

    pub fn name(&self) -> String {
        let mut s = String::new();
        if self.ctrl  { s.push_str("Ctrl+"); }
        if self.shift { s.push_str("Shift+"); }
        if self.alt   { s.push_str("Alt+"); }
        s.push_str(&key_display_name(self.key));
        s
    }

    fn matches(&self, key: Key, shift: bool, ctrl: bool, alt: bool) -> bool {
        self.key == key && self.shift == shift && self.ctrl == ctrl && self.alt == alt
    }
}

// Serde via a plain string-keyed helper so rdev::Key doesn't need to be Serialize.
#[derive(Serialize, Deserialize)]
struct HotkeyKeyData {
    key: String,
    #[serde(default)]
    shift: bool,
    #[serde(default)]
    ctrl: bool,
    #[serde(default)]
    alt: bool,
}

impl Serialize for HotkeyKey {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        HotkeyKeyData { key: key_to_str(self.key), shift: self.shift, ctrl: self.ctrl, alt: self.alt }
            .serialize(s)
    }
}

impl<'de> Deserialize<'de> for HotkeyKey {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let data = HotkeyKeyData::deserialize(d)?;
        let key = str_to_key(&data.key)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown key: {}", data.key)))?;
        Ok(HotkeyKey { key, shift: data.shift, ctrl: data.ctrl, alt: data.alt })
    }
}

pub struct HotkeyConfig {
    pub toggle_key: HotkeyKey,
    pub capture_key: HotkeyKey,
    pub recording: Option<RecordingTarget>,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            toggle_key: HotkeyKey::new(Key::F8, false, false, false),
            capture_key: HotkeyKey::new(Key::F6, false, false, false),
            recording: None,
        }
    }
}

fn is_modifier(key: Key) -> bool {
    matches!(
        key,
        Key::ShiftLeft
            | Key::ShiftRight
            | Key::ControlLeft
            | Key::ControlRight
            | Key::Alt
            | Key::AltGr
            | Key::MetaLeft
            | Key::MetaRight
            | Key::CapsLock
    )
}

pub fn start_listener(tx: mpsc::Sender<HotkeyEvent>, config: Arc<Mutex<HotkeyConfig>>) {
    thread::spawn(move || {
        let mut last_x = 0i32;
        let mut last_y = 0i32;
        let mut shift = false;
        let mut ctrl = false;
        let mut alt = false;

        let callback = move |event: rdev::Event| {
            match event.event_type {
                EventType::MouseMove { x, y } => {
                    last_x = x as i32;
                    last_y = y as i32;
                }
                EventType::KeyPress(key) => {
                    match key {
                        Key::ShiftLeft | Key::ShiftRight => shift = true,
                        Key::ControlLeft | Key::ControlRight => ctrl = true,
                        Key::Alt | Key::AltGr => alt = true,
                        _ => {}
                    }
                    if is_modifier(key) {
                        return;
                    }

                    let mut cfg = config.lock().unwrap();
                    if let Some(target) = cfg.recording {
                        cfg.recording = None;
                        if key == Key::Escape {
                            let _ = tx.send(HotkeyEvent::RecordingCancelled);
                        } else {
                            let hk = HotkeyKey::new(key, shift, ctrl, alt);
                            let _ = tx.send(HotkeyEvent::Recorded { target, key: hk });
                        }
                        return;
                    }

                    if cfg.toggle_key.matches(key, shift, ctrl, alt) {
                        let _ = tx.send(HotkeyEvent::Toggle);
                    } else if cfg.capture_key.matches(key, shift, ctrl, alt) {
                        let _ = tx.send(HotkeyEvent::Capture { x: last_x, y: last_y });
                    }
                }
                EventType::KeyRelease(key) => match key {
                    Key::ShiftLeft | Key::ShiftRight => shift = false,
                    Key::ControlLeft | Key::ControlRight => ctrl = false,
                    Key::Alt | Key::AltGr => alt = false,
                    _ => {}
                },
                _ => {}
            }
        };

        if let Err(e) = rdev::listen(callback) {
            eprintln!("Hotkey listener error: {:?}", e);
        }
    });
}

fn key_display_name(key: Key) -> String {
    match key {
        Key::KeyA => "A",   Key::KeyB => "B",   Key::KeyC => "C",   Key::KeyD => "D",
        Key::KeyE => "E",   Key::KeyF => "F",   Key::KeyG => "G",   Key::KeyH => "H",
        Key::KeyI => "I",   Key::KeyJ => "J",   Key::KeyK => "K",   Key::KeyL => "L",
        Key::KeyM => "M",   Key::KeyN => "N",   Key::KeyO => "O",   Key::KeyP => "P",
        Key::KeyQ => "Q",   Key::KeyR => "R",   Key::KeyS => "S",   Key::KeyT => "T",
        Key::KeyU => "U",   Key::KeyV => "V",   Key::KeyW => "W",   Key::KeyX => "X",
        Key::KeyY => "Y",   Key::KeyZ => "Z",
        Key::Num0 => "0",   Key::Num1 => "1",   Key::Num2 => "2",   Key::Num3 => "3",
        Key::Num4 => "4",   Key::Num5 => "5",   Key::Num6 => "6",   Key::Num7 => "7",
        Key::Num8 => "8",   Key::Num9 => "9",
        Key::F1  => "F1",   Key::F2  => "F2",   Key::F3  => "F3",   Key::F4  => "F4",
        Key::F5  => "F5",   Key::F6  => "F6",   Key::F7  => "F7",   Key::F8  => "F8",
        Key::F9  => "F9",   Key::F10 => "F10",  Key::F11 => "F11",  Key::F12 => "F12",
        Key::Space => "Space",
        Key::Return => "Enter",
        Key::Escape => "Esc",
        Key::Tab => "Tab",
        Key::Backspace => "Backspace",
        Key::Delete => "Del",
        Key::Insert => "Ins",
        Key::Home => "Home",
        Key::End => "End",
        Key::PageUp => "PgUp",
        Key::PageDown => "PgDn",
        Key::UpArrow => "Up",
        Key::DownArrow => "Down",
        Key::LeftArrow => "Left",
        Key::RightArrow => "Right",
        Key::PrintScreen => "PrtSc",
        Key::ScrollLock => "ScrLk",
        Key::Pause => "Pause",
        Key::NumLock => "NumLk",
        Key::BackQuote => "`",
        Key::Minus => "-",
        Key::Equal => "=",
        Key::LeftBracket => "[",
        Key::RightBracket => "]",
        Key::BackSlash => "\\",
        Key::SemiColon => ";",
        Key::Quote => "'",
        Key::Comma => ",",
        Key::Dot => ".",
        Key::Slash => "/",
        Key::Kp0 => "Num0",  Key::Kp1 => "Num1",  Key::Kp2 => "Num2",  Key::Kp3 => "Num3",
        Key::Kp4 => "Num4",  Key::Kp5 => "Num5",  Key::Kp6 => "Num6",  Key::Kp7 => "Num7",
        Key::Kp8 => "Num8",  Key::Kp9 => "Num9",
        Key::KpMinus => "Num-",  Key::KpPlus => "Num+",
        Key::KpMultiply => "Num*",  Key::KpDivide => "Num/",
        Key::KpDelete => "Num.",
        _ => return format!("{:?}", key),
    }.to_string()
}

fn key_to_str(key: Key) -> String {
    match key {
        Key::Unknown(n) => return format!("Unknown:{}", n),
        Key::Alt => "Alt",
        Key::AltGr => "AltGr",
        Key::Backspace => "Backspace",
        Key::CapsLock => "CapsLock",
        Key::ControlLeft => "ControlLeft",
        Key::ControlRight => "ControlRight",
        Key::Delete => "Delete",
        Key::DownArrow => "DownArrow",
        Key::End => "End",
        Key::Escape => "Escape",
        Key::F1  => "F1",   Key::F2  => "F2",   Key::F3  => "F3",   Key::F4  => "F4",
        Key::F5  => "F5",   Key::F6  => "F6",   Key::F7  => "F7",   Key::F8  => "F8",
        Key::F9  => "F9",   Key::F10 => "F10",  Key::F11 => "F11",  Key::F12 => "F12",
        Key::Home => "Home",
        Key::LeftArrow => "LeftArrow",
        Key::MetaLeft => "MetaLeft",
        Key::MetaRight => "MetaRight",
        Key::PageDown => "PageDown",
        Key::PageUp => "PageUp",
        Key::Return => "Return",
        Key::RightArrow => "RightArrow",
        Key::ShiftLeft => "ShiftLeft",
        Key::ShiftRight => "ShiftRight",
        Key::Space => "Space",
        Key::Tab => "Tab",
        Key::UpArrow => "UpArrow",
        Key::PrintScreen => "PrintScreen",
        Key::ScrollLock => "ScrollLock",
        Key::Pause => "Pause",
        Key::NumLock => "NumLock",
        Key::BackQuote => "BackQuote",
        Key::Num0 => "Num0",  Key::Num1 => "Num1",  Key::Num2 => "Num2",  Key::Num3 => "Num3",
        Key::Num4 => "Num4",  Key::Num5 => "Num5",  Key::Num6 => "Num6",  Key::Num7 => "Num7",
        Key::Num8 => "Num8",  Key::Num9 => "Num9",
        Key::Minus => "Minus",
        Key::Equal => "Equal",
        Key::KeyA => "KeyA",  Key::KeyB => "KeyB",  Key::KeyC => "KeyC",  Key::KeyD => "KeyD",
        Key::KeyE => "KeyE",  Key::KeyF => "KeyF",  Key::KeyG => "KeyG",  Key::KeyH => "KeyH",
        Key::KeyI => "KeyI",  Key::KeyJ => "KeyJ",  Key::KeyK => "KeyK",  Key::KeyL => "KeyL",
        Key::KeyM => "KeyM",  Key::KeyN => "KeyN",  Key::KeyO => "KeyO",  Key::KeyP => "KeyP",
        Key::KeyQ => "KeyQ",  Key::KeyR => "KeyR",  Key::KeyS => "KeyS",  Key::KeyT => "KeyT",
        Key::KeyU => "KeyU",  Key::KeyV => "KeyV",  Key::KeyW => "KeyW",  Key::KeyX => "KeyX",
        Key::KeyY => "KeyY",  Key::KeyZ => "KeyZ",
        Key::LeftBracket => "LeftBracket",
        Key::RightBracket => "RightBracket",
        Key::BackSlash => "BackSlash",
        Key::SemiColon => "SemiColon",
        Key::Quote => "Quote",
        Key::Comma => "Comma",
        Key::Dot => "Dot",
        Key::Slash => "Slash",
        Key::Insert => "Insert",
        Key::KpReturn => "KpReturn",
        Key::Kp0 => "Kp0",  Key::Kp1 => "Kp1",  Key::Kp2 => "Kp2",  Key::Kp3 => "Kp3",
        Key::Kp4 => "Kp4",  Key::Kp5 => "Kp5",  Key::Kp6 => "Kp6",  Key::Kp7 => "Kp7",
        Key::Kp8 => "Kp8",  Key::Kp9 => "Kp9",
        Key::KpMinus => "KpMinus",
        Key::KpPlus => "KpPlus",
        Key::KpMultiply => "KpMultiply",
        Key::KpDivide => "KpDivide",
        Key::KpDelete => "KpDelete",
        Key::Function => "Function",
        Key::IntlBackslash => "IntlBackslash",
    }.to_string()
}

fn str_to_key(s: &str) -> Option<Key> {
    if let Some(rest) = s.strip_prefix("Unknown:") {
        return rest.parse::<u32>().ok().map(Key::Unknown);
    }
    Some(match s {
        "Alt" => Key::Alt,
        "AltGr" => Key::AltGr,
        "Backspace" => Key::Backspace,
        "CapsLock" => Key::CapsLock,
        "ControlLeft" => Key::ControlLeft,
        "ControlRight" => Key::ControlRight,
        "Delete" => Key::Delete,
        "DownArrow" => Key::DownArrow,
        "End" => Key::End,
        "Escape" => Key::Escape,
        "F1"  => Key::F1,   "F2"  => Key::F2,   "F3"  => Key::F3,   "F4"  => Key::F4,
        "F5"  => Key::F5,   "F6"  => Key::F6,   "F7"  => Key::F7,   "F8"  => Key::F8,
        "F9"  => Key::F9,   "F10" => Key::F10,  "F11" => Key::F11,  "F12" => Key::F12,
        "Home" => Key::Home,
        "LeftArrow" => Key::LeftArrow,
        "MetaLeft" => Key::MetaLeft,
        "MetaRight" => Key::MetaRight,
        "PageDown" => Key::PageDown,
        "PageUp" => Key::PageUp,
        "Return" => Key::Return,
        "RightArrow" => Key::RightArrow,
        "ShiftLeft" => Key::ShiftLeft,
        "ShiftRight" => Key::ShiftRight,
        "Space" => Key::Space,
        "Tab" => Key::Tab,
        "UpArrow" => Key::UpArrow,
        "PrintScreen" => Key::PrintScreen,
        "ScrollLock" => Key::ScrollLock,
        "Pause" => Key::Pause,
        "NumLock" => Key::NumLock,
        "BackQuote" => Key::BackQuote,
        "Num0" => Key::Num0,  "Num1" => Key::Num1,  "Num2" => Key::Num2,  "Num3" => Key::Num3,
        "Num4" => Key::Num4,  "Num5" => Key::Num5,  "Num6" => Key::Num6,  "Num7" => Key::Num7,
        "Num8" => Key::Num8,  "Num9" => Key::Num9,
        "Minus" => Key::Minus,
        "Equal" => Key::Equal,
        "KeyA" => Key::KeyA,  "KeyB" => Key::KeyB,  "KeyC" => Key::KeyC,  "KeyD" => Key::KeyD,
        "KeyE" => Key::KeyE,  "KeyF" => Key::KeyF,  "KeyG" => Key::KeyG,  "KeyH" => Key::KeyH,
        "KeyI" => Key::KeyI,  "KeyJ" => Key::KeyJ,  "KeyK" => Key::KeyK,  "KeyL" => Key::KeyL,
        "KeyM" => Key::KeyM,  "KeyN" => Key::KeyN,  "KeyO" => Key::KeyO,  "KeyP" => Key::KeyP,
        "KeyQ" => Key::KeyQ,  "KeyR" => Key::KeyR,  "KeyS" => Key::KeyS,  "KeyT" => Key::KeyT,
        "KeyU" => Key::KeyU,  "KeyV" => Key::KeyV,  "KeyW" => Key::KeyW,  "KeyX" => Key::KeyX,
        "KeyY" => Key::KeyY,  "KeyZ" => Key::KeyZ,
        "LeftBracket" => Key::LeftBracket,
        "RightBracket" => Key::RightBracket,
        "BackSlash" => Key::BackSlash,
        "SemiColon" => Key::SemiColon,
        "Quote" => Key::Quote,
        "Comma" => Key::Comma,
        "Dot" => Key::Dot,
        "Slash" => Key::Slash,
        "Insert" => Key::Insert,
        "KpReturn" => Key::KpReturn,
        "Kp0" => Key::Kp0,  "Kp1" => Key::Kp1,  "Kp2" => Key::Kp2,  "Kp3" => Key::Kp3,
        "Kp4" => Key::Kp4,  "Kp5" => Key::Kp5,  "Kp6" => Key::Kp6,  "Kp7" => Key::Kp7,
        "Kp8" => Key::Kp8,  "Kp9" => Key::Kp9,
        "KpMinus" => Key::KpMinus,
        "KpPlus" => Key::KpPlus,
        "KpMultiply" => Key::KpMultiply,
        "KpDivide" => Key::KpDivide,
        "KpDelete" => Key::KpDelete,
        "Function" => Key::Function,
        "IntlBackslash" => Key::IntlBackslash,
        _ => return None,
    })
}
