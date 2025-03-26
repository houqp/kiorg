use eframe::egui;
use egui::{Color32, RichText, Ui, Align2};
use std::fs;
use std::path::PathBuf;
use std::io::Read;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use dirs_next;
use humansize;

// Color scheme configuration
#[derive(Deserialize, Serialize, Clone, Debug)]
struct ColorScheme {
    bg: String,
    bg_dim: String,
    bg_light: String,
    fg: String,
    red: String,
    orange: String,
    yellow: String,
    green: String,
    aqua: String,
    blue: String,
    purple: String,
    selected_bg: String,
    gray: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        // Default to Sonokai color scheme
        Self {
            bg: "#2c2e34".to_string(),
            bg_dim: "#33353f".to_string(),
            bg_light: "#3b3e48".to_string(),
            fg: "#e2e2e3".to_string(),
            red: "#fc5d7c".to_string(),
            orange: "#f39660".to_string(),
            yellow: "#e7c664".to_string(),
            green: "#9ed072".to_string(),
            aqua: "#76cce0".to_string(),
            blue: "#7f84de".to_string(),
            purple: "#b39df3".to_string(),
            selected_bg: "#45475a".to_string(),
            gray: "#7f8490".to_string(),
        }
    }
}

#[derive(Clone)]
struct AppColors {
    bg: Color32,
    bg_dim: Color32,
    bg_light: Color32,
    fg: Color32,
    red: Color32,
    orange: Color32,
    yellow: Color32,
    green: Color32,
    aqua: Color32,
    blue: Color32,
    purple: Color32,
    selected_bg: Color32,
    gray: Color32,
}

impl AppColors {
    fn from_config(config: &ColorScheme) -> Self {
        Self {
            bg: hex_to_color32(&config.bg),
            bg_dim: hex_to_color32(&config.bg_dim),
            bg_light: hex_to_color32(&config.bg_light),
            fg: hex_to_color32(&config.fg),
            red: hex_to_color32(&config.red),
            orange: hex_to_color32(&config.orange),
            yellow: hex_to_color32(&config.yellow),
            green: hex_to_color32(&config.green),
            aqua: hex_to_color32(&config.aqua),
            blue: hex_to_color32(&config.blue),
            purple: hex_to_color32(&config.purple),
            selected_bg: hex_to_color32(&config.selected_bg),
            gray: hex_to_color32(&config.gray),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct Config {
    colors: ColorScheme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            colors: ColorScheme::default(),
        }
    }
}

struct Kiorg {
    current_path: PathBuf,
    entries: Vec<DirEntry>,
    selected_index: usize,
    colors: AppColors,
    ensure_selected_visible: bool,
    show_help: bool,
}

#[derive(Clone)]
struct DirEntry {
    name: String,
    path: PathBuf,
    is_dir: bool,
    modified: std::time::SystemTime,
    size: u64,
}

impl Kiorg {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load configuration from file
        let config = load_config();
        let colors = AppColors::from_config(&config.colors);
        
