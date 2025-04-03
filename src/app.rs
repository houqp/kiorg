use dirs_next as dirs;
use egui::{RichText, TextureHandle, Ui};
use image::io::Reader as ImageReader;
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader, Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;

use crate::config::{self, colors::AppColors};
use crate::models::dir_entry::DirEntry;
use crate::models::tab::TabManager;
use crate::ui::file_list::ROW_HEIGHT;
use crate::ui::path_nav;
use crate::ui::{file_list, help_window};

// Static variable for tracking key press times
static LAST_LOWERCASE_G_PRESS: AtomicU64 = AtomicU64::new(0);

// Layout constants
const PANEL_SPACING: f32 = 10.0; // Space between panels
const SEPARATOR_PADDING: f32 = 5.0; // Padding on each side of separator
const VERTICAL_PADDING: f32 = 4.0; // Vertical padding in panels
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
    // Get the path to the bookmarks file
    fn get_config_dir(&self) -> PathBuf {
        match &self.config_dir_override {
            Some(dir) => dir.clone(),
            None => {
                let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
                dir.push("kiorg");
                dir
            }
        }
    }

    fn get_bookmarks_file_path(&self) -> PathBuf {
        let mut config_dir = self.get_config_dir();
        
        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }
        config_dir.push("bookmarks.txt");
        config_dir
    }

    // Save bookmarks to the config file
    fn save_bookmarks(&self) -> Result<(), Box<dyn Error>> {
        let bookmarks_file = self.get_bookmarks_file_path();
        let mut file = fs::File::create(bookmarks_file)?;

        for bookmark in &self.bookmarks {
            writeln!(file, "{}", bookmark.to_string_lossy())?;
        }

        Ok(())
    }

    // Load bookmarks from the config file
    fn load_bookmarks(&self) -> Vec<PathBuf> {
        let bookmarks_file = self.get_bookmarks_file_path();
        if !bookmarks_file.exists() {
            return Vec::new();
        }

        match fs::File::open(&bookmarks_file) {
            Ok(file) => {
                let reader = BufReader::new(file);
                reader
                    .lines()
                    .map_while(Result::ok)
                    .filter(|line| !line.trim().is_empty())
                    .map(|line| PathBuf::from(line.trim()))
                    .collect()
            }
            Err(_) => Vec::new(),
        }
    }

    pub fn new(cc: &eframe::CreationContext<'_>, initial_dir: PathBuf) -> Self {
        Self::new_with_config_dir(cc, initial_dir, None)
    }

    pub fn new_with_config_dir(
        cc: &eframe::CreationContext<'_>, 
        initial_dir: PathBuf,
        config_dir_override: Option<PathBuf>
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
        app.bookmarks = app.load_bookmarks();


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
            } else if ctx.input(|i| i.key_pressed(egui::Key::Escape) || i.key_pressed(egui::Key::Q)) {
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
                || ctx.input(|i| i.key_pressed(egui::Key::Questionmark)))
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
            if let Some((paths, is_cut)) = self.clipboard.take() {
                let tab = self.tab_manager.current_tab();
                for path in paths {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_default();
                    let mut new_path = tab.current_path.join(name);

                    // Handle duplicate names
                    let mut counter = 1;
                    while new_path.exists() {
                        let stem = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or_default();
                        let ext = path
                            .extension()
                            .and_then(|e| e.to_str())
                            .map(|e| format!(".{}", e))
                            .unwrap_or_default();
                        new_path = tab
                            .current_path
                            .join(format!("{}_{}{}", stem, counter, ext));
                        counter += 1;
                    }

                    if is_cut {
                        if let Err(e) = std::fs::rename(&path, &new_path) {
                            eprintln!("Failed to move: {e}");
                        } else if let Err(e) = std::fs::copy(&path, &new_path) {
                            eprintln!("Failed to copy: {e}");
                        }
                    } else if let Err(e) = std::fs::copy(&path, &new_path) {
                        eprintln!("Failed to copy: {e}");
                    }
                }
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
                        if let Err(e) = self.save_bookmarks() {
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
                    if let Err(e) = self.save_bookmarks() {
                        eprintln!("Failed to save bookmarks: {}", e);
                    }
                }
            }
        }
    }

    fn draw_left_panel(&mut self, ui: &mut Ui, width: f32, height: f32) {
        let tab = self.tab_manager.current_tab_ref();
        let parent_entries = tab.parent_entries.clone();
        let parent_selected_index = tab.parent_selected_index;
        let colors = self.colors.clone();

        let mut path_to_navigate = None;

        ui.vertical(|ui| {
            ui.set_min_width(width);
            ui.set_max_width(width);
            ui.set_min_height(height);
            ui.add_space(VERTICAL_PADDING);
            ui.label(RichText::new("Parent Directory").color(colors.gray));
            ui.separator();

            // Calculate available height for scroll area
            let available_height = height - ROW_HEIGHT - VERTICAL_PADDING * 2.0;

            egui::ScrollArea::vertical()
                .id_salt("parent_list_scroll")
                .auto_shrink([false; 2])
                .max_height(available_height)
                .show(ui, |ui| {
                    // Set the width of the content area
                    let scrollbar_width = 6.0;
                    ui.set_min_width(width - scrollbar_width);
                    ui.set_max_width(width - scrollbar_width);

                    // Draw all rows
                    for (i, entry) in parent_entries.iter().enumerate() {
                        let is_bookmarked = self.bookmarks.contains(&entry.path);
                        let clicked = file_list::draw_parent_entry_row(
                            ui,
                            entry,
                            i == parent_selected_index,
                            &colors,
                            is_bookmarked,
                        );
                        if clicked {
                            path_to_navigate = Some(entry.path.clone());
                            break;
                        }
                    }

                    // Ensure current directory is visible in parent list
                    if !parent_entries.is_empty() {
                        let selected_pos = parent_selected_index as f32 * ROW_HEIGHT;
                        ui.scroll_to_rect(
                            egui::Rect::from_min_size(
                                egui::pos2(0.0, selected_pos),
                                egui::vec2(width, ROW_HEIGHT),
                            ),
                            Some(egui::Align::Center),
                        );
                    }
                });
        });

        // Handle navigation outside the closure
        if let Some(path) = path_to_navigate {
            self.navigate_to(path);
        }
    }

    fn draw_center_panel(&mut self, ui: &mut Ui, width: f32, height: f32) {
        let tab = self.tab_manager.current_tab_ref();
        let entries = tab.entries.clone();
        let selected_index = tab.selected_index;
        let selected_entries = tab.selected_entries.clone();
        let colors = self.colors.clone();
        let rename_mode = self.rename_mode;
        let new_name = &mut self.new_name;
        let rename_focus = self.rename_focus;

        let mut path_to_navigate = None;
        let mut entry_to_rename = None;

        ui.vertical(|ui| {
            ui.set_min_width(width);
            ui.set_max_width(width);
            ui.set_min_height(height);
            ui.add_space(VERTICAL_PADDING);
            file_list::draw_table_header(ui, &colors);

            // Calculate available height for scroll area
            let available_height = height - ROW_HEIGHT - VERTICAL_PADDING * 2.0;

            egui::ScrollArea::vertical()
                .id_salt("current_list_scroll")
                .auto_shrink([false; 2])
                .max_height(available_height)
                .show(ui, |ui| {
                    // Set the width of the content area
                    let scrollbar_width = 6.0;
                    ui.set_min_width(width - scrollbar_width);
                    ui.set_max_width(width - scrollbar_width);

                    // Draw entries
                    for (i, entry) in entries.iter().enumerate() {
                        let is_selected = i == selected_index;
                        let is_in_selection = selected_entries.contains(&entry.path);

                        if file_list::draw_entry_row(
                            ui,
                            file_list::EntryRowParams {
                                entry,
                                is_selected,
                                colors: &colors,
                                rename_mode: rename_mode && is_selected,
                                new_name,
                                rename_focus: rename_focus && is_selected,
                                is_marked: is_in_selection,
                                is_bookmarked: self.bookmarks.contains(&entry.path),
                            },
                        ) {
                            if rename_mode {
                                entry_to_rename = Some((entry.path.clone(), new_name.clone()));
                            } else {
                                path_to_navigate = Some(entry.path.clone());
                            }
                        }
                    }

                    // Handle scrolling to selected item
                    if self.ensure_selected_visible && !entries.is_empty() {
                        let selected_pos = selected_index as f32 * ROW_HEIGHT;
                        ui.scroll_to_rect(
                            egui::Rect::from_min_size(
                                egui::pos2(0.0, selected_pos),
                                egui::vec2(width, ROW_HEIGHT),
                            ),
                            Some(egui::Align::Center),
                        );
                    }
                });
        });

        // Handle navigation outside the closure
        if let Some(path) = path_to_navigate {
            self.navigate_to(path);
        }

        // Handle rename outside the closure
        if let Some((old_path, new_name)) = entry_to_rename {
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

    fn draw_right_panel(&mut self, ui: &mut Ui, width: f32, height: f32) {
        let tab = self.tab_manager.current_tab_ref();
        let colors = self.colors.clone();
        let preview_content = self.preview_content.clone();

        ui.vertical(|ui| {
            ui.set_min_width(width);
            ui.set_max_width(width);
            ui.set_min_height(height);
            ui.add_space(VERTICAL_PADDING);
            ui.label(RichText::new("Preview").color(colors.gray));
            ui.separator();

            // Calculate available height for scroll area
            let available_height = height - ROW_HEIGHT - VERTICAL_PADDING * 4.0;

            egui::ScrollArea::vertical()
                .id_salt("preview_scroll")
                .auto_shrink([false; 2])
                .max_height(available_height)
                .show(ui, |ui| {
                    // Set the width of the content area
                    let scrollbar_width = 6.0;
                    ui.set_min_width(width - scrollbar_width);
                    ui.set_max_width(width - scrollbar_width);

                    // Draw preview content
                    if let Some(entry) = tab.entries.get(tab.selected_index) {
                        if entry.is_dir {
                            ui.label(RichText::new("Directory").color(colors.gray));
                        } else if let Some(image) = &self.current_image {
                            ui.centered_and_justified(|ui| {
                                let available_width = width - PANEL_SPACING * 2.0;
                                let available_height = available_height - PANEL_SPACING * 2.0;
                                let image_size = image.size_vec2();
                                let scale = (available_width / image_size.x)
                                    .min(available_height / image_size.y);
                                let scaled_size = image_size * scale;

                                ui.add(egui::Image::new((image.id(), scaled_size)));
                            });
                        } else {
                            ui.add(
                                egui::TextEdit::multiline(&mut String::from(&preview_content))
                                    .desired_width(width - PANEL_SPACING)
                                    .desired_rows(30)
                                    .interactive(false),
                            );
                        }
                    }
                });

            // Draw help text in its own row at the bottom
            ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
                ui.label(RichText::new("? for help").color(colors.gray));
            });
            ui.add_space(VERTICAL_PADDING);
        });
    }

    fn draw_path_navigation(&mut self, ui: &mut Ui) {
        let tab = self.tab_manager.current_tab();
        if let Some(message) = path_nav::draw_path_navigation(ui, &tab.current_path, &self.colors) {
            match message {
                path_nav::PathNavMessage::Navigate(path) => {
                    self.navigate_to(path);
                }
            }
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
        if let Some(path) = self.entry_to_delete.take() {
            if let Err(e) = if path.is_dir() {
                std::fs::remove_dir_all(&path)
            } else {
                std::fs::remove_file(&path)
            } {
                eprintln!("Failed to delete: {e}");
            } else {
                self.refresh_entries();
            }
        }
        self.show_delete_confirm = false;
    }

    fn cancel_delete(&mut self) {
        self.show_delete_confirm = false;
        self.entry_to_delete = None;
    }

    fn handle_delete_confirmation(&mut self, ctx: &egui::Context) {
        if self.show_delete_confirm {
            if let Some(path) = self.entry_to_delete.clone() {
                self.show_delete_dialog(ctx, &path);
            }
        }
    }

    fn show_delete_dialog(&mut self, ctx: &egui::Context, path: &Path) {
        let mut show_popup = self.show_delete_confirm;
        if let Some(response) = egui::Window::new("Delete Confirmation")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut show_popup)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(format!("Delete {}?", path.display()));
                    let confirm_clicked = ui
                        .link(RichText::new("Press Enter to confirm").color(self.colors.yellow))
                        .clicked();
                    let cancel_clicked = ui
                        .link(RichText::new("Press Esc to cancel").color(self.colors.gray))
                        .clicked();
                    ui.add_space(10.0);

                    if confirm_clicked {
                        self.confirm_delete();
                    } else if cancel_clicked {
                        self.cancel_delete();
                    }
                });
            })
        {
            self.show_delete_confirm = !response.response.clicked_elsewhere();
        }
    }

    fn update_preview(&mut self, ctx: &egui::Context) {
        let tab = self.tab_manager.current_tab();
        if let Some(entry) = tab.entries.get(tab.selected_index) {
            if entry.is_dir {
                self.preview_content = format!("Directory: {}", entry.path.display());
                self.current_image = None;
            } else {
                // Check if it's an image file
                let extension = entry
                    .path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase());

                if let Some(ext) = extension {
                    if ["jpg", "jpeg", "png", "gif", "bmp", "webp"].contains(&ext.as_str()) {
                        if let Ok(bytes) = std::fs::read(&entry.path) {
                            if let Ok(img) = ImageReader::new(Cursor::new(bytes))
                                .with_guessed_format()
                                .unwrap()
                                .decode()
                            {
                                let size = [img.width() as _, img.height() as _];
                                let image = egui::ColorImage::from_rgba_unmultiplied(
                                    size,
                                    img.to_rgba8().as_raw(),
                                );
                                self.current_image = Some(ctx.load_texture(
                                    entry.path.to_string_lossy().to_string(),
                                    image,
                                    egui::TextureOptions::default(),
                                ));
                                self.preview_content =
                                    format!("Image: {}x{}", img.width(), img.height());
                            }
                        }
                    } else {
                        // Clear image texture for non-image files
                        self.current_image = None;
                        match std::fs::read_to_string(&entry.path) {
                            Ok(content) => {
                                // Only show first 1000 characters for text files
                                self.preview_content = content.chars().take(1000).collect();
                            }
                            Err(_) => {
                                // For binary files or files that can't be read
                                self.preview_content = format!("Binary file: {} bytes", entry.size);
                            }
                        }
                    }
                } else {
                    // Clear image texture for files without extension
                    self.current_image = None;
                    match std::fs::read_to_string(&entry.path) {
                        Ok(content) => {
                            // Only show first 1000 characters for text files
                            self.preview_content = content.chars().take(1000).collect();
                        }
                        Err(_) => {
                            // For binary files or files that can't be read
                            self.preview_content = format!("Binary file: {} bytes", entry.size);
                        }
                    }
                }
            }
        } else {
            self.preview_content.clear();
            self.current_image = None;
        }
    }

    fn show_exit_dialog(&mut self, ctx: &egui::Context) {
        let mut show_popup = self.show_exit_confirm;
        if let Some(response) = egui::Window::new("Exit Confirmation")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut show_popup)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    if ui
                        .link(RichText::new("Press Enter to exit").color(self.colors.yellow))
                        .clicked()
                    {
                        std::process::exit(0);
                    }
                    if ui
                        .link(RichText::new("Press Esc or q to cancel").color(self.colors.gray))
                        .clicked()
                    {
                        self.show_exit_confirm = false;
                    }
                    ui.add_space(10.0);
                });
            })
        {
            self.show_exit_confirm = !response.response.clicked_elsewhere();
        }
    }

    fn show_bookmark_popup(&mut self, ctx: &egui::Context) {
        if !self.show_bookmarks {
            return;
        }

        let mut show_popup = self.show_bookmarks;
        let bookmarks = self.bookmarks.to_vec();

        // Initialize a static bookmark index to preserve selection state between frames
        static mut BOOKMARK_SELECTED_INDEX: usize = 0;
        let mut selected_index = unsafe { BOOKMARK_SELECTED_INDEX };

        // Ensure index is valid
        if !bookmarks.is_empty() {
            selected_index = selected_index.min(bookmarks.len() - 1);
        } else {
            selected_index = 0;
        }

        // Track if we need to navigate to a bookmark or remove a bookmark
        let mut navigate_to_path = None;
        let mut remove_bookmark_path = None;

        // Handle keyboard navigation for closing the popup
        if ctx.input(|i| i.key_pressed(egui::Key::Q) || i.key_pressed(egui::Key::Escape)) {
            show_popup = false;
        }

        // Handle keyboard shortcut for deleting bookmarks
        if ctx.input(|i| i.key_pressed(egui::Key::D)) && !bookmarks.is_empty() {
            remove_bookmark_path = Some(bookmarks[selected_index].clone());
        }

        if let Some(response) = egui::Window::new("Bookmarks")
            .resizable(true)
            .default_width(500.0) // Wider window for two columns
            .pivot(egui::Align2::CENTER_CENTER) // Center the window
            .default_pos(ctx.screen_rect().center()) // Position at screen center
            .open(&mut show_popup)
            .show(ctx, |ui| {
                if bookmarks.is_empty() {
                    ui.label("No bookmarks yet. Use 'b' to bookmark folders.");
                    return;
                }

                // Handle keyboard navigation
                if ctx.input(|i| i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown))
                {
                    if !bookmarks.is_empty() {
                        selected_index = (selected_index + 1).min(bookmarks.len() - 1);
                        unsafe {
                            BOOKMARK_SELECTED_INDEX = selected_index;
                        }
                    }
                } else if ctx
                    .input(|i| i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp))
                {
                    selected_index = selected_index.saturating_sub(1);
                    unsafe {
                        BOOKMARK_SELECTED_INDEX = selected_index;
                    }
                } else if ctx
                    .input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::L))
                    && !bookmarks.is_empty()
                {
                    navigate_to_path = Some(bookmarks[selected_index].clone());
                }

                // Display bookmarks in a two-column layout with proper alignment
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Create a grid layout for consistent column alignment
                    egui::Grid::new("bookmarks_grid")
                        .num_columns(2)
                        .spacing([20.0, 6.0]) // Space between columns and rows
                        .striped(true) // Alternate row background for better readability
                        .show(ui, |ui| {
                            for (i, bookmark) in bookmarks.iter().enumerate() {
                                // Extract folder name and parent path
                                let folder_name = bookmark
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_default();

                                let parent_path = bookmark
                                    .parent()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_default();

                                // First column: Folder name with selectable label
                                let folder_response = ui.add(egui::SelectableLabel::new(
                                    i == selected_index,
                                    folder_name,
                                ));

                                // Second column: Parent path
                                let path_color = if i == selected_index {
                                    ui.visuals().strong_text_color()
                                } else {
                                    ui.visuals().weak_text_color()
                                };
                                let path_response = ui.colored_label(path_color, parent_path);
                                ui.end_row();

                                // Combined response for click handling
                                let response = folder_response.union(path_response);

                                if response.clicked() {
                                    navigate_to_path = Some(bookmark.clone());
                                }

                                // Right-click context menu for removing bookmarks
                                response.context_menu(|ui| {
                                    if ui.button("Remove bookmark").clicked() {
                                        remove_bookmark_path = Some(bookmark.clone());
                                        ui.close_menu();
                                    }
                                });
                            }
                        });
                });
            })
        {
            // If we need to navigate, do it now
            if let Some(path) = navigate_to_path {
                self.navigate_to(path);
                show_popup = false;
            }

            // If we need to remove a bookmark, do it now
            if let Some(path) = remove_bookmark_path {
                self.bookmarks.retain(|p| p != &path);

                // Save bookmarks to config file after removal
                if let Err(e) = self.save_bookmarks() {
                    eprintln!("Failed to save bookmarks: {}", e);
                }
            }

            self.show_bookmarks = show_popup && !response.response.clicked_elsewhere();
        } else {
            self.show_bookmarks = show_popup;
        }
    }
}

