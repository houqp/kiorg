//! Ebook preview module

use egui::{RichText, TextureId, Vec2, load::SizedTexture, widgets::ImageSource};

use crate::config::colors::AppColors;
use crate::models::dir_entry::DirEntryMeta;
use crate::models::preview_content::{CachedEbookMeta, CachedPreviewContent, EbookMeta, metadata};
use crate::utils::preview_cache;

/// Render Ebook content
pub fn render(
    ui: &mut egui::Ui,
    ebook_meta: &EbookMeta,
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    // Display ebook title
    ui.label(
        RichText::new(&ebook_meta.title)
            .color(colors.fg)
            .strong()
            .size(20.0),
    );
    ui.add_space(10.0);

    // Display cover image (centered)
    ui.vertical_centered(|ui| {
        ui.add(
            egui::Image::new(ebook_meta.cover.clone())
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
                ui.set_min_width(super::METADATA_TBL_KEY_COL_W);
                ui.set_max_width(super::METADATA_TBL_KEY_COL_W);
                ui.add(
                    egui::Label::new(RichText::new(metadata::PDF_PAGE_COUNT).color(colors.fg))
                        .wrap(),
                );
            });
            ui.add(
                egui::Label::new(RichText::new(ebook_meta.page_count.to_string()).color(colors.fg))
                    .wrap(),
            );
            ui.end_row();

            // Sort keys for consistent display
            let mut sorted_keys: Vec<&String> = ebook_meta.metadata.keys().collect();
            sorted_keys.sort();

            // Display each metadata field in a table row
            for key in sorted_keys {
                if let Some(value) = ebook_meta.metadata.get(key) {
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

/// Extract metadata and cover image from an ebook file and return as `PreviewContent`
pub fn extract_ebook_metadata(entry: DirEntryMeta) -> Result<EbookMeta, String> {
    use rbook::prelude::*;

    let path = &entry.path;
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
    let identifier = epub.metadata().identifiers().next().map_or_else(
        || {
            let now = std::time::SystemTime::now();
            let datetime: chrono::DateTime<chrono::Utc> = now.into();
            datetime.format("%Y-%m-%d_%H:%M:%S").to_string()
        },
        |id| id.value().to_string(),
    );

    let (cover_image, cover_data) = try_extract_rbook_cover(&epub)
        .map(|(img, raw_bytes, _href)| {
            let texture_id = format!("bytes://epub_cover_{identifier}");
            let source = egui::widgets::ImageSource::from((texture_id, raw_bytes.clone()));
            (source, Some((img, raw_bytes)))
        })
        .unwrap_or_else(|| {
            // Create a default "no cover" image if extraction fails
            let sized_texture = SizedTexture::new(TextureId::Managed(0), Vec2::ZERO);
            (ImageSource::Texture(sized_texture), None)
        });

    let meta = crate::models::preview_content::EbookMeta::new(
        metadata.clone(),
        cover_image,
        None,
        page_count,
        path,
    );

    // Spawn background task to encode and save cache after metadata is resolved
    if let Some((img, _raw_bytes)) = cover_data {
        let title_clone = meta.title.clone();

        std::thread::spawn(move || {
            let mut png_bytes = Vec::new();
            if img
                .write_to(
                    &mut std::io::Cursor::new(&mut png_bytes),
                    image::ImageFormat::Png,
                )
                .is_ok()
            {
                let cached_content = CachedPreviewContent::Ebook(CachedEbookMeta {
                    title: title_clone,
                    metadata,
                    page_count,
                    cache_bytes: png_bytes,
                });
                let cache_key = preview_cache::calculate_cache_key(&entry);
                if let Err(e) = preview_cache::save_preview(&cache_key, &cached_content) {
                    tracing::warn!("Failed to save ebook preview cache: {}", e);
                }
            }
        });
    }

    Ok(meta)
}

/// Try to extract cover image from an ebook document using rbook
fn try_extract_rbook_cover(epub: &rbook::Epub) -> Option<(image::DynamicImage, Vec<u8>, String)> {
    use rbook::prelude::*;

    epub.manifest().cover_image().and_then(|cover_image| {
        cover_image.read_bytes().ok().and_then(|cover_data| {
            let href = cover_image.href().name().decode().to_string();
            image::load_from_memory(&cover_data)
                .ok()
                .map(|img| (img, cover_data, href))
        })
    })
}
