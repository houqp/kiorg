//! Video preview module

use egui::RichText;
use ffmpeg_sidecar::event::{FfmpegEvent, OutputVideoFrame, StreamTypeSpecificData};
use image::EncodableLayout;
use rayon::prelude::*;

use crate::config::colors::AppColors;
use crate::models::dir_entry::DirEntryMeta;
use crate::models::preview_content::{
    CachedPreviewContent, CachedVideoMeta, FfmpegMeta, InputMeta, StreamMeta, StreamTypeMeta,
    VideoMeta, metadata,
};
use crate::utils::preview_cache;
use tracing::{debug, warn};

const MIN_SUFFICIENT_QUALITY_SCORE: f64 = 0.5;
const THUMBNAIL_SAMPLE_RATIOS: [f64; 3] = [0.25, 0.5, 0.75];

#[cfg(not(any(test, feature = "testing")))]
mod ffmpeg {
    use ffmpeg_sidecar::command::FfmpegCommand;
    use ffmpeg_sidecar::event::FfmpegEvent;
    use std::time::Instant;
    use tracing::debug;

    pub(super) fn get_metadata_probe_iter(
        path_str: &str,
    ) -> Result<impl Iterator<Item = FfmpegEvent>, String> {
        let start = Instant::now();
        let mut cmd = FfmpegCommand::new()
            .input(path_str)
            .args(["-f", "null", "-vframes", "0", "-"]) // Fast metadata probe
            .spawn()
            .map_err(|e| format!("Failed to spawn ffmpeg probe: {e}"))?;

        debug!("ffmpeg probe spawn took {:?}", start.elapsed());

        cmd.iter()
            .map_err(|e| format!("Failed to extract video metadata: {e}"))
    }

    pub(super) fn get_frame_extraction_iter(
        path_str: &str,
        seek_time: f64,
        available_width: Option<f32>,
        mapping: Option<&str>,
    ) -> Option<impl Iterator<Item = ffmpeg_sidecar::event::OutputVideoFrame>> {
        let start = Instant::now();
        let mut cmd = FfmpegCommand::new();

        // Fast seek: -ss before -i
        cmd.args(["-ss", &seek_time.to_string()]);

        cmd.input(path_str);

        if let Some(m) = mapping {
            // Map specific stream (e.g. attached pic) if requested
            cmd.args(["-map", m]);
        }

        if let Some(w) = available_width {
            cmd.args([
                "-vf",
                // flags=fast_bilinear: Faster scaling algorithm for thumbnails
                &format!("scale={}:-1:flags=fast_bilinear", w as u32),
            ]);
        }

        let mut cmd = cmd
            .args([
                "-an", // Disable audio processing
                "-sn", // Disable subtitle processing
                "-dn", // Disable data stream processing
                "-vframes", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-",
            ])
            .spawn()
            .ok()?;

        debug!(
            "ffmpeg frame extraction spawn took {:?} for {} (map: {:?})",
            start.elapsed(),
            seek_time,
            mapping
        );

        let iter = cmd.iter().ok()?;
        Some(iter.filter_frames())
    }

    pub(super) fn init() -> Result<(), String> {
        let start = Instant::now();
        static INIT: std::sync::OnceLock<Result<(), String>> = std::sync::OnceLock::new();
        let res = INIT
            .get_or_init(|| {
                ffmpeg_sidecar::download::auto_download()
                    .map_err(|e| format!("Failed to auto-download ffmpeg: {e}"))
            })
            .clone();
        debug!("ffmpeg init took {:?}", start.elapsed());
        res
    }
}

#[cfg(any(test, feature = "testing"))]
mod ffmpeg {
    use ffmpeg_sidecar::event::OutputVideoFrame;
    use ffmpeg_sidecar::event::{
        AudioStream, FfmpegEvent, Stream, StreamTypeSpecificData, VideoStream,
    };

