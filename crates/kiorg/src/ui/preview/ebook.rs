//! Ebook preview module

use egui::{RichText, TextureId, Vec2, load::SizedTexture, widgets::ImageSource};
use std::borrow::Cow;
use std::collections::HashMap;

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
    let path = &entry.path;
    let epub = rbook::Epub::options()
        // Skip parsing the table of contents; not required here
        .skip_toc(true)
        .open(path)
        .map_err(|e| format!("Failed to open EPUB file: {e}"))?;

    // Extract epub metadata
    let metadata = create_metadata_map(&epub);
    // Get page count (count readable content items in spine)
    let page_count = epub.spine().len() as isize;
    // Try to extract the cover image
    let cover_data = try_extract_cover_data(&epub)
        // Create a default "no cover" image if extraction fails
        .unwrap_or_else(|| {
            ImageSource::Texture(SizedTexture::new(TextureId::Managed(0), Vec2::ZERO))
        });

    let meta = EbookMeta::new(metadata.clone(), cover_data, None, page_count, path);

    // Spawn background task to encode and save cache after metadata is resolved
    if let ImageSource::Bytes { bytes, .. } = &meta.cover {
        let title_clone = meta.title.clone();
        let raw_bytes = bytes.to_vec();

        std::thread::spawn(move || {
            let mut png_bytes = Vec::new();

            // Convert the cover image to PNG for consistency
            if let Err(error) = cover_image_to_png(&raw_bytes, &mut png_bytes) {
                tracing::warn!("Failed to convert cover image to PNG for cache: {}", error);
                return;
            }

            let cache_key = preview_cache::calculate_cache_key(&entry);
            let cached_content = CachedPreviewContent::Ebook(CachedEbookMeta {
                title: title_clone,
                metadata,
                page_count,
                cache_bytes: png_bytes,
            });

            // Save to cache
            if let Err(e) = preview_cache::save_preview(&cache_key, &cached_content) {
                tracing::warn!("Failed to save ebook preview cache: {}", e);
            }
        });
    }

    Ok(meta)
}

fn create_metadata_map(epub: &rbook::Epub) -> HashMap<String, String> {
    /// Utility to group multiple metadata entries with a single join.
    #[inline]
    fn insert_joined<'a, I, S>(key: &str, metadata: &mut HashMap<String, String>, iter: I)
    where
        I: Iterator<Item = S>,
        // This avoids intermediate allocations (all strings are joined using `Vec::join`)
        S: Into<Cow<'a, str>>,
    {
        let values: Vec<_> = iter.map(Into::into).collect();

        // Avoid inserting if there are no metadata entries
        if !values.is_empty() {
            metadata.insert(key.to_string(), values.join(", "));
        }
    }

    let source = epub.metadata();
    let mut metadata = HashMap::new();

    // Insert ungrouped metadata entries
    if let Some(title) = source.title() {
        metadata.insert("title".to_string(), title.value().to_string());
    }
    if let Some(publisher) = source.publishers().next() {
        metadata.insert("publisher".to_string(), publisher.value().to_string());
    }
    if let Some(description) = source.descriptions().next() {
        metadata.insert("description".to_string(), description.value().to_string());
    }

    // Insert grouped metadata entries
    insert_joined(
        "creator",
        &mut metadata,
        source.creators().map(|creator| creator.value()),
    );
    insert_joined(
        "contributor",
        &mut metadata,
        source.contributors().map(|contributor| contributor.value()),
    );
    insert_joined(
        "language",
        &mut metadata,
        source.languages().map(|language| language.value()),
    );
    insert_joined(
        "identifier",
        &mut metadata,
        source.identifiers().map(|id| match id.scheme() {
            Some(scheme) => Cow::Owned(format!("{scheme:?}: {}", id.value())),
            None => Cow::Borrowed(id.value()),
        }),
    );
    insert_joined("tags", &mut metadata, source.tags().map(|tag| tag.value()));

    metadata
}

/// Try to extract cover image from an ebook document using rbook
fn try_extract_cover_data(epub: &rbook::Epub) -> Option<ImageSource<'static>> {
    let cover_entry = epub.manifest().cover_image()?;
    let raw_bytes = cover_entry.read_bytes().ok()?;

    // Generate a unique id for the ImageSource URI
    let cover_texture_id = epub.metadata().identifiers().next().map_or_else(
        || {
            let now = std::time::SystemTime::now();
            let datetime: chrono::DateTime<chrono::Utc> = now.into();
            datetime.format("%Y-%m-%d_%H:%M:%S").to_string()
        },
        |id| id.value().to_string(),
    );

    Some(ImageSource::Bytes {
        uri: format!("bytes://epub_cover_{cover_texture_id}").into(),
        bytes: raw_bytes.into(),
    })
}

fn cover_image_to_png(raw_cover_image: &[u8], png_buffer: &mut Vec<u8>) -> image::ImageResult<()> {
    let image = image::load_from_memory(raw_cover_image)?;
    let mut cursor = std::io::Cursor::new(png_buffer);

    image.write_to(&mut cursor, image::ImageFormat::Png)
}
