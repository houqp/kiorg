//! Document preview module (PDF, EPUB)

use crate::config::colors::AppColors;
use crate::models::preview_content::{EpubMeta, PdfMeta, PreviewContent};
use egui::{
    ColorImage, RichText, TextureId, TextureOptions, Vec2, load::SizedTexture, widgets::ImageSource,
};
use pdfium_bind::PdfDocument;
use std::path::Path;
use std::sync::{Arc, Mutex};

const METADATA_KEY_COLUMN_WIDTH: f32 = 100.0;

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

/// Render document content
pub fn render(
    ui: &mut egui::Ui,
    pdf_meta: &PdfMeta,
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    render_pdf_preview(ui, pdf_meta, colors, available_width, available_height);
}

/// Render EPUB document content
pub fn render_epub(
    ui: &mut egui::Ui,
    epub_meta: &EpubMeta,
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    render_epub_preview(ui, epub_meta, colors, available_width, available_height);
}

/// Render PDF document preview
fn render_pdf_preview(
    ui: &mut egui::Ui,
    pdf_meta: &PdfMeta,
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    // Display document title
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
                ui.set_min_width(METADATA_KEY_COLUMN_WIDTH);
                ui.set_max_width(METADATA_KEY_COLUMN_WIDTH);
                ui.add(egui::Label::new(RichText::new("Page Count").color(colors.fg)).wrap());
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
                        ui.set_min_width(METADATA_KEY_COLUMN_WIDTH);
                        ui.set_max_width(METADATA_KEY_COLUMN_WIDTH);
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

/// Render EPUB document preview
fn render_epub_preview(
    ui: &mut egui::Ui,
    epub_meta: &EpubMeta,
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    // Display document title
    ui.label(
        RichText::new(&epub_meta.title)
            .color(colors.fg)
            .strong()
            .size(20.0),
    );
    ui.add_space(10.0);

    // Display cover image (centered)
    ui.vertical_centered(|ui| {
        ui.add(
            egui::Image::new(epub_meta.cover.clone())
                .max_size(egui::vec2(available_width, available_height * 0.6))
                .maintain_aspect_ratio(true),
        );
    });
    ui.add_space(15.0);

    egui::Grid::new("epub_metadata_grid")
        .num_columns(2)
        .spacing([10.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            // Add page count first
            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                ui.set_min_width(METADATA_KEY_COLUMN_WIDTH);
                ui.set_max_width(METADATA_KEY_COLUMN_WIDTH);
                ui.add(egui::Label::new(RichText::new("Page Count").color(colors.fg)).wrap());
            });
            ui.add(
                egui::Label::new(RichText::new(epub_meta.page_count.to_string()).color(colors.fg))
                    .wrap(),
            );
            ui.end_row();

            // Sort keys for consistent display
            let mut sorted_keys: Vec<&String> = epub_meta.metadata.keys().collect();
            sorted_keys.sort();

            // Display each metadata field in a table row
            for key in sorted_keys {
                if let Some(value) = epub_meta.metadata.get(key) {
                    // Format the key with proper capitalization for display
                    let display_key = format_metadata_key(key);
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                        ui.set_min_width(METADATA_KEY_COLUMN_WIDTH);
                        ui.set_max_width(METADATA_KEY_COLUMN_WIDTH);
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
pub fn extract_pdf_metadata(path: &Path, ctx: &egui::Context) -> Result<PreviewContent, String> {
    let doc = PdfDocument::open(path)?;
    let file_id = path.to_string_lossy().to_string();
    let (cover_image, texture_handle) = render_pdf_page_low_dpi(&doc, 0, Some(&file_id), ctx)?;

    // Extract metadata
    let mut metadata = std::collections::HashMap::new();
    for &field in &[
        "Title", "Author", "Subject", "Keywords", "Creator", "Producer", "Trapped",
    ] {
        if let Some(value) = doc.get_metadata_value(field) {
            metadata.insert(field.to_string(), value);
        }
    }

    for &field in &["CreationDate", "ModDate"] {
        if let Some(value) = doc.get_metadata_value(field) {
            metadata.insert(field.to_string(), format_pdf_date(&value));
        }
    }

    let version = doc.get_pdf_version();
    metadata.insert("PDF Version".to_string(), format!("{}", version));

    let title = metadata.get("Title").cloned();
    let page_count = doc.page_count();

    // Return PreviewContent with metadata, title, and the rendered first page as cover
    Ok(PreviewContent::pdf_with_file(
        cover_image,
        Some(texture_handle),
        metadata,
        title,
        page_count,
        Arc::new(Mutex::new(doc)),
        path,
    ))
}

/// Extract metadata and cover image from an EPUB file and return as `PreviewContent`
pub fn extract_epub_metadata(path: &Path) -> Result<PreviewContent, String> {
    use rbook::prelude::*;

    let epub = rbook::Epub::options()
        .strict(false)
        .open(path)
        .map_err(|e| format!("Failed to open EPUB file: {e}"))?;

    let epub_metadata = epub.metadata();
    let mut metadata = std::collections::HashMap::new();

    if let Some(title) = epub_metadata.title() {
        metadata.insert("title".to_string(), title.value().to_string());
    }

    let creators: Vec<String> = epub_metadata
        .creators()
        .map(|c| c.value().to_string())
        .collect();
    if !creators.is_empty() {
        metadata.insert("creator".to_string(), creators.join(", "));
    }

    let contributors: Vec<String> = epub_metadata
        .contributors()
        .map(|c| c.value().to_string())
        .collect();
    if !contributors.is_empty() {
        metadata.insert("contributor".to_string(), contributors.join(", "));
    }

    if let Some(publisher) = epub_metadata.publishers().next() {
        metadata.insert("publisher".to_string(), publisher.value().to_string());
    }

    if let Some(description) = epub_metadata.descriptions().next() {
        metadata.insert("description".to_string(), description.value().to_string());
    }

    let languages: Vec<String> = epub_metadata
        .languages()
        .map(|l| l.scheme().code().to_string())
        .collect();
    if !languages.is_empty() {
        metadata.insert("language".to_string(), languages.join(", "));
    }

    let identifiers: Vec<String> = epub_metadata
        .identifiers()
        .map(|i| {
            if let Some(scheme) = i.scheme() {
                format!("{:?}: {}", scheme, i.value())
            } else {
                i.value().to_string()
            }
        })
        .collect();
    if !identifiers.is_empty() {
        metadata.insert("identifier".to_string(), identifiers.join(", "));
    }

    let tags: Vec<String> = epub_metadata
        .tags()
        .map(|s| s.value().to_string())
        .collect();
    if !tags.is_empty() {
        metadata.insert("tags".to_string(), tags.join(", "));
    }

    // Get page count - count readable content items in spine
    let page_count = epub.spine().len() as isize;

    // Try to extract cover image
    let cover_image = try_extract_rbook_cover(&epub).unwrap_or_else(|| {
        // Create a default "no cover" image if extraction fails
        let sized_texture = SizedTexture::new(TextureId::Managed(0), Vec2::ZERO);
        ImageSource::Texture(sized_texture)
    });

    // Return the PreviewContent directly
    Ok(PreviewContent::epub_with_file(
        metadata,
        cover_image,
        page_count,
        path,
    ))
}

/// Helper function to create cover image source from image data and href
fn create_cover_image_source(
    cover_data: Vec<u8>,
    href: &str,
    identifier: &str,
) -> egui::widgets::ImageSource<'static> {
    let extension = href.split('.').next_back().unwrap_or("jpg");
    let texture_id = format!("bytes://epub_cover_{identifier}.{extension}");
    egui::widgets::ImageSource::from((texture_id, cover_data))
}

/// Try to extract cover image from an EPUB document using rbook
fn try_extract_rbook_cover(epub: &rbook::Epub) -> Option<egui::widgets::ImageSource<'static>> {
    use rbook::prelude::*;

    let identifier = epub.metadata().identifiers().next().map_or_else(
        || {
            let now = std::time::SystemTime::now();
            let datetime: chrono::DateTime<chrono::Utc> = now.into();
            datetime.format("%Y-%m-%d_%H:%M:%S").to_string()
        },
        |id| id.value().to_string(),
    );

    epub.manifest().cover_image().and_then(|cover_image| {
        cover_image.read_bytes().ok().map(|cover_data| {
            let href = cover_image.href().name().decode();
            create_cover_image_source(cover_data, &href, &identifier)
        })
    })
}
