//! Video preview module

use crate::config::colors::AppColors;
use crate::models::preview_content::{PreviewContent, VideoMeta};
use egui::{Image, RichText};
use ffmpeg_next::{
    codec::context::Context as CodecContext,
    format, init,
    media::Type,
    software::scaling::{context::Context as ScalerContext, flag::Flags},
    util::{
        format::pixel::Pixel,
        frame::video::Video,
        mathematics::{Rescale, rescale},
    },
};
use std::collections::HashMap;
use std::path::Path;

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

/// Extract a thumbnail from the video file using ffmpeg-next with quality scoring
fn extract_video_thumbnail(
    ctx: &egui::Context,
    path: &Path,
    metadata: &mut HashMap<String, String>,
) -> Result<egui::TextureHandle, String> {
    // Initialize ffmpeg
    init().map_err(|e| format!("Failed to initialize ffmpeg: {e}"))?;

    let path_str = path.to_str().ok_or("Invalid path encoding")?;
    let mut ictx = format::input(path_str).map_err(|e| format!("Failed to open input: {e}"))?;
    let stream = ictx
        .streams()
        .best(Type::Video)
        .ok_or("No video stream found")?;
    let video_stream_index = stream.index();
    let video_params = stream.parameters();

    let mut decoder = CodecContext::from_parameters(video_params)
        .map_err(|e| format!("Failed to create decoder context: {e}"))?
        .decoder()
        .video()
        .map_err(|e| format!("Failed to create video decoder: {e}"))?;

    // Get video dimensions and add to metadata
    let width = decoder.width();
    let height = decoder.height();

    // Check for the pixel aspect ratio
    let par = decoder.aspect_ratio();
    let has_par = par.0 != 0 && par.1 != 0 && !(par.0 == 1 && par.1 == 1);

    let (output_width, output_height) = if has_par {
        // Calculate display dimensions if pixel aspect ratio is present
        let display_width = (decoder.width() as f64 * par.0 as f64 / par.1 as f64) as u32;
        (display_width, decoder.height())
    } else {
        (decoder.width(), decoder.height())
    };

    metadata.insert("Dimensions".to_string(), format!("{width}x{height}"));
    if has_par {
        metadata.insert(
            "Display Dimensions".to_string(),
            format!("{output_width}x{output_height}"),
        );
        metadata.insert(
            "Pixel Aspect Ratio".to_string(),
            format!("{}:{}", par.0, par.1),
        );
    }

    // Get duration of video and its time base
    let duration = stream.duration();
    let time_base = stream.time_base();

    // Calculate the duration of the video in seconds
    let duration_seconds = duration as f64 * time_base.0 as f64 / time_base.1 as f64;

    // Add duration to metadata
    let total_seconds = duration_seconds as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        metadata.insert(
            "Duration".to_string(),
            format!("{hours}:{minutes:02}:{seconds:02}"),
        );
    } else {
        metadata.insert("Duration".to_string(), format!("{minutes}:{seconds:02}"));
    }

    // Create a scaler to convert to RGB24 format and handle pixel aspect ratio
    let mut scaler = ScalerContext::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        output_width,
        output_height,
        Flags::BILINEAR,
    )
    .map_err(|e| format!("Failed to create scaler: {e}"))?;

    // Sample from 0%, 25%, 50%, 75% of the video
    let seek_positions = [0.0, 0.25, 0.5, 0.75];
    let mut frames = Vec::new();
    let mut frame_scores = Vec::new();

    for &seek_ratio in &seek_positions {
        // Convert seek position to seconds, then rescale to FFmpeg's base timebase
        let target_seconds = (duration_seconds * seek_ratio) as i64;
        let target_timestamp = target_seconds.rescale((1, 1), rescale::TIME_BASE);

        // Seek to the target timestamp
        if ictx.seek(target_timestamp, ..target_timestamp).is_err() {
            continue;
        }

        // Flush the decoder after seeking
        decoder.flush();

        // Read a few packets after the seek to get a frame
        let mut packets_processed = 0;
        let max_packets = 20;

        for (stream, packet) in ictx.packets() {
            if stream.index() != video_stream_index {
                continue;
            }

            packets_processed += 1;
            if packets_processed > max_packets {
                break;
            }

            if decoder.send_packet(&packet).is_err() {
                continue;
            }

            let mut frame = Video::empty();
            if decoder.receive_frame(&mut frame).is_ok() {
                // Convert the frame to RGB24 format
                let mut rgb_frame = Video::empty();
                if scaler.run(&frame, &mut rgb_frame).is_err() {
                    continue;
                }

                // Grab raw pixel data and properties
                let data = rgb_frame.data(0);
                let linesize = rgb_frame.stride(0);
                let frame_height = output_height as usize;
                let frame_width = output_width as usize;

                // Extract RGB pixels
                let mut rgb_pixels = Vec::with_capacity(frame_width * frame_height * 3);

                for y in 0..frame_height {
                    for x in 0..frame_width {
                        let src_offset = y * linesize + x * 3;
                        if src_offset + 2 < data.len() {
                            rgb_pixels.push(data[src_offset]);
                            rgb_pixels.push(data[src_offset + 1]);
                            rgb_pixels.push(data[src_offset + 2]);
                        }
                    }
                }

                frames.push((frame_width, frame_height, rgb_pixels.clone()));

                // Calculate quality score for this frame using RGB data
                let rgb_tuples: Vec<(u8, u8, u8)> = rgb_pixels
                    .chunks_exact(3)
                    .map(|chunk| (chunk[0], chunk[1], chunk[2]))
                    .collect();

                let quality_score =
                    calculate_frame_quality(&rgb_tuples, frame_width as u32, frame_height as u32);

                frame_scores.push(quality_score);

                break;
            }
        }
    }

    if frames.is_empty() {
        return Err("No frames could be extracted".to_string());
    }

    // Find the best frame based on quality scores
    let best_frame_index = frame_scores
        .iter()
        .enumerate()
        .max_by(|(_, score_a), (_, score_b)| score_a.partial_cmp(score_b).unwrap())
        .map(|(index, _)| index)
        .unwrap_or(0);

    let (frame_width, frame_height, rgb_data) = &frames[best_frame_index];

    // Create egui image from RGB data
    let color_image = egui::ColorImage::from_rgb([*frame_width, *frame_height], rgb_data);

    // Create the texture with path-based ID for caching
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

            if (-15..=15).contains(&rel_x) && rel_y.abs() <= 15 {
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

/// Calculate frame quality score based on brightness, variance, and sharpness
///
/// This function performs three calculations:
/// 1. Brightness score based on average luminance
/// 2. Variance score based on brightness variance
/// 3. Sharpness score based on Laplacian variance
///
/// It then returns a weighted score between 0.0 and 1.0
///
fn calculate_frame_quality(rgb_data: &[(u8, u8, u8)], width: u32, height: u32) -> f64 {
    let pixel_count = width * height;

    let mut total_brightness = 0.0;
    let mut brightness_values = Vec::new();

    for &(r, g, b) in rgb_data {
        let r = r as f64;
        let g = g as f64;
        let b = b as f64;

        // Calculate luminance using Photometric/digital ITU BT.709
        let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        total_brightness += luminance;
        brightness_values.push(luminance);
    }

    let average_brightness = total_brightness / pixel_count as f64;

    let brightness_score = if average_brightness < 22.0 {
        // Unsuitable too dark: MGV < 22
        0.0
    } else if average_brightness < 56.0 {
        // Underexposed: MGV 22-55
        (average_brightness - 22.0) / (56.0 - 22.0)
    } else if average_brightness <= 171.0 {
        // Suitable brightness: MGV 56-171
        1.0
    } else if average_brightness <= 194.0 {
        // Overexposed: MGV 172-194
        1.0 - (average_brightness - 171.0) / (194.0 - 171.0)
    } else {
        // Unsuitable too bright: MGV > 194
        0.0
    };

    // Calculate variance in brightness
    let variance = brightness_values
        .iter()
        .map(|&v| (v - average_brightness).powi(2))
        .sum::<f64>()
        / pixel_count as f64;

    let variance_score = (variance / 5000.0).min(1.0);

    // Calculate sharpness using Laplacian variance method
    let mut laplacian_sum = 0.0;
    let mut laplacian_count = 0;

    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let center_idx = (y * width + x) as usize;
            if center_idx < brightness_values.len() {
                let center = brightness_values[center_idx];
                let top = brightness_values[((y - 1) * width + x) as usize];
                let bottom = brightness_values[((y + 1) * width + x) as usize];
                let left = brightness_values[(y * width + x - 1) as usize];
                let right = brightness_values[(y * width + x + 1) as usize];

                let laplacian = -top - bottom - left - right + 4.0 * center;
                laplacian_sum += laplacian * laplacian;
                laplacian_count += 1;
            }
        }
    }

    let laplacian_variance = if laplacian_count > 0 {
        laplacian_sum / laplacian_count as f64
    } else {
        0.0
    };

    // Normalize sharpness score
    let sharpness_score = (laplacian_variance / 2000.0).min(1.0);

    // Combined score (weighted average)
    brightness_score * 0.33 + variance_score * 0.33 + sharpness_score * 0.33
}
