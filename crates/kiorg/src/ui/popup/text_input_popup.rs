use crate::config::colors::AppColors;
use egui::{Context, Frame, Key, TextEdit};

use super::window_utils::new_center_popup_window;

/// Configuration for text input selection behavior
pub enum TextSelection {
    /// Select all text
    All,
    /// Select a range of characters from start to end (exclusive)
    Range { start: usize, end: usize },
    /// No selection, cursor at the end
    None,
}

/// Configuration for the text input popup
pub struct TextInputConfig<'a> {
    /// The title of the popup window
    pub title: &'a str,
    /// Hint text shown in the input field when empty
    pub hint: &'a str,
    /// Initial text selection behavior (only applies on first frame)
    pub initial_selection: TextSelection,
}

/// Draw a text input popup and return whether it should stay open
///
/// Returns `true` if the popup should remain open, `false` if it should close
pub fn draw(
    ctx: &Context,
    colors: &AppColors,
    config: &TextInputConfig,
    text: &mut String,
    popup_id: &str,
) -> bool {
    let mut keep_open = true;
    let init_flag_id = egui::Id::new(popup_id);

    // Check for Esc key to close popup
    ctx.input(|i| {
        if i.key_pressed(Key::Escape) {
            keep_open = false;
        }
    });

    new_center_popup_window(config.title)
        .open(&mut keep_open)
        .show(ctx, |ui| {
            Frame::default()
                .fill(colors.bg_extreme)
                .inner_margin(5.0)
                .show(ui, |ui| {
                    ui.set_max_width(500.0);

                    ui.horizontal(|ui| {
                        let is_first_frame = ui.memory(|mem| {
                            !mem.data.get_temp::<bool>(init_flag_id).unwrap_or(false)
                        });

                        // Style the hint text with light color
                        ui.style_mut().visuals.widgets.inactive.fg_stroke.color = colors.fg_light;

                        let text_edit = TextEdit::singleline(text)
                            .hint_text(config.hint)
                            .desired_width(f32::INFINITY)
                            .frame(false);
                        let response = ui.add(text_edit);

                        response.request_focus();

                        // Apply initial selection on first frame
                        if is_first_frame {
                            if let Some(mut state) = TextEdit::load_state(ui.ctx(), response.id) {
                                let cursor_range = match config.initial_selection {
                                    TextSelection::All => {
                                        let len = text.chars().count();
                                        egui::text::CCursorRange::two(
                                            egui::text::CCursor::new(0),
                                            egui::text::CCursor::new(len),
                                        )
                                    }
                                    TextSelection::Range { start, end } => {
                                        egui::text::CCursorRange::two(
                                            egui::text::CCursor::new(start),
                                            egui::text::CCursor::new(end),
                                        )
                                    }
                                    TextSelection::None => {
                                        let len = text.chars().count();
                                        egui::text::CCursorRange::one(egui::text::CCursor::new(len))
                                    }
                                };
                                state.cursor.set_char_range(Some(cursor_range));
                                state.store(ui.ctx(), response.id);
                            }
                            ui.memory_mut(|mem| {
                                mem.data.insert_temp(init_flag_id, true);
                            });
                        }
                    });
                });
        });

    keep_open
}

/// Clear the initialization flag for a text input popup
pub fn clear_init_flag(ctx: &Context, popup_id: &str) {
    let init_flag_id = egui::Id::new(popup_id);
    ctx.memory_mut(|mem| {
        mem.data.insert_temp::<bool>(init_flag_id, false);
    });
}
