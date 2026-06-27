use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc, Mutex};

use eframe::egui;

use crate::clicker::{ClickButton, ClickMode, ClickType, ClickerConfig, ClickerHandle, SequenceItem};
use crate::hotkey::{self, HotkeyConfig, HotkeyEvent};
use crate::profile::Profile;
use crate::ui;

#[derive(PartialEq)]
pub enum Tab {
    Basic,
    Advanced,
    Settings,
}

pub struct App {
    // Basic settings
    pub cps: f64,
    pub jitter_ms: u64,
    pub button: ClickButton,
    pub click_type: ClickType,
    pub use_limit: bool,
    pub max_clicks: u64,

    // Tab / mode
    pub active_tab: Tab,
    pub mode: ClickMode,

    // Sequence
    pub sequence: Vec<SequenceItem>,
    pub selected_idx: Option<usize>,
    pub capture_next: bool,
    pub seq_loop: bool,
    pub seq_repeat_count: u64,

    // Profile
    pub profile_name: String,
    pub profiles_dir: PathBuf,
    pub available_profiles: Vec<PathBuf>,

    // Runtime state
    pub clicker: Option<ClickerHandle>,
    pub status: String,
    pub click_count_display: u64,

    hotkey_rx: mpsc::Receiver<HotkeyEvent>,
    pub hotkey_config: Arc<Mutex<HotkeyConfig>>,
    pub always_on_top: bool,
    pub show_wayland_warning: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let _ = cc;

        let (hotkey_tx, hotkey_rx) = mpsc::channel();
        let hotkey_config = Arc::new(Mutex::new(HotkeyConfig::default()));
        hotkey::start_listener(hotkey_tx, hotkey_config.clone());

        let profiles_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("awesomeclicker");

        let available_profiles = Profile::list(&profiles_dir);

        let show_wayland_warning = cfg!(target_os = "linux")
            && std::env::var("WAYLAND_DISPLAY").is_ok()
            && std::env::var("DISPLAY").is_err();

        App {
            cps: 10.0,
            jitter_ms: 0,
            button: ClickButton::Left,
            click_type: ClickType::Single,
            use_limit: false,
            max_clicks: 100,
            active_tab: Tab::Basic,
            mode: ClickMode::Basic,
            sequence: Vec::new(),
            selected_idx: None,
            capture_next: false,
            seq_loop: false,
            seq_repeat_count: 0,
            profile_name: "Default".into(),
            profiles_dir,
            available_profiles,
            clicker: None,
            status: "Idle".into(),
            click_count_display: 0,
            hotkey_rx,
            hotkey_config,
            always_on_top: false,
            show_wayland_warning,
        }
    }

    pub fn is_running(&self) -> bool {
        self.clicker.is_some()
    }

    pub fn toggle_clicking(&mut self) {
        if self.is_running() {
            self.stop_clicking();
        } else {
            self.start_clicking();
        }
    }

    fn start_clicking(&mut self) {
        let interval_ms = (1000.0 / self.cps).round() as u64;
        let config = ClickerConfig {
            interval_ms,
            jitter_ms: self.jitter_ms,
            button: self.button,
            click_type: self.click_type,
            max_clicks: if self.use_limit { Some(self.max_clicks) } else { None },
            mode: self.mode,
            sequence: self.sequence.clone(),
            seq_loop: self.seq_loop,
            seq_repeat_count: self.seq_repeat_count,
        };
        self.clicker = Some(ClickerHandle::start(config));
        self.click_count_display = 0;
        self.status = "Running…".into();
    }

    fn stop_clicking(&mut self) {
        if let Some(ref h) = self.clicker {
            h.stop();
            self.click_count_display = h.click_count.load(Ordering::Relaxed);
        }
        self.clicker = None;
        self.status = format!("Stopped — {} clicks", self.click_count_display);
    }

    fn poll_hotkeys(&mut self) {
        while let Ok(event) = self.hotkey_rx.try_recv() {
            match event {
                HotkeyEvent::Toggle => self.toggle_clicking(),
                HotkeyEvent::Capture { x, y } => {
                    if self.capture_next {
                        self.sequence.push(SequenceItem {
                            x,
                            y,
                            button: self.button,
                            delay_ms: 100,
                        });
                        self.selected_idx = Some(self.sequence.len() - 1);
                        self.capture_next = false;
                        self.status = format!("Captured ({}, {})", x, y);
                    }
                }
            }
        }
    }

    fn check_clicker_done(&mut self) {
        let done = self.clicker.as_ref().map_or(false, |h| h.is_done());
        if done {
            let count = self
                .clicker
                .as_ref()
                .map_or(0, |h| h.click_count.load(Ordering::Relaxed));
            self.click_count_display = count;
            self.clicker = None;
            self.status = format!("Done — {} clicks", count);
        } else if let Some(ref h) = self.clicker {
            self.click_count_display = h.click_count.load(Ordering::Relaxed);
        }
    }

    pub fn save_profile(&mut self) {
        let profile = Profile {
            name: self.profile_name.clone(),
            cps: self.cps,
            jitter_ms: self.jitter_ms,
            button: self.button,
            click_type: self.click_type,
            use_limit: self.use_limit,
            max_clicks: self.max_clicks,
            mode: self.mode,
            sequence: self.sequence.clone(),
            seq_loop: self.seq_loop,
            seq_repeat_count: self.seq_repeat_count,
        };
        match profile.save(&self.profiles_dir) {
            Ok(()) => {
                self.status = format!("Saved profile '{}'", self.profile_name);
                self.available_profiles = Profile::list(&self.profiles_dir);
            }
            Err(e) => {
                self.status = format!("Save error: {}", e);
            }
        }
    }

    pub fn load_profile(&mut self, path: &std::path::Path) {
        match Profile::load(path) {
            Ok(p) => {
                self.profile_name = p.name.clone();
                self.cps = p.cps;
                self.jitter_ms = p.jitter_ms;
                self.button = p.button;
                self.click_type = p.click_type;
                self.use_limit = p.use_limit;
                self.max_clicks = p.max_clicks;
                self.mode = p.mode;
                self.sequence = p.sequence;
                self.seq_loop = p.seq_loop;
                self.seq_repeat_count = p.seq_repeat_count;
                self.status = format!("Loaded '{}'", p.name);
            }
            Err(e) => {
                self.status = format!("Load error: {}", e);
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_hotkeys();
        self.check_clicker_done();

        if self.is_running() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        ui::show(self, ctx);
    }
}
