use std::path::PathBuf;
use std::fs;
use egui::{RichText, Ui};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::config::{self, colors::AppColors};
use crate::models::dir_entry::DirEntry;
use crate::ui::{help_window, file_list};

// Use atomic for thread-safe last press tracking
static LAST_G_PRESS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct Kiorg {
    pub current_path: PathBuf,
    pub entries: Vec<DirEntry>,
    pub parent_entries: Vec<DirEntry>,
    pub selected_index: usize,
    pub parent_selected_index: usize,
    pub colors: AppColors,
    pub ensure_selected_visible: bool,
    pub show_help: bool,
}

impl Kiorg {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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
        
        let current_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut app = Self {
            current_path,
            entries: Vec::new(),
            parent_entries: Vec::new(),
            selected_index: 0,
            parent_selected_index: 0,
            colors,
            ensure_selected_visible: false,
            show_help: false,
        };
        app.refresh_entries();
        app
    }

    pub fn refresh_entries(&mut self) {
        self.entries.clear();
        self.selected_index = 0;
        self.ensure_selected_visible = true;

        // Refresh parent directory entries
        if let Some(parent) = self.current_path.parent() {
            self.parent_entries.clear();
            self.parent_selected_index = 0;
            
            if let Ok(read_dir) = fs::read_dir(parent) {
                self.parent_entries = read_dir
                    .filter_map(|entry| {
                        let entry = entry.ok()?;
                        let path = entry.path();
                        let is_dir = path.is_dir();
                        let name = entry.file_name().to_string_lossy().into_owned();
                        
                        let metadata = entry.metadata().ok()?;
                        let modified = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
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

            self.parent_entries.sort_by(|a, b| {
                match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name),
                }
            });

            // Find current directory in parent entries
            if let Some(pos) = self.parent_entries.iter().position(|e| e.path == self.current_path) {
                self.parent_selected_index = pos;
            }
        } else {
            self.parent_entries.clear();
        }

        // Refresh current directory entries
        if let Ok(read_dir) = fs::read_dir(&self.current_path) {
            self.entries = read_dir
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    let is_dir = path.is_dir();
                    let name = entry.file_name().to_string_lossy().into_owned();
                    
                    let metadata = entry.metadata().ok()?;
                    let modified = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
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

        self.entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
    }

    pub fn move_selection(&mut self, delta: isize) {
        if self.entries.is_empty() {
            return;
        }
        
        let new_index = self.selected_index as isize + delta;
        if new_index >= 0 && new_index < self.entries.len() as isize {
            self.selected_index = new_index as usize;
            self.ensure_selected_visible = true;
        }
    }

    pub fn navigate_to(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.current_path = path;
            self.selected_index = 0;
            self.refresh_entries();
        } else if path.is_file() {
            if let Err(e) = open::that(&path) {
                eprintln!("Failed to open file: {e}");
            }
        }
    }

    pub fn handle_key_press(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Questionmark)) {
            self.show_help = !self.show_help;
            return;
        }
        
        if self.show_help && (
            ctx.input(|i| i.key_pressed(egui::Key::Enter))
            || ctx.input(|i| i.key_pressed(egui::Key::Questionmark))
        ) {
            self.show_help = false;
            return;
        }
        
        if self.show_help {
            return;
        }
        
        // Handle navigation in current panel
        if ctx.input(|i| i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown)) {
            self.move_selection(1);
        } else if ctx.input(|i| i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp)) {
            self.move_selection(-1);
        } else if ctx.input(|i| i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::ArrowLeft)) {
            if let Some(parent) = self.current_path.parent() {
                self.navigate_to(parent.to_path_buf());
            }
        } else if ctx.input(|i| 
            i.key_pressed(egui::Key::L) 
            || i.key_pressed(egui::Key::ArrowRight) 
            || i.key_pressed(egui::Key::Enter)
        ) {
            if self.selected_index < self.entries.len() {
                let selected_path = self.entries[self.selected_index].path.clone();
                self.navigate_to(selected_path);
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::G)) {
            if ctx.input(|i| i.modifiers.shift) {
                if !self.entries.is_empty() {
                    self.selected_index = self.entries.len() - 1;
                    self.ensure_selected_visible = true;
                }
            } else {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                
                let last = LAST_G_PRESS.load(Ordering::Relaxed);
                if last > 0 && now - last < 500 {
                    self.selected_index = 0;
                    self.ensure_selected_visible = true;
                    LAST_G_PRESS.store(0, Ordering::Relaxed);
                } else {
                    LAST_G_PRESS.store(now, Ordering::Relaxed);
                }
            }
        }
    }

    fn draw_path_navigation(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let path_ui_width = ui.available_width() - 100.0;
            ui.horizontal(|ui| {
                ui.set_min_width(path_ui_width);
                ui.label(RichText::new("$ ").color(self.colors.gray));
                
                let mut components = Vec::new();
                let mut current = PathBuf::new();
                
                for component in self.current_path.components() {
                    let comp_str = component.as_os_str().to_string_lossy().to_string();
                    if !comp_str.is_empty() {
                        current.push(&comp_str);
                        components.push((comp_str, current.clone()));
                    }
                }
                
                if components.is_empty() {
                    ui.label(RichText::new("/").color(self.colors.yellow));
                } else {
                    let mut path_str = String::new();
                    for (i, (name, _)) in components.iter().enumerate() {
                        if i > 1 {
                            path_str.push('/');
                        }
                        path_str.push_str(name);
                    }
                    
                    let estimated_width = path_str.len() as f32 * 7.0;
                    
                    if estimated_width > path_ui_width && components.len() > 4 {
                        if ui.link(RichText::new(&components[0].0).color(self.colors.yellow)).clicked() {
                            self.navigate_to(components[0].1.clone());
                        }
                        
                        ui.label(RichText::new("/").color(self.colors.gray));
                        ui.label(RichText::new("...").color(self.colors.gray));
                        
                        let start_idx = components.len() - 2;
                        for component in components.iter().skip(start_idx) {
                            let (comp_str, path) = component;
                            ui.label(RichText::new("/").color(self.colors.gray));
                            if ui.link(RichText::new(comp_str).color(self.colors.yellow)).clicked() {
                                self.navigate_to(path.clone());
                            }
                        }
                    } else {
                        for (i, (name, path)) in components.iter().enumerate() {
                            if i > 1 {
                                ui.label(RichText::new("/").color(self.colors.gray));
                            }
                            
                            if ui.link(RichText::new(name).color(self.colors.yellow)).clicked() {
                                self.navigate_to(path.clone());
                            }
                        }
                    }
                }
            });
            
            ui.horizontal(|ui| {
                ui.label(RichText::new("? for help").color(self.colors.gray))
                    .on_hover_text("Show keyboard shortcuts");
            });
        });
    }
}

