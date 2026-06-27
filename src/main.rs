mod app;
mod clicker;
mod hotkey;
mod profile;
mod ui;

use app::App;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("AwesomeClicker")
            .with_inner_size([560.0, 480.0])
            .with_min_inner_size([420.0, 360.0]),
        ..Default::default()
    };

    eframe::run_native(
        "AwesomeClicker",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
