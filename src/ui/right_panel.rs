use egui::{Image, RichText, Ui};
use file_type::FileType;
use pathfinder_geometry::transform2d::Transform2F;
use pdf_render::{render_page, Cache, SceneBackend};

use crate::app::Kiorg;
use crate::models::preview_content::PreviewContent;
use crate::ui::style::{section_title_text, HEADER_ROW_HEIGHT};

const PANEL_SPACING: f32 = 10.0;

/// Draws the right panel (preview).
pub fn draw(app: &mut Kiorg, ctx: &egui::Context, ui: &mut Ui, width: f32, height: f32) {
    // No longer need tab reference since we're using the preview_content enum
    // let tab = app.tab_manager.current_tab_ref();
    let colors = &app.colors;

    // Clone the preview content to avoid borrow issues
    let preview_content = app.preview_content.clone();

    ui.vertical(|ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);
        ui.set_min_height(height);
        ui.set_max_height(height);
        ui.label(section_title_text("Preview", colors));
        ui.separator();

        // Calculate available height for scroll area
        let available_height = height - HEADER_ROW_HEIGHT;

        egui::ScrollArea::vertical()
            .id_salt("preview_scroll")
            .auto_shrink([false; 2])
            .max_height(available_height)
            .show(ui, |ui| {
                // Set the width of the content area
                let scrollbar_width = 6.0;
                ui.set_min_width(width - scrollbar_width);
                ui.set_max_width(width - scrollbar_width);

                let available_width = width - PANEL_SPACING * 2.0;
                let available_height = available_height - PANEL_SPACING * 2.0;

                // Draw preview content based on the enum variant
                match preview_content {
                    Some(PreviewContent::Text(text)) => {
                        ui.label(RichText::new(text).color(colors.fg));
                    }
                    Some(PreviewContent::Image(uri)) => {
                        ui.add(
                            Image::new(uri)
                                .max_size(egui::vec2(available_width, available_height))
                                .maintain_aspect_ratio(true),
                        );
                    }
                    Some(PreviewContent::Doc(doc_meta)) => {
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
                                        .max_size(egui::vec2(
                                            available_width,
                                            available_height * 0.6,
                                        ))
                                        .maintain_aspect_ratio(true),
                                );
                            });
                            ui.add_space(15.0);
                        }

                        // Create a table for metadata
                        egui::Grid::new("doc_metadata_grid")
                            .num_columns(2)
                            .spacing([10.0, 6.0])
                            .striped(true)
                            .show(ui, |ui| {
                                // Sort keys for consistent display
                                let mut sorted_keys: Vec<&String> =
                                    doc_meta.metadata.keys().collect();
                                sorted_keys.sort();

                                // Display each metadata field in a table row
                                for key in sorted_keys {
                                    if let Some(value) = doc_meta.metadata.get(key) {
                                        // Format the key with proper capitalization for display
                                        let display_key = format_metadata_key(key);
                                        ui.label(RichText::new(display_key).color(colors.fg));
                                        ui.add(
                                            egui::Label::new(RichText::new(value).color(colors.fg))
                                                .wrap(),
                                        );
                                        ui.end_row();
                                    }
                                }
                            });
                    }
                    Some(PreviewContent::Zip(entries)) => {
                        // Display zip file contents
                        ui.label(
                            RichText::new("Zip Archive Contents:")
                                .color(colors.fg)
                                .strong(),
                        );
                        ui.add_space(5.0);

                        // Constants for the list
                        // TODO: calculate the correct row height
                        const ROW_HEIGHT: f32 = 10.0;

                        // Get the total number of entries
                        let total_rows = entries.len();

                        // Use show_rows for better performance
                        egui::ScrollArea::vertical()
                            .id_salt("zip_entries_scroll")
                            .auto_shrink([false; 2])
                            .show_rows(ui, ROW_HEIGHT, total_rows, |ui, row_range| {
                                // Set width for the content area
                                let available_width = ui.available_width();
                                ui.set_min_width(available_width);

                                // Display entries in the visible range
                                for row_index in row_range {
                                    let entry = &entries[row_index];
                                    // Create a visual indicator for directories
                                    let entry_text = if entry.is_dir {
                                        RichText::new(format!("📁 {}", entry.name)).strong()
                                    } else {
                                        RichText::new(format!("📄 {}", entry.name))
                                    };

                                    ui.label(entry_text.color(colors.fg));
                                }
                            });
                    }

                    Some(PreviewContent::Loading(path, receiver_opt)) => {
                        // Display loading indicator
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.spinner();
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new(format!(
                                    "Loading preview contents for {}...",
                                    path.file_name().unwrap_or_default().to_string_lossy()
                                ))
                                .color(colors.fg),
                            );
                        });

                        // Check if we have a receiver to poll for results
                        let receiver = match receiver_opt {
                            Some(receiver) => receiver,
                            None => return,
                        };
                        // Try to get a lock on the receiver
                        let receiver = receiver.lock().expect("failed to obtain lock");
                        // Try to receive the result without blocking
                        if let Ok(result) = receiver.try_recv() {
                            // Request a repaint to update the UI with the result
                            ctx.request_repaint();
                            // Update the preview content with the result
                            match result {
                                Ok(content) => {
                                    // Set the preview content directly with the received content
                                    app.preview_content = Some(content);
                                }
                                Err(e) => {
                                    app.preview_content = Some(PreviewContent::text(format!(
                                        "Error loading file: {}",
                                        e
                                    )));
                                }
                            }
                        }
                    }
                    None => {
                        ui.label(RichText::new("No file selected").color(colors.fg));
                    }
                }
            });

        // Draw help text in its own row at the bottom
        ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
            ui.add_space(2.0);
            ui.label(RichText::new("? for help").color(colors.fg_light));
        });
    });
}