impl eframe::App for Kiorg {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let total_height = ui.available_height();
            
            // Use a vertical layout to properly stack and size components
            ui.vertical(|ui| {
                ui.set_min_height(total_height);
                
                // Path navigation takes fixed height
                self.draw_path_navigation(ui);
                ui.separator();
                
                // Main content area should take all remaining height
                let content_height = ui.available_height();
                
                // Split the available space into left and right panels
                let available_width = ui.available_width();
                let left_panel_width = available_width * 0.20;
                let right_panel_width = available_width - left_panel_width;
                
                ui.horizontal(|ui| {
                    ui.set_min_height(content_height);
                    
                    // Left panel - Parent directory entries
                    ui.vertical(|ui| {
                        ui.set_min_width(left_panel_width);
                        ui.set_max_width(left_panel_width);
                        ui.set_min_height(content_height);
                        
                        ui.label(RichText::new("Parent Directory").color(self.colors.yellow));
                        ui.separator();
                        
                        let panel_height = content_height - 30.0; // Account for header
                        let row_height = 20.0;
                        let visible_rows = (panel_height / row_height).floor() as usize;
                        
                        let mut scroll_area = egui::ScrollArea::vertical()
                            .id_salt("parent_list_scroll")
                            .auto_shrink([false; 2])
                            .max_height(panel_height)
                            .min_scrolled_height(0.0);

                        if !self.parent_entries.is_empty() && visible_rows > 0 {
                            let total_height = self.parent_entries.len() as f32 * row_height;
                            let selected_y = self.parent_selected_index as f32 * row_height;
                            let ideal_scroll_top = selected_y - (panel_height / 2.0) + (row_height / 2.0);
                            let extra_scroll_padding = row_height * 2.0;
                            let max_scroll = (total_height - panel_height + extra_scroll_padding).max(0.0);
                            let scroll_top = ideal_scroll_top.clamp(0.0, max_scroll);
                            scroll_area = scroll_area.vertical_scroll_offset(scroll_top);
                        }

                        scroll_area.show_rows(
                            ui,
                            row_height,
                            self.parent_entries.len(),
                            |ui: &mut Ui, row_range| {
                                ui.style_mut().spacing.item_spacing.y = 0.0;
                                
                                let entries = self.parent_entries.clone();
                                let mut clicked_path = None;
                                
                                for i in row_range {
                                    if i < entries.len() {
                                        let entry = &entries[i];
                                        let is_selected = i == self.parent_selected_index;
                                        if file_list::draw_parent_entry_row(ui, entry, is_selected, &self.colors) {
                                            clicked_path = Some(entry.path.clone());
                                        }
                                    }
                                }
                                
                                if let Some(path) = clicked_path {
                                    self.navigate_to(path);
                                }
                            }
                        );
                    });
                    
                    ui.separator();
                    
                    // Right panel - Current directory entries
                    ui.vertical(|ui| {
                        ui.set_min_width(right_panel_width);
                        ui.set_max_width(right_panel_width);
                        ui.set_min_height(content_height);
                        
                        file_list::draw_table_header(ui, &self.colors);
                        
                        let panel_height = content_height - 30.0; // Account for header
                        let row_height = 20.0;
                        let visible_rows = (panel_height / row_height).floor() as usize;
                        
                        let mut scroll_area = egui::ScrollArea::vertical()
                            .id_salt("file_list_scroll")
                            .auto_shrink([false; 2])
                            .max_height(panel_height);

                        if self.ensure_selected_visible && !self.entries.is_empty() && visible_rows > 0 {
                            let total_height = self.entries.len() as f32 * row_height;
                            let selected_y = self.selected_index as f32 * row_height;
                            let ideal_scroll_top = selected_y - (panel_height / 2.0) + (row_height / 2.0);
                            let extra_scroll_padding = row_height * 2.0;
                            let max_scroll = (total_height - panel_height + extra_scroll_padding).max(0.0);
                            let scroll_top = ideal_scroll_top.clamp(0.0, max_scroll);
                            
                            scroll_area = scroll_area.vertical_scroll_offset(scroll_top);
                            self.ensure_selected_visible = false;
                        }

                        scroll_area.show_rows(
                            ui,
                            row_height,
                            self.entries.len(),
                            |ui: &mut Ui, row_range| {
                                ui.style_mut().spacing.item_spacing.y = 0.0;
                                
                                let entries = self.entries.clone();
                                let mut clicked_path = None;
                                
                                for i in row_range {
                                    if i < entries.len() {
                                        let entry = &entries[i];
                                        let is_selected = i == self.selected_index;
                                        if file_list::draw_entry_row(ui, entry, is_selected, &self.colors) {
                                            clicked_path = Some(entry.path.clone());
                                        }
                                    }
                                }
                                
                                if let Some(path) = clicked_path {
                                    self.navigate_to(path);
                                }
                            }
                        );
                    });
                });
            });
            
            self.handle_key_press(ctx);
        });
        
        help_window::show_help_window(ctx, &mut self.show_help, &self.colors);
    }
}