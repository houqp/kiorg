use egui::{Context, TextureHandle};
use image::io::Reader as ImageReader;
use std::fs;
use std::io::{BufRead, BufReader, Cursor};
use std::path::{Path, PathBuf};

/// Handles file preview functionality
pub struct Preview;

impl Preview {
    /// Update the preview content based on the selected entry
    pub fn update_preview(
        ctx: &Context,
        selected_path: Option<&PathBuf>,
        preview_content: &mut String,
        current_image: &mut Option<TextureHandle>,
    ) {
        if let Some(path) = selected_path {
            if path.is_file() {
                if Self::is_text_file(path) {
                    // Preview text files
                    *current_image = None;
                    match fs::File::open(path) {
                        Ok(file) => {
                            let reader = BufReader::new(file);
                            let lines: Vec<String> = reader
                                .lines()
                                .take(500) // Limit to 500 lines for performance
                                .filter_map(Result::ok)
                                .collect();
                            *preview_content = lines.join("\n");
                        }
                        Err(_) => {
                            *preview_content = "Error reading file".to_string();
                        }
                    }
                } else if Self::is_image_file(path) {
                    // Preview image files
                    *preview_content = String::new();
                    match Self::load_image(ctx, path) {
                        Some(texture) => {
                            *current_image = Some(texture);
                        }
                        None => {
                            *preview_content = "Error loading image".to_string();
                        }
                    }
                } else {
                    *current_image = None;
                    *preview_content = "Binary file, preview not available".to_string();
                }
            } else {
                *current_image = None;
                *preview_content = String::new();
            }
        } else {
            *current_image = None;
            *preview_content = String::new();
        }
    }

    /// Check if a file is likely to be a text file based on extension
    fn is_text_file(path: &Path) -> bool {
        let text_extensions = [
            "txt", "md", "rs", "toml", "json", "yaml", "yml", "html", "css", 
            "js", "ts", "py", "sh", "bash", "c", "cpp", "h", "hpp", "go", 
            "java", "kt", "swift", "rb", "php", "pl", "lua", "xml", "csv",
            "config", "ini", "log", "gitignore", "dockerignore", "lock",
        ];
        
        path.extension()
            .and_then(|ext| ext.to_str())
            .map_or(false, |ext| {
                text_extensions.iter().any(|&text_ext| ext.eq_ignore_ascii_case(text_ext))
            })
    }

    /// Check if a file is an image based on extension
    fn is_image_file(path: &Path) -> bool {
        let image_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp"];
        
        path.extension()
            .and_then(|ext| ext.to_str())
            .map_or(false, |ext| {
                image_extensions.iter().any(|&img_ext| ext.eq_ignore_ascii_case(img_ext))
            })
    }

    /// Load an image from a file and convert it to a texture
    fn load_image(ctx: &Context, path: &Path) -> Option<TextureHandle> {
        // Try to read file to memory
        let file_data = fs::read(path).ok()?;
        
        // Parse image
        let img = ImageReader::new(Cursor::new(file_data))
            .with_guessed_format().ok()?
            .decode().ok()?
            .to_rgba8();
        
        // Get dimensions
        let width = img.width() as usize;
        let height = img.height() as usize;
        
        // Create texture
        Some(ctx.load_texture(
            path.file_name().unwrap_or_default().to_string_lossy(),
            egui::ColorImage::from_rgba_unmultiplied([width, height], &img),
            egui::TextureOptions::default(),
        ))
    }
}
