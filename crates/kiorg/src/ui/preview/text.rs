//! Text preview module

use std::io::BufRead;
use std::io::Read;
use std::path::PathBuf;
use std::sync::OnceLock;

use egui::RichText;
use egui_extras::syntax_highlighting::{CodeTheme, SyntectSettings};
use file_type::FileType;
use humansize::{BINARY, format_size};
use syntect::{
    dumps,
    parsing::{SyntaxReference, SyntaxSet},
};

use crate::app::Kiorg;
use crate::config::colors::AppColors;
use crate::models::preview_content::PreviewContent;
use crate::ui::preview::loading::load_preview_async;

static SYNTECT_SETTINGS: OnceLock<SyntectSettings> = OnceLock::new();

fn get_syntect_settings() -> &'static SyntectSettings {
    SYNTECT_SETTINGS.get_or_init(|| egui_extras::syntax_highlighting::SyntectSettings {
        ps: dumps::from_uncompressed_data(yazi_prebuilt::syntaxes())
            .expect("Failed to load syntect syntax"),
        ..Default::default()
    })
}

fn get_syntax_set() -> &'static SyntaxSet {
    &get_syntect_settings().ps
}

/// Render text content
pub fn render(ui: &mut egui::Ui, text: &str, colors: &AppColors) {
    ui.label(RichText::new(text).color(colors.fg));
}

pub fn find_syntax_from_path(path: &std::path::Path) -> Option<&'static SyntaxReference> {
    let syntaxes = get_syntax_set();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();
    if let Some(s) = syntaxes.find_syntax_by_extension(&name) {
        return Some(s);
    }

    let ext = path
        .extension()
        .map(|e| e.to_string_lossy())
        .unwrap_or_default();
    if let Some(s) = syntaxes.find_syntax_by_extension(&ext) {
        return Some(s);
    }

    // detect syntax by feeding first line of the file from path
    let reader = match std::fs::File::open(path) {
        Ok(file) => std::io::BufReader::new(file),
        Err(_) => return None,
    };
    if let Some(Ok(line)) = reader.lines().next() {
        // Use the first line to detect syntax
        return syntaxes.find_syntax_by_first_line(&line);
    }
    None
}

/// Render syntax highlighted code content
pub fn render_highlighted(ui: &mut egui::Ui, text: &str, language: &'static str) {
    let theme = CodeTheme::from_memory(ui.ctx(), ui.style());
    let syntect_settings = get_syntect_settings();
    let layout_job = egui_extras::syntax_highlighting::highlight_with(
        ui.ctx(),
        ui.style(),
        &theme,
        text,
        language,
        syntect_settings,
    );

    let available_size = ui.available_size();
    let spacing = ui.spacing().item_spacing;
    // Wrap the label in a container with dark background for consistency across all themes
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(40, 44, 52)) // Dark background color
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            // Make the frame take up all available width
            ui.set_min_width(available_size.x - 2.0 * spacing.x);
            ui.add(egui::Label::new(layout_job).selectable(true));
        });
}

/// Render empty state when no file is selected
pub fn render_empty(ui: &mut egui::Ui, colors: &AppColors) {
    ui.label(RichText::new("No file selected").color(colors.fg));
}

/// Load text content asynchronously
pub fn load_async(app: &mut Kiorg, path: PathBuf, file_size: u64) {
    load_preview_async(app, path, move |path| {
        // Check if file size is larger than 1MB (1,048,576 bytes)
        const MAX_PREVIEW_SIZE: u64 = 1_048_576;
        if file_size > MAX_PREVIEW_SIZE {
            return Ok(PreviewContent::text(format!(
                "Preview disabled for files larger than {}\n\nFile size: {}",
                format_size(MAX_PREVIEW_SIZE, BINARY),
                format_size(file_size, BINARY),
            )));
        }

        // Check if this is a source code file that should be syntax highlighted
        if let Some(syntax) = find_syntax_from_path(&path) {
            // For supported languages, load the full file for syntax highlighting
            load_full_text(&path, Some(syntax.name.as_str()))
        } else {
            // For other files, use the truncated preview
            try_load_utf8_str(path, file_size)
        }
    });
}