        // Customize egui with the loaded colors
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
            selected_index: 0,
            colors,
            ensure_selected_visible: false,
            show_help: false,
        };
        app.refresh_entries();
        app
    }

    fn refresh_entries(&mut self) {
        self.entries.clear();
        self.selected_index = 0;
        self.ensure_selected_visible = true;

        if let Ok(entries) = fs::read_dir(&self.current_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let is_dir = path.is_dir();
                    let name = entry.file_name().to_string_lossy().to_string();
                    
                    // Skip hidden files (optional)
                    // if name.starts_with(".") { continue; }
                    
                    let metadata = match entry.metadata() {
                        Ok(meta) => meta,
                        Err(_) => continue, // Skip entries with inaccessible metadata
                    };
                    
                    let modified = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    let size = if is_dir { 0 } else { metadata.len() };

                    self.entries.push(DirEntry {
                        name,
                        path,
                        is_dir,
                        modified,
                        size,
                    });
                }
            }
        }

        // Sort directories first, then by name
        self.entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
    }

    fn move_selection(&mut self, delta: isize) {
        if self.entries.is_empty() {
            return;
        }
        
        let new_index = self.selected_index as isize + delta;
        if new_index >= 0 && new_index < self.entries.len() as isize {
            self.selected_index = new_index as usize;
            self.ensure_selected_visible = true;
        }
    }

    fn navigate_to(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.current_path = path;
            self.selected_index = 0;
            self.refresh_entries();
        } else if path.is_file() {
            // For files, we could implement opening them with the default application
            // For now, we'll just print the path
            println!("Would open file: {:?}", path);
        }
    }

    fn handle_key_press(&mut self, ctx: &egui::Context) {
        let mut input = ctx.input(|i| i.clone());
        
        // Toggle help window with '?'
        if input.consume_key(egui::Modifiers::NONE, egui::Key::Questionmark) {
            self.show_help = !self.show_help;
            return;
        }
        
        // Close help window with Enter
        if self.show_help && (
            input.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
            ||  input.consume_key(egui::Modifiers::NONE, egui::Key::Questionmark)
        ) {
            self.show_help = false;
            return;
        }
        
        // Skip keyboard navigation when help is shown
        if self.show_help {
            return;
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown)) {
            self.move_selection(1);
        } else if ctx.input(|i| i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp)) {
            self.move_selection(-1);
        } else if ctx.input(|i| i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::ArrowLeft)) {
            // Go to parent directory
            if let Some(parent) = self.current_path.parent() {
                self.navigate_to(parent.to_path_buf());
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::L) || i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::Enter)) {
            // Enter directory or open file
            if self.selected_index < self.entries.len() {
                let selected_path = self.entries[self.selected_index].path.clone();
                self.navigate_to(selected_path);
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::G)) {
            // Check if Shift is pressed for 'G' (go to bottom)
            if ctx.input(|i| i.modifiers.shift) {
                // Go to bottom (last entry)
                if !self.entries.is_empty() {
                    self.selected_index = self.entries.len() - 1;
                    self.ensure_selected_visible = true;
                }
            } else {
                // Track 'g' presses for 'gg' command
                static mut LAST_G_PRESS: Option<std::time::Instant> = None;
                let now = std::time::Instant::now();
                
                unsafe {
                    if let Some(last_time) = LAST_G_PRESS {
                        // If second 'g' press within 500ms, go to top
                        if now.duration_since(last_time).as_millis() < 500 {
                            // Go to top (first entry)
                            self.selected_index = 0;
                            self.ensure_selected_visible = true;
                            LAST_G_PRESS = None;
                        } else {
                            LAST_G_PRESS = Some(now);
                        }
                    } else {
                        LAST_G_PRESS = Some(now);
                    }
                }
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::Questionmark)) {
            // Toggle help window with '?' key
            self.show_help = !self.show_help;
        }
    }
    
    fn draw_path_navigation(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Left side: Current path with $ prefix
            let path_ui_width = ui.available_width() - 100.0;
            ui.horizontal(|ui| {
                ui.set_min_width(path_ui_width);
                ui.label(RichText::new("$ ").color(self.colors.gray));
                
                // Get path components
                let mut components = Vec::new();
                let mut current = PathBuf::new();
                
                // Process all path components
                for component in self.current_path.components() {
                    let comp_str = component.as_os_str().to_string_lossy().to_string();
                    if !comp_str.is_empty() {
                        current.push(&comp_str);
                        components.push((comp_str, current.clone()));
                    }
                }
                
                // Display components as clickable buttons
                if components.is_empty() {
                    // Handle empty path case
                    ui.label(RichText::new("/").color(self.colors.yellow));
                } else {
                    // Check if we need to truncate the path
                    let mut path_str = String::new();
                    for (i, (name, _)) in components.iter().enumerate() {
                        // avoid double slashes due to root being a slash too
                        if i > 1 {
                            path_str.push_str("/");
                        }
                        path_str.push_str(name);
                    }
                    
                    // Estimate path width (rough approximation)
                    let estimated_width = path_str.len() as f32 * 7.0; // Approximate width per character                 
                    
                    if estimated_width > path_ui_width && components.len() > 4 {
                        if ui.link(RichText::new(&components[0].0).color(self.colors.yellow)).clicked() {
                            self.navigate_to(components[0].1.clone());
                        }
                        
                        // Add separator
                        ui.label(RichText::new("/").color(self.colors.gray));
                        ui.label(RichText::new("...").color(self.colors.gray));
                        
                        // Show last two components
                        let start_idx = components.len() - 2;
                        for i in start_idx..components.len() {
                            ui.label(RichText::new("/").color(self.colors.gray));
                            if ui.link(RichText::new(&components[i].0).color(self.colors.yellow)).clicked() {
                                self.navigate_to(components[i].1.clone());
                            }
                        }
                    } else {
                        // Show full path
                        for (i, (name, path)) in components.iter().enumerate() {
                            // skip the slash between root and its folder because root is already a slash
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
            
            // Right side: Navigation hints
            ui.horizontal(|ui| {
                ui.label(RichText::new("? for help").color(self.colors.gray)).on_hover_text("Show keyboard shortcuts");
            });
        });
    }

    fn draw_table_header(&self, ui: &mut Ui) {
        // Reduce vertical spacing
        ui.style_mut().spacing.item_spacing.y = 2.0;
        
        // Fixed height for header to match entry rows
        let row_height = 20.0;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), row_height), egui::Sense::hover());
        
        // Position elements within the row
        let mut cursor = rect.left_top();
        
        // Set up consistent spacing - must match draw_entry_row
        let icon_width = 24.0;
        let date_width = 180.0; // Fixed width for date column
        let size_width = 80.0;  // Fixed width for size column
        let name_width = rect.width() - icon_width - 10.0 - date_width - size_width - 20.0; // Remaining space for name
        
        // Skip icon space
        cursor.x += icon_width + 10.0;
        
        // Name header - use strong text
        ui.painter().text(
            cursor + egui::vec2(0.0, row_height/2.0), 
            Align2::LEFT_CENTER, 
            "Name", 
            egui::FontId::proportional(14.0), 
            self.colors.green
        );
        cursor.x += name_width;
        
        // Modified date header
        ui.painter().text(
            cursor + egui::vec2(0.0, row_height/2.0), 
            Align2::LEFT_CENTER, 
            "Modified", 
            egui::FontId::proportional(14.0), 
            self.colors.green
        );
        cursor.x += date_width;
        
        // Size header
        ui.painter().text(
            cursor + egui::vec2(0.0, row_height/2.0), 
            Align2::LEFT_CENTER, 
            "Size", 
            egui::FontId::proportional(14.0), 
            self.colors.green
        );
        
        ui.separator();
    }

    fn draw_entry_row(&self, ui: &mut Ui, entry: &DirEntry, is_selected: bool) -> bool {
        // Fixed height for each row
        let row_height = 20.0;
        
        // Allocate space for the row and detect interactions
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), row_height),
            egui::Sense::click(),
        );
        
        // Draw selection background if this entry is selected
        if is_selected {
            ui.painter().rect_filled(rect, 0.0, self.colors.selected_bg);
        }
        
        // Position elements within the row
        let mut cursor = rect.left_top();
        
        // Set up consistent spacing - must match draw_table_header
        let icon_width = 24.0;
        let date_width = 180.0; // Fixed width for date column
        let size_width = 80.0;  // Fixed width for size column
        let name_width = rect.width() - icon_width - 10.0 - date_width - size_width - 20.0; // Remaining space for name
        
        // Draw folder/file icon
        let icon = if entry.is_dir { "📁" } else {"📄"};
        let icon_color = if is_selected {
            Color32::WHITE
        } else {
            self.colors.blue
        };
        
        ui.painter().text(
            cursor + egui::vec2(10.0, row_height/2.0), 
            Align2::LEFT_CENTER, 
            icon, 
            egui::FontId::proportional(14.0), 
            icon_color
        );
        cursor.x += icon_width + 10.0;
        
        let name_clip_rect = egui::Rect::from_min_size(
            cursor,
            egui::vec2(name_width, row_height)
        );
        
        // Check if name needs truncation
        // Simple heuristic: assume each character takes about 7 pixels on average
        let estimated_width = entry.name.len() as f32 * 7.0;
        let name_text = if estimated_width > name_width {
            // Name is too long, add ellipsis indicator
            let available_chars = (name_width / 7.0) as usize - 3; // Approximate chars that fit minus "..."
            if available_chars < entry.name.len() {
                let half = available_chars / 2;
                format!("{}...{}", 
                    &entry.name[..half], 
                    &entry.name[entry.name.len() - half..])
            } else {
                entry.name.clone()
            }
        } else {
            entry.name.clone()
        };
        
        // Draw the name with clipping
        let name_color = if is_selected { Color32::WHITE } else { self.colors.fg };
        ui.painter().with_clip_rect(name_clip_rect).text(
            cursor + egui::vec2(0.0, row_height/2.0), 
            Align2::LEFT_CENTER, 
            &name_text, 
            egui::FontId::proportional(14.0), 
            name_color
        );
        cursor.x += name_width;
        
        // Modified date
        let modified_date = DateTime::<Local>::from(entry.modified)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let date_color = if is_selected { Color32::WHITE } else { self.colors.orange };
        ui.painter().text(
            cursor + egui::vec2(0.0, row_height/2.0), 
            Align2::LEFT_CENTER, 
            &modified_date, 
            egui::FontId::proportional(14.0), 
            date_color
        );
        cursor.x += date_width;
        
        // Size (only for files, not directories)
        let size_str = if entry.is_dir {
            "".to_string()
        } else {
            humansize::format_size(entry.size, humansize::BINARY)
        };
        let size_color = if is_selected { Color32::WHITE } else { self.colors.purple };
        ui.painter().text(
            cursor + egui::vec2(0.0, row_height/2.0), 
            Align2::LEFT_CENTER, 
            &size_str, 
            egui::FontId::proportional(14.0), 
            size_color
        );
        
        response.clicked()
    }
    
    fn draw_help_window(&mut self, ctx: &egui::Context) {
        if !self.show_help {
            return;
        }
        
        egui::Window::new("Keyboard Shortcuts")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(10.0, 6.0);
                
                ui.heading(RichText::new("Navigation").color(self.colors.green));
                
                let table = egui::Grid::new("help_grid");
                table.show(ui, |ui| {
                    ui.label(RichText::new("j / ↓").color(self.colors.yellow));
                    ui.label("Move down");
                    ui.end_row();
                    
                    ui.label(RichText::new("k / ↑").color(self.colors.yellow));
                    ui.label("Move up");
                    ui.end_row();
                    
                    ui.label(RichText::new("h / ←").color(self.colors.yellow));
                    ui.label("Go to parent directory");
                    ui.end_row();
                    
                    ui.label(RichText::new("l / → / Enter").color(self.colors.yellow));
                    ui.label("Open directory or file");
                    ui.end_row();
                    
                    ui.label(RichText::new("gg").color(self.colors.yellow));
                    ui.label("Go to first entry");
                    ui.end_row();
                    
                    ui.label(RichText::new("G").color(self.colors.yellow));
                    ui.label("Go to last entry");
                    ui.end_row();
                    
                    ui.label(RichText::new("?").color(self.colors.yellow));
                    ui.label("Toggle this help window");
                    ui.end_row();
                });
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(RichText::new("Close").color(self.colors.aqua)).clicked() {
                            self.show_help = false;
                        }
                    });
                });
            });
    }
}

