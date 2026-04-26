use crate::config::colors::AppColors;
use egui::{Color32, Shadow, Vec2};

/// Creates a centered, frameless popup window with a standard shadow and background color.
/// This is used by search-like popups such as Teleport and GoToPath.
pub fn new_frameless_popup_window<'a>(title: &'a str, colors: &AppColors) -> egui::Window<'a> {
    egui::Window::new(title)
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(
            egui::Frame::default()
                .fill(colors.bg_extreme)
                .inner_margin(8.0)
                .shadow(Shadow {
                    offset: [0, 4],
                    blur: 12,
                    spread: 0,
                    color: Color32::from_black_alpha(60),
                }),
        )
}
