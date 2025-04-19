use egui::TextureHandle;
use std::path::PathBuf;

use crate::config::{self, colors::AppColors};
use crate::models::tab::TabManager;
use crate::ui::add_entry_popup; // Import the new module
use crate::ui::delete_dialog::DeleteDialog;
use crate::ui::dialogs;
use crate::ui::search_bar::{self, SearchBar};
use crate::ui::separator;
use crate::ui::separator::SEPARATOR_PADDING;
use crate::ui::top_banner;
use crate::ui::{bookmark_popup, center_panel, help_window, left_panel, right_panel};

// Layout constants
const PANEL_SPACING: f32 = 5.0; // Space between panels

// Panel size ratios (relative to usable width)
const LEFT_PANEL_RATIO: f32 = 0.15;
const RIGHT_PANEL_RATIO: f32 = 0.25;
const LEFT_PANEL_MIN_WIDTH: f32 = 150.0;
const RIGHT_PANEL_MIN_WIDTH: f32 = 200.0;

const DOUBLE_KEY_PRESS_THRESHOLD_MS: u64 = 500;

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
    pub prev_path: Option<PathBuf>,           // Previous path for selection preservation
    pub cached_preview_path: Option<PathBuf>,
    pub selection_changed: bool, // Flag to track if selection changed
    pub search_bar: SearchBar,
    pub scroll_range: Option<std::ops::Range<usize>>,
    pub add_mode: bool,
    pub new_entry_name: String,
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

        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(colors.fg);
        visuals.widgets.noninteractive.bg_fill = colors.bg;
        visuals.widgets.inactive.bg_fill = colors.bg_dim;
        visuals.widgets.hovered.bg_fill = colors.bg_light;
        visuals.widgets.active.bg_fill = colors.selected_bg;
        visuals.widgets.noninteractive.fg_stroke.color = colors.fg;
        visuals.widgets.inactive.fg_stroke.color = colors.fg;
        visuals.widgets.hovered.fg_stroke.color = colors.fg;
        visuals.widgets.active.fg_stroke.color = colors.fg;
        visuals.window_fill = colors.bg;
        visuals.panel_fill = colors.bg;

        cc.egui_ctx.set_visuals(visuals);

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
        tab.update_selection(index);
        self.ensure_selected_visible = true;
        self.selection_changed = true;
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

    pub fn navigate_to(&mut self, mut path: PathBuf) {
        if path.is_dir() {
            let tab = self.tab_manager.current_tab();
            // Swap current_path with path and store the swapped path as prev_path
            std::mem::swap(&mut tab.current_path, &mut path);
            self.prev_path = Some(path);
            // Reset scroll_range to None when navigating to a new directory
            self.scroll_range = None;
            self.refresh_entries();
        } else if path.is_file() {
            if let Err(e) = open::that(&path) {
                eprintln!("Failed to open file: {e}");
            }
        }
    }

    pub fn handle_key_press(&mut self, ctx: &egui::Context) {
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

        if self.show_exit_confirm {
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                std::process::exit(0);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Escape) || i.key_pressed(egui::Key::Q))
            {
                self.show_exit_confirm = false;
            }
            return;
        }

        if self.show_delete_confirm {
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.confirm_delete();
            } else if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.cancel_delete();
            }
            return;
        }

        if self.rename_mode {
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                let tab = self.tab_manager.current_tab();
                if let Some(entry) = tab.entries.get(tab.selected_index) {
                    let parent = entry.path.parent().unwrap_or(&tab.current_path);
                    let new_path = parent.join(&self.new_name);

                    if let Err(e) = std::fs::rename(&entry.path, &new_path) {
                        eprintln!("Failed to rename: {e}");
                    } else {
                        self.refresh_entries();
                    }
                }
                self.rename_mode = false;
                self.new_name.clear();
                self.rename_focus = false;
            } else if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.rename_mode = false;
                self.new_name.clear();
                self.rename_focus = false;
            }
            return;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Questionmark)) {
            self.show_help = !self.show_help;
            return;
        }

        if self.show_help
            && (ctx.input(|i| i.key_pressed(egui::Key::Enter))
                || ctx.input(|i| i.key_pressed(egui::Key::Questionmark))
                || ctx.input(|i| i.key_pressed(egui::Key::Q)))
        {
            self.show_help = false;
            return;
        }

        if self.show_help {
            return;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Q)) {
            self.show_exit_confirm = true;
            return;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::R)) {
            let tab = self.tab_manager.current_tab();
            if let Some(entry) = tab.entries.get(tab.selected_index) {
                self.new_name = entry.name.clone();
                self.rename_mode = true;
                self.rename_focus = true;
            }
            return;
        }

        // Handle copy/cut/paste
        if ctx.input(|i| i.key_pressed(egui::Key::Y)) {
            let tab = self.tab_manager.current_tab();
            let paths: Vec<PathBuf> = if tab.selected_entries.is_empty() {
                if let Some(entry) = tab.entries.get(tab.selected_index) {
                    vec![entry.path.clone()]
                } else {
                    vec![]
                }
            } else {
                tab.selected_entries.iter().cloned().collect()
            };
            if !paths.is_empty() {
                self.clipboard = Some((paths, false));
            }
            return;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::X)) {
            let tab = self.tab_manager.current_tab();
            let paths: Vec<PathBuf> = if tab.selected_entries.is_empty() {
                if let Some(entry) = tab.entries.get(tab.selected_index) {
                    vec![entry.path.clone()]
                } else {
                    vec![]
                }
            } else {
                tab.selected_entries.iter().cloned().collect()
            };
            if !paths.is_empty() {
                self.clipboard = Some((paths, true));
            }
            return;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::P)) {
            let tab = self.tab_manager.current_tab();
            if center_panel::handle_clipboard_operations(&mut self.clipboard, &tab.current_path) {
                self.refresh_entries();
            }
            return;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::D)) {
            let tab = self.tab_manager.current_tab();
            if let Some(entry) = tab.entries.get(tab.selected_index) {
                self.entry_to_delete = Some(entry.path.clone());
                self.show_delete_confirm = true;
            }
            return;
        }

        // Handle tab creation and switching
        if ctx.input(|i| i.key_pressed(egui::Key::T)) {
            let current_path = self.tab_manager.current_tab_ref().current_path.clone();
            self.tab_manager.add_tab(current_path);
            self.refresh_entries();
            return;
        }

        // Handle tab switching with number keys
        if ctx.input(|i| i.key_pressed(egui::Key::Num1)) {
            self.tab_manager.switch_to_tab(0);
            self.refresh_entries();
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num2)) {
            self.tab_manager.switch_to_tab(1);
            self.refresh_entries();
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num3)) {
            self.tab_manager.switch_to_tab(2);
            self.refresh_entries();
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num4)) {
            self.tab_manager.switch_to_tab(3);
            self.refresh_entries();
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num5)) {
            self.tab_manager.switch_to_tab(4);
            self.refresh_entries();
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num6)) {
            self.tab_manager.switch_to_tab(5);
            self.refresh_entries();
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num7)) {
            self.tab_manager.switch_to_tab(6);
            self.refresh_entries();
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num8)) {
            self.tab_manager.switch_to_tab(7);
            self.refresh_entries();
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num9)) {
            self.tab_manager.switch_to_tab(8);
            self.refresh_entries();
            return;
        }

        // Handle navigation in current panel
        if ctx.input(|i| i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown)) {
            self.move_selection(1);
        } else if ctx.input(|i| i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp)) {
            self.move_selection(-1);
        } else if ctx.input(|i| i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::ArrowLeft))
        {
            let parent_path = self
                .tab_manager
                .current_tab_ref()
                .current_path
                .parent()
                .map(|p| p.to_path_buf());
            if let Some(parent) = parent_path {
                self.navigate_to(parent);
            }
        } else if ctx.input(|i| {
            i.key_pressed(egui::Key::L)
                || i.key_pressed(egui::Key::ArrowRight)
                || i.key_pressed(egui::Key::Enter)
        }) {
            let tab = self.tab_manager.current_tab_ref();
            // Get the entry corresponding to the current `selected_index`.
            // This index always refers to the original `entries` list.
            if let Some(selected_entry) = tab.entries.get(tab.selected_index) {
                self.search_bar.close();
                self.navigate_to(selected_entry.path.clone());
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::G) && i.modifiers.shift) {
            let tab = self.tab_manager.current_tab();
            if !tab.entries.is_empty() {
                tab.update_selection(tab.entries.len() - 1);
                self.ensure_selected_visible = true;
                self.selection_changed = true;
            }
            self.last_lowercase_g_pressed_ms = 0;
        } else if ctx.input(|i| i.key_pressed(egui::Key::G) && !i.modifiers.shift) {
            let tab = self.tab_manager.current_tab();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            let last = self.last_lowercase_g_pressed_ms;
            if last > 0 && now - last < DOUBLE_KEY_PRESS_THRESHOLD_MS {
                tab.update_selection(0);
                self.ensure_selected_visible = true;
                self.selection_changed = true;
                // Reset the timestamp after double g presses has been detected
                self.last_lowercase_g_pressed_ms = 0;
            } else {
                self.last_lowercase_g_pressed_ms = now;
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            let tab = self.tab_manager.current_tab();
            if let Some(entry) = tab.entries.get(tab.selected_index) {
                if tab.selected_entries.contains(&entry.path) {
                    tab.selected_entries.remove(&entry.path);
                } else {
                    tab.selected_entries.insert(entry.path.clone());
                }
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::B)) {
            if ctx.input(|i| i.modifiers.shift) {
                // Toggle bookmark popup visibility
                self.show_bookmarks = !self.show_bookmarks;
                // If we're showing the popup, return immediately to avoid processing other shortcuts
                if self.show_bookmarks {
                    return;
                }
            } else {
                let tab = self.tab_manager.current_tab();
                if tab.selected_index < tab.entries.len() {
                    let selected_entry = &tab.entries[tab.selected_index];

                    // Only allow bookmarking directories, not files
                    if selected_entry.is_dir {
                        let path = selected_entry.path.clone();

                        // Toggle bookmark status
                        if self.bookmarks.contains(&path) {
                            self.bookmarks.retain(|p| p != &path);
                        } else {
                            self.bookmarks.push(path);
                        }

                        // Save bookmarks to config file
                        if let Err(e) = bookmark_popup::save_bookmarks(
                            &self.bookmarks,
                            self.config_dir_override.as_ref(),
                        ) {
                            eprintln!("Failed to save bookmarks: {}", e);
                        }
                    }
                } else {
                    // Bookmark/unbookmark current directory if no entry is selected
                    let current_path = tab.current_path.clone();
                    if self.bookmarks.contains(&current_path) {
                        self.bookmarks.retain(|p| p != &current_path);
                    } else {
                        self.bookmarks.push(current_path);
                    }

                    // Save bookmarks to config file
                    if let Err(e) = bookmark_popup::save_bookmarks(
                        &self.bookmarks,
                        self.config_dir_override.as_ref(),
                    ) {
                        eprintln!("Failed to save bookmarks: {}", e);
                    }
                }
            }
        }

        // Handle search activation
        if ctx.input(|i| i.key_pressed(egui::Key::Slash)) {
            self.search_bar.activate();
            return;
        }

        // Handle Add Entry activation
        if ctx.input(|i| i.key_pressed(egui::Key::A)) {
            // Ensure no other modal/popup is active
            if !self.show_help
                && !self.show_exit_confirm
                && !self.show_delete_confirm
                && !self.rename_mode
                && !self.search_bar.active() // Corrected method call
                && !self.show_bookmarks
            {
                self.add_mode = true;
                self.add_focus = true; // Request focus for the input field
                self.new_entry_name.clear();
            }
            // Consume the 'a' key press
            return;
        }
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

    fn confirm_delete(&mut self) {
        if let Some(path) = self.entry_to_delete.clone() {
            DeleteDialog::perform_delete(&path, || {
                self.refresh_entries();
            });
        }
        self.show_delete_confirm = false;
        self.entry_to_delete = None;
    }

    fn cancel_delete(&mut self) {
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

        self.handle_key_press(ctx);

        // Handle bookmark popup with the new approach
        // Use the bookmark_selected_index field from Kiorg struct
        let bookmark_action =
            bookmark_popup::show_bookmark_popup(ctx, &mut self.show_bookmarks, &mut self.bookmarks, &mut self.bookmark_selected_index);

        // Process the bookmark action
        match bookmark_action {
            bookmark_popup::BookmarkAction::Navigate(path) => self.navigate_to(path),
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
                    self.navigate_to(path);
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

        // Show delete confirmation window if needed
        self.handle_delete_confirmation(ctx);
    }
}
