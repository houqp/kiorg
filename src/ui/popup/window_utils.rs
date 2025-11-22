/// Standard margin for popup content
pub const POPUP_MARGIN: i8 = 10;

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
) -> Option<egui::InnerResponse<R>> {
    let win_resp = new_center_popup_window(title).open(open).show(ctx, |ui| {
        // NOTE: we are wrapping another frame here instead of setting
        // window.frame() because there is window.show_dyn overrides header
        // color with the frame's fill color when frame is not None, making it
        // impossible to set a different color for the window title bar.
        egui::Frame::new()
            .inner_margin(POPUP_MARGIN)
            .show(ui, |ui| content(ui))
    });
    win_resp.and_then(|response| response.inner)
}
