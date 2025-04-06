use egui::{TextureHandle, Ui};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;

use crate::config::{self, colors::AppColors};
use crate::models::dir_entry::DirEntry;
use crate::models::tab::TabManager;
use crate::ui::center_panel::{CenterPanel, CenterPanelDrawParams};
use crate::ui::delete_dialog::DeleteDialog;
use crate::ui::dialogs::Dialogs;
use crate::ui::left_panel::LeftPanel;
use crate::ui::right_panel::{RightPanel, update_preview};
use crate::ui::top_banner::TopBanner;
use crate::ui::{bookmark_popup, help_window};

// Static variable for tracking key press times
static LAST_LOWERCASE_G_PRESS: AtomicU64 = AtomicU64::new(0);

// Layout constants
const PANEL_SPACING: f32 = 10.0; // Space between panels
const SEPARATOR_PADDING: f32 = 5.0; // Padding on each side of separator
const NAV_HEIGHT_RESERVED: f32 = 50.0; // Space reserved for navigation bar

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
    pub config_dir_override: Option<PathBuf>, // Optional override for config directory path
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
        let config = config::load_config();
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

        let tab_manager = TabManager::new(initial_dir);

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
            config_dir_override,
        };

        // Load bookmarks after initializing the app with the config directory
        app.bookmarks = bookmark_popup::load_bookmarks(app.config_dir_override.as_ref());

        app.refresh_entries();
        app
    }

    pub fn refresh_entries(&mut self) {
        let tab = self.tab_manager.current_tab();
        tab.entries.clear();
        tab.selected_index = 0;
        self.ensure_selected_visible = true;

        // Refresh parent directory entries
        if let Some(parent) = tab.current_path.parent() {
            tab.parent_entries.clear();
            tab.parent_selected_index = 0;

            if let Ok(read_dir) = fs::read_dir(parent) {
                tab.parent_entries = read_dir
                    .filter_map(|entry| {
                        let entry = entry.ok()?;
                        let path = entry.path();
                        let is_dir = path.is_dir();
                        let name = entry.file_name().to_string_lossy().into_owned();

                        let metadata = entry.metadata().ok()?;
                        let modified = metadata
                            .modified()
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                        let size = if is_dir { 0 } else { metadata.len() };

                        Some(DirEntry {
                            name,
                            path,
                            is_dir,
                            modified,
                            size,
                        })
                    })
                    .collect();
            }

            tab.parent_entries
                .sort_by(|a, b| match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name),
                });

            // Find current directory in parent entries
            if let Some(pos) = tab
                .parent_entries
                .iter()
                .position(|e| e.path == tab.current_path)
            {
                tab.parent_selected_index = pos;
            }
        } else {
            tab.parent_entries.clear();
        }

        // Refresh current directory entries
        if let Ok(read_dir) = fs::read_dir(&tab.current_path) {
            tab.entries = read_dir
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    let is_dir = path.is_dir();
                    let name = entry.file_name().to_string_lossy().into_owned();

                    let metadata = entry.metadata().ok()?;
                    let modified = metadata
                        .modified()
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    let size = if is_dir { 0 } else { metadata.len() };

                    Some(DirEntry {
                        name,
                        path,
                        is_dir,
                        modified,
                        size,
                    })
                })
                .collect();
        }

        tab.entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });
    }

    pub fn move_selection(&mut self, delta: isize) {
        let tab = self.tab_manager.current_tab();
        if tab.entries.is_empty() {
            return;
        }

        let new_index = tab.selected_index as isize + delta;
        if new_index >= 0 && new_index < tab.entries.len() as isize {
            tab.selected_index = new_index as usize;
            self.ensure_selected_visible = true;
        }
    }

    pub fn navigate_to(&mut self, path: PathBuf) {
        if path.is_dir() {
            let tab = self.tab_manager.current_tab();
            tab.current_path = path;
            tab.selected_index = 0;
            self.refresh_entries();
        } else if path.is_file() {
            if let Err(e) = open::that(&path) {
                eprintln!("Failed to open file: {e}");
            }
        }
    }

    pub fn handle_key_press(&mut self, ctx: &egui::Context) {
        // Don't process keyboard input if the bookmark popup is active
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
            if CenterPanel::handle_clipboard_operations(&mut self.clipboard, &tab.current_path) {
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
            if tab.selected_index < tab.entries.len() {
                let selected_path = tab.entries[tab.selected_index].path.clone();
                self.navigate_to(selected_path);
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::G) && i.modifiers.shift) {
            let tab = self.tab_manager.current_tab();
            if !tab.entries.is_empty() {
                tab.selected_index = tab.entries.len() - 1;
                self.ensure_selected_visible = true;
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::G) && !i.modifiers.shift) {
            let tab = self.tab_manager.current_tab();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            let last = LAST_LOWERCASE_G_PRESS.load(Relaxed);
            if last > 0 && now - last < 500 {
                tab.selected_index = 0;
                self.ensure_selected_visible = true;
                LAST_LOWERCASE_G_PRESS.store(0, Relaxed);
            } else {
                LAST_LOWERCASE_G_PRESS.store(now, Relaxed);
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
    }

    fn draw_center_panel(&mut self, ui: &mut Ui, width: f32, height: f32) {
        let tab = self.tab_manager.current_tab_ref();
        let center_panel = CenterPanel::new(width, height);

        let result = center_panel.draw(
            ui,
            CenterPanelDrawParams {
                tab,
                bookmarks: &self.bookmarks,
                colors: &self.colors,
                rename_mode: self.rename_mode,
                new_name: &mut self.new_name,
                rename_focus: self.rename_focus,
                ensure_selected_visible: self.ensure_selected_visible,
            },
        );

        // Handle navigation
        if let Some(path) = result.path_to_navigate {
            self.navigate_to(path);
        }

        // Handle rename
        if let Some((old_path, new_name)) = result.entry_to_rename {
            if let Some(parent) = old_path.parent() {
                let new_path = parent.join(new_name);
                if let Err(e) = std::fs::rename(&old_path, &new_path) {
                    eprintln!("Failed to rename: {e}");
                } else {
                    self.refresh_entries();
                }
            }
            self.rename_mode = false;
            self.new_name.clear();
            self.rename_focus = false;
        }
    }

    fn draw_vertical_separator(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.set_min_width(SEPARATOR_PADDING);
            ui.set_max_width(SEPARATOR_PADDING);
            ui.separator();
        });
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

    // The show_delete_dialog function has been refactored into the DeleteDialog module
}

