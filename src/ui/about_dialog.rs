use egui::{Context, Image, RichText};

use super::window_utils::new_center_popup_window;
use crate::config::colors::AppColors;
use crate::utils::icon;

/// Show about dialog with application information
pub fn show_about_dialog(ctx: &Context, show_about: &mut bool, colors: &AppColors) {
    if !*show_about {
        return;
    }

    let mut keep_open = *show_about; // Use a temporary variable for the open state

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
                    .link(RichText::new(repo_url).color(colors.link_text))
                    .clicked()
                {
                    if let Err(e) = open::that(repo_url) {
                        eprintln!("Failed to open URL: {}", e);
                    }
                }
                ui.add_space(10.0);
            });
        });

    // Update the state based on window interaction
    if response.is_some() {
        *show_about = keep_open;
    } else {
        *show_about = false;
    }
}
