use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use enigo::{Button, Coordinate, Direction, Enigo, Mouse, Settings};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ClickButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ClickType {
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ClickMode {
    Basic,
    Sequence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceItem {
    pub x: i32,
    pub y: i32,
    pub button: ClickButton,
    pub delay_ms: u64,
}

pub struct ClickerConfig {
    pub interval_ms: u64,
    pub jitter_ms: u64,
    pub button: ClickButton,
    pub click_type: ClickType,
    pub max_clicks: Option<u64>,
    pub mode: ClickMode,
    pub sequence: Vec<SequenceItem>,
    pub seq_loop: bool,
    pub seq_repeat_count: u64,
}

pub struct ClickerHandle {
    pub stop_flag: Arc<AtomicBool>,
    pub click_count: Arc<AtomicU64>,
    thread: Option<JoinHandle<()>>,
}

impl ClickerHandle {
    pub fn start(config: ClickerConfig) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let click_count = Arc::new(AtomicU64::new(0));
        let stop_clone = stop_flag.clone();
        let count_clone = click_count.clone();
        let thread = thread::spawn(move || {
            clicker_loop(config, stop_clone, count_clone);
        });
        ClickerHandle { stop_flag, click_count, thread: Some(thread) }
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    pub fn is_done(&self) -> bool {
        self.stop_flag.load(Ordering::Relaxed)
    }
}

impl Drop for ClickerHandle {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn interruptible_sleep(ms: u64, stop_flag: &AtomicBool) -> bool {
    let mut remaining = ms;
    while remaining > 0 {
        if stop_flag.load(Ordering::Relaxed) {
            return true;
        }
        let chunk = remaining.min(50);
        thread::sleep(Duration::from_millis(chunk));
        remaining -= chunk;
    }
    false
}

fn enigo_button(b: ClickButton) -> Button {
    match b {
        ClickButton::Left => Button::Left,
        ClickButton::Right => Button::Right,
        ClickButton::Middle => Button::Middle,
    }
}

fn do_click(enigo: &mut Enigo, button: ClickButton, click_type: ClickType) {
    let btn = enigo_button(button);
    let _ = enigo.button(btn, Direction::Click);
    if click_type == ClickType::Double {
        thread::sleep(Duration::from_millis(40));
        let _ = enigo.button(btn, Direction::Click);
    }
}

fn clicker_loop(config: ClickerConfig, stop_flag: Arc<AtomicBool>, click_count: Arc<AtomicU64>) {
    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to create Enigo: {:?}", e);
            return;
        }
    };
    let mut rng = rand::thread_rng();

    match config.mode {
        ClickMode::Basic => {
            loop {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                do_click(&mut enigo, config.button, config.click_type);
                let count = click_count.fetch_add(1, Ordering::Relaxed) + 1;
                if config.max_clicks.map_or(false, |m| count >= m) {
                    stop_flag.store(true, Ordering::Relaxed);
                    break;
                }
                let jitter: i64 = if config.jitter_ms > 0 {
                    rng.gen_range(-(config.jitter_ms as i64)..=(config.jitter_ms as i64))
                } else {
                    0
                };
                let sleep_ms = (config.interval_ms as i64 + jitter).max(1) as u64;
                if interruptible_sleep(sleep_ms, &stop_flag) {
                    break;
                }
            }
        }
        ClickMode::Sequence => {
            if config.sequence.is_empty() {
                return;
            }
            let mut repeats_done = 0u64;
            'outer: loop {
                for item in &config.sequence {
                    if stop_flag.load(Ordering::Relaxed) {
                        break 'outer;
                    }
                    if item.delay_ms > 0 && interruptible_sleep(item.delay_ms, &stop_flag) {
                        break 'outer;
                    }
                    let _ = enigo.move_mouse(item.x, item.y, Coordinate::Abs);
                    do_click(&mut enigo, item.button, config.click_type);
                    let count = click_count.fetch_add(1, Ordering::Relaxed) + 1;
                    if config.max_clicks.map_or(false, |m| count >= m) {
                        stop_flag.store(true, Ordering::Relaxed);
                        break 'outer;
                    }
                }
                if !config.seq_loop {
                    stop_flag.store(true, Ordering::Relaxed);
                    break;
                }
                repeats_done += 1;
                if config.seq_repeat_count > 0 && repeats_done >= config.seq_repeat_count {
                    stop_flag.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }
    }
}
