//! Preview content modules for different file types

pub mod directory;
pub mod doc;
pub mod image;
pub mod loading;
pub mod tar;
pub mod text;
pub mod video;
pub mod zip;

use crate::app::Kiorg;
use crate::models::preview_content::PreviewContent;

// return extension if available, otherwise return file name
// returned values are always lowercased
pub fn path_to_ext_info(path: &std::path::Path) -> String {
    let filename = match path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return "<unknown>".into(),
    };
    // for hidden files, avoid spliting on the leading dot
    let split_name = if let Some(stripped) = filename.strip_prefix(".") {
        stripped
    } else {
        &filename[..]
    };
    let parts = split_name.split(".").collect::<Vec<&str>>();
    match parts.len() {
        0 => filename.to_lowercase(),
        1 => {
            // No extension found, return the filename as is
            filename.to_lowercase()
        }
        2 => {
            // Single extension found, return it lowercased
            parts[1].to_lowercase()
        }
        _ => {
            let last = parts[parts.len() - 1].to_lowercase();
            match last.as_str() {
                "zstd" | "gz" | "bz2" | "xz" => {
                    // Handle cases like tar.gz, tar.bz2, etc.
                    let second_last = parts[parts.len() - 2].to_lowercase();
                    if second_last != "tar" {
                        last
                    } else {
                        format!("{second_last}.{last}")
                    }
                }
                _ => last,
            }
        }
    }
}

// Macros for file extension patterns to avoid duplication
#[macro_export]
macro_rules! image_extensions {
    () => {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg"
    };
}

#[macro_export]
macro_rules! video_extensions {
    () => {
        "mp4" | "m4v" | "mkv" | "webm" | "mov" | "avi" | "wmv" | "mpg" | "flv"
    };
}

#[macro_export]
macro_rules! zip_extensions {
    () => {
        "zip" | "jar" | "war" | "ear"
    };
}

#[macro_export]
macro_rules! tar_extensions {
    () => {
        "tar" | "tgz" | "tar.gz" | "tbz" | "tbz2" | "tar.bz2"
    };
}

#[macro_export]
macro_rules! pdf_extensions {
    () => {
        "pdf"
    };
}

#[macro_export]
macro_rules! epub_extensions {
    () => {
        "epub"
    };
}

