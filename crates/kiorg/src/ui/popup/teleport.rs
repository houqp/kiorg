use crate::app::Kiorg;
use crate::ui::popup::PopupType;
use crate::ui::popup::fuzzy_search_popup::{
    FuzzyMatchResult, FuzzySearchAction, FuzzySearchItem, FuzzySearchPopupConfig, FuzzySearchState,
};
use crate::visit_history::VisitHistoryEntry;
use nucleo::{Config as NucleoConfig, Matcher, Utf32Str};
use std::borrow::Cow;
use std::path::PathBuf;

static POPUP_CONFIG: FuzzySearchPopupConfig = FuzzySearchPopupConfig {
    title: "Teleport",
    search_hint: "Teleport to directory...",
    empty_message: "No visit history available",
    no_match_message: "No matching directories found",
    max_visible_results: Some(10),
};

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

/// Represents a search result with visit history data
#[derive(Debug, Clone)]
pub struct TeleportSearchResult {
    pub entry: VisitHistoryEntry,
}

impl FuzzySearchItem for TeleportSearchResult {
    fn display_text(&self) -> Cow<'_, str> {
        self.entry.path.to_string_lossy()
    }

    fn secondary_text(&self) -> Option<Cow<'_, str>> {
        None
    }

    fn search_text(&self) -> Cow<'_, str> {
        self.entry.path.to_string_lossy()
    }
}

/// Filter and sort visit history based on fuzzy search query.
/// This uses custom sorting logic based on access count and timestamp.
pub fn get_search_results(
    query: &str,
    visit_history: &std::collections::HashMap<PathBuf, VisitHistoryEntry>,
) -> Vec<FuzzyMatchResult<TeleportSearchResult>> {
    // If query is empty, just return all directories sorted by access count
    if query.is_empty() {
        let mut results: Vec<FuzzyMatchResult<TeleportSearchResult>> = visit_history
            .iter()
            .filter_map(|(path, entry)| {
                // Only include directories that still exist
                if !path.exists() || !path.is_dir() {
                    return None;
                }

                Some(FuzzyMatchResult {
                    item: TeleportSearchResult {
                        entry: entry.clone(),
                    },
                    score: 0, // Score not relevant for empty query
                })
            })
            .collect();

        // Sort by access count (descending), then by recent access (descending)
        results.sort_by(|a, b| {
            b.item
                .entry
                .count
                .cmp(&a.item.entry.count)
                .then_with(|| b.item.entry.accessed_ts.cmp(&a.item.entry.accessed_ts))
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

    let mut results: Vec<FuzzyMatchResult<TeleportSearchResult>> = visit_history
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
                .map(|score| FuzzyMatchResult {
                    item: TeleportSearchResult {
                        entry: entry.clone(),
                    },
                    score,
                })
        })
        .collect();

    // Sort by score (descending), then by access count (descending), then by recent access (descending)
    results.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| b.item.entry.count.cmp(&a.item.entry.count))
            .then_with(|| b.item.entry.accessed_ts.cmp(&a.item.entry.accessed_ts))
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

    let mut fuzzy_state = FuzzySearchState::new(state.query.clone());
    fuzzy_state.selected_index = state.selected_index;

    // Get search results with custom sorting
    let results = get_search_results(&fuzzy_state.query, &app.visit_history);

    let action = crate::ui::popup::fuzzy_search_popup::draw(
        ctx,
        &POPUP_CONFIG,
        &app.colors,
        &mut fuzzy_state,
        &results,
    );

    match action {
        FuzzySearchAction::KeepOpen => {
            // Update the popup state
            let new_state = TeleportState {
                query: fuzzy_state.query,
                selected_index: fuzzy_state.selected_index,
                focus_input: false,
            };
            app.show_popup = Some(PopupType::Teleport(new_state));
        }
        FuzzySearchAction::Close => {
            app.show_popup = None;
        }
        FuzzySearchAction::Selected(result) => {
            app.show_popup = None;
            app.navigate_to_dir(result.entry.path);
        }
    }
}
