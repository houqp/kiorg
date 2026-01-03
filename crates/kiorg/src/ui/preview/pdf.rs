//! PDF preview module

use crate::config::colors::AppColors;
use crate::models::preview_content::{PdfMeta, metadata};
use egui::{ColorImage, RichText, TextureOptions, widgets::ImageSource};
use pdfium_bind::PdfDocument;
use std::path::Path;
use std::sync::{Arc, Mutex};

fn format_pdf_date(pdf_date: &str) -> String {
    // PDF date format: D:YYYYMMDDHHmmSSOHH'mm'
    // Example: D:20240904003000Z or D:20240904003000+08'00'

    if !pdf_date.starts_with("D:") || pdf_date.len() < 16 {
        return pdf_date.to_string();
    }

    let date_part = &pdf_date[2..]; // Remove "D:" prefix

    // Extract components
    let year = &date_part[0..4];
    let month = &date_part[4..6];
    let day = &date_part[6..8];
    let hour = &date_part[8..10];
    let minute = &date_part[10..12];
    let second = date_part.get(12..14).unwrap_or("00");

    // Format as YYYY-MM-DD HH:mm:ss
    format!("{}-{}-{} {}:{}:{}", year, month, day, hour, minute, second)
}

/// Render PDF content
pub fn render(
    ui: &mut egui::Ui,
    pdf_meta: &PdfMeta,
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    // Display PDF title
    ui.label(
        RichText::new(&pdf_meta.title)
            .color(colors.fg)
            .strong()
            .size(20.0),
    );
    ui.add_space(10.0);

    // Display cover image (centered)
    ui.vertical_centered(|ui| {
        ui.add(
            egui::Image::new(pdf_meta.cover.clone())
                .max_size(egui::vec2(available_width, available_height * 0.6))
                .maintain_aspect_ratio(true),
        );
    });
    ui.add_space(15.0);

    egui::Grid::new("pdf_metadata_grid")
        .num_columns(2)
        .spacing([10.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            // Add page count first
            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                ui.set_min_width(super::METADATA_TBL_KEY_COL_W);
                ui.set_max_width(super::METADATA_TBL_KEY_COL_W);
                ui.add(
                    egui::Label::new(RichText::new(metadata::PDF_PAGE_COUNT).color(colors.fg))
                        .wrap(),
                );
            });
            ui.add(
                egui::Label::new(RichText::new(pdf_meta.page_count.to_string()).color(colors.fg))
                    .wrap(),
            );
            ui.end_row();

            // Sort keys for consistent display
            let mut sorted_keys: Vec<&String> = pdf_meta.metadata.keys().collect();
            sorted_keys.sort();

            // Display each metadata field in a table row
            for key in sorted_keys {
                if let Some(value) = pdf_meta.metadata.get(key) {
                    // Format the key with proper capitalization for display
                    let display_key = format_metadata_key(key);
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                        ui.set_min_width(super::METADATA_TBL_KEY_COL_W);
                        ui.set_max_width(super::METADATA_TBL_KEY_COL_W);
                        ui.add(
                            egui::Label::new(RichText::new(display_key).color(colors.fg)).wrap(),
                        );
                    });
                    ui.add(egui::Label::new(RichText::new(value).color(colors.fg)).wrap());
                    ui.end_row();
                }
            }
        });
}

/// Format metadata key for display by capitalizing words and removing prefixes
fn format_metadata_key(key: &str) -> String {
    // Handle common prefixes like "dc:"
    let clean_key = if key.contains(':') {
        key.split(':').next_back().unwrap_or(key)
    } else {
        key
    };

    // Split by underscores, hyphens, or spaces
    let words: Vec<&str> = clean_key.split(['_', '-', ' ']).collect();

    // Capitalize each word
    let capitalized: Vec<String> = words
        .iter()
        .map(|word| {
            if word.is_empty() {
                String::new()
            } else {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            }
        })
        .collect();

    // Join with spaces
    capitalized.join(" ")
}

/// Render a specific PDF page as an egui `ImageSource`
#[inline]
pub fn render_pdf_page_low_dpi(
    doc: &PdfDocument,
    page_number: isize,
    file_id: Option<&str>,
    ctx: &egui::Context,
) -> Result<(egui::widgets::ImageSource<'static>, egui::TextureHandle), String> {
    render_pdf_page_with_dpi(doc, page_number, file_id, 150.0, ctx) // Use 150 DPI for regular preview
}

/// Render a specific PDF page as an egui `ImageSource` with high DPI for popup view
#[inline]
pub fn render_pdf_page_high_dpi(
    doc: &PdfDocument,
    page_number: isize,
    file_id: Option<&str>,
    ctx: &egui::Context,
) -> Result<(egui::widgets::ImageSource<'static>, egui::TextureHandle), String> {
    render_pdf_page_with_dpi(doc, page_number, file_id, 300.0, ctx) // Use 300 DPI for popup
}

/// Render a specific PDF page as an egui `ImageSource` with configurable DPI
fn render_pdf_page_with_dpi(
    doc: &PdfDocument,
    page_number: isize,
    file_id: Option<&str>,
    dpi: f32,
    ctx: &egui::Context,
) -> Result<(egui::widgets::ImageSource<'static>, egui::TextureHandle), String> {
    let (pixel_data, width, height) = doc.render_page(page_number, dpi)?;

    let color_image =
        ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &pixel_data);

    let texture_id_str = if let Some(id) = file_id {
        format!("pdf_doc_{id}_page_{page_number}_dpi_{dpi}")
    } else {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        format!("pdf_doc_{now}_page_{page_number}_dpi_{dpi}")
    };

    let texture_handle = ctx.load_texture(texture_id_str, color_image, TextureOptions::LINEAR);
    let img_source = ImageSource::from(&texture_handle);

    Ok((img_source, texture_handle))
}

/// Render a PDF page and extract metadata
pub fn extract_pdf_metadata(
    path: &Path,
    ctx: &egui::Context,
) -> Result<crate::models::preview_content::PdfMeta, String> {
    let doc = PdfDocument::open(path)?;
    let file_id = path.to_string_lossy();
    let (cover_image, texture_handle) = render_pdf_page_low_dpi(&doc, 0, Some(&file_id), ctx)?;

    // Extract metadata
    let mut metadata = std::collections::HashMap::new();
    for &field in &[
        metadata::PDF_TITLE,
        metadata::PDF_AUTHOR,
        metadata::PDF_SUBJECT,
        metadata::PDF_KEYWORDS,
        metadata::PDF_CREATOR,
        metadata::PDF_PRODUCER,
        metadata::PDF_TRAPPED,
    ] {
        if let Some(value) = doc.get_metadata_value(field) {
            metadata.insert(field.to_string(), value);
        }
    }

    for &field in &[metadata::PDF_CREATION_DATE, metadata::PDF_MOD_DATE] {
        if let Some(value) = doc.get_metadata_value(field) {
            metadata.insert(field.to_string(), format_pdf_date(&value));
        }
    }

    let version = doc.get_pdf_version();
    metadata.insert(metadata::PDF_VERSION.to_string(), format!("{}", version));

    let title = metadata.get(metadata::PDF_TITLE).cloned();
    let page_count = doc.page_count();

    Ok(crate::models::preview_content::PdfMeta::new(
        cover_image,
        Some(texture_handle),
        metadata,
        title,
        page_count,
        Arc::new(Mutex::new(doc)),
        path,
    ))
}