    pub(super) fn get_metadata_probe_iter(
        _path_str: &str,
    ) -> Result<impl Iterator<Item = FfmpegEvent>, String> {
        let events = vec![
            FfmpegEvent::ParsedInputStream(Stream {
                stream_index: 0,
                format: "h264".to_string(),
                language: "eng".to_string(),
                parent_index: 0,
                raw_log_message: String::new(),
                type_specific_data: StreamTypeSpecificData::Video(VideoStream {
                    width: 1920,
                    height: 1080,
                    pix_fmt: "yuv420p".to_string(),
                    fps: 30.0,
                }),
            }),
            FfmpegEvent::ParsedInputStream(Stream {
                stream_index: 1,
                format: "aac".to_string(),
                language: "eng".to_string(),
                parent_index: 0,
                raw_log_message: String::new(),
                type_specific_data: StreamTypeSpecificData::Audio(AudioStream {
                    sample_rate: 44100,
                    channels: "2".to_string(),
                }),
            }),
        ];
        Ok(events.into_iter())
    }

    pub(super) fn get_frame_extraction_iter(
        _path_str: &str,
        _seek_time: f64,
        _available_width: Option<f32>,
        _mapping: Option<&str>,
    ) -> Option<impl Iterator<Item = OutputVideoFrame>> {
        // Create a fake frame with some variation to pass quality check
        let width = 100;
        let height = 100;
        let mut data = Vec::with_capacity((width * height * 3) as usize);
        for i in 0..(width * height) {
            let ft = i as f64 / (width * height) as f64;
            // Generate a gradient
            data.push((ft * 255.0) as u8);
            data.push(((1.0 - ft) * 255.0) as u8);
            data.push(128);
        }

        let frame = OutputVideoFrame {
            width,
            height,
            data,
            output_index: 0,
            pix_fmt: "rgb24".to_string(),
            frame_num: 0,
            timestamp: 0.0,
        };

        Some(vec![frame].into_iter())
    }

    pub(super) fn init() -> Result<(), String> {
        Ok(())
    }
}

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
            video_meta
                .thumbnail_image
                .clone()
                .max_size(egui::vec2(available_width, available_height * 0.6))
                .maintain_aspect_ratio(true),
        );
    });
    ui.add_space(15.0);

    // Create a table for general video metadata
    ui.label(
        RichText::new("File Metadata")
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
            for (key, value) in &video_meta.ffmpeg.key_metadata {
                render_metadata_row(ui, key, value, colors);
            }
            for (key, value) in &video_meta.ffmpeg.misc_metadata {
                render_metadata_row(ui, key, value, colors);
            }
        });

    // Render inputs and their streams
    for (input_idx, input) in video_meta.ffmpeg.inputs.iter().enumerate() {
        ui.add_space(15.0);
        ui.label(
            RichText::new(format!("Input #{}", input_idx))
                .color(colors.fg_folder)
                .strong()
                .size(14.0),
        );

        for stream in &input.streams {
            ui.add_space(5.0);
            let stream_type_label = match stream.kind {
                StreamTypeMeta::Video { .. } => "Video",
                StreamTypeMeta::Audio { .. } => "Audio",
                StreamTypeMeta::Subtitle => "Subtitle",
                StreamTypeMeta::Unknown => "Unknown",
            };

            ui.label(
                RichText::new(format!("Stream #{}: {}", stream.index, stream_type_label))
                    .color(colors.fg_folder) // Using folder color for section headers
                    .strong(),
            );

            egui::Grid::new(format!("input_{}_stream_{}_grid", input_idx, stream.index))
                .num_columns(2)
                .spacing([10.0, 6.0])
                .striped(true)
                .show(ui, |ui| {
                    render_metadata_row(ui, "Format", &stream.format, colors);
                    if !stream.language.is_empty() && stream.language != "und" {
                        render_metadata_row(ui, "Language", &stream.language, colors);
                    }

                    match &stream.kind {
                        StreamTypeMeta::Video(v) => {
                            render_metadata_row(
                                ui,
                                "Dimensions",
                                &format!("{}x{}", v.width, v.height),
                                colors,
                            );
                            if !v.pix_fmt.is_empty() {
                                render_metadata_row(ui, "Pixel Format", &v.pix_fmt, colors);
                            }
                            if v.fps > 0.0 {
                                render_metadata_row(ui, "FPS", &format!("{:.2}", v.fps), colors);
                            }
                        }
                        StreamTypeMeta::Audio(a) => {
                            render_metadata_row(
                                ui,
                                "Sample Rate",
                                &format!("{} Hz", a.sample_rate),
                                colors,
                            );
                            render_metadata_row(ui, "Channels", &a.channels.to_string(), colors);
                        }
                        _ => {}
                    }
                });
        }
    }
}

