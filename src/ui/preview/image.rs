//! Image preview module

use crate::config::colors::AppColors;
use crate::models::preview_content::{ImageMeta, PreviewContent};
use egui::{Image, RichText};
use image::{GenericImageView, ImageDecoder, ImageFormat};
use std::collections::HashMap;
use std::path::Path;

const METADATA_KEY_COLUMN_WIDTH: f32 = 100.0;

/// Render image content
pub fn render(
    ui: &mut egui::Ui,
    image_meta: &ImageMeta,
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    // Display image title
    ui.label(
        RichText::new(&image_meta.title)
            .color(colors.fg)
            .strong()
            .size(20.0),
    );
    ui.add_space(10.0);

    // Display image (centered)
    ui.vertical_centered(|ui| {
        ui.add(
            Image::new(&image_meta.texture)
                .max_size(egui::vec2(available_width, available_height * 0.6))
                .maintain_aspect_ratio(true),
        );
    });
    ui.add_space(15.0);

    // Create a table for regular metadata
    ui.label(
        RichText::new("Image Metadata")
            .color(colors.fg_folder)
            .strong()
            .size(14.0),
    );
    ui.add_space(5.0);
    egui::Grid::new("image_metadata_grid")
        .num_columns(2)
        .spacing([10.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            // Sort keys for consistent display
            let mut sorted_keys: Vec<&String> = image_meta.metadata.keys().collect();
            sorted_keys.sort();

            // Display each metadata field in a table row
            for key in sorted_keys {
                if let Some(value) = image_meta.metadata.get(key) {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                        ui.set_min_width(METADATA_KEY_COLUMN_WIDTH);
                        ui.set_max_width(METADATA_KEY_COLUMN_WIDTH);
                        ui.add(egui::Label::new(RichText::new(key).color(colors.fg)).wrap());
                    });
                    ui.add(egui::Label::new(RichText::new(value).color(colors.fg)).wrap());
                    ui.end_row();
                }
            }
        });

    // Display EXIF data in a separate table if available
    if let Some(exif_data) = &image_meta.exif_data {
        ui.add_space(15.0);
        ui.label(
            RichText::new("EXIF Data")
                .color(colors.fg_folder)
                .strong()
                .size(14.0),
        );
        ui.add_space(5.0);
        egui::Grid::new("exif_data_grid")
            .num_columns(2)
            .spacing([10.0, 6.0])
            .striped(true)
            .show(ui, |ui| {
                // Sort keys for consistent display
                let mut sorted_keys: Vec<&String> = exif_data.keys().collect();
                sorted_keys.sort();

                // Display each EXIF field in a table row
                for key in sorted_keys {
                    if let Some(value) = exif_data.get(key) {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            ui.set_min_width(METADATA_KEY_COLUMN_WIDTH);
                            ui.set_max_width(METADATA_KEY_COLUMN_WIDTH);
                            ui.add(egui::Label::new(RichText::new(key).color(colors.fg)).wrap());
                        });
                        ui.add(egui::Label::new(RichText::new(value).color(colors.fg)).wrap());
                        ui.end_row();
                    }
                }
            });
    }
}

