/// Standard margin for popup content
pub const POPUP_MARGIN: f32 = 10.0;

pub fn new_center_popup_window(title: &str) -> egui::Window<'_> {
    egui::Window::new(egui::RichText::from(title).size(14.0))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
}

/// Show a centered popup window with standard margin and frame
/// Returns the response from the window if it was shown
pub fn show_center_popup_window<R>(
    title: &str,
    ctx: &egui::Context,
    open: &mut bool,
    content: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<egui::InnerResponse<Option<R>>> {
    let frame = egui::Frame::window(&ctx.style()).inner_margin(POPUP_MARGIN);
    new_center_popup_window(title)
        .frame(frame)
        .open(open)
        .show(ctx, content)
}