impl eframe::App for Kiorg {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_preview(ctx);
        self.handle_key_press(ctx);
        self.show_bookmark_popup(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            let total_height = ui.available_height();

            // Path navigation at the top
            ui.vertical(|ui| {
                ui.add_space(VERTICAL_PADDING);
                ui.horizontal(|ui| {
                    // Path navigation on the left
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                        self.draw_path_navigation(ui);
                    });

                    // Tab numbers on the right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                        for i in (0..self.tab_manager.tabs.len()).rev() {
                            let is_current = i == self.tab_manager.current_tab_index;
                            let text = format!("{}", i + 1);
                            let color = if is_current {
                                self.colors.yellow
                            } else {
                                self.colors.gray
                            };
                            if ui.link(RichText::new(text).color(color)).clicked() {
                                self.tab_manager.switch_to_tab(i);
                            }
                        }
                    });
                });
                ui.add_space(VERTICAL_PADDING);
                ui.separator();
            });

            // Calculate panel widths
            let (left_width, center_width, right_width) =
                self.calculate_panel_widths(ui.available_width());
            let content_height = total_height - NAV_HEIGHT_RESERVED;

            // Main panels layout
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = PANEL_SPACING;
                ui.set_min_height(content_height);

                // Left panel
                self.draw_left_panel(ui, left_width, content_height);

                // Vertical separator after left panel
                self.draw_vertical_separator(ui);

                // Center panel
                self.draw_center_panel(ui, center_width, content_height);

                // Vertical separator after center panel
                self.draw_vertical_separator(ui);

                // Right panel
                self.draw_right_panel(ui, right_width, content_height);

                // Right margin
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
            self.show_exit_dialog(ctx);
        }

        // Show delete confirmation window if needed
        self.handle_delete_confirmation(ctx);
    }
}
