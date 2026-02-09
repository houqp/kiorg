//! Preview popup module for displaying file previews in a popup window

use egui::Context;
use std::sync::{Arc, Mutex};

use tracing::error;

use crate::app::Kiorg;
use crate::config::colors::AppColors;
use crate::models::preview_content::PreviewContent;
use crate::ui::file_list::truncate_text;
use crate::ui::popup::PopupApp;
use crate::ui::popup::PopupType;
use crate::ui::popup::window_utils::new_center_popup_window;
use crate::ui::preview::loading::create_load_popup_meta_task;

fn available_screen_width(ctx: &Context) -> f32 {
    let screen_width = ctx.content_rect().width();
    let pixels_per_point = ctx.pixels_per_point();
    screen_width * pixels_per_point
}

/// Handle the `ShowFilePreview` shortcut action
/// This function was extracted from input.rs to reduce complexity
pub fn handle_show_file_popup(app: &mut Kiorg, ctx: &egui::Context) {
    // Store path and extension information before borrowing app mutably
    let (is_dir, entry, extension) = {
        let tab = app.tab_manager.current_tab_ref();
        if let Some(selected_entry) = tab.selected_entry() {
            (
                selected_entry.is_dir,
                selected_entry.clone(),
                crate::ui::preview::path_to_ext_info(&selected_entry.meta.path),
            )
        } else {
            // No entry selected
            return;
        }
    };
    let path = &entry.meta.path;

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
        let filename = path_buf
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Plugin".to_string());
        let ctx_clone = ctx.clone();

        let available_width = available_screen_width(ctx);
        let (rx, cancel_sender) = create_load_popup_meta_task(entry.meta.clone(), move |entry| {
            let result = plugin.preview_popup(&entry.path.to_string_lossy(), available_width);
            match result {
                Ok(plugin_content) => {
                    let content =
                        PreviewContent::plugin_preview_from_components(plugin_content, &ctx_clone);
                    // Extract components from PreviewContent
                    match content {
                        PreviewContent::PluginPreview { components } => {
                            Ok(crate::ui::popup::plugin_viewer::PluginContent {
                                filename,
                                components,
                            })
                        }
                        _ => Err("Unexpected content type for Plugin viewer".into()),
                    }
                }
                Err(e) => Err(format!("Plugin error: {}", e)),
            }
        });
        app.show_popup = Some(PopupType::Plugin(Box::new(PopupApp::loading(
            path_buf,
            rx,
            cancel_sender,
        ))));
        return;
    }

    // Handle different file types based on extension
    match extension.as_str() {
        crate::ui::preview::pdf_extensions!() => {
            // Not loaded or different type, start a new high-DPI load for PdfViewer
            let ctx_clone = ctx.clone();
            let path_buf = path.to_path_buf();
            let (rx, cancel_sender) =
                create_load_popup_meta_task(entry.meta.clone(), move |entry| {
                    let (mut meta, doc) =
                        crate::ui::preview::pdf::extract_pdf_metadata(entry, &ctx_clone)?;
                    let doc_arc = Arc::new(Mutex::new(doc));
                    // Upgrade to high DPI for the popup
                    {
                        let doc_lock = doc_arc.lock().map_err(|_| "Failed to lock PDF doc")?;
                        let rendered = crate::ui::preview::pdf::render_pdf_page_high_dpi(
                            &doc_lock,
                            0,
                            Some(&meta.file_id),
                            &ctx_clone,
                        )?;
                        meta.cover = rendered.img_source;
                        meta._texture_handle = Some(rendered.texture_handle);
                    }
                    Ok(crate::ui::popup::pdf_viewer::PdfViewerContent { meta, doc: doc_arc })
                });
            app.show_popup = Some(PopupType::Pdf(Box::new(PopupApp::loading(
                path_buf,
                rx,
                cancel_sender,
            ))));
        }
        crate::ui::preview::epub_extensions!() => {
            let path_buf = path.to_path_buf();
            let (rx, cancel_sender) = create_load_popup_meta_task(entry.meta.clone(), |entry| {
                crate::ui::preview::ebook::extract_ebook_metadata(entry)
            });
            app.show_popup = Some(PopupType::Ebook(Box::new(PopupApp::loading(
                path_buf,
                rx,
                cancel_sender,
            ))));
        }
        crate::ui::preview::image_extensions!() => {
            let path_buf = path.to_path_buf();
            let ctx_clone = ctx.clone();
            let available_width = available_screen_width(ctx);
            let (rx, cancel_sender) =
                create_load_popup_meta_task(entry.meta.clone(), move |entry| {
                    crate::ui::preview::image::read_image_with_metadata(
                        entry,
                        &ctx_clone,
                        Some(available_width),
                    )
                });
            app.show_popup = Some(PopupType::Image(Box::new(PopupApp::loading(
                path_buf,
                rx,
                cancel_sender,
            ))));
        }
        crate::ui::preview::zip_extensions!() | crate::ui::preview::tar_extensions!() => {
            app.show_popup = Some(PopupType::Preview);
        }
        crate::ui::preview::video_extensions!() => {
            let path_buf = path.to_path_buf();
            let ctx_clone = ctx.clone();
            let available_width = available_screen_width(ctx);
            let (rx, cancel_sender) =
                create_load_popup_meta_task(entry.meta.clone(), move |entry| {
                    crate::ui::preview::video::read_video_with_metadata(
                        entry,
                        &ctx_clone,
                        Some(available_width),
                    )
                });
            app.show_popup = Some(PopupType::Video(Box::new(PopupApp::loading(
                path_buf,
                rx,
                cancel_sender,
            ))));
        }
        v => {
            if let Some(syntax) = crate::ui::preview::text::find_syntax_from_path(path) {
                match crate::ui::preview::text::load_full_text(path, Some(syntax.name.as_str())) {
                    Ok(content) => {
                        app.preview_content = Some(content);
                        app.show_popup = Some(PopupType::Preview);
                    }
                    Err(_) => {
                        app.toasts.error("Failed to load text content for preview.");
                    }
                }
            } else {
                app.toasts
                    .error(format!("Preview not implemented for file type: {v}."));
            }
        }
    }
}

