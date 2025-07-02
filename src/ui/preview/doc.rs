//! Document preview module (PDF, EPUB)

use crate::config::colors::AppColors;
use crate::models::preview_content::{EpubMeta, PdfMeta, PreviewContent};
use egui::{RichText, TextureId, Vec2, load::SizedTexture, widgets::ImageSource}; // Corrected import for SizedTexture and ImageSource
use pathfinder_geometry::transform2d::Transform2F;
use pdf::file::{NoCache, NoLog};
use pdf_render::{Cache, SceneBackend, render_page};
use std::path::Path;
// Removed unused import: use std::sync::Arc;

const METADATA_KEY_COLUMN_WIDTH: f32 = 100.0;

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
pub fn render_pdf_page(
    file: &pdf::file::File<Vec<u8>, NoCache, NoCache, NoLog>,
    page_number: usize,
    file_id: Option<&str>,
) -> Result<egui::widgets::ImageSource<'static>, String> {
    render_pdf_page_with_dpi(file, page_number, file_id, 150.0) // Use 150 DPI for regular preview
}

/// Render a specific PDF page as an egui `ImageSource` with high DPI for popup view
pub fn render_pdf_page_high_dpi(
    file: &pdf::file::File<Vec<u8>, NoCache, NoCache, NoLog>,
    page_number: usize,
    file_id: Option<&str>,
) -> Result<egui::widgets::ImageSource<'static>, String> {
    render_pdf_page_with_dpi(file, page_number, file_id, 300.0) // Use 300 DPI for popup
}

/// Render a specific PDF page as an egui `ImageSource` with configurable DPI
fn render_pdf_page_with_dpi(
    file: &pdf::file::File<Vec<u8>, NoCache, NoCache, NoLog>,
    page_number: usize,
    file_id: Option<&str>,
    dpi: f32,
) -> Result<egui::widgets::ImageSource<'static>, String> {
    let resolver = file.resolver();

    // Get the page for rendering
    let page = file
        .get_page(page_number as u32)
        .map_err(|e| format!("Failed to get page {page_number}: {e}"))?;

    // Set up rendering with configurable DPI
    let mut cache = Cache::new();
    let mut backend = SceneBackend::new(&mut cache);

    render_page(
        &mut backend,
        &resolver,
        &page,
        Transform2F::from_scale(dpi / 25.4),
    )
    .map_err(|e| format!("Failed to render page: {e}"))?;

    let scene = backend.finish();

    // Export as SVG for resolution-independent rendering
    use pathfinder_export::Export;
    use pathfinder_export::FileFormat;

    let mut svg_data = Vec::new();
    scene.export(&mut svg_data, FileFormat::SVG).unwrap();
    let svg_bytes: egui::load::Bytes = svg_data.into();

    // Create a unique texture ID that includes the file identifier, page number, and DPI
    // This ensures that different documents and different DPI levels don't share texture cache entries
    let texture_id = if let Some(id) = file_id {
        format!(
            "bytes://pdf_doc_{}_page_{}_dpi_{}.svg",
            id, page_number, dpi as u32
        )
    } else {
        // Generate a timestamp-based ID if no file_id is provided
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        format!(
            "bytes://pdf_doc_{}_page_{}_dpi_{}.svg",
            now, page_number, dpi as u32
        )
    };

    let img_source = ImageSource::from((texture_id, svg_bytes));

    Ok(img_source)
}

