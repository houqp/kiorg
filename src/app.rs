use egui::TextureHandle;
use notify::RecursiveMode;
use notify::Watcher;
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

// Constants
const STATE_FILE_NAME: &str = "state.json";

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

fn create_fs_watcher(watch_dir: &Path) -> (notify::RecommendedWatcher, Arc<AtomicBool>) {
    let notify_fs_change = Arc::new(AtomicBool::new(false));
    let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
    let mut fs_watcher = notify::recommended_watcher(tx).expect("Failed to create watcher");
    fs_watcher
        .watch(watch_dir, RecursiveMode::NonRecursive)
        .expect("Failed to watch path");
    let notify_fs_change_clone = notify_fs_change.clone();
    std::thread::spawn(move || loop {
        for res in &rx {
            match res {
                Ok(event) => match event.kind {
                    notify::EventKind::Remove(_)
                    | notify::EventKind::Modify(_)
                    | notify::EventKind::Create(_) => {
                        notify_fs_change_clone.store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                    _ => {}
                },
                // TODO: print error in console
                Err(_e) => {}
            }
        }
    });
    (fs_watcher, notify_fs_change)
}

/// Application state that can be serialized and persisted between app restarts
#[derive(Serialize, Deserialize)]
pub struct AppState {
    pub tab_manager: TabManager,
    pub bookmarks: Vec<PathBuf>,
    pub config_dir_override: Option<PathBuf>, // Optional override for config directory path
}

impl AppState {
    pub fn new(tab_manager: TabManager, config_dir_override: Option<PathBuf>) -> Self {
        Self {
            tab_manager,
            bookmarks: Vec::new(),
            config_dir_override,
        }
    }
}

pub struct Kiorg {
    // Serializable app state
    pub state: AppState,

    // Application colors
    pub colors: AppColors,

    // Fields that get reset after refresh_entries
    pub selection_changed: bool, // Flag to track if selection changed
    pub ensure_selected_visible: bool,
    pub prev_path: Option<PathBuf>, // Previous path for selection preservation
    pub cached_preview_path: Option<PathBuf>,
    pub preview_content: String,
    pub current_image: Option<TextureHandle>,

    // fields that get reset after changing directories
    // TODO: will it crash the app if large amount of entries are deleted in the same dir?
    pub scroll_range: Option<std::ops::Range<usize>>,

    // TODO: replace rename_mode with Option<new_name>?
    pub new_name: String,
    pub rename_mode: bool,
    pub bookmark_selected_index: usize, // Store bookmark selection index in app state
    pub entry_to_delete: Option<PathBuf>,
    pub show_delete_confirm: bool,
    pub clipboard: Option<(Vec<PathBuf>, bool)>, // (paths, is_cut)
    pub show_bookmarks: bool,
    pub search_bar: SearchBar,
    pub show_exit_confirm: bool,
    pub terminal_ctx: Option<terminal::TerminalContext>,
    pub show_help: bool,
    pub shutdown_requested: bool,
    pub notify_fs_change: Arc<AtomicBool>,
    pub fs_watcher: notify::RecommendedWatcher,

    pub add_mode: bool,
    pub new_entry_name: String, // name for newly created file/directory
    // TODO: is this neeeded if we already have add_mode?
    pub add_focus: bool,

    // ts variable for tracking key press times
    pub last_lowercase_g_pressed_ms: u64,
}

impl Kiorg {
    pub fn new(cc: &eframe::CreationContext<'_>, initial_dir: Option<PathBuf>) -> Self {
        Self::new_with_config_dir(cc, initial_dir, None)
    }

    pub fn new_with_config_dir(
        cc: &eframe::CreationContext<'_>,
        initial_dir: Option<PathBuf>,
        config_dir_override: Option<PathBuf>,
    ) -> Self {
        let config = config::load_config_with_override(config_dir_override.as_ref());
        let colors = AppColors::from_config(&config.colors);

        cc.egui_ctx.set_visuals(colors.to_visuals());

        // Determine the initial path and whether to use saved state
        let (use_saved_state, initial_path) = match initial_dir {
            // If initial directory is provided, use it
            Some(path) => (None, path),

            // If no initial directory is provided, try to load from saved state
            None => {
                if let Some(saved_state) = Self::load_app_state(config_dir_override.as_ref()) {
                    // Use the saved state's path
                    let path = saved_state
                        .tab_manager
                        .current_tab_ref()
                        .current_path
                        .clone();
                    (Some(saved_state), path)
                } else {
                    // No saved state, use current directory
                    let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    (None, path)
                }
            }
        };

        // Create file system watcher
        let (fs_watcher, notify_fs_change) = create_fs_watcher(initial_path.as_path());

        // Create the app with common initialization
        let mut app = Self {
            state: match use_saved_state {
                Some(saved_state) => saved_state,
                None => {
                    let tab_manager = TabManager::new_with_config(initial_path, Some(&config));
                    let mut app_state = AppState::new(tab_manager, config_dir_override.clone());

                    // Load bookmarks for new state
                    app_state.bookmarks =
                        bookmark_popup::load_bookmarks(app_state.config_dir_override.as_ref());
                    app_state
                }
            },
            colors, // Add the colors field here
            current_image: None,
            selection_changed: true,
            ensure_selected_visible: false,
            prev_path: None,
            cached_preview_path: None,
            preview_content: String::new(),
            scroll_range: None,
            rename_mode: false,
            new_name: String::new(),
            clipboard: None,
            show_delete_confirm: false,
            entry_to_delete: None,
            show_bookmarks: false,
            bookmark_selected_index: 0,
            search_bar: SearchBar::new(),
            show_exit_confirm: false,
            add_mode: false,
            new_entry_name: String::new(),
            add_focus: false,
            last_lowercase_g_pressed_ms: 0,
            terminal_ctx: None,
            show_help: false,
            shutdown_requested: false,
            notify_fs_change,
            fs_watcher,
        };

        app.refresh_entries();
        app
    }

    pub fn refresh_entries(&mut self) {
        self.state.tab_manager.refresh_entries();

        // --- Start: Restore Selection Preservation (Post-Sort) ---
        let mut selection_restored = false;
        if let Some(prev_path) = &self.prev_path {
            selection_restored = self.state.tab_manager.select_child(prev_path)
        };
        if !selection_restored {
            // Default to the first item if selection wasn't restored
            self.state.tab_manager.reset_selection();
        }
        self.selection_changed = true;
        // Clear prev_path after attempting to use it
        self.prev_path = None;

        // Always ensure selection is visible and invalidate preview cache
        self.ensure_selected_visible = true;
        self.cached_preview_path = None; // Invalidate preview cache
    }

    pub fn set_selection(&mut self, index: usize) {
        let tab = self.state.tab_manager.current_tab();
        if tab.selected_index == index {
            return;
        }
        tab.update_selection(index);
        self.ensure_selected_visible = true;
        self.selection_changed = true;
    }

    pub fn delete_selected_entry(&mut self) {
        let tab = self.state.tab_manager.current_tab();
        if let Some(entry) = tab.selected_entry() {
            self.entry_to_delete = Some(entry.path.clone());
            self.show_delete_confirm = true;
        }
    }

    pub fn rename_selected_entry(&mut self) {
        let tab = self.state.tab_manager.current_tab();
        if let Some(entry) = tab.selected_entry() {
            self.new_name = entry.name.clone();
            self.rename_mode = true;
        }
    }

    fn get_selected_entries(&mut self) -> Vec<PathBuf> {
        let tab = self.state.tab_manager.current_tab();
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
        let tab = self.state.tab_manager.current_tab();
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
        let tab = self.state.tab_manager.current_tab();
        // Swap current_path with path and store the swapped path as prev_path
        std::mem::swap(&mut tab.current_path, &mut path);
        self.prev_path = Some(path);
        // Reset scroll_range to None when navigating to a new directory
        self.scroll_range = None;
        self.search_bar.close();
        self.fs_watcher
            .watch(tab.current_path.as_path(), RecursiveMode::NonRecursive)
            .expect("Failed to watch path");
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

    fn graceful_shutdown(&mut self, ctx: &egui::Context) {
        // Save application state before shutting down
        if let Err(e) = self.save_app_state() {
            eprintln!("Failed to save application state: {}", e);
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn save_app_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_dir = config::get_kiorg_config_dir(self.state.config_dir_override.as_ref());

        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
        }

        let state_path = config_dir.join(STATE_FILE_NAME);
        let state_json = serde_json::to_string_pretty(&self.state)?;
        std::fs::write(&state_path, state_json)?;

        Ok(())
    }

    fn load_app_state(config_dir_override: Option<&PathBuf>) -> Option<AppState> {
        let config_dir = config::get_kiorg_config_dir(config_dir_override);
        let state_path = config_dir.join(STATE_FILE_NAME);

        if !state_path.exists() {
            return None;
        }

        match std::fs::read_to_string(&state_path) {
            Ok(json_str) => match serde_json::from_str(&json_str) {
                Ok(state) => Some(state),
                Err(e) => {
                    eprintln!("Failed to parse app state: {}", e);
                    None
                }
            },
            Err(e) => {
                eprintln!("Failed to read app state file: {}", e);
                None
            }
        }
    }
}

impl eframe::App for Kiorg {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self
            .notify_fs_change
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            self.refresh_entries();
            self.notify_fs_change
                .store(false, std::sync::atomic::Ordering::Relaxed);
        }

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
            &mut self.state.bookmarks,
            &mut self.bookmark_selected_index,
        );
        // Process the bookmark action
        match bookmark_action {
            bookmark_popup::BookmarkAction::Navigate(path) => self.navigate_to_dir(path),
            bookmark_popup::BookmarkAction::SaveBookmarks => {
                // Save bookmarks when the popup signals a change (e.g., deletion)
                if let Err(e) = bookmark_popup::save_bookmarks(
                    &self.state.bookmarks,
                    self.state.config_dir_override.as_ref(),
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

        if self.shutdown_requested {
            self.graceful_shutdown(ctx);
        }
    }
}
