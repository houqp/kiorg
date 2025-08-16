//! Text preview module

use crate::app::Kiorg;
use crate::config::colors::AppColors;
use crate::models::preview_content::PreviewContent;
use crate::ui::preview::loading::load_preview_async;
use egui::RichText;
use egui_extras::syntax_highlighting::CodeTheme;
use file_type::FileType;
use std::io::Read;
use std::path::PathBuf;

/// Get language type from file extension or filename
pub fn lang_type_from_ext(ext: &str) -> Option<&'static str> {
    match ext {
        "rs" => Some("rs"),
        "js" | "mjs" | "cjs" => Some("js"),
        "ts" | "tsx" => Some("ts"),
        "py" | "pyw" | "pyi" => Some("py"),
        "java" => Some("java"),
        "c" | "h" => Some("c"),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some("cpp"),
        "cs" => Some("cs"),
        "go" => Some("go"),
        "rb" => Some("ruby"),
        "php" => Some("php"),
        "swift" => Some("swift"),
        "kt" | "kts" => Some("kotlin"),
        "scala" => Some("scala"),
        "clj" | "cljs" | "cljc" => Some("clojure"),
        "hs" => Some("haskell"),
        "elm" => Some("elm"),
        "dart" => Some("dart"),
        "lua" => Some("lua"),
        "r" => Some("r"),
        "m" => Some("objc"),
        "sh" | "bash" | "zsh" | "fish" => Some("bash"),
        "txt" | "text" | "log" => Some("txt"),
        "ps1" => Some("powershell"),
        "sql" => Some("sql"),
        "html" | "htm" => Some("html"),
        "css" => Some("css"),
        "scss" | "sass" => Some("scss"),
        "less" => Some("less"),
        "xml" => Some("xml"),
        "json" => Some("json"),
        "yaml" | "yml" => Some("yaml"),
        "toml" => Some("toml"),
        "ini" | "cfg" | "conf" => Some("ini"),
        "dockerfile" => Some("dockerfile"),
        "makefile" => Some("makefile"),
        "cmake" => Some("cmake"),
        "tex" | "latex" => Some("latex"),
        "md" => Some("md"),
        "vim" => Some("vim"),
        ".gitignore" => Some("gitignore"),
        ".gitmodules" => Some("gitmodules"),
        _ => None,
    }
}

/// Render text content
pub fn render(ui: &mut egui::Ui, text: &str, colors: &AppColors) {
    ui.label(RichText::new(text).color(colors.fg));
}

/// Render syntax highlighted code content
pub fn render_highlighted(ui: &mut egui::Ui, text: &str, language: &'static str) {
    let theme = CodeTheme::from_memory(ui.ctx(), ui.style());
    egui_extras::syntax_highlighting::code_view_ui(ui, &theme, text, language);
}

/// Render empty state when no file is selected
pub fn render_empty(ui: &mut egui::Ui, colors: &AppColors) {
    ui.label(RichText::new("No file selected").color(colors.fg));
}