fn render_metadata_row(ui: &mut egui::Ui, key: &str, value: &str, colors: &AppColors) {
    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
        ui.set_min_width(super::METADATA_TBL_KEY_COL_W);
        ui.set_max_width(super::METADATA_TBL_KEY_COL_W);
        ui.add(egui::Label::new(RichText::new(key).color(colors.fg)).wrap());
    });
    ui.add(egui::Label::new(RichText::new(value).color(colors.fg)).wrap());
    ui.end_row();
}

pub fn read_video_with_metadata(
    entry: DirEntryMeta,
    ctx: &egui::Context,
    available_width: Option<f32>,
) -> Result<VideoMeta, String> {
    ffmpeg::init()?;

    let title = entry
        .path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Create FfmpegMeta to store all gathered metadata
    let mut ffmpeg_meta = FfmpegMeta::default();

    // Try to extract a real thumbnail from the video
    let thumbnail_texture =
        match extract_video_thumbnail(ctx, entry, &mut ffmpeg_meta, available_width) {
            Ok(texture) => texture,
            Err(_) => {
                // Fall back to placeholder thumbnail
                generate_placeholder_thumbnail(ctx)
                    .map_err(|e| format!("Failed to generate thumbnail: {e}"))?
            }
        };

    let meta = VideoMeta::new(title, ffmpeg_meta.clone(), thumbnail_texture);

    Ok(meta)
}

