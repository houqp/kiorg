use egui::{Context, Image, RichText};

use super::window_utils::new_center_popup_window;
use crate::app::Kiorg;
use crate::utils::icon;

/// Show about dialog with application information
pub fn show_about_dialog(ctx: &Context, app: &mut Kiorg) {
    // Check if the dialog should be shown based on the show_dialog field
    if app.show_dialog != Some(crate::app::DialogType::About) {
        return;
    }

    let mut keep_open = true; // Use a temporary variable for the open state

    let response = new_center_popup_window("About")
        .open(&mut keep_open) // Control window visibility
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);

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
                {
                    if let Err(e) = open::that(repo_url) {
                        // Call toasts directly
                        app.toasts.error(format!("Failed to open URL: {}", e));
                    }
                }
                ui.add_space(10.0);

                // Add a hint about closing the dialog
                if ui
                    .link(RichText::new("Press Esc or q to close").color(app.colors.fg_light))
                    .clicked()
                {
                    app.show_dialog = None;
                }
                ui.add_space(5.0);
            });
        });

    // Update the state based on window interaction
    if response.is_some() {
        if !keep_open {
            app.show_dialog = None;
        }
    } else {
        app.show_dialog = None;
    }
}
