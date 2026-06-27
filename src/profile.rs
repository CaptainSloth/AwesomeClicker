use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::clicker::{ClickButton, ClickMode, ClickType, SequenceItem};

#[derive(Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub cps: f64,
    pub jitter_ms: u64,
    pub button: ClickButton,
    pub click_type: ClickType,
    pub use_limit: bool,
    pub max_clicks: u64,
    pub mode: ClickMode,
    pub sequence: Vec<SequenceItem>,
    pub seq_loop: bool,
    pub seq_repeat_count: u64,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "Default".into(),
            cps: 10.0,
            jitter_ms: 0,
            button: ClickButton::Left,
            click_type: ClickType::Single,
            use_limit: false,
            max_clicks: 100,
            mode: ClickMode::Basic,
            sequence: Vec::new(),
            seq_loop: false,
            seq_repeat_count: 0,
        }
    }
}

impl Profile {
    pub fn save(&self, dir: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dir)?;
        let path = dir.join(format!("{}.json", sanitize_name(&self.name)));
        let json = serde_json::to_string_pretty(self).unwrap();
        fs::write(path, json)
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        let data = fs::read_to_string(path)?;
        serde_json::from_str(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn list(dir: &Path) -> Vec<PathBuf> {
        let Ok(entries) = fs::read_dir(dir) else {
            return vec![];
        };
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map_or(false, |e| e == "json"))
            .collect();
        paths.sort();
        paths
    }
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}
