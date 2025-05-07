use notify::RecursiveMode;
use notify::Watcher;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::models::preview_content::PreviewContent;

/// Popup types that can be shown in the application
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupType {
    About,
    Help,
    Exit,
    Delete,
    Rename,
}

/// Clipboard operation types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Clipboard {
    Copy(Vec<PathBuf>),
    Cut(Vec<PathBuf>),
}

// Constants
const STATE_FILE_NAME: &str = "state.json";

use crate::config::{self, colors::AppColors};
use crate::input;
use crate::models::tab::{TabManager, TabManagerState};
use crate::ui::add_entry_popup; // Import the new module
use crate::ui::delete_popup::{self, DeleteConfirmResult, DeleteConfirmState};
use crate::ui::exit_popup;
use crate::ui::search_bar::{self, SearchBar};
use crate::ui::separator;
use crate::ui::separator::SEPARATOR_PADDING;
use crate::ui::terminal;
use crate::ui::top_banner;
use crate::ui::{
    about_popup, bookmark_popup, center_panel, help_window, left_panel, rename_popup, right_panel,
};
use egui_notify::Toasts;

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
                Err(e) => {
                    eprintln!("File system watcher error: {}", e);
                }
            }
        }
    });
    (fs_watcher, notify_fs_change)
}

/// Serializable app state structure
#[derive(Serialize, Deserialize)]
pub struct AppState {
    pub tab_manager: TabManagerState,
    // Add more fields here in the future
}

pub struct Kiorg {
    // Tab manager for file navigation
    pub tab_manager: TabManager,

    // Fields moved from AppState
    pub bookmarks: Vec<PathBuf>,
    pub config_dir_override: Option<PathBuf>,

    // Application configuration
    pub config: config::Config,

    // Application colors
    pub colors: AppColors,

    // Toast notifications
    pub toasts: Toasts,

    // Fields that get reset after refresh_entries
    pub selection_changed: bool, // Flag to track if selection changed
    pub ensure_selected_visible: bool,
    pub prev_path: Option<PathBuf>, // Previous path for selection preservation
    pub cached_preview_path: Option<PathBuf>,
    pub preview_content: Option<PreviewContent>,

    // fields that get reset after changing directories
    // TODO: will it crash the app if large amount of entries are deleted in the same dir?
    pub scroll_range: Option<std::ops::Range<usize>>,

    // Popup management
    pub show_popup: Option<PopupType>,

    // TODO: move new_name into PopupType::Rename?
    pub new_name: String,
    pub bookmark_selected_index: usize, // Store bookmark selection index in app state
    pub entry_to_delete: Option<PathBuf>,
    pub delete_popup_state: DeleteConfirmState, // State for delete confirmation popup
    pub clipboard: Option<Clipboard>,
    pub show_bookmarks: bool,
    pub search_bar: SearchBar,
    pub terminal_ctx: Option<terminal::TerminalContext>,
    pub shutdown_requested: bool,
    pub notify_fs_change: Arc<AtomicBool>,
    pub fs_watcher: notify::RecommendedWatcher,

    pub new_entry_name: Option<String>, // None when not in add mode, Some when in add mode

    // Track files that are currently being opened
    pub files_being_opened: HashMap<PathBuf, Arc<AtomicBool>>,

    // Error channel for background operations
    pub error_sender: std::sync::mpsc::Sender<String>,
    pub error_receiver: std::sync::mpsc::Receiver<String>,

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
        let config = config::load_config_with_override(config_dir_override.as_ref())
            .expect("Invalid config");
        let colors = match &config.colors {
            Some(color_scheme) => AppColors::from_config(color_scheme),
            None => AppColors::default(),
        };

        cc.egui_ctx.set_visuals(colors.to_visuals());

        // Determine the initial path and tab manager
        let (tab_manager, initial_path) = match initial_dir {
            // If initial directory is provided, use it
            Some(path) => {
                let tab_manager = TabManager::new_with_config(path.clone(), Some(&config));
                (tab_manager, path)
            }
            // If no initial directory is provided, try to load from saved state
            None => {
                if let Some(tab_manager) = Self::load_app_state(config_dir_override.as_ref()) {
                    // Use the saved state's path
                    let path = tab_manager.current_tab_ref().current_path.clone();
                    (tab_manager, path)
                } else {
                    // No saved state, use current directory
                    let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    let tab_manager = TabManager::new_with_config(path.clone(), Some(&config));
                    (tab_manager, path)
                }
            }
        };

