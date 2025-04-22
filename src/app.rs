use egui::TextureHandle;
use std::path::PathBuf;

use crate::config::{self, colors::AppColors};
use crate::input;
use crate::models::tab::TabManager;
use crate::ui::add_entry_popup; // Import the new module
use crate::ui::delete_dialog::DeleteDialog;
use crate::ui::dialogs;
use crate::ui::search_bar::{self, SearchBar};
use crate::ui::separator;
use crate::ui::separator::SEPARATOR_PADDING;
use crate::ui::terminal;
use crate::ui::top_banner;
use crate::ui::{bookmark_popup, center_panel, help_window, left_panel, right_panel};

// Layout constants
const PANEL_SPACING: f32 = 5.0; // Space between panels

// Panel size ratios (relative to usable width)
const LEFT_PANEL_RATIO: f32 = 0.15;
const RIGHT_PANEL_RATIO: f32 = 0.25;
const LEFT_PANEL_MIN_WIDTH: f32 = 150.0;
const RIGHT_PANEL_MIN_WIDTH: f32 = 200.0;

pub struct Kiorg {
    pub tab_manager: TabManager,
    pub colors: AppColors,
    pub ensure_selected_visible: bool,
    pub show_help: bool,
    pub preview_content: String,
    pub show_exit_confirm: bool,
    pub current_image: Option<TextureHandle>,
    pub rename_mode: bool,
    pub new_name: String,
    pub rename_focus: bool,
    pub clipboard: Option<(Vec<PathBuf>, bool)>, // (paths, is_cut)
    pub show_delete_confirm: bool,
    pub entry_to_delete: Option<PathBuf>,
    pub bookmarks: Vec<PathBuf>,
    pub show_bookmarks: bool,
    pub bookmark_selected_index: usize, // Store bookmark selection index in app state
    pub config_dir_override: Option<PathBuf>, // Optional override for config directory path
    pub prev_path: Option<PathBuf>,     // Previous path for selection preservation
    pub cached_preview_path: Option<PathBuf>,
    pub selection_changed: bool, // Flag to track if selection changed
    pub search_bar: SearchBar,
    pub scroll_range: Option<std::ops::Range<usize>>,

    pub terminal_ctx: Option<terminal::TerminalContext>,

    pub add_mode: bool,
    pub new_entry_name: String,
    // TODO: is this neeeded if we already have add_mode?
    pub add_focus: bool,

    // ts variable for tracking key press times
    pub last_lowercase_g_pressed_ms: u64,
}

impl Kiorg {
    pub fn new(cc: &eframe::CreationContext<'_>, initial_dir: PathBuf) -> Self {
        Self::new_with_config_dir(cc, initial_dir, None)
    }

    pub fn new_with_config_dir(
        cc: &eframe::CreationContext<'_>,
        initial_dir: PathBuf,
        config_dir_override: Option<PathBuf>,
    ) -> Self {
        let config = config::load_config_with_override(config_dir_override.as_ref());
        let colors = AppColors::from_config(&config.colors);

        cc.egui_ctx.set_visuals(colors.to_visuals());

        let tab_manager = TabManager::new_with_config(initial_dir, Some(&config));

        let mut app = Self {
            tab_manager,
            colors,
            ensure_selected_visible: false,
            show_help: false,
            preview_content: String::new(),
            show_exit_confirm: false,
            current_image: None,
            rename_mode: false,
            new_name: String::new(),
            rename_focus: false,
            clipboard: None,
            show_delete_confirm: false,
            entry_to_delete: None,
            bookmarks: Vec::new(),
            show_bookmarks: false,
            bookmark_selected_index: 0, // Initialize the bookmark selection index
            config_dir_override,
            prev_path: None,
            cached_preview_path: None,
            selection_changed: true, // Initialize flag to true
            search_bar: SearchBar::new(),
            scroll_range: None,
            add_mode: false,
            new_entry_name: String::new(),
            add_focus: false,
            last_lowercase_g_pressed_ms: 0,
            terminal_ctx: None,
        };

        // Load bookmarks after initializing the app with the config directory
        app.bookmarks = bookmark_popup::load_bookmarks(app.config_dir_override.as_ref());

        app.refresh_entries();
        app
    }

