//! Directory preview module

use crate::config::colors::AppColors;
use crate::models::preview_content::DirectoryEntry;
use crate::ui::preview::{prefix_dir_name, prefix_file_name};
use egui::RichText;
use std::fs;
use std::path::Path;

/// Render directory content
pub fn render(ui: &mut egui::Ui, entries: &[DirectoryEntry], colors: &AppColors) {
    // Display directory contents
    ui.label(
        RichText::new("Directory Contents:")
            .color(colors.fg)
            .strong(),
    );
    ui.add_space(5.0);

    // Constants for the list
    const ROW_HEIGHT: f32 = 10.0; // TODO: calculate the correct row height

    // Get the total number of entries
    let total_rows = entries.len();

    // Use show_rows for better performance
    egui::ScrollArea::vertical()
        .id_salt("dir_entries_scroll")
        .auto_shrink([false; 2])
        .show_rows(ui, ROW_HEIGHT, total_rows, |ui, row_range| {
            // Set width for the content area
            let available_width = ui.available_width();
            ui.set_min_width(available_width);

            // Display entries in the visible range
            for row_index in row_range {
                let entry = &entries[row_index];
                // Create a visual indicator for directories
                let entry_text = if entry.is_dir {
                    RichText::new(prefix_dir_name(&entry.name)).strong()
                } else {
                    RichText::new(prefix_file_name(&entry.name))
                };

                ui.label(entry_text.color(colors.fg));
            }
        });
}

/// Reuses `DirectoryEntry` for simplicity, as it has the required fields (name, is_dir)
pub fn read_dir_entries(path: &Path) -> Result<Vec<DirectoryEntry>, String> {
    let mut entries = Vec::new();
    let read_dir = fs::read_dir(path).map_err(|e| format!("Failed to read directory: {e}"))?;

    for entry_result in read_dir {
        let entry = entry_result.map_err(|e| format!("Failed to read directory entry: {e}"))?;
        let path = entry.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let is_dir = path.is_dir();

        entries.push(DirectoryEntry { name, is_dir });
    }

    // Sort entries: directories first, then by name
    entries.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(entries)
}