// Public macros for use in other modules
pub use epub_extensions;
pub use image_extensions;
pub use pdf_extensions;
pub use tar_extensions;
pub use video_extensions;
pub use zip_extensions;

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

    let ext = path_to_ext_info(&entry.path);
    match ext.as_str() {
        image_extensions!() => {
            let ctx_clone = ctx.clone();
            loading::load_preview_async(app, entry.path, move |path| {
                image::read_image_with_metadata(&path, &ctx_clone)
            });
        }
        video_extensions!() => {
            let ctx_clone = ctx.clone();
            loading::load_preview_async(app, entry.path, move |path| {
                video::read_video_with_metadata(&path, &ctx_clone)
            });
        }
        zip_extensions!() => {
            loading::load_preview_async(app, entry.path, |path| {
                let result = zip::read_zip_entries(&path);
                result.map(PreviewContent::zip)
            });
        }
        tar_extensions!() => {
            loading::load_preview_async(app, entry.path, |path| {
                let result = tar::read_tar_entries(&path);
                result.map(PreviewContent::tar)
            });
        }
        epub_extensions!() => {
            loading::load_preview_async(app, entry.path, |path| doc::extract_epub_metadata(&path));
        }
        pdf_extensions!() => {
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
    use std::path::Path;

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

    #[test]
    fn test_path_to_ext_info_no_extension() {
        assert_eq!(path_to_ext_info(Path::new("filename")), "filename");
        assert_eq!(path_to_ext_info(Path::new("README")), "readme");
        assert_eq!(path_to_ext_info(Path::new("Makefile")), "makefile");
    }

    #[test]
    fn test_path_to_ext_info_single_extension() {
        assert_eq!(path_to_ext_info(Path::new("file.txt")), "txt");
        assert_eq!(path_to_ext_info(Path::new("image.PNG")), "png");
        assert_eq!(path_to_ext_info(Path::new("script.py")), "py");
        assert_eq!(path_to_ext_info(Path::new("document.PDF")), "pdf");
    }

    #[test]
    fn test_path_to_ext_info_multiple_extensions() {
        assert_eq!(path_to_ext_info(Path::new("file.backup.txt")), "txt");
        assert_eq!(path_to_ext_info(Path::new("archive.tar.gz")), "tar.gz");
        assert_eq!(path_to_ext_info(Path::new("data.tar.bz2")), "tar.bz2");
        assert_eq!(path_to_ext_info(Path::new("backup.tar.xz")), "tar.xz");
        assert_eq!(path_to_ext_info(Path::new("file.tar.zstd")), "tar.zstd");
    }

    #[test]
    fn test_path_to_ext_info_compression_extensions() {
        assert_eq!(path_to_ext_info(Path::new("file.gz")), "gz");
        assert_eq!(path_to_ext_info(Path::new("file.bz2")), "bz2");
        assert_eq!(path_to_ext_info(Path::new("file.xz")), "xz");
        assert_eq!(path_to_ext_info(Path::new("file.zstd")), "zstd");
    }

    #[test]
    fn test_path_to_ext_info_hidden_files() {
        assert_eq!(path_to_ext_info(Path::new(".bashrc")), ".bashrc");
        assert_eq!(path_to_ext_info(Path::new(".gitignore")), ".gitignore");
        assert_eq!(path_to_ext_info(Path::new(".config.json")), "json");
        assert_eq!(path_to_ext_info(Path::new(".hidden.tar.gz")), "tar.gz");
    }

    #[test]
    fn test_path_to_ext_info_parent_directory() {
        assert_eq!(path_to_ext_info(Path::new("..")), "<unknown>");
    }

    #[test]
    fn test_path_to_ext_info_no_filename() {
        assert_eq!(path_to_ext_info(Path::new("")), "<unknown>");
        assert_eq!(path_to_ext_info(Path::new("/")), "<unknown>");
    }

    #[test]
    fn test_path_to_ext_info_edge_cases() {
        // File starting with dot but no extension
        assert_eq!(path_to_ext_info(Path::new(".file")), ".file");

        // Multiple dots but compression extension only
        assert_eq!(path_to_ext_info(Path::new("file.name.gz")), "gz");
        assert_eq!(path_to_ext_info(Path::new("long.file.name.bz2")), "bz2");

        // Empty extension parts
        assert_eq!(path_to_ext_info(Path::new("file.")), "");
        assert_eq!(path_to_ext_info(Path::new("file..")), "");

        // Case sensitivity
        assert_eq!(path_to_ext_info(Path::new("FILE.TXT")), "txt");
        assert_eq!(path_to_ext_info(Path::new("Archive.TAR.GZ")), "tar.gz");
    }

    #[test]
    fn test_path_to_ext_info_with_path() {
        assert_eq!(path_to_ext_info(Path::new("/path/to/file.txt")), "txt");
        assert_eq!(
            path_to_ext_info(Path::new("../relative/path/file.tar.gz")),
            "tar.gz"
        );
        assert_eq!(path_to_ext_info(Path::new("./current/dir/file")), "file");
    }

    #[test]
    fn test_path_to_ext_info_special_tar_cases() {
        // Regular tar files
        assert_eq!(path_to_ext_info(Path::new("archive.tar")), "tar");
        assert_eq!(path_to_ext_info(Path::new("backup.tgz")), "tgz");
        assert_eq!(path_to_ext_info(Path::new("data.tbz")), "tbz");
        assert_eq!(path_to_ext_info(Path::new("files.tbz2")), "tbz2");

        // Edge case: empty second-to-last part
        assert_eq!(path_to_ext_info(Path::new("file..gz")), "gz");
    }
}
