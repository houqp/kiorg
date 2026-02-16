//! Tar archive preview module

use egui::RichText;
use std::fs::File;
use std::io::BufReader;
use tar::Archive;

use crate::config::colors::AppColors;
use crate::models::dir_entry::DirEntryMeta;
use crate::models::preview_content::{CachedPreviewContent, TarEntry};
use crate::ui::preview::{prefix_dir_name, prefix_file_name};
use crate::utils::preview_cache;

/// Render tar archive content
pub fn render(ui: &mut egui::Ui, entries: &[TarEntry], colors: &AppColors) {
    // Display tar file contents
    ui.label(
        RichText::new("Tar Archive Contents:")
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
        .id_salt("tar_entries_scroll")
        .auto_shrink([false; 2])
        .show_rows(ui, ROW_HEIGHT, total_rows, |ui, row_range| {
            // Set width for the content area
            let available_width = ui.available_width();
            ui.set_min_width(available_width);

            // Display entries in the visible range
            for row_index in row_range {
                let entry = &entries[row_index];
                ui.horizontal(|ui| {
                    // Display permissions
                    ui.label(
                        RichText::new(&entry.permissions)
                            .color(colors.fg_light)
                            .family(egui::FontFamily::Monospace),
                    );
                    ui.add_space(2.0);

                    // Format entry name with prefix
                    let name_text = if entry.is_dir {
                        prefix_dir_name(&entry.name)
                    } else {
                        prefix_file_name(&entry.name)
                    };
                    ui.label(RichText::new(&name_text).color(colors.fg));

                    // Push size to the right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if !entry.is_dir {
                            ui.label(
                                RichText::new(crate::utils::format::format_size(
                                    entry.size,
                                    entry.is_dir,
                                ))
                                .color(colors.fg_light),
                            );
                        }
                    });
                });
            }
        });
}

/// Read entries from a tar file and return them as a vector of `TarEntry`
pub fn read_tar_entries(entry: DirEntryMeta) -> Result<Vec<TarEntry>, String> {
    let path = &entry.path;
    let file = File::open(path).map_err(|e| format!("Failed to open tar file: {e}"))?;

    // Try to determine if it's compressed by the file extension
    let mut archive: Box<dyn std::io::Read> = Box::new(BufReader::new(file));

    // Handle compressed tar files
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext.to_lowercase().as_str() {
            "gz" | "tgz" => {
                let file =
                    File::open(path).map_err(|e| format!("Failed to reopen tar.gz file: {e}"))?;
                let gz = flate2::read::GzDecoder::new(BufReader::new(file));
                archive = Box::new(gz);
            }
            "bz2" | "tbz" | "tbz2" => {
                let file =
                    File::open(path).map_err(|e| format!("Failed to reopen tar.bz2 file: {e}"))?;
                let bz2 = bzip2::read::BzDecoder::new(BufReader::new(file));
                archive = Box::new(bz2);
            }
            _ => {
                let file =
                    File::open(path).map_err(|e| format!("Failed to reopen tar file: {e}"))?;
                archive = Box::new(BufReader::new(file));
            }
        }
    }

    let mut tar = Archive::new(archive);
    let mut entries = Vec::new();

    let tar_entries = tar
        .entries()
        .map_err(|e| format!("Failed to read tar entries: {e}"))?;

    for entry_result in tar_entries {
        let entry = entry_result.map_err(|e| format!("Failed to read tar entry: {e}"))?;
        let header = entry.header();

        let path = entry
            .path()
            .map_err(|e| format!("Failed to get entry path: {e}"))?;
        let name = path.to_string_lossy().to_string();

        let is_dir = header.entry_type() == tar::EntryType::Directory;
        let size = header.size().unwrap_or(0);
        let mode = header.mode().unwrap_or(0);
        let permissions = format!("{:o}", mode & 0o777);

        entries.push(TarEntry {
            name,
            size,
            is_dir,
            permissions,
        });
    }

    // Spawn background task to save cache
    let cached = CachedPreviewContent::Tar(entries.clone());
    std::thread::spawn(move || {
        let cache_key = preview_cache::calculate_cache_key(&entry);
        if let Err(e) = preview_cache::save_preview(&cache_key, &cached) {
            tracing::warn!("Failed to save tar preview cache: {}", e);
        }
    });

    Ok(entries)
}
