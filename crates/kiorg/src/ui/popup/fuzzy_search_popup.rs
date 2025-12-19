//! Shared abstraction for popups with fuzzy search functionality.
//!
//! This module provides a reusable component for rendering popups that feature
//! a search bar with fuzzy matching and a scrollable list of selectable items.

use crate::config::colors::AppColors;
use egui::{Align, Color32, Frame, Key, Layout, Shadow, TextEdit, Vec2};
use nucleo::{Config as NucleoConfig, Matcher, Utf32Str};
use std::borrow::Cow;

/// Trait for items that can be displayed in a fuzzy search popup.
pub trait FuzzySearchItem: Clone {
    /// The text to display as the primary label.
    fn display_text(&self) -> Cow<'_, str>;

    /// Optional secondary text (displayed as weak/dimmed text).
    fn secondary_text(&self) -> Option<Cow<'_, str>> {
        None
    }

    /// The text to match against when fuzzy searching.
    fn search_text(&self) -> Cow<'_, str> {
        self.display_text()
    }
}

/// Configuration for the fuzzy search popup.
pub struct FuzzySearchPopupConfig<'a> {
    /// The title of the popup window.
    pub title: &'a str,
    /// Hint text shown in the search bar when empty.
    pub search_hint: &'a str,
    /// Message shown when no items are available at all.
    pub empty_message: &'a str,
    /// Message shown when no items match the search query.
    pub no_match_message: &'a str,
    /// Maximum number of visible results (None for unlimited).
    pub max_visible_results: Option<usize>,
}

/// State for the fuzzy search popup UI.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FuzzySearchState {
    pub query: String,
    pub selected_index: usize,
    last_query: String,
}

impl FuzzySearchState {
    pub fn new(query: String) -> Self {
        Self {
            query: query.clone(),
            selected_index: 0,
            last_query: query,
        }
    }

    /// Update the state when the query changes, resetting selection if needed.
    pub fn update_query(&mut self) {
        if self.query != self.last_query {
            self.selected_index = 0;
            self.last_query = self.query.clone();
        }
    }
}

/// Result of fuzzy matching with score.
pub struct FuzzyMatchResult<T> {
    pub item: T,
    pub score: u16,
}

/// Perform fuzzy filtering on a list of items.
pub fn fuzzy_filter<T: FuzzySearchItem>(query: &str, items: &[T]) -> Vec<FuzzyMatchResult<T>> {
    if query.is_empty() {
        return items
            .iter()
            .map(|item| FuzzyMatchResult {
                item: item.clone(),
                score: 0,
            })
            .collect();
    }

    let mut config = NucleoConfig::DEFAULT;
    config.ignore_case = true;
    let mut matcher = Matcher::new(config);

    let mut needle_buf = Vec::new();
    let needle = query.to_lowercase();
    let needle_utf32 = Utf32Str::new(&needle, &mut needle_buf);

    let mut results: Vec<FuzzyMatchResult<T>> = items
        .iter()
        .filter_map(|item| {
            let search_text = item.search_text();
            let mut haystack_buf = Vec::new();
            let haystack_utf32 = Utf32Str::new(&search_text, &mut haystack_buf);
            matcher
                .fuzzy_match(haystack_utf32, needle_utf32)
                .map(|score| FuzzyMatchResult {
                    item: item.clone(),
                    score,
                })
        })
        .collect();

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results
}

/// Action returned by the fuzzy search popup.
pub enum FuzzySearchAction<T> {
    /// Keep the popup open.
    KeepOpen,
    /// Close the popup without selecting anything.
    Close,
    /// An item was selected.
    Selected(T),
}

