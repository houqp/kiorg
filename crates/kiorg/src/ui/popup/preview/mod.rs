//! Preview popup module for displaying file previews in a popup window

use crate::app::Kiorg;
use crate::models::preview_content::PreviewContent;
use crate::ui::file_list::truncate_text;
use crate::ui::popup::PopupType;
use crate::ui::popup::window_utils::new_center_popup_window;
use egui::Context;

pub mod doc;
pub mod image;

/// Handle the `ShowFilePreview` shortcut action
/// This function was extracted from input.rs to reduce complexity
pub fn handle_show_file_preview(app: &mut Kiorg, _ctx: &egui::Context) {
    // Store path and extension information before borrowing app mutably
    let (is_dir, path, extension) = {
        let tab = app.tab_manager.current_tab_ref();
        if let Some(selected_entry) = tab.selected_entry() {
            (
                selected_entry.is_dir,
                &selected_entry.path,
                crate::ui::preview::path_to_ext_info(&selected_entry.path),
            )
        } else {
            // No entry selected
            return;
        }
    };

    if is_dir {
        // Show preview popup for directories
        app.show_popup = Some(PopupType::Preview);
        return;
    }

    // First check if any plugins can handle this file
    let plugin_result = if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        app.plugin_manager.get_preview_plugin_for_file(file_name)
    } else {
        None
    };
    if let Some(plugin) = plugin_result {
        // Trigger a fresh load specifically for the popup using the PreviewPopup command
        let path_buf = path.to_path_buf();
        let ctx_clone = _ctx.clone();
        crate::ui::preview::loading::load_preview_async(app, path_buf, move |path| {
            let result = plugin.preview_popup(&path.to_string_lossy());
            match result {
                Ok(plugin_content) => Ok(PreviewContent::plugin_preview_from_components(
                    plugin_content,
                    &ctx_clone,
                )),
                Err(e) => Ok(PreviewContent::text(format!("Plugin error: {}", e))),
            }
        });
        app.show_popup = Some(PopupType::Preview);
        return;
    }

    // Handle different file types based on extension
    match extension.as_str() {
        crate::ui::preview::pdf_extensions!() => {
            // Get the current selected path
            let selected_path = {
                let tab = app.tab_manager.current_tab_ref();
                tab.selected_entry().map(|entry| entry.path.clone())
            };

            if let Some(path) = selected_path {
                // We can assume preview_content will always be Pdf due to right panel loading
                if let Some(PreviewContent::Pdf(ref mut pdf_meta)) = app.preview_content {
                    // We already have pdf meta with correct metadata, just update the cover with high DPI
                    // Generate a unique file ID based on the path
                    let file_id = path.to_string_lossy().to_string();

                    match crate::ui::preview::doc::render_pdf_page_high_dpi(
                        &pdf_meta.pdf_file.lock().unwrap(),
                        pdf_meta.current_page, // Use current page from existing meta
                        Some(&file_id),
                        _ctx,
                    ) {
                        Ok((img_source, texture_handle)) => {
                            // Update the cover with high DPI version
                            pdf_meta.cover = img_source;
                            pdf_meta._texture_handle = Some(texture_handle);

                            // Show preview popup after successful rendering
                            app.show_popup = Some(PopupType::Preview);
                        }
                        Err(_) => {
                            // If error rendering, don't show popup
                        }
                    }
                } else {
                    // For EPUB or other doc types, just show the popup directly
                    app.show_popup = Some(PopupType::Preview);
                }
            }
        }
        crate::ui::preview::epub_extensions!() => {
            // Show preview popup for EPUB files
            app.show_popup = Some(PopupType::Preview);
        }
        crate::ui::preview::zip_extensions!() => {
            // Show preview popup for zip files
            app.show_popup = Some(PopupType::Preview);
        }
        crate::ui::preview::tar_extensions!() => {
            // Show preview popup for tar files
            app.show_popup = Some(PopupType::Preview);
        }
        crate::ui::preview::image_extensions!() => {
            // Show preview popup for image files
            app.show_popup = Some(PopupType::Preview);
        }
        v => {
            if let Some(syntax) = crate::ui::preview::text::find_syntax_from_path(path) {
                match crate::ui::preview::text::load_full_text(path, Some(syntax.name.as_str())) {
                    Ok(preview_content) => {
                        app.preview_content = Some(preview_content);
                        app.show_popup = Some(PopupType::Preview);
                    }
                    Err(_) => {
                        app.toasts.error("Failed to load text content for preview.");
                    }
                }
            } else {
                // send notification for unsupported file types
                app.toasts
                    .error(format!("Preview not implemented for file type: {v}."));
            }
        }
    }
}

