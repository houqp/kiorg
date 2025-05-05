use egui::{Image, RichText, Ui};
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
                        ui.centered_and_justified(|ui| {
                            ui.add(
                                Image::new(uri)
                                    .max_size(egui::vec2(available_width, available_height))
                                    .maintain_aspect_ratio(true),
                            );
                        });
                    }
                    Some(PreviewContent::Pdf(img_source)) => {
                        let image = Image::new(img_source)
                            .max_size(egui::vec2(available_width, available_height))
                            .maintain_aspect_ratio(true);
                        ui.centered_and_justified(|ui| {
                            ui.add(image);
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
                                    path.display()
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

fn render_pdf_page(
    path: &std::path::Path,
    page_number: u32,
) -> Result<egui::widgets::ImageSource<'static>, String> {
    let file = pdf::file::FileOptions::uncached()
        .open(path)
        .map_err(|e| format!("Failed to open PDF file: {}", e))?;
    let resolver = file.resolver();
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
        Ok(img_source)
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
            entry.path.display()
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
            let path = entry.path.clone();

            // Create a channel for communication
            let (sender, receiver) = std::sync::mpsc::channel();

            // Set the initial loading state
            app.preview_content = Some(PreviewContent::loading_with_receiver(
                path.clone(),
                receiver,
            ));

            // Spawn a thread to load the zip file
            std::thread::spawn(move || {
                let result = read_zip_entries(&path);
                let preview_result = result.map(PreviewContent::zip);
                let _ = sender.send(preview_result);
            });
        }
        // PDF extension
        "pdf" => {
            // Handle PDF files asynchronously
            let path = entry.path.clone();

            // Create a channel for communication
            let (sender, receiver) = std::sync::mpsc::channel();

            // Set the initial loading state with the receiver
            app.preview_content = Some(PreviewContent::loading_with_receiver(
                path.clone(),
                receiver,
            ));

            std::thread::spawn(move || {
                let preview_result = render_pdf_page(&path, 0).map(PreviewContent::pdf);
                let _ = sender.send(preview_result);
            });
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
                    // For binary files or files that can't be read
                    app.preview_content = Some(PreviewContent::text(format!(
                        "Binary file: {} bytes",
                        entry.size
                    )));
                }
            }
        }
    }
}
