use egui::Ui;
use std::path::PathBuf;

use crate::config::colors::AppColors;
use crate::models::tab::Tab;
use crate::ui::file_list;
use crate::ui::style::VERTICAL_PADDING;

const ROW_HEIGHT: f32 = 24.0;

pub struct CenterPanel {
    width: f32,
    height: f32,
}

pub struct CenterPanelResult {
    pub path_to_navigate: Option<PathBuf>,
    pub entry_to_rename: Option<(PathBuf, String)>,
}

// Group parameters for the draw function to avoid too many arguments warning
pub struct CenterPanelDrawParams<'a> {
    pub tab: &'a Tab,
    pub bookmarks: &'a [PathBuf],
    pub colors: &'a AppColors,
    pub rename_mode: bool,
    pub new_name: &'a mut String,
    pub rename_focus: bool,
    pub ensure_selected_visible: bool,
}

impl CenterPanel {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Handles clipboard paste operations (copy/cut)
    /// Returns true if any operation was performed
    pub fn handle_clipboard_operations(
        clipboard: &mut Option<(Vec<PathBuf>, bool)>,
        current_path: &std::path::Path,
    ) -> bool {
        if let Some((paths, is_cut)) = clipboard.take() {
            for path in paths {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();
                let mut new_path = current_path.join(name);

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
                    new_path = current_path.join(format!("{}_{}{}", stem, counter, ext));
                    counter += 1;
                }

                if is_cut {
                    if let Err(e) = std::fs::rename(&path, &new_path) {
                        eprintln!("Failed to move: {e}");
                    }
                } else if let Err(e) = std::fs::copy(&path, &new_path) {
                    eprintln!("Failed to copy: {e}");
                }
            }
            true
        } else {
            false
        }
    }

    pub fn draw(&self, ui: &mut Ui, params: CenterPanelDrawParams) -> CenterPanelResult {
        let entries = params.tab.entries.clone();
        let selected_index = params.tab.selected_index;
        let selected_entries = params.tab.selected_entries.clone();

        let mut result = CenterPanelResult {
            path_to_navigate: None,
            entry_to_rename: None,
        };

        ui.vertical(|ui| {
            ui.set_min_width(self.width);
            ui.set_max_width(self.width);
            ui.set_min_height(self.height);
            ui.add_space(VERTICAL_PADDING);
            file_list::draw_table_header(ui, params.colors);

            // Calculate available height for scroll area
            let available_height = self.height - ROW_HEIGHT - VERTICAL_PADDING * 2.0;

            egui::ScrollArea::vertical()
                .id_salt("current_list_scroll")
                .auto_shrink([false; 2])
                .max_height(available_height)
                .show(ui, |ui| {
                    // Set the width of the content area
                    let scrollbar_width = 6.0;
                    ui.set_min_width(self.width - scrollbar_width);
                    ui.set_max_width(self.width - scrollbar_width);

                    // Draw entries
                    for (i, entry) in entries.iter().enumerate() {
                        let is_selected = i == selected_index;
                        let is_in_selection = selected_entries.contains(&entry.path);

                        if file_list::draw_entry_row(
                            ui,
                            file_list::EntryRowParams {
                                entry,
                                is_selected,
                                colors: params.colors,
                                rename_mode: params.rename_mode && is_selected,
                                new_name: params.new_name,
                                rename_focus: params.rename_focus && is_selected,
                                is_marked: is_in_selection,
                                is_bookmarked: params.bookmarks.contains(&entry.path),
                            },
                        ) {
                            if params.rename_mode {
                                result.entry_to_rename =
                                    Some((entry.path.clone(), params.new_name.clone()));
                            } else {
                                result.path_to_navigate = Some(entry.path.clone());
                            }
                        }
                    }

                    // Handle scrolling to selected item
                    if params.ensure_selected_visible && !entries.is_empty() {
                        let selected_pos = selected_index as f32 * ROW_HEIGHT;
                        ui.scroll_to_rect(
                            egui::Rect::from_min_size(
                                egui::pos2(0.0, selected_pos),
                                egui::vec2(self.width, ROW_HEIGHT),
                            ),
                            Some(egui::Align::Center),
                        );
                    }
                });
        });

        result
    }
}
