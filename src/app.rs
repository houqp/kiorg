use notify::RecursiveMode;
use notify::Watcher;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::config::{self, LEFT_PANEL_RATIO, PREVIEW_PANEL_RATIO, colors::AppColors};
use crate::input;
use crate::models::preview_content::PreviewContent;
use crate::models::tab::{TabManager, TabManagerState};
use crate::ui::egui_notify::Toasts;
use crate::ui::popup::delete::DeleteConfirmResult;
use crate::ui::popup::{
    PopupType, about, add_entry, bookmark, delete, exit, file_drop, open_with,
    preview as popup_preview, rename, teleport, theme,
};
use crate::ui::search_bar::{self, SearchBar};
use crate::ui::separator;
use crate::ui::separator::SEPARATOR_PADDING;
use crate::ui::terminal;
use crate::ui::top_banner;
use crate::ui::update;
use crate::ui::{center_panel, help_window, left_panel, notification, preview, right_panel};
use crate::visit_history::{self, VisitHistoryEntry};

/// Error type for Kiorg application
#[derive(Debug)]
pub enum KiorgError {
    /// Configuration error
    ConfigError(config::ConfigError),
    /// Directory does not exist
    DirectoryNotFound(PathBuf),
    /// Path is not a directory
    NotADirectory(PathBuf),
    /// File system watcher error
    WatcherError(String),
    /// Other error
    Other(String),
}

impl fmt::Display for KiorgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KiorgError::ConfigError(e) => write!(f, "Configuration error: {e}"),
            KiorgError::DirectoryNotFound(path) => {
                write!(f, "Directory not found: {}", path.display())
            }
            KiorgError::NotADirectory(path) => write!(f, "Not a directory: {}", path.display()),
            KiorgError::WatcherError(e) => write!(f, "File system watcher error: {e}"),
            KiorgError::Other(e) => write!(f, "{e}"),
        }
    }
}

impl Error for KiorgError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            KiorgError::ConfigError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<config::ConfigError> for KiorgError {
    fn from(err: config::ConfigError) -> Self {
        KiorgError::ConfigError(err)
    }
}

/// Clipboard operation types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Clipboard {
    Copy(Vec<PathBuf>),
    Cut(Vec<PathBuf>),
}

// Constants
const STATE_FILE_NAME: &str = "state.json";

// Layout constants
const PANEL_SPACING: f32 = 5.0; // Space between panels

fn create_fs_watcher(
    watch_dir: &Path,
) -> Result<(notify::RecommendedWatcher, Arc<AtomicBool>), std::io::Error> {
    let notify_fs_change = Arc::new(AtomicBool::new(false));
    let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();

    let mut fs_watcher = match notify::recommended_watcher(tx) {
        Ok(watcher) => watcher,
        Err(e) => return Err(std::io::Error::other(e.to_string())),
    };

    if let Err(e) = fs_watcher.watch(watch_dir, RecursiveMode::NonRecursive) {
        return Err(std::io::Error::other(format!("Failed to watch path: {e}")));
    }

    let notify_fs_change_clone = notify_fs_change.clone();
    std::thread::spawn(move || {
        loop {
            for res in &rx {
                match res {
                    Ok(event) => match event.kind {
                        notify::EventKind::Remove(_)
                        | notify::EventKind::Modify(_)
                        | notify::EventKind::Create(_) => {
                            notify_fs_change_clone
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                        _ => {}
                    },
                    Err(e) => {
                        eprintln!("File system watcher error: {e}");
                    }
                }
            }
        }
    });

    Ok((fs_watcher, notify_fs_change))
}

/// Returns the fallback directory path to use when no valid path is available.
/// Uses the user's home directory, with a fallback to "." if that fails.
fn fallback_initial_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
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
    // Merged shortcuts (defaults + user overrides) for runtime use
    pub merged_shortcuts: config::shortcuts::Shortcuts,
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
    pub clipboard: Option<Clipboard>,
    pub search_bar: SearchBar,
    pub terminal_ctx: Option<terminal::TerminalContext>,
    pub notify_fs_change: Arc<AtomicBool>,
    pub fs_watcher: notify::RecommendedWatcher,
    // Track files that are currently being opened
    pub files_being_opened: HashMap<PathBuf, Arc<AtomicBool>>,
    // Async notification system for background operations
    pub notification_system: notification::AsyncNotification,
    // Key buffer for tracking unprocessed key presses
    pub key_buffer: Vec<egui::Key>,
    pub shutdown_requested: bool,
    // Signal whether to scroll to display current directory in the left panel
    pub scroll_left_panel: bool,
    // Global visit history tracking
    pub visit_history: HashMap<PathBuf, VisitHistoryEntry>,
    // Async history saver for non-blocking save operations
    pub history_saver: visit_history::HistorySaver,
}

