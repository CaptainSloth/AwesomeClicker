use eframe::egui;

use crate::app::App;
use crate::clicker::{ClickButton, ClickType};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let is_running = app.is_running();

    egui::Grid::new("basic_grid")
        .num_columns(2)
        .spacing([10.0, 8.0])
        .show(ui, |ui| {
            ui.label("Speed:");
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut app.cps, 0.1f64..=50.0)
                        .logarithmic(true)
                        .suffix(" CPS"),
                );
                let interval_ms = (1000.0 / app.cps).round() as u64;
                ui.weak(format!("[{} ms interval]", interval_ms));
            });
            ui.end_row();

            ui.label("Mouse Button:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut app.button, ClickButton::Left, "Left");
                ui.radio_value(&mut app.button, ClickButton::Right, "Right");
                ui.radio_value(&mut app.button, ClickButton::Middle, "Middle");
            });
            ui.end_row();

            ui.label("Click Type:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut app.click_type, ClickType::Single, "Single");
                ui.radio_value(&mut app.click_type, ClickType::Double, "Double");
            });
            ui.end_row();

            ui.label("Jitter (±ms):");
            ui.add(egui::Slider::new(&mut app.jitter_ms, 0u64..=500).suffix(" ms"));
            ui.end_row();

            ui.label("Click Limit:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut app.use_limit, false, "Unlimited");
                ui.radio_value(&mut app.use_limit, true, "Stop after");
                if app.use_limit {
                    ui.add(
                        egui::DragValue::new(&mut app.max_clicks)
                            .range(1u64..=10_000_000),
                    );
                    ui.label("clicks");
                }
            });
            ui.end_row();
        });

    ui.separator();
    show_start_stop(app, ui, is_running);
}

pub fn show_start_stop(app: &mut App, ui: &mut egui::Ui, is_running: bool) {
    ui.horizontal(|ui| {
        let (label, color) = if is_running {
            ("■  Stop  (F8)", egui::Color32::from_rgb(200, 60, 60))
        } else {
            ("▶  Start  (F8)", egui::Color32::from_rgb(50, 150, 70))
        };

        let btn = egui::Button::new(egui::RichText::new(label).color(egui::Color32::WHITE))
            .fill(color)
            .min_size(egui::vec2(140.0, 34.0));

        if ui.add(btn).clicked() {
            app.toggle_clicking();
        }

        ui.add_space(10.0);

        if is_running {
            ui.colored_label(
                egui::Color32::from_rgb(80, 200, 80),
                format!("● Running  —  {} clicks", app.click_count_display),
            );
        } else {
            ui.label(&app.status);
        }
    });
}