/// Draw a fuzzy search popup and return the action to take.
///
/// # Arguments
/// * `ctx` - The egui context
/// * `config` - Configuration for the popup
/// * `colors` - App colors to use for rendering
/// * `state` - Mutable reference to the popup state
/// * `items` - The items to display and search through
///
/// # Returns
/// The action to take based on user interaction.
pub fn draw<T: FuzzySearchItem>(
    ctx: &egui::Context,
    config: &FuzzySearchPopupConfig,
    colors: &AppColors,
    state: &mut FuzzySearchState,
    items: &[FuzzyMatchResult<T>],
) -> FuzzySearchAction<T> {
    let mut action = FuzzySearchAction::KeepOpen;

    let shadow = Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 0,
        color: Color32::from_black_alpha(60),
    };

    egui::Window::new(config.title)
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(
            Frame::default()
                .fill(colors.bg_extreme)
                .inner_margin(8.0)
                .shadow(shadow),
        )
        .show(ctx, |ui| {
            let popup_height = ui.available_height() - 60.0;
            ui.set_min_width(600.0);
            ui.set_max_width(ui.available_width() - 100.0);
            ui.set_min_height(popup_height);
            ui.set_max_height(popup_height);

            // Determine visible items count
            let visible_count = if let Some(max) = config.max_visible_results {
                items.len().min(max)
            } else {
                items.len()
            };

            // Handle keyboard input
            action = handle_keyboard_input(ctx, state, items, visible_count);
            if !matches!(action, FuzzySearchAction::KeepOpen) {
                return;
            }

            // Render search bar
            if render_search_bar(ui, &mut state.query, config.search_hint) {
                action = FuzzySearchAction::Close;
                return;
            }

            ui.separator();

            // Render content
            if items.is_empty() {
                ui.centered_and_justified(|ui| {
                    let message = if state.query.is_empty() {
                        config.empty_message
                    } else {
                        config.no_match_message
                    };
                    ui.label(message);
                });
            } else if let Some(selected) = render_item_list(ui, colors, state, items, visible_count)
            {
                action = FuzzySearchAction::Selected(selected);
            }
        });

    action
}

/// Handle keyboard input for the popup.
fn handle_keyboard_input<T: FuzzySearchItem>(
    ctx: &egui::Context,
    state: &mut FuzzySearchState,
    items: &[FuzzyMatchResult<T>],
    visible_count: usize,
) -> FuzzySearchAction<T> {
    let mut action = FuzzySearchAction::KeepOpen;

    ctx.input(|i| {
        for event in &i.events {
            if let egui::Event::Key {
                key, pressed: true, ..
            } = event
            {
                match *key {
                    Key::Escape => {
                        action = FuzzySearchAction::Close;
                    }
                    Key::Enter => {
                        if !items.is_empty() && state.selected_index < visible_count {
                            action = FuzzySearchAction::Selected(
                                items[state.selected_index].item.clone(),
                            );
                        }
                    }
                    Key::ArrowDown => {
                        if !items.is_empty() {
                            let max_index = visible_count.saturating_sub(1);
                            state.selected_index = (state.selected_index + 1).min(max_index);
                        }
                    }
                    Key::ArrowUp => {
                        if state.selected_index > 0 {
                            state.selected_index -= 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    action
}

/// Render the search bar. Returns true if close button was clicked.
fn render_search_bar(ui: &mut egui::Ui, query: &mut String, hint: &str) -> bool {
    let mut close_clicked = false;

    ui.horizontal(|ui| {
        let text_edit = TextEdit::singleline(query)
            .hint_text(hint)
            .desired_width(f32::INFINITY)
            .frame(false);

        let response = ui.add(text_edit);
        response.request_focus();

        if ui.button("×").clicked() {
            close_clicked = true;
        }
    });

    close_clicked
}

/// Render the list of items. Returns the selected item if one was clicked.
fn render_item_list<T: FuzzySearchItem>(
    ui: &mut egui::Ui,
    colors: &AppColors,
    state: &mut FuzzySearchState,
    items: &[FuzzyMatchResult<T>],
    visible_count: usize,
) -> Option<T> {
    let mut selected_item = None;

    egui::ScrollArea::vertical()
        .max_height(ui.available_height())
        .show(ui, |ui| {
            for (index, result) in items.iter().take(visible_count).enumerate() {
                let is_selected = index == state.selected_index;

                let (bg_color, text_color) = if is_selected {
                    (colors.bg_selected, colors.fg_selected)
                } else {
                    (Color32::TRANSPARENT, colors.fg)
                };

                let response = ui
                    .allocate_response(Vec2::new(ui.available_width(), 30.0), egui::Sense::click());

                if response.clicked() {
                    selected_item = Some(result.item.clone());
                }

                if response.hovered() {
                    state.selected_index = index;
                }

                if is_selected || response.hovered() {
                    ui.painter().rect_filled(response.rect, 0.0, bg_color);
                }

                let mut content_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(response.rect)
                        .layout(Layout::left_to_right(Align::Center)),
                );
                content_ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(result.item.display_text().as_ref())
                            .color(text_color)
                            .size(14.0),
                    );
                    if let Some(secondary) = result.item.secondary_text() {
                        ui.weak(secondary.as_ref());
                    }
                });
            }
        });

    selected_item
}