/// Try to load a file as UTF-8 text
pub fn try_load_utf8_str(path: PathBuf, file_size: u64) -> Result<PreviewContent, String> {
    // TODO: reuse the buffer between file reads
    let mut bytes: Vec<u8> = vec![0; std::cmp::min(1000, file_size as usize)];

    let file = match std::fs::File::open(&path) {
        Ok(file) => file,
        Err(e) => return Ok(PreviewContent::text(format!("Error opening file: {e}"))),
    };
    let bytes_read = match std::io::BufReader::new(file).read(&mut bytes) {
        Ok(bytes_read) => bytes_read,
        Err(e) => return Ok(PreviewContent::text(format!("Error reading file: {e}"))),
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

/// Detect file type and return a `PreviewContent` with generic file information
pub fn render_generic_file(path: PathBuf, size: u64) -> Result<PreviewContent, String> {
    // Try to detect the file type using file_type crate
    let file_type_info = match FileType::try_from_file(&path) {
        Ok(file_type) => {
            let media_types = file_type.media_types().join(", ");
            let extensions = file_type.extensions().join(", ");

            if !media_types.is_empty() {
                format!("File type: {media_types} ({extensions})")
            } else if !extensions.is_empty() {
                format!("File type: {extensions}")
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

/// Load full text content (not limited to first 1000 bytes like the regular preview)
pub fn load_full_text(
    path: &PathBuf,
    lang: Option<&'static str>,
) -> Result<PreviewContent, String> {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            // Use provided language or detect from extension
            let lang_name = lang.or_else(|| find_syntax_from_path(path).map(|s| s.name.as_str()));
            if let Some(language) = lang_name {
                Ok(PreviewContent::HighlightedCode { content, language })
            } else {
                Ok(PreviewContent::text(content))
            }
        }
        Err(_) => {
            // If UTF-8 reading fails, try to read as bytes and convert with lossy conversion
            // Don't use syntax highlighting for lossy converted content as it may be corrupted
            match std::fs::read(path) {
                Ok(bytes) => {
                    let content = String::from_utf8_lossy(&bytes).to_string();
                    Ok(PreviewContent::text(content))
                }
                Err(e) => Err(format!("Failed to read file: {e}")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_find_syntax_from_path_rust() {
        assert_eq!(
            find_syntax_from_path(Path::new("test.rs")).unwrap().name,
            "Rust"
        );
    }

    #[test]
    fn test_find_syntax_from_path_javascript() {
        assert_eq!(
            find_syntax_from_path(Path::new("test.js")).unwrap().name,
            "JavaScript (Babel)"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.mjs")).unwrap().name,
            "JavaScript (Babel)"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.cjs")).unwrap().name,
            "JavaScript (Babel)"
        );
    }

    #[test]
    fn test_find_syntax_from_path_typescript() {
        assert_eq!(
            find_syntax_from_path(Path::new("test.ts")).unwrap().name,
            "TypeScript"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.tsx")).unwrap().name,
            "TypeScriptReact"
        );
    }

    #[test]
    fn test_find_syntax_from_path_python() {
        assert_eq!(
            find_syntax_from_path(Path::new("test.py")).unwrap().name,
            "Python"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.pyw")).unwrap().name,
            "Python"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.pyi")).unwrap().name,
            "Python"
        );
    }

    #[test]
    fn test_find_syntax_from_path_c_cpp() {
        assert_eq!(
            find_syntax_from_path(Path::new("test.c")).unwrap().name,
            "C"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.h")).unwrap().name,
            "Objective-C"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.cpp")).unwrap().name,
            "C++"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.cc")).unwrap().name,
            "C++"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.cxx")).unwrap().name,
            "C++"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.hpp")).unwrap().name,
            "C++"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.hxx")).unwrap().name,
            "C++"
        );
    }

    #[test]
    fn test_find_syntax_from_path_shell_scripts() {
        assert_eq!(
            find_syntax_from_path(Path::new("test.sh")).unwrap().name,
            "Bourne Again Shell (bash)"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.bash")).unwrap().name,
            "Bourne Again Shell (bash)"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.zsh")).unwrap().name,
            "Bourne Again Shell (bash)"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.fish")).unwrap().name,
            "Fish"
        );
    }

    #[test]
    fn test_find_syntax_from_path_markup_languages() {
        assert_eq!(
            find_syntax_from_path(Path::new("test.html")).unwrap().name,
            "HTML"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.htm")).unwrap().name,
            "HTML"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.xml")).unwrap().name,
            "XML"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.css")).unwrap().name,
            "CSS"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.scss")).unwrap().name,
            "SCSS"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.sass")).unwrap().name,
            "Sass"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.md")).unwrap().name,
            "Markdown"
        );
    }

    #[test]
    fn test_find_syntax_from_path_config_files() {
        assert_eq!(
            find_syntax_from_path(Path::new("test.json")).unwrap().name,
            "JSON"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.yaml")).unwrap().name,
            "YAML"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.yml")).unwrap().name,
            "YAML"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.toml")).unwrap().name,
            "TOML"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.ini")).unwrap().name,
            "INI"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.cfg")).unwrap().name,
            "INI"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("test.conf")).unwrap().name,
            "nginx"
        );
    }

    #[test]
    fn test_find_syntax_from_path_special_filenames() {
        assert_eq!(
            find_syntax_from_path(Path::new("Makefile")).unwrap().name,
            "Makefile"
        );
        assert_eq!(
            find_syntax_from_path(Path::new("Dockerfile")).unwrap().name,
            "Dockerfile"
        );
        assert_eq!(
            find_syntax_from_path(Path::new(".gitignore")).unwrap().name,
            "Git Ignore"
        );
        assert_eq!(
            find_syntax_from_path(Path::new(".gitmodules"))
                .unwrap()
                .name,
            "Git Config"
        );
    }

    #[test]
    fn test_find_syntax_from_path_text_files() {
        assert_eq!(
            find_syntax_from_path(Path::new("test.txt")).unwrap().name,
            "Plain Text"
        );

        // .text files are not recognized by syntect
        assert!(find_syntax_from_path(Path::new("test.text")).is_none());

        // .log files are recognized with syntax name "log"
        assert_eq!(
            find_syntax_from_path(Path::new("test.log")).unwrap().name,
            "log"
        );
    }

    #[test]
    fn test_find_syntax_from_path_unknown() {
        assert!(find_syntax_from_path(Path::new("test.unknown")).is_none());
        assert!(find_syntax_from_path(Path::new("test.xyz")).is_none());
        assert!(find_syntax_from_path(Path::new("test")).is_none());
    }

    #[test]
    fn test_find_syntax_from_first_line() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Test Python shebang detection
        let mut python_file = NamedTempFile::new().unwrap();
        writeln!(python_file, "#!/usr/bin/env python3").unwrap();
        writeln!(python_file, "print('Hello, World!')").unwrap();
        python_file.flush().unwrap();

        let syntax = find_syntax_from_path(python_file.path());
        assert!(syntax.is_some());
        assert_eq!(syntax.unwrap().name, "Python");

        // Test shell script shebang detection
        let mut shell_file = NamedTempFile::new().unwrap();
        writeln!(shell_file, "#!/bin/bash").unwrap();
        writeln!(shell_file, "echo 'Hello, World!'").unwrap();
        shell_file.flush().unwrap();

        let syntax = find_syntax_from_path(shell_file.path());
        assert!(syntax.is_some());
        assert_eq!(syntax.unwrap().name, "Bourne Again Shell (bash)");

        // Test file that can't be opened (non-existent file)
        let non_existent_path = Path::new("/non/existent/file.unknown");
        assert!(find_syntax_from_path(non_existent_path).is_none());

        // Test empty file (no first line)
        let mut empty_file = NamedTempFile::new().unwrap();
        empty_file.flush().unwrap();

        let syntax = find_syntax_from_path(empty_file.path());
        assert!(syntax.is_none());

        // Test file with first line that doesn't match any syntax
        let mut unknown_file = NamedTempFile::new().unwrap();
        writeln!(unknown_file, "This is just some random text").unwrap();
        writeln!(unknown_file, "With no recognizable syntax").unwrap();
        unknown_file.flush().unwrap();

        let syntax = find_syntax_from_path(unknown_file.path());
        assert!(syntax.is_none());
    }
}