/// Read image file, extract metadata, and create a `PreviewContent`
///
/// This function:
/// 1. Reads the image file and extracts metadata
/// 2. Creates a texture from the image data
/// 3. Returns a `PreviewContent::Image` with the texture
///
/// # Arguments
/// * `path` - The path to the image file
/// * `ctx` - The egui context for creating textures
pub fn read_image_with_metadata(
    path: &Path,
    ctx: &egui::Context,
) -> Result<PreviewContent, String> {
    // Get the filename for the title
    let title = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Create a HashMap to store metadata
    let mut metadata = HashMap::new();

    // Open the image file
    let mut decoder = image::ImageReader::open(path)
        .map_err(|e| format!("failed to open image: {e}"))?
        .into_decoder()
        .map_err(|e| format!("failed to create decoder for image: {e}"))?;
    let exif_bytes = decoder
        .exif_metadata()
        .map_err(|e| format!("failed to extract exif metadata: {e}"))?;
    let orientation = decoder
        .orientation()
        .map_err(|e| format!("failed to get image orientation: {e}"))?;
    let mut img = match image::DynamicImage::from_decoder(decoder) {
        Ok(img) => img,
        Err(e) => return Err(format!("Failed to decode image: {e}")),
    };

    img.apply_orientation(orientation);

    // Create a separate HashMap for EXIF data
    let mut exif_data = None;

    if let Some(v) = exif_bytes {
        let (fields, _) =
            exif::parse_exif(v.as_ref()).map_err(|e| format!("failed to parse EXIF data: {e}"))?;

        // Only create the HashMap if we have EXIF data
        if !fields.is_empty() {
            let mut exif_map = HashMap::new();
            for field in fields {
                exif_map.insert(
                    format!("{}", field.tag),
                    format!("{}", field.display_value()),
                );
            }
            exif_data = Some(exif_map);
        }
    }

    // Extract basic image information
    let dimensions = img.dimensions();
    metadata.insert(
        "Dimensions".to_string(),
        format!("{}x{} pixels", dimensions.0, dimensions.1),
    );

    // Get color type
    metadata.insert("Color Type".to_string(), format!("{:?}", img.color()));

    // Add color depth information
    match img.color() {
        image::ColorType::Rgb8 | image::ColorType::Rgba8 => {
            metadata.insert("Bit Depth".to_string(), "8 bits per channel".to_string());
        }
        image::ColorType::Rgb16 | image::ColorType::Rgba16 => {
            metadata.insert("Bit Depth".to_string(), "16 bits per channel".to_string());
        }
        image::ColorType::L8 | image::ColorType::La8 => {
            metadata.insert("Bit Depth".to_string(), "8 bits (grayscale)".to_string());
        }
        image::ColorType::L16 | image::ColorType::La16 => {
            metadata.insert("Bit Depth".to_string(), "16 bits (grayscale)".to_string());
        }
        _ => {
            // Other color types
        }
    }

    // Try to get format-specific information
    if let Ok(format) = image::ImageFormat::from_path(path) {
        // Format the image format in a more readable way
        let format_name = match format {
            ImageFormat::Jpeg => "JPEG".to_string(),
            ImageFormat::Png => "PNG".to_string(),
            ImageFormat::Gif => "GIF".to_string(),
            ImageFormat::WebP => "WebP".to_string(),
            ImageFormat::Tiff => "TIFF".to_string(),
            ImageFormat::Bmp => "BMP".to_string(),
            ImageFormat::Ico => "ICO".to_string(),
            ImageFormat::Tga => "TGA".to_string(),
            ImageFormat::Dds => "DDS".to_string(),
            ImageFormat::Farbfeld => "Farbfeld".to_string(),
            ImageFormat::Avif => "AVIF".to_string(),
            ImageFormat::Qoi => "QOI".to_string(),
            ImageFormat::Pcx => "PCX".to_string(),
            _ => format!("{format:?}"),
        };
        metadata.insert("Format".to_string(), format_name);

        // Add format-specific metadata
        if format == ImageFormat::Gif {
            // For GIF, we can check if it's animated by examining the file
            // A simple heuristic: try to read the file and check if it has multiple frames
            if let Ok(file_content) = std::fs::read(path) {
                // Look for multiple image descriptors in the GIF file
                // This is a very simplified approach - not 100% reliable
                // The byte sequence 0x2C (image descriptor marker) appears for each frame
                let image_descriptor_count = file_content
                    .windows(2)
                    .filter(|window| window[0] == 0x2C)
                    .count();

                if image_descriptor_count > 1 {
                    metadata.insert("Animation".to_string(), "Animated GIF".to_string());
                }
            }
        }
    }

    // Add file size
    if let Ok(metadata_os) = std::fs::metadata(path) {
        let size = metadata_os.len();
        metadata.insert(
            "File Size".to_string(),
            humansize::format_size(size, humansize::BINARY),
        );
    }

    // Convert the image to RGBA8 format for egui
    let rgba8_img = img.to_rgba8();
    let dimensions = rgba8_img.dimensions();

    // Create egui::ColorImage from the image data
    let size = [dimensions.0 as _, dimensions.1 as _];
    let pixels = rgba8_img.as_flat_samples();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

    // Create a unique texture ID based on the path
    let texture_id = format!("image_{}", path.display());

    // Create a texture from the color image
    let texture = ctx.load_texture(texture_id, color_image, egui::TextureOptions::default());

    // Create the image preview content with the texture and exif data
    Ok(PreviewContent::image(title, metadata, texture, exif_data))
}