/// Extract a thumbnail from the video file using ffmpeg-sidecar
fn extract_video_thumbnail(
    ctx: &egui::Context,
    entry: DirEntryMeta,
    ffmpeg_meta: &mut FfmpegMeta,
    available_width: Option<f32>,
) -> Result<egui::TextureHandle, String> {
    let path = &entry.path;
    let start = std::time::Instant::now();
    let path_str = path.to_str().ok_or("Invalid path encoding")?;

    // 1. Probe metadata and extract the first sample frame concurrently
    // Note: we can't use disp:attached_pic here yet because we don't know if it exists until probe finishes.
    // However, for most videos, the first frame (0.0) is what we want anyway.
    let (probe_res, first_sample_res) = rayon::join(
        || probe_metadata(path_str, ffmpeg_meta),
        || extract_and_score_frame(path_str, 0.0, available_width, None),
    );

    probe_res.map_err(|e| format!("Failed to probe metadata: {e}"))?;

    let has_attached_pic = ffmpeg_meta
        .inputs
        .iter()
        .any(|i| i.streams.iter().any(|s| s.is_attached_pic));

    // Collect all samples
    let mut samples = Vec::new();
    let mut skip_remaining = false;
    if let Some(first_res) = first_sample_res {
        let first = first_res?;
        // Optimization: if the first frame has a good enough score, skip the remaining position frame sampling to save time
        if first.0 > MIN_SUFFICIENT_QUALITY_SCORE {
            skip_remaining = true;
        }
        samples.push(first);
    }

    // Use provided cover if already exists
    if !skip_remaining && has_attached_pic {
        if let Some(Ok(pic_frame)) =
            extract_and_score_frame(path_str, 0.0, available_width, Some("disp:attached_pic"))
        {
            samples.push(pic_frame);
            skip_remaining = true;
        }
    }

    // 2. Sample remaining positions in parallel if duration is known
    if !skip_remaining
        && let Some(duration_secs) = ffmpeg_meta.duration_secs
        && duration_secs > 0.0
    {
        let mut remaining_samples: Vec<_> = THUMBNAIL_SAMPLE_RATIOS
            .par_iter()
            .filter_map(|&ratio| {
                match extract_and_score_frame(
                    path_str,
                    duration_secs * ratio,
                    available_width,
                    None,
                ) {
                    Some(Ok(frame)) => Some(frame),
                    Some(Err(e)) => {
                        warn!("Failed to extract frame at ratio {}: {}", ratio, e);
                        None
                    }
                    None => None,
                }
            })
            .collect();
        samples.append(&mut remaining_samples);
    }

    let num_samples = samples.len();
    // Find the best sample
    let best_result = samples
        .into_iter()
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let (best_score, frame) =
        best_result.ok_or_else(|| "No frames could be extracted".to_string())?;

    debug!(
        "extracted video thumbnail in {:?} (score: {:.2}, samples: {}, skipped remaining: {})",
        start.elapsed(),
        best_score,
        num_samples,
        skip_remaining
    );

    // Creates the image buffer once from the raw frame data
    let img = image::RgbImage::from_raw(frame.width, frame.height, frame.data)
        .ok_or("Failed to create image from raw frame data")?;

    // Create texture from the image buffer
    let color_image =
        egui::ColorImage::from_rgb([frame.width as usize, frame.height as usize], &img);
    let texture_id = format!("video_thumbnail_{}", path.display());
    let texture = ctx.load_texture(texture_id, color_image, egui::TextureOptions::default());

    // Spawn background task to encode and save cache
    let title_clone = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let ffmpeg_meta_clone = ffmpeg_meta.clone();
    let img_dyn = image::DynamicImage::ImageRgb8(img);
    let img_clone = img_dyn.clone();

    std::thread::spawn(move || {
        let mut png_bytes = Vec::new();
        if img_clone
            .write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageFormat::Png,
            )
            .is_ok()
        {
            let cached_content = CachedPreviewContent::Video(CachedVideoMeta {
                title: title_clone,
                ffmpeg: ffmpeg_meta_clone.into(),
                cache_bytes: png_bytes,
            });
            let cache_key = preview_cache::calculate_cache_key(&entry);
            if let Err(e) = preview_cache::save_preview(&cache_key, &cached_content) {
                tracing::warn!("Failed to save video preview cache: {}", e);
            }
        }
    });

    Ok(texture)
}

