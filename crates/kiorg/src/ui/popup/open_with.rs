use crate::app::Kiorg;
use crate::ui::popup::PopupType;
use crate::ui::popup::fuzzy_search_popup::{
    FuzzySearchAction, FuzzySearchItem, FuzzySearchPopupConfig, FuzzySearchState, fuzzy_filter,
};
use crate::ui::popup::text_input_popup::{TextInputConfig, TextSelection, draw as draw_text_input};
use mimeapps::AppInfo;
use std::borrow::Cow;

const OPEN_WITH_POPUP_ID: &str = "open_with_popup";

static POPUP_CONFIG: FuzzySearchPopupConfig = FuzzySearchPopupConfig {
    title: "Open with",
    search_hint: "Type to filter application to open with...",
    empty_message: "No applications available",
    no_match_message: "No matching application found",
    max_visible_results: None,
};

impl FuzzySearchItem for AppInfo {
    fn display_text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.name)
    }

    fn secondary_text(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(&self.path))
    }

    fn search_text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.name)
    }
}

#[cfg(not(test))]
fn get_apps_for_file(path: &std::path::Path) -> Vec<AppInfo> {
    mimeapps::get_apps_for_file(path)
}

#[cfg(test)]
fn get_apps_for_file(_path: &std::path::Path) -> Vec<AppInfo> {
    Vec::new()
}

#[derive(Default, Clone)]
struct OpenWithUiState {
    apps: Vec<AppInfo>,
    fuzzy_state: FuzzySearchState,
    apps_loaded: bool,
}

impl OpenWithUiState {
    fn load_if_needed(&mut self, app: &Kiorg) {
        if !self.apps_loaded
            && let Some(entry) = app.tab_manager.current_tab_ref().selected_entry()
        {
            self.apps = get_apps_for_file(&entry.path);
            self.apps_loaded = true;
        }
    }
}

/// Draw the open with popup dialog
pub fn draw(ctx: &egui::Context, app: &mut Kiorg) {
    // Early return if not in open with mode
    if !matches!(app.show_popup, Some(PopupType::OpenWith)) {
        return;
    }

    let mut ui_state = ctx.data_mut(|d| {
        d.get_temp_mut_or_insert_with(egui::Id::new("open_with_ui_state"), || {
            OpenWithUiState::default()
        })
        .clone()
    });

    ui_state.load_if_needed(app);
    ui_state.fuzzy_state.update_query();

    // If no apps are found, show a custom command input popup instead
    if ui_state.apps.is_empty() {
        draw_custom_command_popup(ctx, app, &mut ui_state);
    } else {
        draw_app_selection_popup(ctx, app, &mut ui_state);
    }
}

fn draw_custom_command_popup(ctx: &egui::Context, app: &mut Kiorg, ui_state: &mut OpenWithUiState) {
    let config = TextInputConfig {
        title: "Open with",
        hint: "No associated application found for file type, enter custom command to open...",
        initial_selection: TextSelection::None,
    };

    let keep_open = draw_text_input(
        ctx,
        &app.colors,
        &config,
        &mut ui_state.fuzzy_state.query,
        OPEN_WITH_POPUP_ID,
    );

    if !keep_open {
        clear_state(ctx);
        close_popup(app);
    } else {
        // Save state back
        ctx.data_mut(|d| {
            d.insert_temp(egui::Id::new("open_with_ui_state"), ui_state.clone());
        });
    }
}

fn draw_app_selection_popup(ctx: &egui::Context, app: &mut Kiorg, ui_state: &mut OpenWithUiState) {
    let filtered_apps = fuzzy_filter(&ui_state.fuzzy_state.query, &ui_state.apps);

    let action = crate::ui::popup::fuzzy_search_popup::draw(
        ctx,
        &POPUP_CONFIG,
        &app.colors,
        &mut ui_state.fuzzy_state,
        &filtered_apps,
    );

    // Save state back
    ctx.data_mut(|d| {
        d.insert_temp(egui::Id::new("open_with_ui_state"), ui_state.clone());
    });

    match action {
        FuzzySearchAction::KeepOpen => {}
        FuzzySearchAction::Close => {
            clear_state(ctx);
            close_popup(app);
        }
        FuzzySearchAction::Selected(app_info) => {
            clear_state(ctx);
            confirm_open_with(app, app_info.path);
        }
    }
}

fn clear_state(ctx: &egui::Context) {
    ctx.data_mut(|d| {
        d.remove::<OpenWithUiState>(egui::Id::new("open_with_ui_state"));
    });
}

pub fn confirm_open_with(app: &mut Kiorg, command: String) {
    if command.is_empty() {
        app.notify_error("Cannot open: No command provided");
        return;
    }

    let path_to_open = {
        let tab = app.tab_manager.current_tab_ref();
        tab.selected_entry().map(|entry| entry.path.clone())
    };

    if let Some(path) = path_to_open {
        app.open_file_with_command(path, command);
    }

    close_popup(app);
}

pub fn close_popup(app: &mut Kiorg) {
    app.show_popup = None;
}