pub fn close_popup(app: &mut Kiorg) {
    app.show_popup = None;
}

/// Shows the generic preview popup for the currently selected file
pub fn draw(ctx: &Context, app: &mut Kiorg) {
    if !matches!(app.show_popup, Some(PopupType::Preview)) {
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
            // Calculate available space in the popup
            let available_width = ui.available_width();
            let available_height = ui.available_height();

            if let Some(content) = &mut app.preview_content {
                render_content(ui, content, &app.colors, available_width, available_height);
            } else {
                ui.vertical_centered(|ui| {
                    ui.label("No preview content available");
                });
            }
        });

    if !keep_open {
        close_popup(app);
    }
}

fn render_content(
    ui: &mut egui::Ui,
    content: &mut PreviewContent,
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
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
                            .text_color(colors.fg)
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
            crate::ui::popup::image_viewer::render_popup(
                ui,
                image_meta,
                available_width,
                available_height,
            );
        }
        PreviewContent::Video(_) | PreviewContent::Pdf(_) => {
            error!("Video and PDF should be rendered through their respective viewers");
        }
        PreviewContent::Ebook(ebook_meta) => {
            crate::ui::popup::ebook_viewer::render_popup(
                ui,
                ebook_meta,
                colors,
                available_width,
                available_height,
            );
        }
        PreviewContent::Zip(zip_entries) => {
            egui::ScrollArea::vertical()
                .id_salt("zip_popup_scroll")
                .show(ui, |ui| {
                    crate::ui::preview::zip::render(ui, zip_entries, colors);
                });
        }
        PreviewContent::Tar(tar_entries) => {
            egui::ScrollArea::vertical()
                .id_salt("tar_popup_scroll")
                .show(ui, |ui| {
                    crate::ui::preview::tar::render(ui, tar_entries, colors);
                });
        }
        PreviewContent::PluginPreview { components } => {
            crate::ui::preview::plugin::render(
                ui,
                components,
                colors,
                available_width,
                available_height,
            );
        }
        PreviewContent::Loading { path, .. } => {
            render_loading(ui, path, colors);
        }
        // For other file types
        _ => {
            ui.vertical_centered(|ui| {
                ui.label("Preview not implemented for this file type yet.");
            });
        }
    }
}

pub fn render_loading(ui: &mut egui::Ui, path: &std::path::Path, colors: &AppColors) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        ui.spinner();
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new(format!(
                "Loading preview contents for {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ))
            .color(colors.fg),
        );
        ui.add_space(20.0);
    });
}

pub fn render_error(ui: &mut egui::Ui, error: &str, _colors: &AppColors) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        ui.label(egui::RichText::new(format!("Error: {error}")).color(egui::Color32::RED));
        ui.add_space(20.0);
    });
}