/// Render a PDF page and extract metadata
pub fn extract_pdf_metadata(path: &Path, _: u32) -> Result<PreviewContent, String> {
    let file = pdf::file::FileOptions::uncached()
        .open(path)
        .map_err(|e| format!("Failed to open PDF file: {e}"))?;

    // Generate a unique file ID based on the path
    let file_id = path.to_string_lossy().to_string();

    // Render the first page as the cover
    let cover_image = render_pdf_page(&file, 0, Some(&file_id))?;

    // Extract metadata
    let mut metadata = std::collections::HashMap::new();

    // Get PDF version
    if let Ok(version) = file.version() {
        let version_str = format!("{version:?}").trim_matches('"').to_string();
        metadata.insert("PDF Version".to_string(), version_str);
    }

    fn format_pdf_date(date: &pdf::primitive::Date) -> String {
        format!(
            "{}-{}-{} {}:{}:{}",
            date.year, date.month, date.day, date.hour, date.minute, date.second
        )
    }

    // Get document info if available
    let mut title = None;
    if let Some(ref info) = file.trailer.info_dict {
        // Extract common metadata fields
        if let Some(v) = &info.title {
            title = Some(v.to_string_lossy());
        }
        if let Some(author) = &info.author {
            metadata.insert("Author".to_string(), author.to_string_lossy());
        }
        if let Some(subject) = &info.subject {
            metadata.insert("Subject".to_string(), subject.to_string_lossy());
        }
        if let Some(keywords) = &info.keywords {
            metadata.insert("Keywords".to_string(), keywords.to_string_lossy());
        }
        if let Some(creator) = &info.creator {
            metadata.insert("Creator".to_string(), creator.to_string_lossy());
        }
        if let Some(producer) = &info.producer {
            metadata.insert("Producer".to_string(), producer.to_string_lossy());
        }
        if let Some(creation_date) = &info.creation_date {
            metadata.insert("Creation Date".to_string(), format_pdf_date(creation_date));
        }
        if let Some(mod_date) = &info.mod_date {
            metadata.insert("Modification Date".to_string(), format_pdf_date(mod_date));
        }
    }

    // Get catalog information
    let catalog = file.get_root();
    if let Some(ref version) = catalog.version {
        metadata.insert("PDF Catalog Version".to_string(), version.to_string());
    }

    // Get page count
    let page_count = file.num_pages() as usize;

    // Store the PDF file in an Arc for sharing across page navigation
    let pdf_file = std::sync::Arc::new(file);

    // Return PreviewContent with metadata, title, and the rendered first page as cover
    Ok(PreviewContent::pdf_with_file(
        cover_image,
        metadata,
        title,
        page_count,
        pdf_file,
        path,
    ))
}

/// Extract metadata and cover image from an EPUB file and return as `PreviewContent`
pub fn extract_epub_metadata(path: &Path) -> Result<PreviewContent, String> {
    use epub::doc::EpubDoc;

    // Open the EPUB file
    let mut doc = EpubDoc::new(path).map_err(|e| format!("Failed to open EPUB file: {e}"))?;

    // Get metadata
    let metadata = doc.metadata.clone();

    // Get page count using get_num_pages method
    let page_count = doc.get_num_pages();

    // Try to extract cover image
    let cover_image = try_extract_epub_cover(&mut doc).unwrap_or_else(|| {
        // Create a default "no cover" image if extraction fails
        // This is a placeholder and won't actually render anything meaningful
        // without a proper texture upload to the egui context.
        // For a real solution, a default image should be loaded as a texture.
        let sized_texture = SizedTexture::new(TextureId::Managed(0), Vec2::ZERO);
        ImageSource::Texture(sized_texture)
    });

    // Return the PreviewContent directly
    Ok(PreviewContent::epub(metadata, cover_image, page_count))
}

/// Try to extract cover image from an EPUB document
fn try_extract_epub_cover<R: std::io::Read + std::io::Seek>(
    doc: &mut epub::doc::EpubDoc<R>,
) -> Option<egui::widgets::ImageSource<'static>> {
    // Try to get the cover image
    let cover_result = doc.get_cover()?;
    let (cover_data, mime_type) = cover_result;
    let texture_id = format!(
        "bytes://epub_cover_{}.{}",
        doc.metadata
            .get("identifier")
            .map(|v| v[0].clone())
            .unwrap_or_else(|| {
                let now = std::time::SystemTime::now();
                let datetime: chrono::DateTime<chrono::Utc> = now.into();
                datetime.format("%Y-%m-%d_%H:%M:%S").to_string()
            }),
        mime_type,
    );
    Some(egui::widgets::ImageSource::from((texture_id, cover_data)))
}
