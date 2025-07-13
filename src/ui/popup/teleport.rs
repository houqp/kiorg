use crate::app::Kiorg;
use crate::ui::popup::PopupType;
use crate::visit_history::VisitHistoryEntry;
use egui::{self, Align, Color32, Key, Layout, Shadow, TextEdit, Vec2};
use nucleo::{Config as NucleoConfig, Matcher, Utf32Str};
use std::path::PathBuf;

/// State for the teleport popup
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeleportState {
    pub query: String,
    pub selected_index: usize,
    pub focus_input: bool,
}

impl Default for TeleportState {
    fn default() -> Self {
        Self {
            query: String::new(),
            selected_index: 0,
            focus_input: true,
        }
    }
}

/// Represents a search result with fuzzy matching score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub score: u16,
    pub entry: VisitHistoryEntry,
}

/// Filter and sort visit history based on fuzzy search query
pub fn get_search_results(
    query: &str,
    visit_history: &std::collections::HashMap<PathBuf, VisitHistoryEntry>,
) -> Vec<SearchResult> {
    // If query is empty, just return all directories sorted by access count
    if query.is_empty() {
        let mut results: Vec<SearchResult> = visit_history
            .iter()
            .filter_map(|(path, entry)| {
                // Only include directories that still exist
                if !path.exists() || !path.is_dir() {
                    return None;
                }

                Some(SearchResult {
                    path: path.clone(),
                    score: 0, // Score not relevant for empty query
                    entry: entry.clone(),
                })
            })
            .collect();

        // Sort by access count (descending), then by recent access (descending)
        results.sort_by(|a, b| {
            b.entry
                .count
                .cmp(&a.entry.count)
                .then_with(|| b.entry.accessed_ts.cmp(&a.entry.accessed_ts))
        });

        return results;
    }

    // For non-empty queries, use fuzzy matching
    let mut config = NucleoConfig::DEFAULT;
    config.ignore_case = true;
    let mut matcher = Matcher::new(config);

    // Create needle UTF32 once outside the loop for efficiency
    let mut needle_buf = Vec::new();
    let needle = query.to_lowercase();
    let needle_utf32 = Utf32Str::new(&needle, &mut needle_buf);

    let mut results: Vec<SearchResult> = visit_history
        .iter()
        .filter_map(|(path, entry)| {
            // Only include directories that still exist
            if !path.exists() || !path.is_dir() {
                return None;
            }

            let path_str = path.to_string_lossy();
            let mut haystack_buf = Vec::new();
            let haystack_utf32 = Utf32Str::new(&path_str, &mut haystack_buf);

            matcher
                .fuzzy_match(haystack_utf32, needle_utf32)
                .map(|score| SearchResult {
                    path: path.clone(),
                    score,
                    entry: entry.clone(),
                })
        })
        .collect();

    // Sort by score (descending), then by access count (descending), then by recent access (descending)
    results.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| b.entry.count.cmp(&a.entry.count))
            .then_with(|| b.entry.accessed_ts.cmp(&a.entry.accessed_ts))
    });

    results
}

/// Draw the teleport popup
pub fn draw(ctx: &egui::Context, app: &mut Kiorg) {
    let state = if let Some(PopupType::Teleport(ref state)) = app.show_popup {
        state.clone()
    } else {
        return;
    };

    let mut keep_open = true;
    let mut navigate_to: Option<PathBuf> = None;
    let mut new_state = state.clone();

    // Create shadow similar to search bar
    let shadow = Shadow {
        offset: [0, 4],                       // 4px downward shadow
        blur: 12,                             // 12px blur
        spread: 0,                            // No spread
        color: Color32::from_black_alpha(60), // Semi-transparent black shadow
    };

    egui::Window::new("Teleport")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(
            egui::Frame::default()
                .fill(app.colors.bg_extreme)
                .inner_margin(8.0)
                .shadow(shadow),
        )
        .show(ctx, |ui| {
            let popup_height = ui.available_height() - 60.0;
            ui.set_min_width(600.0);
            ui.set_max_width(ui.available_width() - 100.0);
            ui.set_min_height(popup_height); // Set a fixed minimum height
            ui.set_max_height(popup_height); // Set a fixed minimum height

            // Limit the number of visible results
            let max_visible_results = 10;

            // Search results
            let results = get_search_results(&new_state.query, &app.visit_history);

            // Handle keyboard input
            ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Key {
                        key, pressed: true, ..
                    } = event
                    {
                        match *key {
                            Key::Escape => {
                                keep_open = false;
                            }
                            Key::Enter => {
                                // Navigate to selected result
                                let visible_count = results.len().min(max_visible_results);
                                if !results.is_empty() && new_state.selected_index < visible_count {
                                    navigate_to =
                                        Some(results[new_state.selected_index].path.clone());
                                    keep_open = false;
                                }
                            }
                            Key::ArrowDown => {
                                if !results.is_empty() {
                                    let max_index = (results.len().min(max_visible_results)) - 1;
                                    new_state.selected_index =
                                        (new_state.selected_index + 1).min(max_index);
                                }
                            }
                            Key::ArrowUp => {
                                if new_state.selected_index > 0 {
                                    new_state.selected_index -= 1;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            });

            // Search input
            ui.horizontal(|ui| {
                let response = ui.add(
                    TextEdit::singleline(&mut new_state.query)
                        .hint_text("Teleport to directory...")
                        .desired_width(f32::INFINITY)
                        .frame(false),
                );

                if new_state.focus_input {
                    response.request_focus();
                    new_state.focus_input = false;
                }

                // Reset selection when query changes
                if new_state.query != state.query {
                    new_state.selected_index = 0;
                }

                // Close button
                if ui.button("×").clicked() {
                    keep_open = false;
                }
            });

            ui.separator();

            if results.is_empty() {
                ui.centered_and_justified(|ui| {
                    if app.visit_history.is_empty() {
                        ui.label("No visit history available");
                    } else {
                        ui.label("No matching directories found");
                    }
                });
                return;
            }

            egui::ScrollArea::vertical()
                .max_height(popup_height) // Fixed height to keep popup size constant
                .show(ui, |ui| {
                    for (index, result) in results.iter().take(max_visible_results).enumerate() {
                        let is_selected = index == new_state.selected_index;

                        let (bg_color, text_color) = if is_selected {
                            (app.colors.bg_selected, app.colors.fg_selected)
                        } else {
                            (Color32::TRANSPARENT, app.colors.fg)
                        };

                        let response = ui.allocate_response(
                            Vec2::new(ui.available_width(), 30.0),
                            egui::Sense::click(),
                        );

                        if response.clicked() {
                            navigate_to = Some(result.path.clone());
                            keep_open = false;
                        }

                        // Handle mouse hover
                        if response.hovered() {
                            new_state.selected_index = index;
                        }

                        // Draw background
                        if is_selected || response.hovered() {
                            ui.painter().rect_filled(response.rect, 0.0, bg_color);
                        }

                        // Draw content
                        let mut content_ui = ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(response.rect)
                                .layout(Layout::left_to_right(Align::Center)),
                        );
                        content_ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            // Path
                            ui.label(
                                egui::RichText::new(result.path.to_string_lossy())
                                    .color(text_color)
                                    .size(14.0),
                            );
                        });
                    }
                });
        });

    // Update the popup state
    if keep_open {
        app.show_popup = Some(PopupType::Teleport(new_state));
    } else {
        app.show_popup = None;
    }

    // Navigate if a path was selected
    if let Some(path) = navigate_to {
        app.navigate_to_dir(path);
    }
}