impl Kiorg {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        initial_dir: Option<PathBuf>,
    ) -> Result<Self, KiorgError> {
        Self::new_with_config_dir(cc, initial_dir, None)
    }

    pub fn new_with_config_dir(
        cc: &eframe::CreationContext<'_>,
        initial_dir: Option<PathBuf>,
        config_dir_override: Option<PathBuf>,
    ) -> Result<Self, KiorgError> {
        let config = config::load_config_with_override(config_dir_override.as_ref())?;

        // Create merged shortcuts: start with defaults and apply user overrides
        let mut merged_shortcuts = config::shortcuts::default_shortcuts();
        if let Some(user_shortcuts) = &config.shortcuts {
            // Apply user shortcuts over defaults
            for (action, shortcuts_list) in user_shortcuts {
                merged_shortcuts.set_shortcuts(*action, shortcuts_list.clone());
            }
        }

        // Load colors based on theme name from config
        let colors = crate::theme::Theme::load_colors_from_config(&config);
        cc.egui_ctx.set_visuals(colors.to_visuals());

        // Determine the initial path and tab manager
        let (tab_manager, initial_path) = match initial_dir {
            // If initial directory is provided, use it
            Some(path) => {
                // For explicitly provided paths, validate and return error if invalid
                if !path.exists() {
                    return Err(KiorgError::DirectoryNotFound(path.clone()));
                }
                if !path.is_dir() {
                    return Err(KiorgError::NotADirectory(path.clone()));
                }

                let tab_manager = TabManager::new_with_config(path.clone(), Some(&config));
                (tab_manager, path)
            }
            // If no initial directory is provided, try to load from saved state
            None => {
                if let Some(tab_manager) = Self::load_app_state(config_dir_override.as_ref()) {
                    // Use the saved state's path
                    let path = tab_manager.current_tab_ref().current_path.clone();

                    // Verify that the saved path still exists
                    if !path.exists() || !path.is_dir() {
                        // If saved path doesn't exist, fall back to home directory
                        tracing::error!(
                            "Saved path in state '{}' is invalid, falling back to home directory",
                            path.display()
                        );
                        let fallback_path = fallback_initial_dir();
                        let fallback_tab_manager =
                            TabManager::new_with_config(fallback_path.clone(), Some(&config));
                        (fallback_tab_manager, fallback_path)
                    } else {
                        (tab_manager, path)
                    }
                } else {
                    // No saved state, use fallback directory
                    let path = fallback_initial_dir();
                    let tab_manager = TabManager::new_with_config(path.clone(), Some(&config));
                    (tab_manager, path)
                }
            }
        };

        let (fs_watcher, notify_fs_change) = match create_fs_watcher(initial_path.as_path()) {
            Ok(watcher) => watcher,
            Err(e) => return Err(KiorgError::WatcherError(e.to_string())),
        };

        let bookmarks = bookmark::load_bookmarks(config_dir_override.as_ref());

        // Load visit history
        let visit_history = visit_history::load_visit_history(config_dir_override.as_ref())
            .unwrap_or_else(|e| {
                tracing::error!(err =? e, "Failed to load visit history");
                HashMap::new()
            });

        // Create async notification system
        let notification_system = notification::AsyncNotification::default();

        // Create async history saver
        let history_saver = visit_history::HistorySaver::new();

        let mut app = Self {
            tab_manager,
            bookmarks,
            config_dir_override, // Use the provided config_dir_override
            config,              // Store the loaded config
            merged_shortcuts,    // Initialize merged_shortcuts
            colors,              // Add the colors field here
            toasts: Toasts::default().with_anchor(crate::ui::egui_notify::Anchor::BottomLeft),
            selection_changed: true,
            ensure_selected_visible: false,
            prev_path: None,
            cached_preview_path: None,
            preview_content: None,
            scroll_range: None,
            show_popup: None,
            clipboard: None,
            search_bar: SearchBar::new(),
            files_being_opened: HashMap::new(),
            notification_system,
            key_buffer: Vec::new(),
            terminal_ctx: None,
            shutdown_requested: false,
            notify_fs_change,
            scroll_left_panel: false,
            fs_watcher,
            visit_history,
            history_saver,
        };

        app.refresh_entries();
        Ok(app)
    }

    /// Display an error notification with a consistent timeout
    pub fn notify_error<T: ToString>(&mut self, message: T) {
        notification::notify_error(&mut self.toasts, message);
    }

    /// Display an info notification with a consistent timeout
    pub fn notify_info<T: ToString>(&mut self, message: T) {
        notification::notify_info(&mut self.toasts, message);
    }

    /// Display a success notification with a consistent timeout
    pub fn notify_success<T: ToString>(&mut self, message: T) {
        notification::notify_success(&mut self.toasts, message);
    }

    /// Check and process notification messages from background operations
    pub fn check_notifications(&mut self) {
        notification::check_notifications(self);
    }

    /// Get shortcuts from config or use defaults
    /// This method provides a centralized way to access shortcuts configuration
    /// that can be reused across the main input handler and popup components
    pub fn get_shortcuts(&self) -> &crate::config::shortcuts::Shortcuts {
        &self.merged_shortcuts
    }

    /// Extract shortcut action from egui input events
    /// This method provides a centralized way to process keyboard input and convert it to shortcut actions
    /// that can be reused across the main input handler and popup components
    pub fn get_shortcut_action_from_input(
        &self,
        ctx: &egui::Context,
        namespace: bool,
    ) -> Option<crate::config::shortcuts::ShortcutAction> {
        let shortcuts = self.get_shortcuts();
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key {
                    key,
                    modifiers,
                    pressed: true,
                    ..
                } = event
                {
                    if let Some(action) = crate::config::shortcuts::shortcuts_helpers::find_action(
                        shortcuts, *key, *modifiers, namespace,
                    ) {
                        return Some(action);
                    }
                }
            }
            None
        })
    }

    pub fn refresh_entries(&mut self) {
        self.tab_manager.refresh_entries();
        // tab_manager.refresh_entries() will refresh both parent and current directory entries
        // so always refocus left panel after refresh
        self.scroll_left_panel = true;

        // Restore search filter if it was active before refresh
        if self.search_bar.query.is_some() {
            let case_insensitive = self.search_bar.case_insensitive;
            let tab = self.tab_manager.current_tab_mut();
            tab.update_filtered_cache(
                &self.search_bar.query,
                case_insensitive,
                self.search_bar.fuzzy,
            );
        }

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

        if tab.is_range_selection_active() {
            tab.apply_range_selection_to_marked();
            tab.range_selection_start = None;
        }

        let entries_to_delete = if !tab.marked_entries.is_empty() {
            // Use marked entries for bulk deletion
            tab.marked_entries.iter().cloned().collect()
        } else if let Some(entry) = tab.selected_entry() {
            // Fall back to the currently selected entry if no entries are marked
            vec![entry.path.clone()]
        } else {
            // No entries to delete
            return;
        };

        self.show_popup = Some(PopupType::Delete(
            crate::ui::popup::delete::DeleteConfirmState::Initial,
            entries_to_delete,
        ));
    }

    pub fn rename_selected_entry(&mut self) {
        let tab = self.tab_manager.current_tab_mut();
        if let Some(entry) = tab.selected_entry() {
            self.show_popup = Some(PopupType::Rename(entry.name.clone()));
        }
    }

    /// Common logic for copy/cut operations
    /// Returns the paths to operate on, handling range selection and marked entries
    fn prepare_clipboard_operation(&mut self) -> Vec<PathBuf> {
        let tab = self.tab_manager.current_tab_mut();

        // copy/cut exits range selection mode if active
        if tab.is_range_selection_active() {
            tab.apply_range_selection_to_marked();
            tab.range_selection_start = None;
        }

        // Check if we're operating on a single unmarked file while other files are marked
        // In such case, we should reset the marked state
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
            tab.marked_entries.clear();
            vec![selected_path]
        } else {
            // Otherwise, proceed with marked entries
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
    }

    pub fn cut_selected_entries(&mut self) {
        let paths = self.prepare_clipboard_operation();
        if !paths.is_empty() {
            self.clipboard = Some(Clipboard::Cut(paths));
        }
    }

    pub fn copy_selected_entries(&mut self) {
        let paths = self.prepare_clipboard_operation();
        if !paths.is_empty() {
            self.clipboard = Some(Clipboard::Copy(paths));
        }
    }

    pub fn select_all_entries(&mut self) {
        let tab = self.tab_manager.current_tab_mut();
        tab.marked_entries.clear();
        let filtered_entries = tab.get_cached_filtered_entries().clone();
        for (entry, _original_index) in filtered_entries {
            tab.marked_entries.insert(entry.path);
        }
    }

    pub fn move_selection(&mut self, delta: isize) {
        let tab = self.tab_manager.current_tab_mut();
        let entries = tab.get_cached_filtered_entries(); // Get filtered entries with original indices

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

    pub fn move_selection_by_page(&mut self, direction: isize) {
        // Calculate page size from scroll_range if available
        let page_size = if let Some(ref range) = self.scroll_range {
            // Use the visible range size as page size, with a minimum of 1
            (range.end - range.start).max(1) as isize - 1
        } else {
            // Default page size if scroll_range is not available
            10
        };

        // Move by the effective page size in the specified direction
        self.move_selection(direction * page_size);
    }

    fn navigate_to_dir_without_history(&mut self, mut path: PathBuf) {
        let tab = self.tab_manager.current_tab_mut();
        // Swap current_path with path and store the swapped path as prev_path
        std::mem::swap(&mut tab.current_path, &mut path);
        self.prev_path = Some(path);
        // Reset scroll_range to None when navigating to a new directory
        self.scroll_range = None;
        // Exit range selection mode when changing directories
        tab.range_selection_start = None;
        self.search_bar.close();
        // Reset filter when closing search bar
        tab.update_filtered_cache(&None, false, false);

        // Watch the new directory
        if let Err(e) = self
            .fs_watcher
            .watch(tab.current_path.as_path(), RecursiveMode::NonRecursive)
        {
            self.notify_error(format!("Failed to watch directory: {e}"));
        }

        self.refresh_entries();
    }

    pub fn navigate_to_dir(&mut self, path: PathBuf) {
        if !path.exists() || !path.is_dir() {
            if self.visit_history.remove(&path).is_some() {
                // Save updated visit history asynchronously
                self.history_saver
                    .save_async(&self.visit_history, self.config_dir_override.as_ref());
            }
            self.notify_error(format!(
                "Cannot navigate to '{}': Path is not a directory or doesn't exist",
                path.display()
            ));
            return;
        }
        self.navigate_to_dir_without_history(path.clone());

        // Track visit in global history
        visit_history::update_visit_history(&mut self.visit_history, &path);
        // Save visit history asynchronously (non-blocking)
        self.history_saver
            .save_async(&self.visit_history, self.config_dir_override.as_ref());

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

    /// Helper function to handle common file opening logic
    fn open_file_internal<F, E>(&mut self, path: PathBuf, open_fn: F)
    where
        F: FnOnce() -> Result<(), E> + Send + 'static,
        E: std::fmt::Display + 'static,
        String: From<E>,
    {
        // Add the file to the list of files being opened
        let signal = Arc::new(AtomicBool::new(true));
        self.files_being_opened.insert(path.clone(), signal.clone());

        // Clone the notification sender for the thread
        let notification_sender = self.notification_system.get_sender();

        // Spawn a thread to open the file asynchronously
        std::thread::spawn(move || {
            match open_fn() {
                Ok(_) => {}
                Err(e) => {
                    // Send the error message back to the main thread
                    let _ = notification_sender
                        .send(notification::NotificationMessage::Error(format!("{e}")));
                }
            }
            signal.store(false, std::sync::atomic::Ordering::Relaxed);
        });
    }

    /// Open a file with the default application
    pub fn open_file(&mut self, path: PathBuf) {
        let path_clone = path.clone();
        self.open_file_internal(path, move || {
            open::that(&path_clone).map_err(|e| format!("Failed to open file: {e}"))
        });
    }

    /// Open a file with a custom command
    pub fn open_file_with_command(&mut self, path: PathBuf, command: String) {
        let path_clone = path.clone();
        let command_clone = command.clone();
        self.open_file_internal(path, move || {
            open::with(&path_clone, &command_clone)
                .map_err(|e| format!("Failed to open file with '{command_clone}': {e}"))
        });
    }

    pub fn process_input(&mut self, ctx: &egui::Context) {
        // Let terminal widget process all the inputs
        if self.terminal_ctx.is_some() {
            return;
        }

        // Prioritize Search Mode Input
        if search_bar::handle_key_press(ctx, self) {
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
        let left_width = usable_width * LEFT_PANEL_RATIO;
        let right_width = usable_width
            * self
                .config
                .layout
                .as_ref()
                .and_then(|l| l.preview)
                .unwrap_or(PREVIEW_PANEL_RATIO);
        let center_width = usable_width - left_width - right_width;

        (left_width, center_width, right_width)
    }

    fn handle_delete_confirmation(&mut self, ctx: &egui::Context) {
        if let Some(PopupType::Delete(ref mut state, ref entries_to_delete)) = self.show_popup {
            if entries_to_delete.is_empty() {
                return;
            }

            let mut show_delete_confirm = true; // Temporary variable for compatibility

            let result = delete::handle_delete_confirmation(
                ctx,
                &mut show_delete_confirm,
                entries_to_delete,
                &self.colors,
                state,
            );

            if !show_delete_confirm {
                self.show_popup = None;
            }

            match result {
                DeleteConfirmResult::Confirm => {
                    delete::confirm_delete(self);
                }
                DeleteConfirmResult::Cancel => {
                    delete::cancel_delete(self);
                }
                DeleteConfirmResult::None => {
                    // No action taken yet
                }
            }
        }
    }

    pub fn persist_app_state(&mut self) {
        self.history_saver.shutdown();
        // Save application state before shutting down
        if let Err(e) = self.save_app_state() {
            self.toasts
                .error(format!("Failed to save application state: {e}"));
        }
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
                                eprintln!("Failed to parse app state: {e}");
                                None
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read app state file: {e}");
                None
            }
        }
    }
}

