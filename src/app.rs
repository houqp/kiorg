use std::path::PathBuf;
use std::fs;
use egui::{RichText, Ui, TextureHandle};
use std::sync::atomic::{AtomicU64, Ordering};
use image::io::Reader as ImageReader;
use std::io::Cursor;

use crate::config::{self, colors::AppColors};
use crate::models::dir_entry::DirEntry;
use crate::ui::{help_window, file_list};
use crate::ui::file_list::ROW_HEIGHT;

// Layout constants
const PANEL_SPACING: f32 = 10.0;         // Space between panels
const SEPARATOR_PADDING: f32 = 5.0;      // Padding on each side of separator
const VERTICAL_PADDING: f32 = 4.0;       // Vertical padding in panels
const NAV_HEIGHT_RESERVED: f32 = 50.0;   // Space reserved for navigation bar

// Panel size ratios (relative to usable width)
const LEFT_PANEL_RATIO: f32 = 0.15;
const RIGHT_PANEL_RATIO: f32 = 0.25;
const LEFT_PANEL_MIN_WIDTH: f32 = 150.0;
const RIGHT_PANEL_MIN_WIDTH: f32 = 200.0;

// Use atomic for thread-safe last press tracking
static LAST_G_PRESS: AtomicU64 = AtomicU64::new(0);

