use eframe::egui;

use crate::app::App;
use crate::hotkey::RecordingTarget;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(4.0);
    ui.strong("Hotkeys");
    ui.separator();
    ui.label("Click a key button then press any key (with Ctrl/Shift/Alt if desired) to reassign it.");
    ui.add_space(6.0);

    let (toggle_name, capture_name) = {
        let cfg = app.hotkey_config.lock().unwrap();
        (cfg.toggle_key.name(), cfg.capture_key.name())  // both return String now
    };

    egui::Grid::new("hotkey_grid")
        .num_columns(2)
        .spacing([10.0, 10.0])
        .show(ui, |ui| {
            ui.label("Toggle Start/Stop:");
            key_record_button(ui, "toggle_btn", &toggle_name,
                app.recording_hotkey == Some(RecordingTarget::Toggle),
                || {
                    app.recording_hotkey = Some(RecordingTarget::Toggle);
                    app.hotkey_config.lock().unwrap().recording = Some(RecordingTarget::Toggle);
                });
            ui.end_row();

            ui.label("Capture Location:");
            key_record_button(ui, "capture_btn", &capture_name,
                app.recording_hotkey == Some(RecordingTarget::Capture),
                || {
                    app.recording_hotkey = Some(RecordingTarget::Capture);
                    app.hotkey_config.lock().unwrap().recording = Some(RecordingTarget::Capture);
                });
            ui.end_row();
        });

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

fn key_record_button(
    ui: &mut egui::Ui,
    _id: &str,
    current_name: &str,
    is_recording: bool,
    mut on_click: impl FnMut(),
) {
    let (label, fill) = if is_recording {
        ("Press any key…".to_string(), egui::Color32::from_rgb(240, 160, 0))
    } else {
        (current_name.to_string(), ui.visuals().widgets.inactive.bg_fill)
    };

    let btn = egui::Button::new(
        egui::RichText::new(&label)
            .monospace()
            .color(if is_recording { egui::Color32::BLACK } else { ui.visuals().text_color() }),
    )
    .fill(fill)
    .min_size(egui::vec2(110.0, 28.0));

    if ui.add(btn).on_hover_text("Click to reassign").clicked() && !is_recording {
        on_click();
    }
}