impl eframe::App for Kiorg {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Interactive path navigation with navigation hints
            self.draw_path_navigation(ui);
            
            // Handle keyboard input
            self.handle_key_press(ctx);
            
            ui.separator();
            
            // Table header
            self.draw_table_header(ui);
            
            // Calculate available height for scroll area
            let available_height = ui.available_height();
            let row_height = 20.0;
            let visible_rows = (available_height / row_height).floor() as usize;
            
            // Use a ScrollArea with ID to maintain scroll state
            let scroll_id = ui.id().with("file_list_scroll");
            let mut scroll_area = egui::ScrollArea::vertical()
                .id_salt(scroll_id)
                .auto_shrink([false; 2])
                .max_height(available_height);

            if self.ensure_selected_visible && !self.entries.is_empty() && visible_rows > 0 {
                // Calculate the scroll position to center the selected item
                let total_height = self.entries.len() as f32 * row_height;
                let selected_y = self.selected_index as f32 * row_height;
                
                // Always center the selected item in the view
                let ideal_scroll_top = selected_y - (available_height / 2.0) + (row_height / 2.0);
                
                // Add extra padding to ensure we can scroll all the way to the bottom
                let extra_scroll_padding = row_height * 2.0;
                let max_scroll = (total_height - available_height + extra_scroll_padding).max(0.0);
                let scroll_top = ideal_scroll_top.clamp(0.0, max_scroll);
                
                // Set the scroll position directly
                scroll_area = scroll_area.vertical_scroll_offset(scroll_top);
                
                // Reset the flag after applying
                self.ensure_selected_visible = false;
            }

