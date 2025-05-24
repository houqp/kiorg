//! Document preview module for popup display (PDF, EPUB)

use crate::config::colors::AppColors;
use crate::models::preview_content::{DocMeta, EpubMeta, PdfMeta};
use egui::{Button, Image, RichText};

/// Generate a consistent input ID for page navigation based on file path
fn get_page_input_id(file_path: &std::path::Path) -> egui::Id {
    let file_id = file_path.to_string_lossy().to_string();
    egui::Id::new(format!("page_input_{}", file_id))
}

/// Render document content optimized for popup view
///
/// This version focuses on displaying the document cover/first page at a large size
/// without detailed metadata tables. Only shows page navigation for PDF documents.
pub fn render_popup(
    ui: &mut egui::Ui,
    doc_meta: &mut DocMeta,
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
    file_path: &std::path::Path,
) {
    match doc_meta {
        DocMeta::Pdf(pdf_meta) => {
            render_pdf_popup(
                ui,
                pdf_meta,
                colors,
                available_width,
                available_height,
                file_path,
            );
        }
        DocMeta::Epub(epub_meta) => {
            render_epub_popup(ui, epub_meta, colors, available_width, available_height);
        }
    }
}

/// Render PDF document in popup with page navigation
fn render_pdf_popup(
    ui: &mut egui::Ui,
    pdf_meta: &mut PdfMeta,
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
    file_path: &std::path::Path,
) {
    // Get current page and total pages
    let current_page = pdf_meta.current_page;
    let total_pages = pdf_meta.page_count;

    ui.vertical_centered(|ui| {
        // Create a constrained horizontal container that only takes the space it needs
        ui.allocate_ui_with_layout(
            egui::vec2(200.0, 30.0), // Fixed width container for the navigation controls
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                // Previous page button (left arrow)
                if ui
                    .add_enabled(
                        current_page > 0,
                        Button::new(RichText::new("▲").size(16.0).color(colors.fg))
                            .min_size(egui::vec2(24.0, 24.0)),
                    )
                    .clicked()
                    && current_page > 0
                {
                    pdf_meta.current_page = current_page.saturating_sub(1);
                    // Update the input text in memory
                    let input_id = get_page_input_id(file_path);
                    let new_text = (pdf_meta.current_page + 1).to_string();
                    ui.ctx().data_mut(|d| d.insert_temp(input_id, new_text));
                    // Trigger PDF re-rendering for the new page
                    render_pdf_page_for_popup(ui, pdf_meta, file_path);
                }

                // Editable page input
                ui.horizontal(|ui| {
                    // Use egui's memory to store the page input text per document
                    let input_id = get_page_input_id(file_path);

                    // Get or initialize the input text
                    let mut page_input_text = ui.ctx().data(|d| {
                        d.get_temp::<String>(input_id)
                            .unwrap_or_else(|| (current_page + 1).to_string())
                    });

                    // Update input text if page changed via navigation buttons
                    let expected_text = (current_page + 1).to_string();
                    if !ui.memory(|m| m.has_focus(input_id)) && page_input_text != expected_text {
                        page_input_text = expected_text.clone();
                        ui.ctx()
                            .data_mut(|d| d.insert_temp(input_id, page_input_text.clone()));
                    }

                    // Input field for current page number
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut page_input_text)
                            .id(input_id)
                            .desired_width(40.0)
                            .clip_text(false),
                    );

                    // Store the updated text
                    ui.ctx()
                        .data_mut(|d| d.insert_temp(input_id, page_input_text.clone()));

                    // Handle Enter key to jump to page
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Ok(new_page) = page_input_text.parse::<usize>() {
                            if new_page >= 1 && new_page <= total_pages {
                                pdf_meta.current_page = new_page - 1; // Convert to 0-based
                                render_pdf_page_for_popup(ui, pdf_meta, file_path);
                            } else {
                                // Invalid page number, reset to current page
                                let reset_text = (current_page + 1).to_string();
                                ui.ctx().data_mut(|d| d.insert_temp(input_id, reset_text));
                            }
                        } else {
                            // Invalid input, reset to current page
                            let reset_text = (current_page + 1).to_string();
                            ui.ctx().data_mut(|d| d.insert_temp(input_id, reset_text));
                        }
                    }

                    // Label showing "of X"
                    ui.label(
                        RichText::new(format!("of {}", total_pages))
                            .color(colors.fg)
                            .size(14.0),
                    );
                });

                // Next page button (right arrow)
                if ui
                    .add_enabled(
                        current_page < total_pages - 1,
                        Button::new(RichText::new("▼").size(16.0).color(colors.fg))
                            .min_size(egui::vec2(24.0, 24.0)),
                    )
                    .clicked()
                    && current_page < total_pages - 1
                {
                    pdf_meta.current_page += 1;
                    // Update the input text in memory
                    let input_id = get_page_input_id(file_path);
                    let new_text = (pdf_meta.current_page + 1).to_string();
                    ui.ctx().data_mut(|d| d.insert_temp(input_id, new_text));
                    // Trigger PDF re-rendering for the new page
                    render_pdf_page_for_popup(ui, pdf_meta, file_path);
                }
            },
        );
    });

    // Add a small space after the navigation bar
    ui.add_space(5.0);

    // Display cover image (using most of the available space)
    // Calculate available space for the document, accounting for the navigation bar
    let nav_bar_height = 30.0; // Navigation bar + spacing
    let max_height = available_height - nav_bar_height;
    let max_width = available_width * 0.95;

    // Add the document preview with maximum possible size
    ui.add_sized(
        egui::vec2(max_width, max_height),
        Image::new(pdf_meta.cover.clone()).maintain_aspect_ratio(true),
    );
}

/// Render EPUB document in popup without page navigation
fn render_epub_popup(
    ui: &mut egui::Ui,
    epub_meta: &EpubMeta,
    _colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    ui.vertical_centered(|ui| {
        // Add a small space at the top
        ui.add_space(10.0);

        // For EPUB, just display the cover without any page navigation controls or page count
        // Use maximum available space since there's no navigation bar or page count display
        let max_height = available_height * 0.95;
        let max_width = available_width * 0.95;

        // Add the document preview with maximum possible size
        ui.add_sized(
            egui::vec2(max_width, max_height),
            Image::new(epub_meta.cover.clone()).maintain_aspect_ratio(true),
        );
    });
}

/// Helper function to render PDF page when navigation buttons are clicked
fn render_pdf_page_for_popup(
    ui: &mut egui::Ui,
    pdf_meta: &mut PdfMeta,
    file_path: &std::path::Path,
) {
    // Open the PDF file
    if let Ok(pdf_file) = pdf::file::FileOptions::uncached().open(file_path) {
        // Generate a unique file ID based on the path
        let file_id = file_path.to_string_lossy().to_string();

        // Render the new page
        if let Ok(img_source) = crate::ui::preview::doc::render_pdf_page_high_dpi(
            &pdf_file,
            pdf_meta.current_page,
            Some(&file_id),
        ) {
            // Update the cover with the new page image
            pdf_meta.cover = img_source;
        }
    }
    // Request repaint to show the updated image
    ui.ctx().request_repaint();
}