    pub fn refresh_entries(&mut self) {
        self.tab_manager.refresh_entries();

        // --- Start: Restore Selection Preservation (Post-Sort) ---
        let mut selection_restored = false;
        if let Some(prev_path) = &self.prev_path {
            selection_restored = self.tab_manager.select_child(prev_path)
        };
        if !selection_restored {
            // Default to the first item if selection wasn't restored
            self.tab_manager.reset_selection();
        }
        self.selection_changed = true;
        // Clear prev_path after attempting to use it
        self.prev_path = None;

        // Always ensure selection is visible and invalidate preview cache
        self.ensure_selected_visible = true;
        self.cached_preview_path = None; // Invalidate preview cache
    }

    pub fn set_selection(&mut self, index: usize) {
        let tab = self.tab_manager.current_tab();
        if tab.selected_index == index {
            return;
        }
        tab.update_selection(index);
        self.ensure_selected_visible = true;
        self.selection_changed = true;
    }

    pub fn delete_selected_entry(&mut self) {
        let tab = self.tab_manager.current_tab();
        if let Some(entry) = tab.selected_entry() {
            self.entry_to_delete = Some(entry.path.clone());
            self.show_delete_confirm = true;
        }
    }

    pub fn rename_selected_entry(&mut self) {
        let tab = self.tab_manager.current_tab();
        if let Some(entry) = tab.selected_entry() {
            self.new_name = entry.name.clone();
            self.rename_mode = true;
            self.rename_focus = true;
        }
    }

    fn get_selected_entries(&mut self) -> Vec<PathBuf> {
        let tab = self.tab_manager.current_tab();
        if tab.selected_entries.is_empty() {
            if let Some(entry) = tab.selected_entry() {
                vec![entry.path.clone()]
            } else {
                vec![]
            }
        } else {
            tab.selected_entries.iter().cloned().collect()
        }
    }

    pub fn cut_selected_entries(&mut self) {
        let paths = self.get_selected_entries();
        if !paths.is_empty() {
            self.clipboard = Some((paths, true));
        }
    }

    pub fn copy_selected_entries(&mut self) {
        let paths = self.get_selected_entries();
        if !paths.is_empty() {
            self.clipboard = Some((paths, false));
        }
    }

    pub fn move_selection(&mut self, delta: isize) {
        let tab = self.tab_manager.current_tab();
        let entries = tab.get_filtered_entries_with_indices(&self.search_bar.query); // Get filtered entries with original indices

        if entries.is_empty() {
            return;
        }

        // Find the current position in the *filtered* list
        let current_filtered_index = entries
            .iter()
            .position(|(_, original_index)| *original_index == tab.selected_index);

        if let Some(current_idx) = current_filtered_index {
            let new_filtered_index = current_idx as isize + delta;

            // Clamp the new index to the bounds of the filtered list
            if new_filtered_index >= 0 && new_filtered_index < entries.len() as isize {
                // Get the original index from the new position in the filtered list
                let new_original_index = entries[new_filtered_index as usize].1;
                tab.update_selection(new_original_index);
                self.ensure_selected_visible = true;
                self.selection_changed = true;
            }
        } else {
            // If the current selection is not in the filtered list (e.g., after filter change),
            // select the first item in the filtered list.
            if let Some((_, first_original_index)) = entries.first() {
                tab.update_selection(*first_original_index);
                self.ensure_selected_visible = true;
                self.selection_changed = true;
            }
        }
    }

    pub fn navigate_to_dir(&mut self, mut path: PathBuf) {
        let tab = self.tab_manager.current_tab();
        // Swap current_path with path and store the swapped path as prev_path
        std::mem::swap(&mut tab.current_path, &mut path);
        self.prev_path = Some(path);
        // Reset scroll_range to None when navigating to a new directory
        self.scroll_range = None;
        self.search_bar.close();
        self.refresh_entries();
    }

    pub fn open_file(&mut self, path: PathBuf) {
        if let Err(e) = open::that(&path) {
            eprintln!("Failed to open file: {e}");
        }
    }

    pub fn process_input(&mut self, ctx: &egui::Context) {
        // Let terminal widget process all the inputs
        if self.terminal_ctx.is_some() {
            return;
        }

        // Prioritize Add Mode Input
        if add_entry_popup::handle_key_press(ctx, self) {
            return;
        }

        // Prioritize Search Mode Input
        if search_bar::handle_key_press(ctx, self) {
            return;
        }

        // Don't process other keyboard input if the bookmark popup is active
        if self.show_bookmarks {
            return;
        }

        input::process_input_events(self, ctx);
    }