            // Display directory entries with proper scrolling
            scroll_area.show_rows(
                ui,
                row_height,
                self.entries.len() + 3,
                |ui: &mut Ui, row_range| {
                    // Reduce vertical spacing between items
                    ui.style_mut().spacing.item_spacing.y = 0.0;
                    
                    let entries = self.entries.clone(); // Clone to avoid borrowing issues
                    let mut clicked_path = None;
                    
                    for i in row_range {
                        if i < entries.len() {
                            let entry = &entries[i];
                            let is_selected = i == self.selected_index;
                            if self.draw_entry_row(ui, entry, is_selected) {
                                clicked_path = Some(entry.path.clone());
                            }
                        } else {
                            // Add empty space for extra rows at the end
                            ui.allocate_space(egui::vec2(ui.available_width(), row_height));
                        }
                    }
                    
                    // Handle navigation if a directory was clicked
                    if let Some(path) = clicked_path {
                        self.navigate_to(path);
                    }
                }
            );
        });
        
        // Draw help window if enabled
        self.draw_help_window(ctx);
    }
}

// Load configuration from file
fn load_config() -> Config {
    let config_dir = dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("kiorg");
    
    // Create config directory if it doesn't exist
    if !config_dir.exists() {
        let _ = fs::create_dir_all(&config_dir);
    }
    
    let config_path = config_dir.join("config.toml");
    
    // If config file doesn't exist, create it with default values
    if !config_path.exists() {
        let default_config = Config::default();
        let toml_str = toml::to_string_pretty(&default_config).unwrap_or_default();
        let _ = fs::write(&config_path, toml_str);
        return default_config;
    }
    
    // Read and parse config file
    let mut file = match fs::File::open(&config_path) {
        Ok(file) => file,
        Err(_) => return Config::default(),
    };
    
    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return Config::default();
    }
    
    toml::from_str(&contents).unwrap_or_default()
}

fn hex_to_color32(hex: &str) -> Color32 {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[..2], 16).unwrap();
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
    Color32::from_rgb(r, g, b)
}

fn main() -> Result<(), eframe::Error> {
    // Add dirs_next dependency for config directory
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Kiorg",
        options,
        Box::new(|cc| Ok(Box::new(Kiorg::new(cc)))),
    )
}
