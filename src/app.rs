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
    pub preview_content: String,
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

    fn update_preview(&mut self) {
        if let Some(entry) = self.entries.get(self.selected_index) {
            if entry.is_dir {
                self.preview_content = format!("Directory: {}", entry.path.display());
            } else {
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
            self.preview_content.clear();
        }
    }

    fn draw_preview_panel(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Preview").size(16.0).strong());
        ui.separator();
        let available_height = ui.available_height() - 50.0; // Account for header and spacing
        egui::ScrollArea::vertical()
            .id_salt("preview_scroll")
            .max_height(available_height)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.preview_content)
                        .desired_width(f32::INFINITY)
                        .desired_rows(30)
                        .interactive(false)
                );
            });
    }
}

impl eframe::App for Kiorg {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Update preview when selection changes
            self.update_preview();

            let total_height = ui.available_height();
            
            // Path navigation at the top
            ui.vertical(|ui| {
                ui.add_space(4.0);
                self.draw_path_navigation(ui);
                ui.add_space(4.0);
                ui.separator();
            });

            // Main layout with three columns
            let available_width = ui.available_width();
            let left_width = (available_width * 0.15).max(150.0); // Minimum width of 150
            let right_width = (available_width * 0.25).max(200.0); // Minimum width of 200
            let center_width = available_width - left_width - right_width - 20.0; // Account for spacing

            let content_height = total_height - 50.0; // Account for path navigation

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                ui.set_min_height(content_height);

                // Left panel (15% width) - Parent directory
                ui.vertical(|ui| {
                    ui.set_min_width(left_width);
                    ui.set_max_width(left_width);
                    ui.set_min_height(content_height);
                    ui.add_space(4.0);
                    ui.label(RichText::new("Parent Directory").color(self.colors.gray));
                    ui.separator();
                    if let Some(path) = file_list::draw_file_list(
                        ui,
                        &self.parent_entries,
                        self.parent_selected_index,
                        &self.colors,
                        self.ensure_selected_visible,
                        true,
                        "parent_list_scroll",
                    ) {
                        self.navigate_to(path);
                    }
                });

                // Vertical separator after left panel
                ui.add_space(5.0);
                ui.vertical(|ui| {
                    let rect = ui.available_rect_before_wrap();
                    ui.painter().vline(
                        rect.left(),
                        rect.top()..=rect.bottom(),
                        ui.visuals().widgets.noninteractive.bg_stroke,
                    );
                });
                ui.add_space(5.0);

                // Center panel (60% width) - Current directory
                ui.vertical(|ui| {
                    ui.set_min_width(center_width);
                    ui.set_max_width(center_width);
                    ui.set_min_height(content_height);
                    ui.add_space(4.0);
                    if let Some(path) = file_list::draw_file_list(
                        ui,
                        &self.entries,
                        self.selected_index,
                        &self.colors,
                        self.ensure_selected_visible,
                        false,
                        "current_list_scroll",
                    ) {
                        self.navigate_to(path);
                    }
                });

                // Vertical separator after center panel
                ui.add_space(5.0);
                ui.vertical(|ui| {
                    let rect = ui.available_rect_before_wrap();
                    ui.painter().vline(
                        rect.left(),
                        rect.top()..=rect.bottom(),
                        ui.visuals().widgets.noninteractive.bg_stroke,
                    );
                });
                ui.add_space(5.0);

                // Right panel (25% width) - Preview
                ui.vertical(|ui| {
                    ui.set_min_width(right_width);
                    ui.set_max_width(right_width);
                    ui.set_min_height(content_height);
                    ui.add_space(4.0);
                    ui.label(RichText::new("Preview").color(self.colors.gray));
                    ui.separator();
                    let available_height = ui.available_height() - 50.0; // Account for header and spacing
                    egui::ScrollArea::vertical()
                        .id_salt("preview_scroll")
                        .max_height(available_height)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.preview_content)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(30)
                                    .interactive(false)
                            );
                        });
                });
            });
        });

        // Handle keyboard input
        self.handle_key_press(ctx);
        
        // Show help window if needed
        if self.show_help {
            help_window::show_help_window(ctx, &mut self.show_help, &self.colors);
        }
    }
}