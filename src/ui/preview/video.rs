//! Video preview module

use crate::config::colors::AppColors;
use crate::models::preview_content::{VideoMeta, PreviewContent};
use egui::{Image, RichText};
use std::collections::HashMap;
use std::path::Path;
use video_rs::decode::Decoder;
use video_rs::Url;

const METADATA_KEY_COLUMN_WIDTH: f32 = 100.0;

/// Render video content
pub fn render(
    ui: &mut egui::Ui,
    video_meta: &VideoMeta,
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    // Display video title
    ui.label(
        RichText::new(&video_meta.title)
            .color(colors.fg)
            .strong()
            .size(20.0),
    );
    ui.add_space(10.0);

    // Display video thumbnail (centered)
    ui.vertical_centered(|ui| {
        ui.add(
            Image::new(video_meta.thumbnail.clone())
                .max_size(egui::vec2(available_width, available_height * 0.6))
                .maintain_aspect_ratio(true),
        );
    });
    ui.add_space(15.0);

    // Create a table for video metadata
    ui.label(
        RichText::new("Video Metadata")
            .color(colors.fg_folder)
            .strong()
            .size(14.0),
    );
    ui.add_space(5.0);

    egui::Grid::new("video_metadata_grid")
        .num_columns(2)
        .spacing([10.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            // Sort keys for consistent display
            let mut sorted_keys: Vec<&String> = video_meta.metadata.keys().collect();
            sorted_keys.sort();

            // Display each metadata field in a table row
            for key in sorted_keys {
                if let Some(value) = video_meta.metadata.get(key) {
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

/// Read video file, extract metadata and generate thumbnail, and create a `PreviewContent`
pub fn read_video_with_metadata(
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

    // Add file size
    if let Ok(file_metadata) = std::fs::metadata(path) {
        let size = file_metadata.len();
        metadata.insert(
            "File Size".to_string(),
            humansize::format_size(size, humansize::BINARY),
        );
    }

    // Get file extension for file type information
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_uppercase();
        metadata.insert("File Type".to_string(), ext_str.to_string());
    }

    // Try to extract a real thumbnail from the video
    let thumbnail_texture = match extract_video_thumbnail(ctx, path, &mut metadata) {
        Ok(texture) => texture,
        Err(_e) => {
            // Fall back to placeholder thumbnail
            generate_placeholder_thumbnail(ctx, path)
                .map_err(|e| format!("Failed to generate thumbnail: {e}"))?
        }
    };

    Ok(PreviewContent::video(title, metadata, thumbnail_texture))
}

/// Extract a thumbnail from the video file using video-rs
fn extract_video_thumbnail(
    ctx: &egui::Context,
    path: &Path,
    metadata: &mut HashMap<String, String>,
) -> Result<egui::TextureHandle, String> {    
    // Initialize video-rs (a wrapper for ffmpeg-next)
    video_rs::init().map_err(|e| format!("Failed to initialize video-rs: {e}"))?;

    // Convert path to URL format
    let absolute_path = path
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize path: {e}"))?;
    let source = format!("file://{}", absolute_path.display())
        .parse::<Url>()
        .map_err(|e| format!("Failed to parse URL: {e}"))?;

    // Create decoder
    let mut decoder = Decoder::new(source)
        .map_err(|e| format!("Failed to create decoder: {e}"))?;

    // Get video dimensions and add to metadata
    let (width, height) = decoder.size();
    metadata.insert("Dimensions".to_string(), format!("{}x{}", width, height));

    // Try to get duration information
    if let Ok(duration) = decoder.duration() {
        let total_seconds = duration.as_secs() as u64;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        
        if hours > 0 {
            metadata.insert("Duration".to_string(), format!("{}:{:02}:{:02}", hours, minutes, seconds));
        } else {
            metadata.insert("Duration".to_string(), format!("{}:{:02}", minutes, seconds));
        }
    }

    // Decode the first frame
    let (_, frame) = decoder
        .decode()
        .map_err(|e| format!("Failed to decode frame: {e}"))?;

    // Convert frame to raw RGB data
    let (raw, _) = frame.into_raw_vec_and_offset();

    // Convert RGB to RGBA from raw data
    let rgba_data: Vec<u8> = raw
        .chunks_exact(3)
        .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255u8])
        .collect();

    // Create egui::ColorImage from the RGBA data
    let color_image = egui::ColorImage::from_rgba_unmultiplied([width as _, height as _], &rgba_data);

    // Create the texture with path-based ID for uniqueness
    let texture_id = format!("video_thumbnail_{}", path.display());
    let texture = ctx.load_texture(texture_id, color_image, egui::TextureOptions::default());

    Ok(texture)
}

/// Generate a placeholder thumbnail for video files if extraction fails
fn generate_placeholder_thumbnail(
    ctx: &egui::Context,
    path: &Path,
) -> Result<egui::TextureHandle, String> {
    let width = 320;
    let height = 240;
    
    let mut rgb_data = Vec::with_capacity(width * height * 3);
    
    // Create a dark background with a play button symbol
    for y in 0..height {
        for x in 0..width {
            // Create a dark gray background
            let mut r = 40u8;
            let mut g = 40u8;
            let mut b = 40u8;
            
            // Add a border
            if x < 2 || x >= width - 2 || y < 2 || y >= height - 2 {
                r = 80;
                g = 80;
                b = 80;
            }
            
            // Add a triangular play button in the center
            let center_x = width / 2;
            let center_y = height / 2;
            let rel_x = x as i32 - center_x as i32;
            let rel_y = y as i32 - center_y as i32;
            
            if rel_x >= -15 && rel_x <= 15 && rel_y.abs() <= 15 {
                let max_y = if rel_x <= 0 {
                    15
                } else {
                    15 - (rel_x * 15) / 15
                };
                
                if rel_y.abs() <= max_y {
                    r = 220;
                    g = 220;
                    b = 220;
                }
            }
            
            rgb_data.push(r);
            rgb_data.push(g);
            rgb_data.push(b);
        }
    }

    // Create the texture
    let color_image = egui::ColorImage::from_rgb([width, height], &rgb_data);
    let texture_id = format!("video_placeholder_{}", path.display());
    let texture = ctx.load_texture(texture_id, color_image, egui::TextureOptions::default());

    Ok(texture)
}