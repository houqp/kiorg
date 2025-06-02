pub fn new_center_popup_window(title: &str) -> egui::Window {
    egui::Window::new(egui::RichText::from(title).size(14.0))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
}
