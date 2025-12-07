#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod executor;
mod i18n;
mod logger;
mod model;

use app::App;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let icon = load_icon();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_resizable(false)
            .with_title("WForge")
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "WForge",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

fn load_icon() -> egui::IconData {
    let icon_bytes = include_bytes!("../assets/WForge.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}