fn probe_metadata(path_str: &str, meta: &mut FfmpegMeta) -> Result<(), String> {
    for event in ffmpeg::get_metadata_probe_iter(path_str)? {
        match event {
            FfmpegEvent::Log(_level, msg) => {
                // Parse generic metadata lines
                // Example: "[info]     major_brand     : mp42"
                if let Some(colon_idx) = msg.find(':') {
                    let key_part = &msg[..colon_idx];
                    // Metadata keys are usually indented with 4 spaces after "[info] "
                    if key_part.contains("    ") {
                        let key = key_part.split_whitespace().last().unwrap_or("").trim();
                        let value = msg[colon_idx + 1..].trim();
                        if !key.is_empty() && !value.is_empty() {
                            meta.misc_metadata
                                .insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
            FfmpegEvent::ParsedDuration(d) => {
                meta.duration_secs = Some(d.duration);
                // format duration back to HH:MM:SS.ss for display
                let hrs = (d.duration / 3600.0) as u32;
                let mins = ((d.duration % 3600.0) / 60.0) as u32;
                let secs = d.duration % 60.0;
                let duration_str = if hrs > 0 {
                    format!("{:02}:{:02}:{:05.2}", hrs, mins, secs)
                } else {
                    format!("{:02}:{:05.2}", mins, secs)
                };
                meta.key_metadata
                    .insert(metadata::VID_DURATION.to_string(), duration_str);
            }
            FfmpegEvent::ParsedInputStream(stream) => {
                let kind = match stream.type_specific_data {
                    StreamTypeSpecificData::Video(v) => StreamTypeMeta::Video(v),
                    StreamTypeSpecificData::Audio(a) => StreamTypeMeta::Audio(a),
                    _ => StreamTypeMeta::Unknown,
                };

                let parent_idx = stream.parent_index as usize;
                if meta.inputs.len() <= parent_idx {
                    meta.inputs.resize(parent_idx + 1, InputMeta::default());
                }

                let is_attached_pic = stream.raw_log_message.contains("(attached pic)");
                meta.inputs[parent_idx].streams.push(StreamMeta {
                    index: stream.stream_index as usize,
                    format: stream.format,
                    language: stream.language,
                    kind,
                    is_attached_pic,
                });
            }
            _ => {}
        }
    }
    Ok(())
}

fn extract_and_score_frame(
    path_str: &str,
    seek_time: f64,
    available_width: Option<f32>,
    mapping: Option<&str>,
) -> Option<Result<(f64, OutputVideoFrame), String>> {
    let mut iter =
        ffmpeg::get_frame_extraction_iter(path_str, seek_time, available_width, mapping)?;
    let frame = iter.next()?;

    let score = {
        let rgb_img = match image::RgbImage::from_raw(frame.width, frame.height, frame.data.clone())
        {
            Some(img) => img,
            None => {
                return Some(Err(format!(
                    "Failed to create image buffer from raw data for frame at {}",
                    seek_time
                )));
            }
        };
        // Use grayscale version for faster scoring
        const CLIP_WIDTH: u32 = 320;
        let clip_width = available_width
            .map(|w| (w as u32).min(CLIP_WIDTH))
            .unwrap_or(CLIP_WIDTH);
        let gray = if frame.width > clip_width {
            let n_width = clip_width;
            let n_height = (frame.height as f64 * (n_width as f64 / frame.width as f64)) as u32;
            let resized = image::imageops::resize(
                &rgb_img,
                n_width,
                n_height,
                image::imageops::FilterType::Triangle,
            );
            image::DynamicImage::ImageRgb8(resized)
        } else {
            image::DynamicImage::ImageRgb8(rgb_img)
        }
        .grayscale();
        calculate_frame_quality(&gray)
    };

    Some(Ok((score, frame)))
}

fn calculate_frame_quality(dynamic_img: &image::DynamicImage) -> f64 {
    let img = dynamic_img.to_luma8();
    let (width, height) = img.dimensions();
    let total_pixels = (width * height) as f64;
    if total_pixels == 0.0 {
        return 0.0;
    }
    let pixels = img.as_bytes();

    // Pass 1: Calculate the mean brightness
    let sum: u64 = pixels.iter().map(|p| *p as u64).sum();
    let avg_brightness = sum as f64 / total_pixels;

    // Pass 2: Calculate the population variance (sum of squared differences from the mean)
    let sum_sq_diff: f64 = pixels
        .iter()
        .map(|p| {
            let diff = *p as f64 - avg_brightness;
            diff * diff
        })
        .sum();
    let variance = sum_sq_diff / total_pixels;

    // Brightness suitability score
    let brightness_score = if !(22.0..=194.0).contains(&avg_brightness) {
        0.0
    } else if avg_brightness < 56.0 {
        (avg_brightness - 22.0) / (56.0 - 22.0)
    } else if avg_brightness <= 171.0 {
        1.0
    } else {
        1.0 - (avg_brightness - 171.0) / (194.0 - 171.0)
    };

    // Variance score (diversity of brightness)
    let variance_score = (variance / 5000.0).min(1.0);

    // Sharpness (Laplacian variance)
    let mut lap_sum = 0.0;
    let w = width as usize;
    let h = height as usize;

    if h > 2 && w > 2 {
        for y in 1..h - 1 {
            let row_offset = y * w;
            let prev_row_offset = (y - 1) * w;
            let next_row_offset = (y + 1) * w;

            for x in 1..w - 1 {
                let center = pixels[row_offset + x] as f64;
                let top = pixels[prev_row_offset + x] as f64;
                let bottom = pixels[next_row_offset + x] as f64;
                let left = pixels[row_offset + x - 1] as f64;
                let right = pixels[row_offset + x + 1] as f64;

                // Laplacian kernel: [[0,-1,0], [-1,4,-1], [0,-1,0]]
                let laplacian = 4.0 * center - top - bottom - left - right;
                lap_sum += laplacian * laplacian;
            }
        }
    }

    let lap_count = (w - 2) * (h - 2);
    let lap_variance = if lap_count > 0 {
        lap_sum / lap_count as f64
    } else {
        0.0
    };
    let sharpness_score = (lap_variance / 2000.0).min(1.0);

    // Weighted average of all factors
    brightness_score * 0.33 + variance_score * 0.33 + sharpness_score * 0.34
}

/// Generate a placeholder thumbnail for video files if extraction fails
const PLACEHOLDER_WIDTH: usize = 320;
const PLACEHOLDER_HEIGHT: usize = 240;
const PLACEHOLDER_SIZE: usize = PLACEHOLDER_WIDTH * PLACEHOLDER_HEIGHT * 3;

static PLACEHOLDER_THUMBNAIL: [u8; PLACEHOLDER_SIZE] = {
    let mut data = [0u8; PLACEHOLDER_SIZE];
    let mut y = 0;
    while y < PLACEHOLDER_HEIGHT {
        let mut x = 0;
        while x < PLACEHOLDER_WIDTH {
            let idx = (y * PLACEHOLDER_WIDTH + x) * 3;
            let mut r = 40u8;
            let mut g = 40u8;
            let mut b = 40u8;

            if x < 2 || x >= PLACEHOLDER_WIDTH - 2 || y < 2 || y >= PLACEHOLDER_HEIGHT - 2 {
                r = 80;
                g = 80;
                b = 80;
            }

            let center_x = (PLACEHOLDER_WIDTH / 2) as i32;
            let center_y = (PLACEHOLDER_HEIGHT / 2) as i32;
            let rel_x = x as i32 - center_x;
            let rel_y = y as i32 - center_y;

            let abs_rel_y = if rel_y < 0 { -rel_y } else { rel_y };

            if rel_x >= -15 && rel_x <= 15 && abs_rel_y <= 15 {
                let max_y = if rel_x <= 0 {
                    15
                } else {
                    15 - (rel_x * 15) / 15
                };

                if abs_rel_y <= max_y {
                    r = 220;
                    g = 220;
                    b = 220;
                }
            }

            data[idx] = r;
            data[idx + 1] = g;
            data[idx + 2] = b;

            x += 1;
        }
        y += 1;
    }
    data
};

/// Generate a placeholder thumbnail for video files if extraction fails
fn generate_placeholder_thumbnail(ctx: &egui::Context) -> Result<egui::TextureHandle, String> {
    let color_image = egui::ColorImage::from_rgb(
        [PLACEHOLDER_WIDTH, PLACEHOLDER_HEIGHT],
        &PLACEHOLDER_THUMBNAIL,
    );
    let texture = ctx.load_texture(
        "video_placeholder",
        color_image,
        egui::TextureOptions::default(),
    );

    Ok(texture)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, GrayImage, Luma};

    #[test]
    fn test_calculate_frame_quality_black_image() {
        let width = 100;
        let height = 100;
        let img = GrayImage::new(width, height);
        let dynamic_img = DynamicImage::ImageLuma8(img);

        let score = calculate_frame_quality(&dynamic_img);

        assert!(
            score < 0.01,
            "Black image should have a very low score, got {}",
            score
        );
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_calculate_frame_quality_gradient_image() {
        let width = 100;
        let height = 100;
        let mut img = GrayImage::new(width, height);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let val = (((x + y) as f64) / ((width + height) as f64) * 255.0) as u8;
            *pixel = Luma([val]);
        }
        let dynamic_img = DynamicImage::ImageLuma8(img);

        let score = calculate_frame_quality(&dynamic_img);

        assert!(
            score > 0.5,
            "Gradient image should have a higher score than black image, got {}",
            score
        );
    }
}