impl eframe::App for Kiorg {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update preview using the new module
        update_preview(
            self.tab_manager.current_tab_ref(),
            ctx,
            &mut self.preview_content,
            &mut self.current_image,
        );
        
        self.handle_key_press(ctx);

        // Handle bookmark popup with the new approach
        let bookmark_action =
            bookmark_popup::show_bookmark_popup(ctx, &mut self.show_bookmarks, &mut self.bookmarks);

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
            let total_height = ui.available_height();

            // Path navigation at the top
            TopBanner::new(self).draw(ui);

            // Calculate panel widths
            let (left_width, center_width, right_width) =
                self.calculate_panel_widths(ui.available_width());
            let content_height = total_height - NAV_HEIGHT_RESERVED;

            // Main panels layout
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = PANEL_SPACING;
                ui.set_min_height(content_height);

                if let Some(path) = LeftPanel::new(left_width, content_height).draw(
                    ui,
                    self.tab_manager.current_tab_ref(),
                    &self.bookmarks,
                    &self.colors,
                ) {
                    self.navigate_to(path);
                }
                // Vertical separator after left panel
                self.draw_vertical_separator(ui);
                self.draw_center_panel(ui, center_width, content_height);
                self.draw_vertical_separator(ui);
                RightPanel::new(right_width, content_height).draw(
                    ui,
                    self.tab_manager.current_tab_ref(),
                    &self.colors,
                    &self.preview_content,
                    &self.current_image,
                );
                ui.add_space(PANEL_SPACING);
            });

            // Reset ensure_selected_visible flag after drawing
            self.ensure_selected_visible = false;
        });

        // Show help window if needed
        if self.show_help {
            help_window::show_help_window(ctx, &mut self.show_help, &self.colors);
        }

        // Show exit confirmation window if needed
        if self.show_exit_confirm {
            // Call the refactored dialog function
            Dialogs::show_exit_dialog(ctx, &mut self.show_exit_confirm, &self.colors);
        }

        // Show delete confirmation window if needed
        self.handle_delete_confirmation(ctx);
    }
}