//! Zip archive preview module

use egui::RichText;
use std::fs::File;
use zip::ZipArchive;

use crate::config::colors::AppColors;
use crate::models::dir_entry::DirEntryMeta;
use crate::models::preview_content::{CachedPreviewContent, ZipEntry};
use crate::ui::preview::{prefix_dir_name, prefix_file_name};
use crate::utils::preview_cache;

/// Render zip archive content
pub fn render(ui: &mut egui::Ui, entries: &[ZipEntry], colors: &AppColors) {
    // Display zip file contents
    ui.label(
        RichText::new("Zip Archive Contents:")
            .color(colors.fg)
            .strong(),
    );
    ui.add_space(5.0);

    // TODO: calculate the correct row height
    const ROW_HEIGHT: f32 = 10.0;

    // Get the total number of entries
    let total_rows = entries.len();

    // Use show_rows for better performance
    egui::ScrollArea::vertical()
        .id_salt("zip_entries_scroll")
        .auto_shrink([false; 2])
        .show_rows(ui, ROW_HEIGHT, total_rows, |ui, row_range| {
            // Set width for the content area
            let available_width = ui.available_width();
            ui.set_min_width(available_width);

            // Display entries in the visible range
            for row_index in row_range {
                let entry = &entries[row_index];
                let entry_text = if entry.is_dir {
                    RichText::new(prefix_dir_name(&entry.name)).strong()
                } else {
                    RichText::new(prefix_file_name(&entry.name))
                };

                ui.horizontal(|ui| {
                    ui.label(entry_text.color(colors.fg));
                    if !entry.is_dir {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(crate::utils::format::format_size(
                                    entry.size,
                                    entry.is_dir,
                                ))
                                .color(colors.fg_light),
                            );
                        });
                    }
                });
            }
        });
}

/// Read entries from a zip file and return them as a vector of `ZipEntry`
pub fn read_zip_entries(entry: DirEntryMeta) -> Result<Vec<ZipEntry>, String> {
    let path = &entry.path;
    // Open the zip file
    let file = File::open(path).map_err(|e| format!("Failed to open zip file: {e}"))?;

    // Create a zip archive from the file
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read zip archive: {e}"))?;

    // Create a vector to store the entries
    let mut entries = Vec::new();

    // Process each file in the archive
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {e}"))?;

        let size = file.size();
        let is_dir = file.is_dir();
        // Create a ZipEntry from the file
        let entry = ZipEntry {
            name: file.name().to_string(),
            size,
            is_dir,
        };

        entries.push(entry);
    }

    // Sort entries: directories first, then files, both alphabetically
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    // Spawn background task to save cache
    let cached = CachedPreviewContent::Zip(entries.clone());
    std::thread::spawn(move || {
        let cache_key = preview_cache::calculate_cache_key(&entry);
        if let Err(e) = preview_cache::save_preview(&cache_key, &cached) {
            tracing::warn!("Failed to save zip preview cache: {}", e);
        }
    });

    Ok(entries)
}