pub struct Kiorg {
    pub current_path: PathBuf,
    pub entries: Vec<DirEntry>,
    pub parent_entries: Vec<DirEntry>,
    pub selected_index: usize,
    pub parent_selected_index: usize,
    pub colors: AppColors,
    pub ensure_selected_visible: bool,
    pub show_help: bool,
    pub preview_content: String,
    pub show_exit_confirm: bool,
    pub current_image: Option<TextureHandle>,
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
            preview_content: String::new(),
            show_exit_confirm: false,
            current_image: None,
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
        if self.show_exit_confirm {
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                std::process::exit(0);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_exit_confirm = false;
            }
            return;
        }

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

        if ctx.input(|i| i.key_pressed(egui::Key::Q)) {
            self.show_exit_confirm = true;
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
                
                let available_width = ui.available_width() - 100.0; // Reserve space for help text
                let estimated_width = path_str.len() as f32 * 7.0;
                
                if estimated_width > available_width && components.len() > 4 {
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

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new("? for help").color(self.colors.gray))
                    .on_hover_text("Show keyboard shortcuts");
            });
        });
    }

    fn update_preview(&mut self, ctx: &egui::Context) {
        if let Some(entry) = self.entries.get(self.selected_index) {
            if entry.is_dir {
                self.preview_content = format!("Directory: {}", entry.path.display());
                self.current_image = None;
            } else {
                // Check if it's an image file
                let extension = entry.path.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase());
                
                if let Some(ext) = extension {
                    if ["jpg", "jpeg", "png", "gif", "bmp", "webp"].contains(&ext.as_str()) {
                        if let Ok(bytes) = std::fs::read(&entry.path) {
                            if let Ok(img) = ImageReader::new(Cursor::new(bytes)).with_guessed_format().unwrap().decode() {
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
                                self.preview_content = format!("Image: {}x{}", img.width(), img.height());
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

    fn draw_left_panel(&mut self, ui: &mut Ui, width: f32, height: f32) {
        ui.vertical(|ui| {
            ui.set_min_width(width);
            ui.set_max_width(width);
            ui.set_min_height(height);
            ui.add_space(VERTICAL_PADDING);
            ui.label(RichText::new("Parent Directory").color(self.colors.gray));
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
                    let mut path_to_navigate = None;
                    for (i, entry) in self.parent_entries.iter().enumerate() {
                        let clicked = file_list::draw_parent_entry_row(
                            ui,
                            entry,
                            i == self.parent_selected_index,
                            &self.colors,
                        );
                        if clicked {
                            path_to_navigate = Some(entry.path.clone());
                            break;
                        }
                    }

                    // Handle navigation
                    if let Some(path) = path_to_navigate {
                        self.navigate_to(path);
                    }

                    // Ensure current directory is visible in parent list
                    if !self.parent_entries.is_empty() {
                        let selected_pos = self.parent_selected_index as f32 * ROW_HEIGHT;
                        ui.scroll_to_rect(
                            egui::Rect::from_min_size(
                                egui::pos2(0.0, selected_pos),
                                egui::vec2(width, ROW_HEIGHT)
                            ),
                            Some(egui::Align::Center)
                        );
                    }
                });
        });
    }

    fn draw_center_panel(&mut self, ui: &mut Ui, width: f32, height: f32) {
        ui.vertical(|ui| {
            ui.set_min_width(width);
            ui.set_max_width(width);
            ui.set_min_height(height);
            ui.add_space(VERTICAL_PADDING);
            file_list::draw_table_header(ui, &self.colors);

            // Calculate available height for scroll area
            let available_height = height - ROW_HEIGHT - VERTICAL_PADDING * 2.0;
            
            egui::ScrollArea::vertical()
                .id_salt("current_list_scroll")
                .auto_shrink([false; 2])
                .max_height(available_height)
                .show(ui, |ui| {
                    // Set the width of the content area
                    let scrollbar_width = 6.0; // Standard scrollbar width in egui
                    ui.set_min_width(width - scrollbar_width);
                    ui.set_max_width(width - scrollbar_width);

                    // Draw all rows
                    let mut path_to_navigate = None;
                    for (i, entry) in self.entries.iter().enumerate() {
                        let clicked = file_list::draw_entry_row(
                            ui,
                            entry,
                            i == self.selected_index,
                            &self.colors,
                        );
                        if clicked {
                            path_to_navigate = Some(entry.path.clone());
                            break;
                        }
                    }

                    // Handle navigation
                    if let Some(path) = path_to_navigate {
                        self.navigate_to(path);
                    }

                    // Handle scrolling to selected item
                    if self.ensure_selected_visible && !self.entries.is_empty() {
                        let selected_pos = self.selected_index as f32 * ROW_HEIGHT;
                        ui.scroll_to_rect(
                            egui::Rect::from_min_size(
                                egui::pos2(0.0, selected_pos),
                                egui::vec2(width, ROW_HEIGHT)
                            ),
                            Some(egui::Align::Center)
                        );
                    }
                });
        });
    }

    fn draw_right_panel(&mut self, ui: &mut Ui, width: f32, height: f32) {
        ui.vertical(|ui| {
            ui.set_min_width(width);
            ui.set_max_width(width);
            ui.set_min_height(height);
            ui.add_space(VERTICAL_PADDING);
            ui.label(RichText::new("Preview").color(self.colors.gray));
            ui.separator();
            let preview_height = height - NAV_HEIGHT_RESERVED;
            
            if let Some(image) = &self.current_image {
                egui::ScrollArea::vertical()
                    .id_salt("preview_scroll")
                    .max_height(preview_height)
                    .show(ui, |ui| {
                        ui.centered_and_justified(|ui| {
                            let available_width = width - PANEL_SPACING * 2.0;
                            let available_height = preview_height - PANEL_SPACING * 2.0;
                            let image_size = image.size_vec2();
                            let scale = (available_width / image_size.x).min(available_height / image_size.y);
                            let scaled_size = image_size * scale;
                            
                            ui.add(egui::Image::new((image.id(), scaled_size)));
                        });
                    });
            } else {
                egui::ScrollArea::vertical()
                    .id_salt("preview_scroll")
                    .max_height(preview_height)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.preview_content)
                                .desired_width(width - PANEL_SPACING)
                                .desired_rows(30)
                                .interactive(false)
                        );
                    });
            }
        });
    }

    fn draw_vertical_separator(&mut self, ui: &mut Ui) {
        ui.add_space(SEPARATOR_PADDING);
        ui.vertical(|ui| {
            let rect = ui.available_rect_before_wrap();
            ui.painter().vline(
                rect.left(),
                rect.top()..=rect.bottom(),
                ui.visuals().widgets.noninteractive.bg_stroke,
            );
        });
        ui.add_space(SEPARATOR_PADDING);
    }

    fn calculate_panel_widths(&self, available_width: f32) -> (f32, f32, f32) {
        let total_spacing = (PANEL_SPACING * 2.0) +                    // Space between panels
                          (SEPARATOR_PADDING * 4.0) +                  // Padding around two separators
                          PANEL_SPACING +                             // Right margin
                          8.0;                                        // Margins from both sides
        
        let usable_width = available_width - total_spacing;
        let left_width = (usable_width * LEFT_PANEL_RATIO).max(LEFT_PANEL_MIN_WIDTH);
        let right_width = (usable_width * RIGHT_PANEL_RATIO).max(RIGHT_PANEL_MIN_WIDTH);
        let center_width = usable_width - left_width - right_width;

        (left_width, center_width, right_width)
    }
}

impl eframe::App for Kiorg {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_preview(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            let total_height = ui.available_height();
            
            // Path navigation at the top
            ui.vertical(|ui| {
                ui.add_space(VERTICAL_PADDING);
                self.draw_path_navigation(ui);
                ui.add_space(VERTICAL_PADDING);
                ui.separator();
            });

            // Calculate panel widths
            let (left_width, center_width, right_width) = self.calculate_panel_widths(ui.available_width());
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

        // Handle keyboard input
        self.handle_key_press(ctx);
        
        // Show help window if needed
        if self.show_help {
            help_window::show_help_window(ctx, &mut self.show_help, &self.colors);
        }

        // Show exit confirmation window if needed
        if self.show_exit_confirm {
            egui::Window::new("Exit Confirmation")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.label(RichText::new("Press Enter to exit").color(self.colors.yellow));
                        ui.label(RichText::new("Press Esc to cancel").color(self.colors.gray));
                        ui.add_space(10.0);
                    });
                });
        }
    }
}