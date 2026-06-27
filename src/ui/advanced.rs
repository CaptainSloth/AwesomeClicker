use eframe::egui;

use crate::app::App;
use crate::clicker::{ClickButton, ClickMode, SequenceItem};
use crate::ui::basic::show_start_stop;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let is_running = app.is_running();

    ui.horizontal(|ui| {
        ui.label("Click Mode:");
        ui.radio_value(&mut app.mode, ClickMode::Basic, "Basic (current position)");
        ui.radio_value(&mut app.mode, ClickMode::Sequence, "Sequence (list of positions)");
    });

    ui.separator();

    if app.mode == ClickMode::Basic {
        ui.label("Clicks happen at the current mouse position. Use the Basic tab for per-click settings.");
        ui.add_space(8.0);
        show_start_stop(app, ui, is_running);
        return;
    }

    // Toolbar
    ui.horizontal(|ui| {
        if ui.button("＋ Add Row").clicked() {
            app.sequence.push(SequenceItem {
                x: 0,
                y: 0,
                button: app.button,
                delay_ms: 100,
            });
            app.selected_idx = Some(app.sequence.len() - 1);
        }

        let has_sel = app.selected_idx.is_some();
        if ui.add_enabled(has_sel, egui::Button::new("✕ Delete")).clicked() {
            if let Some(idx) = app.selected_idx {
                app.sequence.remove(idx);
                app.selected_idx = if app.sequence.is_empty() {
                    None
                } else {
                    Some(idx.min(app.sequence.len() - 1))
                };
            }
        }

        let can_up = app.selected_idx.map_or(false, |i| i > 0);
        if ui.add_enabled(can_up, egui::Button::new("↑")).clicked() {
            if let Some(idx) = app.selected_idx {
                app.sequence.swap(idx, idx - 1);
                app.selected_idx = Some(idx - 1);
            }
        }

        let can_down = app
            .selected_idx
            .map_or(false, |i| i + 1 < app.sequence.len());
        if ui.add_enabled(can_down, egui::Button::new("↓")).clicked() {
            if let Some(idx) = app.selected_idx {
                app.sequence.swap(idx, idx + 1);
                app.selected_idx = Some(idx + 1);
            }
        }

        ui.separator();

        let (capture_label, capture_color) = if app.capture_next {
            (
                "⊙ Waiting… hover & press F6",
                egui::Color32::from_rgb(240, 160, 0),
            )
        } else {
            ("⊙ Capture next with F6", egui::Color32::TRANSPARENT)
        };
        if ui
            .add(egui::Button::new(capture_label).fill(capture_color))
            .clicked()
        {
            app.capture_next = !app.capture_next;
        }
    });

    ui.add_space(4.0);

    // Sequence table
    egui::ScrollArea::vertical()
        .max_height(180.0)
        .id_salt("seq_scroll")
        .show(ui, |ui| {
            egui::Grid::new("seq_grid")
                .num_columns(6)
                .striped(true)
                .spacing([6.0, 4.0])
                .show(ui, |ui| {
                    ui.strong("#");
                    ui.strong("X");
                    ui.strong("Y");
                    ui.strong("Button");
                    ui.strong("Delay (ms)");
                    ui.strong("Sel");
                    ui.end_row();

                    let len = app.sequence.len();
                    for i in 0..len {
                        ui.label(format!("{}", i + 1));
                        ui.add(
                            egui::DragValue::new(&mut app.sequence[i].x)
                                .speed(1)
                                .range(0i32..=9999),
                        );
                        ui.add(
                            egui::DragValue::new(&mut app.sequence[i].y)
                                .speed(1)
                                .range(0i32..=9999),
                        );

                        // Button selector per-row (use ComboBox without app borrow in closure)
                        let current_btn = app.sequence[i].button;
                        let btn_label = match current_btn {
                            ClickButton::Left => "Left",
                            ClickButton::Right => "Right",
                            ClickButton::Middle => "Middle",
                        };
                        egui::ComboBox::from_id_salt(format!("row_btn_{}", i))
                            .selected_text(btn_label)
                            .width(70.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut app.sequence[i].button,
                                    ClickButton::Left,
                                    "Left",
                                );
                                ui.selectable_value(
                                    &mut app.sequence[i].button,
                                    ClickButton::Right,
                                    "Right",
                                );
                                ui.selectable_value(
                                    &mut app.sequence[i].button,
                                    ClickButton::Middle,
                                    "Middle",
                                );
                            });

                        ui.add(
                            egui::DragValue::new(&mut app.sequence[i].delay_ms)
                                .range(0u64..=60_000),
                        );

                        let selected = app.selected_idx == Some(i);
                        if ui.selectable_label(selected, "●").clicked() {
                            app.selected_idx = Some(i);
                        }
                        ui.end_row();
                    }
                });
        });

    ui.separator();

    // Loop settings
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.seq_loop, "Loop sequence");
        if app.seq_loop {
            ui.label("Repeat:");
            let forever = app.seq_repeat_count == 0;
            if ui.radio(forever, "Forever").clicked() {
                app.seq_repeat_count = 0;
            }
            if ui.radio(!forever, "N times:").clicked() && forever {
                app.seq_repeat_count = 5;
            }
            if !forever {
                ui.add(egui::DragValue::new(&mut app.seq_repeat_count).range(1u64..=10_000));
            }
        }
    });

    ui.separator();
    show_start_stop(app, ui, is_running);
}
