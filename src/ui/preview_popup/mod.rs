//! Preview popup module for displaying file previews in a popup window

use crate::app::{Kiorg, PopupType};
use crate::models::preview_content::{PreviewContent, DocMeta};
use crate::ui::file_list::truncate_text;
use crate::ui::window_utils::new_center_popup_window;
use egui::Context;

pub mod doc;
pub mod image;

/// Handle the ShowFilePreview shortcut action
/// This function was extracted from input.rs to reduce complexity
pub fn handle_show_file_preview(app: &mut Kiorg, _ctx: &egui::Context) {
    // Store path and extension information before borrowing app mutably
    let (is_dir, extension) = {
        let tab = app.tab_manager.current_tab_ref();
        if let Some(selected_entry) = tab.selected_entry() {
            (
                selected_entry.is_dir,
                selected_entry
                    .path
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase()),
            )
        } else {
            // No entry selected
            return;
        }
    };

    if is_dir {
        // TODO: preview directory by storage size?
        return;
    }

    // Handle different file types based on extension
    match extension.as_deref() {
        Some("pdf") => {
            // Get the current selected path
            let selected_path = {
                let tab = app.tab_manager.current_tab_ref();
                tab.selected_entry().map(|entry| entry.path.clone())
            };

            if let Some(path) = selected_path {
                // We can assume preview_content will always be Doc due to right panel loading
                if let Some(PreviewContent::Doc(ref mut existing_doc_meta)) = app.preview_content {
                    // Only handle PDF documents for page navigation updates
                    if let DocMeta::Pdf(ref mut pdf_meta) = existing_doc_meta {
                        // We already have doc meta with correct metadata, just update the cover with high DPI
                        match pdf::file::FileOptions::uncached().open(&path) {
                            Ok(pdf_file) => {
                                // Generate a unique file ID based on the path
                                let file_id = path.to_string_lossy().to_string();

                                match crate::ui::preview::doc::render_pdf_page_high_dpi(
                                    &pdf_file,
                                    pdf_meta.current_page, // Use current page from existing meta
                                    Some(&file_id),
                                ) {
                                    Ok(img_source) => {
                                        // Update the cover with high DPI version
                                        pdf_meta.cover = img_source;

                                        // Show preview popup after successful rendering
                                        app.show_popup = Some(PopupType::Preview);
                                    }
                                    Err(_) => {
                                        // If error rendering, don't show popup
                                    }
                                }
                            }
                            Err(_) => {
                                // If error opening file, don't show popup
                            }
                        }
                    } else {
                        // For EPUB or other doc types, just show the popup directly
                        app.show_popup = Some(PopupType::Preview);
                    }
                }
            }
        }
        Some("epub") => {
            // Show preview popup for EPUB files
            app.show_popup = Some(PopupType::Preview);
        }
        Some("jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg") => {
            // Show preview popup for image files
            app.show_popup = Some(PopupType::Preview);
        }
        _ => {
            // Ignore unsupported file types
        }
    }
}

/// Shows the preview popup for the currently selected file
pub fn show_preview_popup(ctx: &Context, app: &mut Kiorg) {
    // Check if preview popup should be shown
    if let Some(PopupType::Preview) = app.show_popup {
        // Get selected file path for rendering the popup
        let selected_path = {
            let tab = app.tab_manager.current_tab_ref();
            tab.selected_entry().map(|entry| entry.path.clone())
        };

        let mut keep_open = true;
        let screen_size = ctx.screen_rect().size();
        let popup_size = egui::vec2(screen_size.x * 0.9, screen_size.y * 0.9);
        let popup_content_width = popup_size.x * 0.9; // Calculate once

        let window_title = {
            let tab = app.tab_manager.current_tab_ref();
            let selected_entry = tab.selected_entry();
            selected_entry
                .map(|entry| entry.name.clone())
                .unwrap_or_else(|| "File Preview".to_string())
        };

        new_center_popup_window(&truncate_text(&window_title, popup_content_width))
            .max_size(popup_size)
            .min_size(popup_size)
            .open(&mut keep_open)
            .show(ctx, |ui| {
                if let Some(ref mut content) = app.preview_content {
                    // Calculate available space in the popup
                    let available_width = ui.available_width();
                    let available_height = ui.available_height();

                    // Display the preview content based on its type
                    match content {
                        PreviewContent::Image(ref image_meta) => {
                            // Use our specialized popup image renderer
                            image::render_popup(
                                ui,
                                image_meta,
                                &app.colors,
                                available_width,
                                available_height,
                            );
                        }
                        PreviewContent::Doc(ref mut doc_meta) => {
                            // Use specialized PDF/document popup renderer with navigation
                            if let Some(path) = &selected_path {
                                doc::render_popup(
                                    ui,
                                    doc_meta,
                                    &app.colors,
                                    available_width,
                                    available_height,
                                    path,
                                );
                            }
                        }
                        PreviewContent::Loading(path, _) => {
                            ui.vertical_centered(|ui| {
                                ui.add_space(20.0);
                                ui.spinner();
                                ui.add_space(10.0);
                                ui.label(egui::RichText::new(format!(
                                    "Loading preview contents for {}",
                                    path.file_name().unwrap_or_default().to_string_lossy()
                                )));
                                ui.add_space(20.0);
                            });
                        }
                        // For other file types
                        _ => {
                            ui.vertical_centered(|ui| {
                                ui.label("Preview not implemented for this file type yet.");
                            });
                        }
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.label("No preview content available");
                    });
                }
            });

        // Handle popup close
        if !keep_open {
            app.show_popup = None;
        }
    }
}