    fn calculate_panel_widths(&self, available_width: f32) -> (f32, f32, f32) {
        let total_spacing = (PANEL_SPACING * 2.0) +                    // Space between panels
                          (SEPARATOR_PADDING * 4.0) +                  // Padding around two separators
                          PANEL_SPACING +                             // Right margin
                          8.0; // Margins from both sides

        let usable_width = available_width - total_spacing;
        let left_width = (usable_width * LEFT_PANEL_RATIO).max(LEFT_PANEL_MIN_WIDTH);
        let right_width = (usable_width * RIGHT_PANEL_RATIO).max(RIGHT_PANEL_MIN_WIDTH);
        let center_width = usable_width - left_width - right_width;

        (left_width, center_width, right_width)
    }

    pub fn confirm_delete(&mut self) {
        if let Some(path) = self.entry_to_delete.clone() {
            DeleteDialog::perform_delete(&path, || {
                self.refresh_entries();
            });
        }
        self.show_delete_confirm = false;
        self.entry_to_delete = None;
    }

    pub fn cancel_delete(&mut self) {
        self.show_delete_confirm = false;
        self.entry_to_delete = None;
    }

    fn handle_delete_confirmation(&mut self, ctx: &egui::Context) {
        let mut should_confirm = false;
        let mut should_cancel = false;

        DeleteDialog::handle_delete_confirmation(
            ctx,
            &mut self.show_delete_confirm,
            &self.entry_to_delete,
            &self.colors,
            || should_confirm = true,
            || should_cancel = true,
        );

        if should_confirm {
            self.confirm_delete();
        } else if should_cancel {
            self.cancel_delete();
        }
    }
}

impl eframe::App for Kiorg {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update preview cache only if selection changed
        if self.selection_changed {
            right_panel::update_preview_cache(self, ctx);
            self.selection_changed = false; // Reset flag after update
        }

        terminal::draw(ctx, self);

        self.process_input(ctx);

        // Handle bookmark popup with the new approach
        // Use the bookmark_selected_index field from Kiorg struct
        let bookmark_action = bookmark_popup::show_bookmark_popup(
            ctx,
            &mut self.show_bookmarks,
            &mut self.bookmarks,
            &mut self.bookmark_selected_index,
        );
        // Process the bookmark action
        match bookmark_action {
            bookmark_popup::BookmarkAction::Navigate(path) => self.navigate_to_dir(path),
            bookmark_popup::BookmarkAction::SaveBookmarks => {
                // Save bookmarks when the popup signals a change (e.g., deletion)
                if let Err(e) = bookmark_popup::save_bookmarks(
                    &self.bookmarks,
                    self.config_dir_override.as_ref(),
                ) {
                    eprintln!("Failed to save bookmarks: {}", e);
                }
            }
            bookmark_popup::BookmarkAction::None => {}
        };

        // Show delete confirmation window if needed
        // NOTE: important to keep it before the center panel so the popup can
        // be triggered through the right click context menu
        self.handle_delete_confirmation(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            let total_available_height = ui.available_height();

            // Draw top banner and measure its height
            let top_banner_response = ui.scope(|ui| {
                top_banner::draw(self, ui);
            });
            let top_banner_height = top_banner_response.response.rect.height();

            // Calculate panel widths
            let (left_width, center_width, right_width) =
                self.calculate_panel_widths(ui.available_width());

            // Calculate content height based on actual top banner height
            let content_height = total_available_height - top_banner_height;

            // Main panels layout
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = PANEL_SPACING;
                ui.set_min_height(content_height);

                // Call the new left_panel::draw function directly
                if let Some(path) = left_panel::draw(self, ui, left_width, content_height) {
                    self.navigate_to_dir(path);
                }
                separator::draw_vertical_separator(ui);
                center_panel::draw(self, ui, center_width, content_height);
                separator::draw_vertical_separator(ui);
                // Call the new right_panel::draw function directly
                right_panel::draw(self, ui, right_width, content_height);
                ui.add_space(PANEL_SPACING);
            });
        });

        search_bar::draw(ctx, self);

        // Show add entry popup if needed
        if self.add_mode {
            add_entry_popup::draw(ctx, self);
        }

        // Show help window if needed
        if self.show_help {
            help_window::show_help_window(ctx, &mut self.show_help, &self.colors);
        }

        // Show exit confirmation window if needed
        if self.show_exit_confirm {
            // Call the refactored dialog function
            dialogs::show_exit_dialog(ctx, &mut self.show_exit_confirm, &self.colors);
        }
    }
}
