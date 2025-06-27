//! Preview content modules for different file types

pub mod directory;
pub mod doc;
pub mod image;
pub mod loading;
pub mod text;
pub mod zip;

use crate::app::Kiorg;
use crate::models::preview_content::PreviewContent;

#[inline]
pub fn prefix_file_name(name: &str) -> String {
    format!("📄 {name}")
}

#[inline]
pub fn prefix_dir_name(name: &str) -> String {
    format!("📁 {name}")
}

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
        loading::load_preview_async(app, entry.path, |path| {
            let result = directory::read_dir_entries(&path);
            result.map(PreviewContent::directory)
        });
        return;
    }

    let ext = entry
        .path
        .extension()
        .and_then(|e| e.to_str())
        .map_or_else(|| "__unknown__".to_string(), str::to_lowercase);

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
            loading::load_preview_async(app, entry.path, |path| doc::extract_epub_metadata(&path));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_file_name_uses_nbsp() {
        let file_name = "test_file.txt";
        let prefixed_name = prefix_file_name(file_name);
        // Expected: "📄 test_file.txt" (with NBSP between icon and name)
        assert_eq!(prefixed_name, format!("📄{}test_file.txt", '\u{00A0}'));
        assert_eq!(prefixed_name.chars().nth(1), Some('\u{00A0}'));
    }

    #[test]
    fn test_prefix_dir_name_uses_nbsp() {
        let dir_name = "test_dir";
        let prefixed_name = prefix_dir_name(dir_name);
        // Expected: "📁 test_dir" (with NBSP between icon and name)
        assert_eq!(prefixed_name, format!("📁{}test_dir", '\u{00A0}'));
        assert_eq!(prefixed_name.chars().nth(1), Some('\u{00A0}'));
    }
}
