use crate::app::Kiorg;
use egui::{Color32, Context, Shadow};

#[derive(Default)]
pub struct SearchBar {
    pub query: Option<String>,
    pub focus: bool,
}

impl SearchBar {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            query: None,
            focus: false,
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
                    } else {
                        // Select the first matched entry
                        let tab = app.tab_manager.current_tab_mut();
                        if let Some(first_filtered_index) =
                            tab.get_first_filtered_entry_index(query.as_str())
                        {
                            tab.update_selection(first_filtered_index);
                            app.ensure_selected_visible = true;
                            app.selection_changed = true;
                        }
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

                        // Close button
                        if ui.button("×").clicked() {
                            app.search_bar.close();
                        }
                    });
                });
        });
}
