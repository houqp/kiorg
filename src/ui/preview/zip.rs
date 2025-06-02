//! Zip archive preview module

use crate::config::colors::AppColors;
use crate::models::preview_content::ZipEntry;
use egui::RichText;
use std::fs::File;
use std::path::Path;
use zip::ZipArchive;

/// Render zip archive content
pub fn render(ui: &mut egui::Ui, entries: &[ZipEntry], colors: &AppColors) {
    // Display zip file contents
    ui.label(
        RichText::new("Zip Archive Contents:")
            .color(colors.fg)
            .strong(),
    );
    ui.add_space(5.0);

    // Constants for the list
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
                // Create a visual indicator for directories
                let entry_text = if entry.is_dir {
                    RichText::new(format!("📁 {}", entry.name)).strong()
                } else {
                    RichText::new(format!("📄 {}", entry.name))
                };

                ui.label(entry_text.color(colors.fg));
            }
        });
}

/// Read entries from a zip file and return them as a vector of `ZipEntry`
pub fn read_zip_entries(path: &Path) -> Result<Vec<ZipEntry>, String> {
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

        // Create a ZipEntry from the file
        let entry = ZipEntry {
            name: file.name().to_string(),
            size: file.size(),
            is_dir: file.is_dir(),
        };

        entries.push(entry);
    }

    // Sort entries: directories first, then files, both alphabetically
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    Ok(entries)
}