        // Create file system watcher
        let (fs_watcher, notify_fs_change) = create_fs_watcher(initial_path.as_path());

        // Load bookmarks
        let bookmarks = bookmark_popup::load_bookmarks(config_dir_override.as_ref());

        // Create a channel for error messages
        let (tx, rx) = std::sync::mpsc::channel::<String>();
        let error_sender = tx;
        let error_receiver = rx;

        let mut app = Self {
            tab_manager,
            bookmarks,
            config_dir_override, // Use the provided config_dir_override
            config,              // Store the loaded config
            colors,              // Add the colors field here
            toasts: Toasts::default().with_anchor(egui_notify::Anchor::BottomLeft),
            selection_changed: true,
            ensure_selected_visible: false,
            prev_path: None,
            cached_preview_path: None,
            preview_content: None,
            scroll_range: None,
            show_popup: None,
            new_name: String::new(),
            clipboard: None,
            entry_to_delete: None,
            delete_popup_state: DeleteConfirmState::Initial, // Initialize delete popup state
            show_bookmarks: false,
            bookmark_selected_index: 0,
            search_bar: SearchBar::new(),
            new_entry_name: None,
            files_being_opened: HashMap::new(),
            error_sender,
            error_receiver,
            last_lowercase_g_pressed_ms: 0,
            terminal_ctx: None,
            shutdown_requested: false,
            notify_fs_change,
            fs_watcher,
        };

