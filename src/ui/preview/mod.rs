//! Preview content modules for different file types

pub mod doc;
pub mod image;
pub mod loading;
pub mod text;
pub mod zip;

use crate::app::Kiorg;
use crate::models::preview_content::PreviewContent;

/// Update the preview cache based on the selected file
pub fn update_cache(app: &mut Kiorg, ctx: &egui::Context) {
    let tab = app.tab_manager.current_tab_ref();
    let selected_path = tab.entries.get(tab.selected_index).map(|e| e.path.clone());

    // Check if the selected file is the same as the cached one in app
    if selected_path.as_ref() == app.cached_preview_path.as_ref() {
        return; // Cache hit, no need to update
    }

    // Cache miss, update the preview content in app
    let maybe_entry = selected_path.as_ref().and_then(|p| {
        tab.entries.iter().find(|entry| &entry.path == p).cloned() // Clone the entry data if found
    });
    app.cached_preview_path = selected_path; // Update the cached path in app regardless

    let entry = match maybe_entry {
        Some(entry) => entry,
        None => {
            app.preview_content = None; // No content to display
            app.cached_preview_path = None; // Clear cache in app if no file is selected
            return;
        } // No entry selected, clear the preview content
    };

    if entry.is_dir {
        app.preview_content = Some(PreviewContent::text(format!(
            "Directory: {}",
            entry.path.file_name().unwrap_or_default().to_string_lossy()
        )));
        return;
    }

    let ext = entry
        .path
        .extension()
        .and_then(|e| e.to_str()).map_or_else(|| "__unknown__".to_string(), str::to_lowercase);

    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" => {
            let ctx_clone = ctx.clone();
            loading::load_preview_async(app, entry.path, move |path| {
                image::read_image_with_metadata(&path, &ctx_clone)
            });
        }
        "zip" | "jar" | "war" | "ear" => {
            loading::load_preview_async(app, entry.path, |path| {
                let result = zip::read_zip_entries(&path);
                result.map(PreviewContent::zip)
            });
        }
        "epub" => {
            loading::load_preview_async(app, entry.path, |path| {
                doc::extract_epub_metadata(&path)
            });
        }
        "pdf" => {
            loading::load_preview_async(app, entry.path, |path| {
                doc::extract_pdf_metadata(&path, 0)
            });
        }
        // All other files
        _ => {
            let size = entry.size;
            if size == 0 {
                app.preview_content = Some(PreviewContent::text("Empty file".to_string()));
                return;
            }
            text::load_async(app, entry.path, size);
        }
    }
}
