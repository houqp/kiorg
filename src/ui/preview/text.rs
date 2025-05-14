//! Text preview module

use crate::app::Kiorg;
use crate::config::colors::AppColors;
use crate::models::preview_content::PreviewContent;
use crate::ui::preview::loading::load_preview_async;
use egui::RichText;
use file_type::FileType;
use std::io::Read;
use std::path::PathBuf;

/// Render text content
pub fn render(ui: &mut egui::Ui, text: &str, colors: &AppColors) {
    ui.label(RichText::new(text).color(colors.fg));
}

/// Render empty state when no file is selected
pub fn render_empty(ui: &mut egui::Ui, colors: &AppColors) {
    ui.label(RichText::new("No file selected").color(colors.fg));
}

/// Load text content asynchronously
pub fn load_async(app: &mut Kiorg, path: PathBuf, file_size: u64) {
    load_preview_async(app, path, move |path| try_load_utf8_str(path, file_size));
}

/// Try to load a file as UTF-8 text
pub fn try_load_utf8_str(path: PathBuf, file_size: u64) -> Result<PreviewContent, String> {
    // TODO: reuse the buffer between file reads
    let mut bytes: Vec<u8> = vec![0; std::cmp::min(1000, file_size as usize)];

    let mut file = match std::fs::File::open(&path) {
        Ok(file) => file,
        Err(e) => return Ok(PreviewContent::text(format!("Error opening file: {}", e))),
    };
    let bytes_read = match file.read(&mut bytes) {
        Ok(bytes_read) => bytes_read,
        Err(e) => return Ok(PreviewContent::text(format!("Error reading file: {}", e))),
    };
    let content = match std::str::from_utf8(&bytes[..bytes_read]) {
        Ok(content) => Some(content.to_string()),
        Err(e) => {
            // Extract valid UTF-8 up to the error
            let valid_up_to = e.valid_up_to();

            // If we have a substantial amount of valid UTF-8 (within 4 bytes of 1000),
            // use from_utf8_lossy to display what we can
            if valid_up_to > bytes_read - 4 {
                Some(String::from_utf8_lossy(&bytes[..valid_up_to]).to_string())
            } else {
                None
            }
        }
    };

    match content {
        Some(content) => Ok(PreviewContent::text(content)),
        None => render_generic_file(path, file_size),
    }
}

/// Detect file type and return a PreviewContent with generic file information
pub fn render_generic_file(path: PathBuf, size: u64) -> Result<PreviewContent, String> {
    // Try to detect the file type using file_type crate
    let file_type_info = match FileType::try_from_file(&path) {
        Ok(file_type) => {
            let media_types = file_type.media_types().join(", ");
            let extensions = file_type.extensions().join(", ");

            if !media_types.is_empty() {
                format!("File type: {} ({})", media_types, extensions)
            } else if !extensions.is_empty() {
                format!("File type: {}", extensions)
            } else {
                "Unknown file type".to_string()
            }
        }
        Err(_) => "Unknown file type".to_string(),
    };

    // Return the PreviewContent directly
    Ok(PreviewContent::text(format!(
        "{}\n\n{}\n\nSize: {} bytes",
        path.as_path()
            .file_name()
            .unwrap_or_default()
            .to_string_lossy(),
        file_type_info,
        size
    )))
}