        app.refresh_entries();
        app
    }

    pub fn refresh_entries(&mut self) {
        self.tab_manager.refresh_entries();

        // --- Start: Restore Selection Preservation (Post-Sort) ---
        if let Some(prev_path) = &self.prev_path {
            self.tab_manager.select_child(prev_path);
        }
        self.selection_changed = true;
        // Clear prev_path after attempting to use it
        self.prev_path = None;

        // Always ensure selection is visible and invalidate preview cache
        self.ensure_selected_visible = true;
        self.cached_preview_path = None; // Invalidate preview cache
    }

    pub fn set_selection(&mut self, index: usize) {
        let tab = self.tab_manager.current_tab_mut();
        if tab.selected_index == index {
            return;
        }
        tab.update_selection(index);
        self.ensure_selected_visible = true;
        self.selection_changed = true;
    }

    pub fn delete_selected_entry(&mut self) {
        let tab = self.tab_manager.current_tab_mut();
        if let Some(entry) = tab.selected_entry() {
            self.entry_to_delete = Some(entry.path.clone());
            self.show_popup = Some(PopupType::Delete);
        }
    }

    pub fn rename_selected_entry(&mut self) {
        let tab = self.tab_manager.current_tab_mut();
        if let Some(entry) = tab.selected_entry() {
            self.new_name = entry.name.clone();
            self.show_popup = Some(PopupType::Rename);
        }
    }

    fn get_marked_entries(&mut self) -> Vec<PathBuf> {
        let tab = self.tab_manager.current_tab_mut();
        if tab.marked_entries.is_empty() {
            if let Some(entry) = tab.selected_entry() {
                vec![entry.path.clone()]
            } else {
                vec![]
            }
        } else {
            tab.marked_entries.iter().cloned().collect()
        }
    }

    pub fn cut_selected_entries(&mut self) {
        let tab = self.tab_manager.current_tab_mut();

        // Check if we're cutting a single unmarked file while other files are marked
        let should_clear_marked = if let Some(entry) = tab.selected_entry() {
            let selected_path = &entry.path;

            // If the selected file is not marked but there are other marked files
            !tab.marked_entries.contains(selected_path) && !tab.marked_entries.is_empty()
        } else {
            false
        };

        if should_clear_marked {
            // Get the selected entry path before clearing marked entries
            let selected_path = tab.selected_entry().unwrap().path.clone();

            // Clear all marked entries
            tab.marked_entries.clear();

            // Update the cut buffer with only the newly selected file
            self.clipboard = Some(Clipboard::Cut(vec![selected_path]));
            return;
        }

        // Otherwise, proceed with the normal behavior
        let paths = self.get_marked_entries();
        if !paths.is_empty() {
            self.clipboard = Some(Clipboard::Cut(paths));
        }
    }

    pub fn copy_selected_entries(&mut self) {
        let tab = self.tab_manager.current_tab_mut();

        // Check if we're copying a single unmarked file while other files are marked
        let should_clear_marked = if let Some(entry) = tab.selected_entry() {
            let selected_path = &entry.path;

            // If the selected file is not marked but there are other marked files
            !tab.marked_entries.contains(selected_path) && !tab.marked_entries.is_empty()
        } else {
            false
        };

        if should_clear_marked {
            // Get the selected entry path before clearing marked entries
            let selected_path = tab.selected_entry().unwrap().path.clone();

            // Clear all marked entries
            tab.marked_entries.clear();

            // Update the clipboard with only the newly selected file
            self.clipboard = Some(Clipboard::Copy(vec![selected_path]));
            return;
        }

        // Otherwise, proceed with the normal behavior
        let paths = self.get_marked_entries();
        if !paths.is_empty() {
            self.clipboard = Some(Clipboard::Copy(paths));
        }
    }

    pub fn move_selection(&mut self, delta: isize) {
        let tab = self.tab_manager.current_tab_mut();
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

    fn navigate_to_dir_without_history(&mut self, mut path: PathBuf) {
        let tab = self.tab_manager.current_tab_mut();
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

    pub fn navigate_to_dir(&mut self, path: PathBuf) {
        self.navigate_to_dir_without_history(path.clone());
        self.tab_manager.current_tab_mut().add_to_history(path);
    }

    pub fn navigate_history_back(&mut self) {
        let tab = self.tab_manager.current_tab_mut();
        if let Some(path) = tab.history_back() {
            self.navigate_to_dir_without_history(path);
        }
    }

    pub fn navigate_history_forward(&mut self) {
        let tab = self.tab_manager.current_tab_mut();
        if let Some(path) = tab.history_forward() {
            self.navigate_to_dir_without_history(path);
        }
    }

    pub fn open_file(&mut self, path: PathBuf) {
        // Add the file to the list of files being opened
        let signal = Arc::new(AtomicBool::new(true));
        self.files_being_opened.insert(path.clone(), signal.clone());

        // Clone the path for the thread
        let path_clone = path.clone();

        // Clone the error sender for the thread
        let error_sender = self.error_sender.clone();

        // Spawn a thread to open the file asynchronously
        std::thread::spawn(move || match open::that(&path_clone) {
            Ok(_) => {
                signal.store(false, std::sync::atomic::Ordering::Relaxed);
            }
            Err(e) => {
                // Send the error message back to the main thread
                let _ = error_sender.send(format!("Failed to open file: {}", e));
            }
        });
    }

    pub fn process_input(&mut self, ctx: &egui::Context) {
        // Let terminal widget process all the inputs
        if self.terminal_ctx.is_some() {
            return;
        }

        // Prioritize Add Mode Input
        if self.new_entry_name.is_some() && add_entry_popup::handle_key_press(ctx, self) {
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

    fn handle_delete_confirmation(&mut self, ctx: &egui::Context) {
        if self.show_popup != Some(PopupType::Delete) || self.entry_to_delete.is_none() {
            return;
        }

        let mut show_delete_confirm = true; // Temporary variable for compatibility

        let result = delete_popup::handle_delete_confirmation(
            ctx,
            &mut show_delete_confirm,
            &self.entry_to_delete,
            &self.colors,
            &mut self.delete_popup_state,
        );

        if !show_delete_confirm {
            self.show_popup = None;
        }

        match result {
            DeleteConfirmResult::Confirm => {
                delete_popup::confirm_delete(self);
            }
            DeleteConfirmResult::Cancel => {
                delete_popup::cancel_delete(self);
            }
            DeleteConfirmResult::None => {
                // No action taken yet
            }
        }
    }

    fn graceful_shutdown(&mut self, ctx: &egui::Context) {
        // Save application state before shutting down
        if let Err(e) = self.save_app_state() {
            self.toasts
                .error(format!("Failed to save application state: {}", e));
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn save_app_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_dir = config::get_kiorg_config_dir(self.config_dir_override.as_ref());

        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
        }

        // Save app state with tab_manager as a top-level key
        let state_path = config_dir.join(STATE_FILE_NAME);
        let app_state = AppState {
            tab_manager: self.tab_manager.to_state(),
            // Add more fields here in the future
        };
        let state_json = serde_json::to_string_pretty(&app_state)?;
        std::fs::write(&state_path, state_json)?;

        Ok(())
    }

    fn load_app_state(config_dir_override: Option<&PathBuf>) -> Option<TabManager> {
        let config_dir = config::get_kiorg_config_dir(config_dir_override);
        let state_path = config_dir.join(STATE_FILE_NAME);

        if !state_path.exists() {
            return None;
        }

        match std::fs::read_to_string(&state_path) {
            Ok(json_str) => {
                // First try to parse as the new format (AppState)
                match serde_json::from_str::<AppState>(&json_str) {
                    Ok(app_state) => {
                        // Convert TabManagerState to TabManager
                        let tab_manager = TabManager::from_state(app_state.tab_manager);
                        Some(tab_manager)
                    }
                    Err(_) => {
                        // If that fails, try the old format (direct TabManagerState)
                        match serde_json::from_str::<TabManagerState>(&json_str) {
                            Ok(tab_manager_state) => {
                                // Convert TabManagerState to TabManager
                                let tab_manager = TabManager::from_state(tab_manager_state);
                                Some(tab_manager)
                            }
                            Err(e) => {
                                eprintln!("Failed to parse app state: {}", e);
                                None
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read app state file: {}", e);
                None
            }
        }
    }
}

impl eframe::App for Kiorg {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Store shortcuts in the context for the help window to access
        if let Some(shortcuts) = &self.config.shortcuts {
            ctx.data_mut(|d| d.insert_temp(egui::Id::new("shortcuts"), shortcuts.clone()));
        }

        if self
            .notify_fs_change
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            self.refresh_entries();
            self.notify_fs_change
                .store(false, std::sync::atomic::Ordering::Relaxed);
        }

        // Check for error messages from background threads
        // Process all pending error messages
        while let Ok(error_message) = self.error_receiver.try_recv() {
            // Add the error to the toasts
            self.toasts.error(error_message);
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
                    self.toasts
                        .error(format!("Failed to save bookmarks: {}", e));
                }
            }
            bookmark_popup::BookmarkAction::None => {}
        };

        // Show delete confirmation window if needed
        // TODO: write a test for triggering delete popup from right click
        // NOTE: important to keep it before the center panel so the popup can
        // be triggered through the right click context menu
        //
        // Handle popups based on the show_popup field
        match self.show_popup {
            Some(PopupType::Help) => {
                let mut keep_open = true;
                help_window::show_help_window(ctx, &mut keep_open, &self.colors);
                if !keep_open {
                    self.show_popup = None;
                }
            }
            Some(PopupType::About) => {
                about_popup::show_about_popup(ctx, self);
            }
            Some(PopupType::Exit) => {
                let mut keep_open = true;
                exit_popup::show(ctx, &mut keep_open, &self.colors);
                if !keep_open {
                    self.show_popup = None;
                }
            }
            Some(PopupType::Delete) => {
                self.handle_delete_confirmation(ctx);
            }
            Some(PopupType::Rename) => {
                // Draw the rename popup
                rename_popup::draw(ctx, self);
            }
            None => {}
        }

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

            // Main panels layout
            ui.horizontal(|ui| {
                let container_height = total_available_height - top_banner_height;
                ui.spacing_mut().item_spacing.x = PANEL_SPACING;
                ui.set_min_height(container_height);

                let content_height =
                    container_height - ui.spacing().item_spacing.x * 2.0 - PANEL_SPACING;

                if let Some(path) = left_panel::draw(self, ui, left_width, content_height) {
                    self.navigate_to_dir(path);
                }
                separator::draw_vertical_separator(ui);

                center_panel::draw(self, ui, center_width, content_height);
                separator::draw_vertical_separator(ui);

                right_panel::draw(self, ctx, ui, right_width, content_height);
                ui.add_space(PANEL_SPACING);
            });
        });

        search_bar::draw(ctx, self);

        // Show add entry popup if needed
        if self.new_entry_name.is_some() {
            add_entry_popup::draw(ctx, self);
        }

        if self.shutdown_requested {
            self.graceful_shutdown(ctx);
        }

        // Draw toast notifications
        self.toasts.show(ctx);
    }
}