pub fn close_popup(app: &mut Kiorg) {
    app.show_popup = None;
    // For plugins, we need to clear the content/cache because the popup loads
    // specific content via `preview_popup` that might differ from the
    // standard right panel preview.
    app.preview_content = None;
    app.cached_preview_path = None;
    // Force preview update in the main loop since we cleared the content
    app.selection_changed = true;
}

/// Shows the preview popup for the currently selected file
pub fn draw(ctx: &Context, app: &mut Kiorg) {
    if app.show_popup != Some(PopupType::Preview) {
        return;
    }

    let mut keep_open = true;
    let screen_size = ctx.content_rect().size();
    let popup_size = egui::vec2(screen_size.x * 0.9, screen_size.y * 0.9);
    let popup_content_width = popup_size.x * 0.9; // Calculate once

    let window_title = {
        let tab = app.tab_manager.current_tab_ref();
        let selected_entry = tab.selected_entry();
        selected_entry.map_or_else(|| "File Preview".to_string(), |entry| entry.name.clone())
    };

    new_center_popup_window(&truncate_text(&window_title, popup_content_width))
        .max_size(popup_size)
        .min_size(popup_size)
        .open(&mut keep_open)
        .show(ctx, |ui| {
            let content = if let Some(content) = &mut app.preview_content {
                content
            } else {
                ui.vertical_centered(|ui| {
                    ui.label("No preview content available");
                });
                return;
            };

            // Calculate available space in the popup
            let available_width = ui.available_width();
            let available_height = ui.available_height();

            // Display the preview content based on its type
            match content {
                PreviewContent::Text(text) => {
                    // Display text with syntax highlighting if it's source code
                    egui::ScrollArea::both()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let mut text_str = text.as_str();
                            ui.add(
                                egui::TextEdit::multiline(&mut text_str)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(0)
                                    .font(egui::TextStyle::Monospace)
                                    .text_color(app.colors.fg)
                                    .interactive(false),
                            );
                        });
                }
                PreviewContent::HighlightedCode { content, language } => {
                    // Display syntax highlighted code with both horizontal and vertical scrolling
                    egui::ScrollArea::both()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            crate::ui::preview::text::render_highlighted(ui, content, language);
                        });
                }
                PreviewContent::Image(image_meta) => {
                    image::render_popup(ui, image_meta, available_width, available_height);
                }
                PreviewContent::Pdf(pdf_meta) => {
                    doc::render_pdf_popup(
                        ui,
                        pdf_meta,
                        &app.colors,
                        available_width,
                        available_height,
                    );
                }
                PreviewContent::Epub(epub_meta) => {
                    doc::render_epub_popup(
                        ui,
                        epub_meta,
                        &app.colors,
                        available_width,
                        available_height,
                    );
                }
                PreviewContent::Zip(zip_entries) => {
                    egui::ScrollArea::vertical()
                        .id_salt("zip_popup_scroll")
                        .show(ui, |ui| {
                            crate::ui::preview::zip::render(ui, zip_entries, &app.colors);
                        });
                }
                PreviewContent::Tar(tar_entries) => {
                    egui::ScrollArea::vertical()
                        .id_salt("tar_popup_scroll")
                        .show(ui, |ui| {
                            crate::ui::preview::tar::render(ui, tar_entries, &app.colors);
                        });
                }
                PreviewContent::PluginPreview { components } => {
                    crate::ui::preview::plugin::render(
                        ui,
                        components,
                        &app.colors,
                        available_width,
                        available_height,
                    );
                }
                PreviewContent::Loading(path, _, _) => {
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
        });

    if !keep_open {
        close_popup(app);
    }
}
