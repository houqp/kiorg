use eframe::egui;

mod app;
mod config;
mod models;
mod ui;
mod utils;

use app::Kiorg;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Kiorg",
        options,
        Box::new(|cc| Ok(Box::new(Kiorg::new(cc))))
    )
}
