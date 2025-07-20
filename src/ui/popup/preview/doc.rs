//! Document preview module for popup display (PDF, EPUB)

use crate::config::colors::AppColors;
use crate::models::preview_content::{EpubMeta, PdfMeta};
use egui::{Button, Image, Key, Modifiers, RichText};

/// Generate a consistent input ID for page navigation based on file ID
fn get_page_input_id(file_id: &str) -> egui::Id {
    egui::Id::new(format!("page_input_{file_id}"))
}

/// Render PDF document in popup with page navigation
pub fn render_pdf_popup(
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
                    navigate_to_previous_page(pdf_meta, ui.ctx());
                }

                // Editable page input
                ui.horizontal(|ui| {
                    // Use egui's memory to store the page input text per document
                    let input_id = get_page_input_id(&pdf_meta.file_id);

                    // Get or initialize the input text
                    let mut page_input_text = ui.ctx().data(|d| {
                        d.get_temp::<String>(input_id)
                            .unwrap_or_else(|| (current_page + 1).to_string())
                    });

                    // Update input text if page changed via navigation buttons
                    let expected_text = (current_page + 1).to_string();
                    if !ui.memory(|m| m.has_focus(input_id)) && page_input_text != expected_text {
                        page_input_text = expected_text;
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
                        RichText::new(format!("of {total_pages}"))
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
                    navigate_to_next_page(pdf_meta, ui.ctx());
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
pub fn render_epub_popup(
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

/// Helper function to navigate to the next page in PDF
pub fn navigate_to_next_page(pdf_meta: &mut PdfMeta, ctx: &egui::Context) -> bool {
    let current_page = pdf_meta.current_page;
    let total_pages = pdf_meta.page_count;

    if current_page < total_pages - 1 {
        pdf_meta.current_page += 1;

        // Update the input text in memory
        let input_id = get_page_input_id(&pdf_meta.file_id);
        let new_text = (pdf_meta.current_page + 1).to_string();
        ctx.data_mut(|d| d.insert_temp(input_id, new_text));

        // Re-render the PDF page using cached file
        if let Ok(img_source) = crate::ui::preview::doc::render_pdf_page_high_dpi(
            &pdf_meta.pdf_file,
            pdf_meta.current_page,
            Some(&pdf_meta.file_id),
        ) {
            pdf_meta.cover = img_source;
        }

        ctx.request_repaint();
        true
    } else {
        false
    }
}

/// Helper function to navigate to the previous page in PDF
pub fn navigate_to_previous_page(pdf_meta: &mut PdfMeta, ctx: &egui::Context) -> bool {
    let current_page = pdf_meta.current_page;

    if current_page > 0 {
        pdf_meta.current_page = current_page.saturating_sub(1);

        // Update the input text in memory
        let input_id = get_page_input_id(&pdf_meta.file_id);
        let new_text = (pdf_meta.current_page + 1).to_string();
        ctx.data_mut(|d| d.insert_temp(input_id, new_text));

        // Re-render the PDF page using cached file
        if let Ok(img_source) = crate::ui::preview::doc::render_pdf_page_high_dpi(
            &pdf_meta.pdf_file,
            pdf_meta.current_page,
            Some(&pdf_meta.file_id),
        ) {
            pdf_meta.cover = img_source;
        }

        ctx.request_repaint();
        true
    } else {
        false
    }
}

/// Helper function to render PDF page when navigation buttons are clicked
fn render_pdf_page_for_popup(
    ui: &mut egui::Ui,
    pdf_meta: &mut PdfMeta,
    file_path: &std::path::Path,
) {
    // Generate a unique file ID based on the path
    let file_id = file_path.to_string_lossy().to_string();

    // Render the new page using cached file
    if let Ok(img_source) = crate::ui::preview::doc::render_pdf_page_high_dpi(
        &pdf_meta.pdf_file,
        pdf_meta.current_page,
        Some(&file_id),
    ) {
        // Update the cover with the new page image
        pdf_meta.cover = img_source;
    }
    // Request repaint to show the updated image
    ui.ctx().request_repaint();
}

/// Handle key input events for the PDF preview popup
/// Returns true if the key was handled, false otherwise
pub fn handle_preview_popup_input_pdf(
    pdf_meta: &mut PdfMeta,
    key: Key,
    modifiers: Modifiers,
    ctx: &egui::Context,
) {
    use crate::config::shortcuts::{self, ShortcutAction, shortcuts_helpers};

    // Get shortcuts from config or use defaults
    let shortcuts = shortcuts::get_default_shortcuts();

    // Try to find a matching action for the key combination
    if let Some(action) = shortcuts_helpers::find_action(shortcuts, key, modifiers, false) {
        match action {
            ShortcutAction::PageUp => {
                handle_page_up(pdf_meta, ctx);
            }
            ShortcutAction::PageDown => {
                handle_page_down(pdf_meta, ctx);
            }
            _ => {
                // Other actions are not handled in preview popup
            }
        }
    }
}

/// Handle page up navigation for PDF documents
pub fn handle_page_up(pdf_meta: &mut PdfMeta, ctx: &egui::Context) {
    navigate_to_previous_page(pdf_meta, ctx);
}

/// Handle page down navigation for PDF documents
pub fn handle_page_down(pdf_meta: &mut PdfMeta, ctx: &egui::Context) {
    navigate_to_next_page(pdf_meta, ctx);
}
