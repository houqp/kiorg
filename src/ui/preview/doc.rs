//! Document preview module (PDF, EPUB)

use crate::config::colors::AppColors;
use crate::models::preview_content::{DocMeta, PreviewContent};
use egui::{Image, RichText};
use pathfinder_geometry::transform2d::Transform2F;
use pdf_render::{render_page, Cache, SceneBackend};
use std::path::Path;

const METADATA_KEY_COLUMN_WIDTH: f32 = 100.0;

/// Render document content
pub fn render(
    ui: &mut egui::Ui,
    doc_meta: &DocMeta,
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    // Display document title
    ui.label(
        RichText::new(&doc_meta.title)
            .color(colors.fg)
            .strong()
            .size(20.0),
    );
    ui.add_space(10.0);

    // Display cover image if available (centered)
    if let Some(cover) = &doc_meta.cover {
        ui.vertical_centered(|ui| {
            ui.add(
                Image::new(cover.clone())
                    .max_size(egui::vec2(available_width, available_height * 0.6))
                    .maintain_aspect_ratio(true),
            );
        });
        ui.add_space(15.0);
    }

    egui::Grid::new("doc_metadata_grid")
        .num_columns(2)
        .spacing([10.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            // Sort keys for consistent display
            let mut sorted_keys: Vec<&String> = doc_meta.metadata.keys().collect();
            sorted_keys.sort();

            // Display each metadata field in a table row
            for key in sorted_keys {
                if let Some(value) = doc_meta.metadata.get(key) {
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

/// Render a PDF page and extract metadata
pub fn extract_pdf_metadata(path: &Path, page_number: u32) -> Result<PreviewContent, String> {
    let file = pdf::file::FileOptions::uncached()
        .open(path)
        .map_err(|e| format!("Failed to open PDF file: {}", e))?;
    let resolver = file.resolver();

    // Extract metadata
    let mut metadata = std::collections::HashMap::new();

    // Get PDF version
    if let Ok(version) = file.version() {
        let version_str = format!("{:?}", version).trim_matches('"').to_string();
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
    metadata.insert("Page Count".to_string(), file.num_pages().to_string());

    // Get the page for rendering
    let page = file
        .get_page(page_number)
        .map_err(|e| format!("Failed to get page {}: {}", page_number, e))?;

    let mut cache = Cache::new();
    let mut backend = SceneBackend::new(&mut cache);

    let dpi = 150.0; // Adjust DPI as needed for quality vs performance

    render_page(
        &mut backend,
        &resolver,
        &page,
        Transform2F::from_scale(dpi / 25.4),
    )
    .map_err(|e| format!("Failed to render page: {}", e))?;

    let scene = backend.finish();

    {
        use pathfinder_export::Export;
        use pathfinder_export::FileFormat;

        let mut svg_data = Vec::new();
        scene.export(&mut svg_data, FileFormat::SVG).unwrap();
        let svg_bytes: egui::load::Bytes = svg_data.into();
        let img_source =
            egui::widgets::ImageSource::from((format!("bytes:://{:?}.svg", path), svg_bytes));

        // Return PreviewContent directly
        Ok(PreviewContent::pdf(img_source, metadata, title))
    }

    // Rasterize the scene to an RGBA image using the pathfinder_rasterize crate
    // {
    // use pathfinder_rasterize::Rasterizer;
    // let image = Rasterizer::new().rasterize(scene, None);

    // // Convert the RGBA image to egui's TextureHandle
    // let width = image.width();
    // let height = image.height();
    // let size = [width, height];

    // // Get raw pixel data
    // let pixels_data = image.data();

    // // Create a ColorImage from the raw RGBA data
    // let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels_data);

    // // Create a texture from the color image
    // let texture_handle: egui::TextureHandle = ctx.load_texture(
    //     format!("pdf_page_{}_{}", page_number, path.display()),
    //     color_image,
    //     egui::TextureOptions::default(),
    // );

    // Ok(texture_handle)
    // }
}

/// Extract metadata and cover image from an EPUB file and return as PreviewContent
pub fn extract_epub_metadata(path: &Path) -> Result<PreviewContent, String> {
    use epub::doc::EpubDoc;

    // Open the EPUB file
    let mut doc = EpubDoc::new(path).map_err(|e| format!("Failed to open EPUB file: {}", e))?;

    // Get metadata
    let metadata = doc.metadata.clone();

    // Try to extract cover image
    let cover_image = try_extract_epub_cover(&mut doc);

    // Return the PreviewContent directly
    Ok(PreviewContent::epub(metadata, cover_image))
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
