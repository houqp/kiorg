use crate::config::colors::AppColors;

pub const HEADER_FONT_SIZE: f32 = 12.0;
pub const HEADER_ROW_HEIGHT: f32 = HEADER_FONT_SIZE + 4.0;

pub fn section_title_text(text: &str, colors: &AppColors) -> egui::RichText {
    egui::RichText::new(text)
        .color(colors.gray)
        .font(egui::FontId::proportional(HEADER_FONT_SIZE))
}