/// Read entries from a zip file and return them as a vector of ZipEntry
fn read_zip_entries(
    path: &std::path::Path,
) -> Result<Vec<crate::models::preview_content::ZipEntry>, String> {
    use std::fs::File;
    use zip::ZipArchive;

    // Open the zip file
    let file = File::open(path).map_err(|e| format!("Failed to open zip file: {}", e))?;

    // Create a zip archive from the file
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read zip archive: {}", e))?;

    // Create a vector to store the entries
    let mut entries = Vec::new();

    // Process each file in the archive
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;

        // Create a ZipEntry from the file
        let entry = crate::models::preview_content::ZipEntry {
            name: file.name().to_string(),
            size: file.size(),
            is_dir: file.is_dir(),
        };

        entries.push(entry);
    }

    // Sort entries: directories first, then files, both alphabetically
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    Ok(entries)
}

/// Extract metadata and cover image from an EPUB file and return as PreviewContent
fn read_epub_metadata(path: &std::path::Path) -> Result<PreviewContent, String> {
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

fn render_pdf_page(path: &std::path::Path, page_number: u32) -> Result<PreviewContent, String> {
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

/// Detect file type asynchronously and return a PreviewContent
fn render_generic_file(path: &std::path::Path, size: u64) -> Result<PreviewContent, String> {
    // Try to detect the file type using file_type crate
    let file_type_info = match FileType::try_from_file(path) {
        Ok(file_type) => {
            let media_types = file_type.media_types().join(", ");
            let extensions = file_type.extensions().join(", ");

            if !media_types.is_empty() {
                format!("File type: {} ({})", media_types, extensions)
            } else if !extensions.is_empty() {
                format!("File type: {}", extensions)
            } else {
                "Unknown file type".to_string()
            }
        }
        Err(_) => "Unknown file type".to_string(),
    };

    // Return the PreviewContent directly
    Ok(PreviewContent::text(format!(
        "{}\n\n{}\n\nSize: {} bytes",
        path.file_name().unwrap_or_default().to_string_lossy(),
        file_type_info,
        size
    )))
}

pub fn update_preview_cache(app: &mut Kiorg, _ctx: &egui::Context) {
    let tab = app.tab_manager.current_tab_ref();
    let selected_path = tab.entries.get(tab.selected_index).map(|e| e.path.clone());

    // Check if the selected file is the same as the cached one in app
    if selected_path.as_ref() == app.cached_preview_path.as_ref() {
        return; // Cache hit, no need to update
    }

    // Cache miss, update the preview content in app
    let maybe_entry = selected_path.as_ref().and_then(|p| {
        tab.entries.iter().find(|entry| &entry.path == p).cloned() // Clone the entry data if found
    });
    app.cached_preview_path = selected_path; // Update the cached path in app regardless

    let entry = match maybe_entry {
        Some(entry) => entry,
        None => {
            app.preview_content = None; // No content to display
            app.cached_preview_path = None; // Clear cache in app if no file is selected
            return;
        } // No entry selected, clear the preview content
    };

    if entry.is_dir {
        app.preview_content = Some(PreviewContent::text(format!(
            "Directory: {}",
            entry.path.file_name().unwrap_or_default().to_string_lossy()
        )));
        return;
    }

    let ext = entry
        .path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_else(|| "__unknown__".to_string());

    match ext.as_str() {
        // Image extensions
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" => {
            app.preview_content = Some(PreviewContent::image(entry.path));
        }
        // Zip extensions
        "zip" | "jar" | "war" | "ear" => {
            // Handle zip files asynchronously
            load_preview_async(app, entry.path.clone(), |path| {
                let result = read_zip_entries(&path);
                result.map(PreviewContent::zip)
            });
        }
        // EPUB extension
        "epub" => {
            // Handle EPUB files asynchronously
            load_preview_async(app, entry.path.clone(), |path| read_epub_metadata(&path));
        }
        // PDF extension
        "pdf" => {
            // Handle PDF files asynchronously
            load_preview_async(app, entry.path.clone(), |path| render_pdf_page(&path, 0));
        }
        // All other files
        _ => {
            match std::fs::read_to_string(&entry.path) {
                Ok(content) => {
                    // Only show first 1000 characters for text files
                    let preview_text = content.chars().take(1000).collect::<String>();
                    app.preview_content = Some(PreviewContent::text(preview_text));
                }
                Err(_) => {
                    // For binary files or files that can't be read as text
                    // Handle file type detection asynchronously
                    let path = entry.path.clone();
                    let size = entry.size;

                    load_preview_async(app, path, move |path| render_generic_file(&path, size));
                }
            }
        }
    }
}

/// Helper function to load preview content asynchronously
///
/// This function handles the common pattern of:
/// - Creating a channel for communication
/// - Setting up the loading state with receiver
/// - Spawning a thread to process the file
/// - Sending the result back through the channel
///
/// # Arguments
/// * `app` - The application state
/// * `path` - The path to the file to load
/// * `processor` - A closure that processes the file and returns a Result<PreviewContent, String>
fn load_preview_async<F>(app: &mut Kiorg, path: std::path::PathBuf, processor: F)
where
    F: FnOnce(std::path::PathBuf) -> Result<PreviewContent, String> + Send + 'static,
{
    // Create a channel for communication
    let (sender, receiver) = std::sync::mpsc::channel();

    // Set the initial loading state with the receiver
    app.preview_content = Some(PreviewContent::loading_with_receiver(
        path.clone(),
        receiver,
    ));

    // Spawn a thread to process the file
    std::thread::spawn(move || {
        let preview_result = processor(path);
        let _ = sender.send(preview_result);
    });
}
