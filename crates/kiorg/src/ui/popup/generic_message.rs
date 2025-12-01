// Generic message popup implementation
use egui::Context;

use super::PopupType;
use super::window_utils::new_center_popup_window;
use crate::app::Kiorg;
use crate::ui::style::section_title_text;

/// Show generic message popup with custom title and message
pub fn show_generic_message_popup(ctx: &Context, app: &mut Kiorg) {
    // Check if the popup should be shown and extract title and message
    let (title, message) = match &app.show_popup {
        Some(PopupType::GenericMessage(title, msg)) => (title.clone(), msg.clone()),
        _ => return,
    };

    let mut keep_open = true; // Use a temporary variable for the open state

    let response = new_center_popup_window(&title)
        .open(&mut keep_open) // Control window visibility
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);

                // Display the message using section_title_text for consistent styling
                ui.label(message);

                ui.add_space(10.0);

                // Add a hint about closing the popup
                if ui
                    .link(section_title_text("Press Esc or q to close", &app.colors))
                    .clicked()
                {
                    app.show_popup = None;
                }
                ui.add_space(5.0);
            });
        });

    // Update the state based on window interaction
    if response.is_some() {
        if !keep_open {
            app.show_popup = None;
        }
    } else {
        app.show_popup = None;
    }
}
