use egui::{Ui, Separator};

pub const SEPARATOR_PADDING: f32 = 1.0;

pub fn draw_vertical_separator(ui: &mut Ui) { // Add padding argument
    ui.vertical(|ui| {
        ui.set_min_width(SEPARATOR_PADDING); // Use padding argument
        ui.set_max_width(SEPARATOR_PADDING); // Use padding argument
        ui.add(Separator::default().vertical());
    });
}