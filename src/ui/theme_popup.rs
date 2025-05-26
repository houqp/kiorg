use crate::app::Kiorg;
use crate::config;
use crate::config::shortcuts::ShortcutAction;
use crate::theme::Theme;

use super::window_utils::new_center_popup_window;

/// Helper function to apply a theme and save it to the configuration
fn apply_and_save_theme(app: &mut Kiorg, theme_selection: &Theme, ctx: &egui::Context) {
    let new_colors = theme_selection.get_colors();

    // Apply the new theme to the app colors
    app.colors = new_colors.clone();

    // Apply the new theme to the UI context
    ctx.set_visuals(app.colors.to_visuals());

    // Update the configuration with theme key only, set colors to None to keep config short
    app.config.colors = None;
    app.config.theme = Some(theme_selection.theme_key().to_string());

    // Save the configuration
    if let Err(e) = config::save_config_with_override(&app.config, app.config_dir_override.as_ref())
    {
        app.notify_error(format!("Failed to save theme: {}", e));
    }
}

/// Helper function to display themes in a grid layout
fn display_themes_grid(
    ui: &mut egui::Ui,
    themes: &[&Theme],
    selected_theme_key: &str,
    current_theme: &Theme,
    colors: &crate::config::colors::AppColors,
) -> Option<Theme> {
    let mut selected_theme: Option<&Theme> = None;
    let bg_selected = colors.bg_selected.to_owned();
    let selected_key = selected_theme_key.to_owned(); // Clone for closure capture

    // Create a vector of theme keys for the closure to use
    let theme_keys: Vec<String> = themes.iter().map(|t| t.theme_key().to_string()).collect();

    egui::Grid::new("themes_grid")
        .num_columns(2)
        .min_col_width(200.0)
        .spacing([20.0, 2.0]) // 20px horizontal spacing, 2px vertical spacing
        .with_row_color(move |i, _| {
            // Check if this row index corresponds to the selected theme
            if let Some(theme_key) = theme_keys.get(i) {
                if theme_key == &selected_key {
                    Some(bg_selected)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .show(ui, |ui| {
            for theme in themes.iter() {
                let is_selected = theme.theme_key() == selected_theme_key;
                let is_current = theme.theme_key() == current_theme.theme_key();

                let theme_text = theme.display_name();
                let theme_color = if is_current {
                    colors.highlight
                } else if is_selected {
                    colors.fg_selected
                } else {
                    colors.fg
                };

                let response = ui.colored_label(theme_color, theme_text);
                ui.end_row();

                // Show clickable hand cursor on hover and handle clicks
                let response = if response.hovered() {
                    response.on_hover_cursor(egui::CursorIcon::PointingHand)
                } else {
                    response
                };
                if response.clicked() {
                    selected_theme = Some(theme);
                }
            }
        });

    selected_theme.cloned()
}

pub fn draw(app: &mut Kiorg, ctx: &egui::Context) {
    // Extract the selected theme key from the app's popup state
    let selected_theme_key = if let Some(crate::app::PopupType::Themes(key)) = &app.show_popup {
        key.clone()
    } else {
        // Default to the first theme if no key is found
        "dark_kiorg".to_string()
    };

    // Get current theme from the selected theme key
    let current_theme =
        Theme::from_theme_key(&selected_theme_key).unwrap_or_else(crate::theme::get_default_theme); // Default fallback

    let mut keep_open = true;
    // Get list of all themes
    let themes = Theme::all_themes();

    // Find current theme index for navigation
    let current_selected_index = themes
        .iter()
        .position(|t| t.theme_key() == selected_theme_key)
        .unwrap_or(0);

    // Handle keyboard navigation using shortcuts
    let mut theme_key_changed = false;
    let mut new_selected_theme_key = selected_theme_key.to_string();

    // Check for shortcut actions based on input
    let action = app.get_shortcut_action_from_input(ctx, false);

    if let Some(action) = action {
        match action {
            ShortcutAction::Exit => {
                app.show_popup = None;
                return;
            }
            ShortcutAction::MoveDown => {
                if !themes.is_empty() {
                    let new_index = (current_selected_index + 1).min(themes.len() - 1);
                    if new_index != current_selected_index {
                        new_selected_theme_key = themes[new_index].theme_key().to_string();
                        theme_key_changed = true;
                    }
                }
            }
            ShortcutAction::MoveUp => {
                let new_index = current_selected_index.saturating_sub(1);
                if new_index != current_selected_index {
                    new_selected_theme_key = themes[new_index].theme_key().to_string();
                    theme_key_changed = true;
                }
            }
            ShortcutAction::OpenDirectoryOrFile => {
                if !themes.is_empty() {
                    if let Some(selected_theme) = Theme::from_theme_key(&new_selected_theme_key) {
                        // Apply and save the selected theme
                        apply_and_save_theme(app, selected_theme, ctx);
                        app.show_popup = None;
                        return;
                    }
                }
            }
            _ => {} // Ignore other actions
        }
    }

    // Apply preview theme when theme key changes
    if theme_key_changed {
        if let Some(preview_theme) = Theme::from_theme_key(&new_selected_theme_key) {
            // Apply the theme immediately for preview
            let new_colors = preview_theme.get_colors();
            app.colors = new_colors.clone();
            ctx.set_visuals(app.colors.to_visuals());

            // Update the popup with the new selected theme key
            app.show_popup = Some(crate::app::PopupType::Themes(
                new_selected_theme_key.clone(),
            ));
        }
    }

    let mut selected_theme = None;

    new_center_popup_window("Themes")
        .open(&mut keep_open)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(theme) = display_themes_grid(
                    ui,
                    themes,
                    &new_selected_theme_key,
                    current_theme,
                    &app.colors,
                ) {
                    selected_theme = Some(theme);
                }
            });
        });

    // Handle theme selection from mouse clicks
    if let Some(theme) = selected_theme {
        // Apply and save the selected theme
        apply_and_save_theme(app, &theme, ctx);
        app.show_popup = None;
        return;
    }

    // Handle popup close via window controls
    if !keep_open {
        app.show_popup = None;
    }
}
