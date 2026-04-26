pub mod center_panel;
pub mod egui_notify;
pub mod file_list;
pub mod help_window;
pub mod left_panel;
pub mod notification;
pub mod path_nav;
pub mod popup;
pub mod preview;
pub mod rename;
pub mod right_panel;
pub mod search_bar;
pub mod separator;
pub mod style;
pub mod terminal;
pub mod top_banner;
pub mod update;

#[inline]
pub fn clamp_height(h: f32) -> f32 {
    #[cfg(debug_assertions)]
    return h;
    #[cfg(not(debug_assertions))]
    return h.max(0.0);
}
