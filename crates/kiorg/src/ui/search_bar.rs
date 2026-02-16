use crate::app::Kiorg;
use egui::{Color32, Context, Shadow};

#[derive(Default)]
pub struct SearchBar {
    pub query: Option<String>,
    pub focus: bool,
    pub case_insensitive: bool,
    pub fuzzy: bool,
}

impl SearchBar {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            query: None,
            focus: false,
            case_insensitive: true, // Default to case insensitive
            fuzzy: true,            // Default to fuzzy search
        }
    }

    #[must_use]
    pub const fn active(&self) -> bool {
        self.query.is_some()
    }

    pub fn activate(&mut self) {
        if self.query.is_none() {
            self.query = Some(String::new());
        }
        self.focus = true;
    }

    pub fn close(&mut self) {
        self.focus = false;
        self.query = None;
    }
}

fn apply_new_query(app: &mut Kiorg) {
    // only need to apply search filter to the current active tab
    let tab = app.tab_manager.current_tab_mut();
    tab.update_filtered_cache(
        &app.search_bar.query,
        app.search_bar.case_insensitive,
        app.search_bar.fuzzy,
    );

    if let Some(&index) = tab.get_cached_filtered_entries().first() {
        tab.update_selection(index);
        app.ensure_selected_visible = true;
        app.selection_changed = true;
    }
}

pub fn handle_key_press(ctx: &Context, app: &mut Kiorg) -> bool {
    match &app.search_bar.query {
        Some(query) => {
            if !app.search_bar.focus {
                return false;
            }

            let mut close_search_bar = false;

            let consumed = ctx.input(|i| {
                if i.key_pressed(egui::Key::Enter) {
                    // Keep search mode active if there's a non-empty search query
                    if query.is_empty() {
                        close_search_bar = true;
                    }
                    app.search_bar.focus = false;
                    return true; // Consume Enter key
                }

                if i.key_pressed(egui::Key::Escape) {
                    close_search_bar = true;
                    return true; // Consume Escape key
                }

                // Block all other keyboard inputs when search bar has focus
                true
            });

            if close_search_bar {
                app.search_bar.close();
                // Reset filter when closing search bar
                let tab = app.tab_manager.current_tab_mut();
                tab.update_filtered_cache(&None, false, false);
            }

            consumed
        }
        None => false,
    }
}

pub fn draw(ctx: &Context, app: &mut Kiorg) {
    if app.search_bar.query.is_none() {
        return;
    }

    egui::Area::new(egui::Id::new("search_bar"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 10.0)) // Center-top with offset
        .interactable(true)
        .movable(false)
        .show(ctx, |ui| {
            // Create a shadow similar to window popups
            let shadow = Shadow {
                offset: [0, 4],                       // 4px downward shadow
                blur: 12,                             // 12px blur
                spread: 0,                            // No spread
                color: Color32::from_black_alpha(60), // Semi-transparent black shadow
            };

            egui::Frame::default()
                .fill(app.colors.bg_extreme)
                .inner_margin(5.0)
                .shadow(shadow)
                .show(ui, |ui| {
                    ui.set_max_width(400.0); // Limit width

                    ui.horizontal(|ui| {
                        // Search input
                        let text_edit =
                            egui::TextEdit::singleline(app.search_bar.query.as_mut().unwrap())
                                .hint_text("Search...")
                                .desired_width(f32::INFINITY) // Take available width
                                .frame(false);
                        let response = ui.add(text_edit);

                        // Set focus when search mode is first activated
                        if app.search_bar.focus {
                            response.request_focus();
                            app.search_bar.focus = false;
                        }

                        // Update focus state based on whether the text edit has focus
                        app.search_bar.focus = response.has_focus();

                        // Update filter when text changes
                        if response.changed() {
                            apply_new_query(app);
                        }

                        // Case sensitivity toggle button
                        let toggle_color = if app.search_bar.case_insensitive {
                            app.colors.fg_light
                        } else {
                            app.colors.highlight
                        };
                        let tooltip_text = if app.search_bar.case_insensitive {
                            "Click to enable case sensitive search"
                        } else {
                            "Click to enable case insensitive search"
                        };
                        let case_button_clicked = ui
                            .add(
                                egui::Button::new(egui::RichText::new("Aa").color(toggle_color))
                                    .small()
                                    .frame(false),
                            )
                            .on_hover_text(tooltip_text)
                            .clicked();
                        if case_button_clicked {
                            app.search_bar.case_insensitive = !app.search_bar.case_insensitive;
                            apply_new_query(app);
                        }

                        // Fuzzy search toggle button
                        let fuzzy_toggle_color = if app.search_bar.fuzzy {
                            app.colors.highlight
                        } else {
                            app.colors.fg_light
                        };
                        let fuzzy_tooltip_text = if app.search_bar.fuzzy {
                            "Click to disable fuzzy search (exact match)"
                        } else {
                            "Click to enable fuzzy search"
                        };
                        let fuzzy_button_clicked = ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Fz").color(fuzzy_toggle_color),
                                )
                                .small()
                                .frame(false),
                            )
                            .on_hover_text(fuzzy_tooltip_text)
                            .clicked();
                        if fuzzy_button_clicked {
                            app.search_bar.fuzzy = !app.search_bar.fuzzy;
                            apply_new_query(app);
                        }

                        // Close button
                        if ui.button("×").clicked() {
                            app.search_bar.close();
                            // Reset filter when closing search bar
                            let tab = app.tab_manager.current_tab_mut();
                            tab.update_filtered_cache(&None, false, false);
                        }
                    });
                });
        });
}