/// Load text content asynchronously
pub fn load_async(app: &mut Kiorg, path: PathBuf, file_size: u64) {
    load_preview_async(app, path, move |path| {
        // Check if this is a source code file that should be syntax highlighted
        let ext = super::path_to_ext_info(&path);
        if let Some(lang) = lang_type_from_ext(&ext) {
            // For supported languages, load the full file for syntax highlighting
            load_full_text(&path, Some(lang))
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

    let mut file = match std::fs::File::open(&path) {
        Ok(file) => file,
        Err(e) => return Ok(PreviewContent::text(format!("Error opening file: {e}"))),
    };
    let bytes_read = match file.read(&mut bytes) {
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
            let language = lang.or_else(|| {
                let ext = super::path_to_ext_info(path);
                lang_type_from_ext(&ext)
            });

            if let Some(lang) = language {
                Ok(PreviewContent::HighlightedCode {
                    content,
                    language: lang,
                })
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

    #[test]
    fn test_lang_type_from_ext_rust() {
        assert_eq!(lang_type_from_ext("rs"), Some("rs"));
    }

    #[test]
    fn test_lang_type_from_ext_javascript() {
        assert_eq!(lang_type_from_ext("js"), Some("js"));
        assert_eq!(lang_type_from_ext("mjs"), Some("js"));
        assert_eq!(lang_type_from_ext("cjs"), Some("js"));
    }

    #[test]
    fn test_lang_type_from_ext_typescript() {
        assert_eq!(lang_type_from_ext("ts"), Some("ts"));
        assert_eq!(lang_type_from_ext("tsx"), Some("ts"));
    }

    #[test]
    fn test_lang_type_from_ext_python() {
        assert_eq!(lang_type_from_ext("py"), Some("py"));
        assert_eq!(lang_type_from_ext("pyw"), Some("py"));
        assert_eq!(lang_type_from_ext("pyi"), Some("py"));
    }

    #[test]
    fn test_lang_type_from_ext_c_cpp() {
        assert_eq!(lang_type_from_ext("c"), Some("c"));
        assert_eq!(lang_type_from_ext("h"), Some("c"));
        assert_eq!(lang_type_from_ext("cpp"), Some("cpp"));
        assert_eq!(lang_type_from_ext("cc"), Some("cpp"));
        assert_eq!(lang_type_from_ext("cxx"), Some("cpp"));
        assert_eq!(lang_type_from_ext("hpp"), Some("cpp"));
        assert_eq!(lang_type_from_ext("hxx"), Some("cpp"));
    }

    #[test]
    fn test_lang_type_from_ext_shell_scripts() {
        assert_eq!(lang_type_from_ext("sh"), Some("bash"));
        assert_eq!(lang_type_from_ext("bash"), Some("bash"));
        assert_eq!(lang_type_from_ext("zsh"), Some("bash"));
        assert_eq!(lang_type_from_ext("fish"), Some("bash"));
    }

    #[test]
    fn test_lang_type_from_ext_markup_languages() {
        assert_eq!(lang_type_from_ext("html"), Some("html"));
        assert_eq!(lang_type_from_ext("htm"), Some("html"));
        assert_eq!(lang_type_from_ext("xml"), Some("xml"));
        assert_eq!(lang_type_from_ext("css"), Some("css"));
        assert_eq!(lang_type_from_ext("scss"), Some("scss"));
        assert_eq!(lang_type_from_ext("sass"), Some("scss"));
        assert_eq!(lang_type_from_ext("md"), Some("md"));
    }

    #[test]
    fn test_lang_type_from_ext_config_files() {
        assert_eq!(lang_type_from_ext("json"), Some("json"));
        assert_eq!(lang_type_from_ext("yaml"), Some("yaml"));
        assert_eq!(lang_type_from_ext("yml"), Some("yaml"));
        assert_eq!(lang_type_from_ext("toml"), Some("toml"));
        assert_eq!(lang_type_from_ext("ini"), Some("ini"));
        assert_eq!(lang_type_from_ext("cfg"), Some("ini"));
        assert_eq!(lang_type_from_ext("conf"), Some("ini"));
    }

    #[test]
    fn test_lang_type_from_ext_special_filenames() {
        // Test full filenames without extensions
        assert_eq!(lang_type_from_ext("makefile"), Some("makefile"));
        assert_eq!(lang_type_from_ext("dockerfile"), Some("dockerfile"));
        assert_eq!(lang_type_from_ext(".gitignore"), Some("gitignore"));
        assert_eq!(lang_type_from_ext(".gitmodules"), Some("gitmodules"));
    }

    #[test]
    fn test_lang_type_from_ext_text_files() {
        assert_eq!(lang_type_from_ext("txt"), Some("txt"));
        assert_eq!(lang_type_from_ext("text"), Some("txt"));
        assert_eq!(lang_type_from_ext("log"), Some("txt"));
    }

    #[test]
    fn test_lang_type_from_ext_unknown() {
        assert_eq!(lang_type_from_ext("unknown"), None);
        assert_eq!(lang_type_from_ext("xyz"), None);
        assert_eq!(lang_type_from_ext(""), None);
    }

    #[test]
    fn test_lang_type_from_ext_case_sensitivity() {
        // The function should work with lowercase extensions
        assert_eq!(lang_type_from_ext("rs"), Some("rs"));
        // Test that uppercase doesn't match (function expects lowercase)
        assert_eq!(lang_type_from_ext("RS"), None);
        assert_eq!(lang_type_from_ext("Makefile"), None);
    }
}
