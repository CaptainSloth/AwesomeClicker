use eframe::egui;

use crate::app::App;
use crate::hotkey::HotkeyKey;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(4.0);

    ui.strong("Hotkeys");
    ui.separator();

    // Read both keys up front so we can pass the "other" key as the blocked value.
    let (mut toggle_key, mut capture_key) = {
        let cfg = app.hotkey_config.lock().unwrap();
        (cfg.toggle_key, cfg.capture_key)
    };

    egui::Grid::new("hotkey_grid")
        .num_columns(2)
        .spacing([10.0, 8.0])
        .show(ui, |ui| {
            ui.label("Toggle Start/Stop:");
            hotkey_picker(ui, "toggle_key", &mut toggle_key, Some(capture_key));
            ui.end_row();

            ui.label("Capture Location (Advanced):");
            hotkey_picker(ui, "capture_key", &mut capture_key, Some(toggle_key));
            ui.end_row();
        });

    // Write back any changes.
    let mut cfg = app.hotkey_config.lock().unwrap();
    cfg.toggle_key = toggle_key;
    cfg.capture_key = capture_key;

    ui.add_space(12.0);
    ui.strong("Window");
    ui.separator();

    let resp = ui.checkbox(&mut app.always_on_top, "Always on top");
    if resp.changed() {
        let level = if app.always_on_top {
            egui::WindowLevel::AlwaysOnTop
        } else {
            egui::WindowLevel::Normal
        };
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
    }
}

fn hotkey_picker(ui: &mut egui::Ui, id: &str, key: &mut HotkeyKey, blocked: Option<HotkeyKey>) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(key.name())
        .width(60.0)
        .show_ui(ui, |ui| {
            for &k in HotkeyKey::ALL {
                let is_blocked = blocked == Some(k);
                ui.add_enabled_ui(!is_blocked, |ui| {
                    let label = if is_blocked {
                        format!("{} (in use)", k.name())
                    } else {
                        k.name().to_string()
                    };
                    ui.selectable_value(key, k, label);
                });
            }
        });
}
