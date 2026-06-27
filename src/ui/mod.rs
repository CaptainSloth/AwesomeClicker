pub mod advanced;
pub mod basic;
pub mod settings;

use eframe::egui;

use crate::app::{App, Tab};

pub fn show(app: &mut App, ctx: &egui::Context) {
    if app.show_wayland_warning {
        egui::TopBottomPanel::top("wayland_warning").show(ctx, |ui| {
            ui.colored_label(
                egui::Color32::from_rgb(240, 160, 0),
                "⚠  Wayland detected: global hotkeys may not work. \
                 Run the app under XWayland or with DISPLAY set.",
            );
        });
    }

    egui::TopBottomPanel::top("profile_bar").show(ctx, |ui| {
        show_profile_bar(app, ui);
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut app.active_tab, Tab::Basic, "Basic");
            ui.selectable_value(&mut app.active_tab, Tab::Advanced, "Advanced");
            ui.selectable_value(&mut app.active_tab, Tab::Settings, "Settings");
        });
        ui.separator();

        match app.active_tab {
            Tab::Basic => basic::show(app, ui),
            Tab::Advanced => advanced::show(app, ui),
            Tab::Settings => settings::show(app, ui),
        }
    });
}

fn show_profile_bar(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Profile:");
        ui.add(egui::TextEdit::singleline(&mut app.profile_name).desired_width(140.0));

        if ui.button("💾 Save").clicked() {
            app.save_profile();
        }

        let profiles = app.available_profiles.clone();
        if !profiles.is_empty() {
            let mut load_path: Option<std::path::PathBuf> = None;
            egui::ComboBox::from_id_salt("profile_load")
                .selected_text("📂 Load…")
                .show_ui(ui, |ui| {
                    for path in &profiles {
                        let name = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("?");
                        if ui.selectable_label(false, name).clicked() {
                            load_path = Some(path.clone());
                        }
                    }
                });
            if let Some(path) = load_path {
                app.load_profile(&path);
            }
        }
    });
}
