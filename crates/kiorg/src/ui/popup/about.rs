use egui::{Context, Image, RichText};

use super::PopupType;
use super::window_utils::show_center_popup_window;
use crate::app::Kiorg;
use crate::utils::icon;

/// Show about popup with application information
pub fn show_about_popup(ctx: &Context, app: &mut Kiorg) {
    // Check if the popup should be shown based on the show_popup field
    if !matches!(app.show_popup, Some(PopupType::About)) {
        return;
    }

    let mut keep_open = true; // Use a temporary variable for the open state

    let response = show_center_popup_window("About", ctx, &mut keep_open, |ui| {
        ui.vertical_centered(|ui| {
            // Load and display the app icon
            let texture = icon::load_app_icon_texture(ctx);

            // Display the image with a fixed size
            ui.add(Image::new(&texture).max_width(128.0));

            ui.label(format!("Kiorg v{}", env!("CARGO_PKG_VERSION")));

            // Repository URL as a clickable link
            let repo_url = env!("CARGO_PKG_REPOSITORY");
            if ui
                .link(RichText::new(repo_url).color(app.colors.link_text))
                .clicked()
                && let Err(e) = open::that(repo_url)
            {
                // Call notify_error wrapper
                app.notify_error(format!("Failed to open URL: {e}"));
            }
            ui.add_space(10.0);

            // Add a hint about closing the popup
            if ui
                .link(RichText::new("Press Esc or q to close").color(app.colors.fg_light))
                .clicked()
            {
                app.show_popup = None;
            }
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
