mod app;
mod clicker;
mod hotkey;
mod profile;
mod ui;

use app::App;

fn main() -> eframe::Result<()> {
    // On Linux, prefer X11 over native Wayland when an X display is available.
    // rdev global hotkeys and ViewportCommand::WindowLevel both require X11 —
    // they silently fail under native Wayland compositors. XWayland is present
    // on all major desktop environments (GNOME, KDE, Pop!_OS) so this is safe.
    #[cfg(target_os = "linux")]
    if std::env::var("DISPLAY").is_ok() {
        std::env::remove_var("WAYLAND_DISPLAY");
    }

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