impl eframe::App for Kiorg {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        #[cfg(feature = "debug")]
        ctx.set_debug_on_hover(true);

        if self
            .notify_fs_change
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            // Store the currently selected file path in prev_path for refresh_entries to handle
            self.prev_path = {
                let tab = self.tab_manager.current_tab_ref();
                tab.selected_entry().map(|entry| entry.path.clone())
            };

            self.refresh_entries();

            self.notify_fs_change
                .store(false, std::sync::atomic::Ordering::Relaxed);
        }

        // Check for notification messages from background threads
        self.check_notifications();

        // Update preview cache only if selection changed
        if self.selection_changed {
            preview::update_cache(self, ctx);
            self.selection_changed = false; // Reset flag after update
        }

        terminal::draw(ctx, self);

        self.process_input(ctx);

        match &self.show_popup {
            Some(PopupType::Help) => {
                let mut keep_open = true;
                help_window::show_help_window(
                    ctx,
                    self.get_shortcuts(),
                    &mut keep_open,
                    &self.colors,
                );
                if !keep_open {
                    self.show_popup = None;
                }
            }
            Some(PopupType::About) => {
                about::show_about_popup(ctx, self);
            }
            Some(PopupType::Exit) => {
                exit::draw(ctx, self);
            }
            Some(PopupType::Delete(_, _)) => {
                self.handle_delete_confirmation(ctx);
            }
            Some(PopupType::DeleteProgress(_)) => {
                delete::handle_delete_progress(ctx, self);
            }
            Some(PopupType::Rename(_)) => {
                rename::draw(ctx, self);
            }
            Some(PopupType::OpenWith(_)) => {
                open_with::draw(ctx, self);
            }
            Some(PopupType::AddEntry(_)) => {
                add_entry::draw(ctx, self);
            }
            Some(PopupType::Bookmarks(_)) => {
                // Handle bookmark popup
                let bookmark_action = bookmark::show_bookmark_popup(ctx, self);
                // Process the bookmark action
                match bookmark_action {
                    bookmark::BookmarkAction::Navigate(path) => self.navigate_to_dir(path),
                    bookmark::BookmarkAction::SaveBookmarks => {
                        // Save bookmarks when the popup signals a change (e.g., deletion)
                        if let Err(e) = bookmark::save_bookmarks(
                            &self.bookmarks,
                            self.config_dir_override.as_ref(),
                        ) {
                            self.notify_error(format!("Failed to save bookmarks: {e}"));
                        }
                    }
                    bookmark::BookmarkAction::None => {}
                };
            }
            Some(PopupType::Preview) => {
                popup_preview::show_preview_popup(ctx, self);
            }
            Some(PopupType::Themes(_)) => {
                theme::draw(self, ctx);
            }
            Some(PopupType::FileDrop(_)) => {
                file_drop::draw(ctx, self);
            }
            Some(PopupType::Teleport(_)) => {
                teleport::draw(ctx, self);
            }
            Some(PopupType::UpdateConfirm(_)) => {
                update::show_update_confirm_popup(ctx, self);
            }
            Some(PopupType::UpdateProgress(_)) => {
                update::show_update_progress(ctx, self);
            }
            Some(PopupType::UpdateRestart) => {
                update::show_update_restart_popup(ctx, self);
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

        if self.shutdown_requested {
            self.persist_app_state();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Draw toast notifications
        self.toasts.show(ctx);
    }
}
