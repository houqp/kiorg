use crate::config::colors::AppColors;
use crate::models::preview_content::PdfMeta;
use crate::ui::file_list::truncate_text;
use crate::ui::popup::window_utils::new_center_popup_window;
use egui::{Button, Key, Modifiers, RichText};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
use tracing::error;

/// Type alias for PDF meta receiver
pub type PdfMetaReceiver = Arc<Mutex<mpsc::Receiver<Result<PdfViewerContent, String>>>>;

/// Content for the PDF viewer, owning the document handle
pub struct PdfViewerContent {
    pub meta: PdfMeta,
    pub doc: Arc<Mutex<pdfium_bind::PdfDocument>>,
}

impl std::fmt::Debug for PdfViewerContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PdfViewerContent")
            .field("meta", &self.meta)
            .field("doc", &"<PdfDocument>")
            .finish()
    }
}

impl PdfViewerContent {
    pub fn render_page(&mut self, ctx: &egui::Context) -> Result<(), String> {
        let doc_lock = self.doc.lock().map_err(|_| "Failed to lock PDF doc")?;
        let rendered = crate::ui::preview::pdf::render_pdf_page_high_dpi(
            &doc_lock,
            self.meta.current_page,
            Some(&self.meta.file_id),
            ctx,
        )?;

        self.meta.cover = rendered.img_source;
        self.meta._texture_handle = Some(rendered.texture_handle);
        Ok(())
    }

    pub fn update_page_num_text(&self, ctx: &egui::Context) {
        let input_id = self.meta.page_input_id();
        // in the UI, we display the first page as 1 instead of 0
        let new_text = (self.meta.current_page + 1).to_string();
        ctx.data_mut(|d| d.insert_temp(input_id, new_text));
    }
}

/// Dedicated state for the PDF viewer app
#[derive(Debug)]
pub enum PdfViewer {
    Loading(PathBuf, PdfMetaReceiver, std::sync::mpsc::Sender<()>),
    Loaded(PdfViewerContent),
    Error(String),
}

impl crate::ui::popup::PopupApp for PdfViewer {
    type Content = PdfViewerContent;

    fn loading(
        path: PathBuf,
        receiver: Arc<Mutex<mpsc::Receiver<Result<Self::Content, String>>>>,
        cancel_sender: mpsc::Sender<()>,
    ) -> Self {
        Self::Loading(path, receiver, cancel_sender)
    }

    fn loaded(content: Self::Content) -> Self {
        Self::Loaded(content)
    }

    fn error(message: String) -> Self {
        Self::Error(message)
    }

    fn as_loading(&self) -> Option<&Arc<Mutex<mpsc::Receiver<Result<Self::Content, String>>>>> {
        match self {
            Self::Loading(_, receiver, _) => Some(receiver),
            _ => None,
        }
    }

    fn title(&self) -> String {
        "PDF Viewer".to_string()
    }
}

impl PdfViewer {
    pub fn draw(&mut self, ctx: &egui::Context, colors: &AppColors) -> bool {
        let mut keep_open = true;
        let screen_size = ctx.content_rect().size();
        let popup_size = egui::vec2(screen_size.x * 0.9, screen_size.y * 0.9);
        let popup_content_width = popup_size.x * 0.9;

        new_center_popup_window(&truncate_text("PDF Viewer", popup_content_width))
            .max_size(popup_size)
            .min_size(popup_size)
            .open(&mut keep_open)
            .show(ctx, |ui| {
                let available_width = ui.available_width();
                let available_height = ui.available_height();

                match self {
                    Self::Loaded(pdf_meta) => {
                        render_popup(ui, pdf_meta, colors, available_width, available_height);
                    }
                    Self::Loading(path, _, _) => {
                        crate::ui::popup::preview::render_loading(ui, path, colors);
                    }
                    Self::Error(e) => {
                        crate::ui::popup::preview::render_error(ui, e, colors);
                    }
                }
            });

        keep_open
    }
}

/// Render PDF in popup with page navigation
pub fn render_popup(
    ui: &mut egui::Ui,
    viewer_content: &mut PdfViewerContent,
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    // Get current page and total pages
    let current_page = viewer_content.meta.current_page;
    let total_pages = viewer_content.meta.page_count;

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
                    navigate_to_previous_page(viewer_content, ui.ctx());
                }

                // Editable page input
                ui.horizontal(|ui| {
                    // Use egui's memory to store the page input text per PDF
                    let input_id = viewer_content.meta.page_input_id();

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
                        if let Ok(new_page) = page_input_text.parse::<isize>() {
                            if new_page >= 1 && new_page <= total_pages {
                                viewer_content.meta.current_page = new_page - 1; // Convert to 0-based
                                if let Err(e) = render_pdf_page_for_popup(ui, viewer_content) {
                                    error!("Error rendering PDF page: {}", e);
                                }
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
                    navigate_to_next_page(viewer_content, ui.ctx());
                }
            },
        );
    });

    // Add a small space after the navigation bar
    ui.add_space(5.0);

    // Display cover image (using most of the available space)
    // Calculate available space for the PDF, accounting for the navigation bar
    let nav_bar_height = 30.0; // Navigation bar + spacing
    let max_height = available_height - nav_bar_height;
    let max_width = available_width * 0.95;

    // Add the PDF preview with maximum possible size
    ui.add_sized(
        egui::vec2(max_width, max_height),
        egui::Image::new(viewer_content.meta.cover.clone())
            .max_size(egui::vec2(max_width, max_height))
            .maintain_aspect_ratio(true),
    );
}

/// Helper function to navigate to the next page in PDF
pub fn navigate_to_next_page(viewer_content: &mut PdfViewerContent, ctx: &egui::Context) {
    let current_page = viewer_content.meta.current_page;
    let total_pages = viewer_content.meta.page_count;
    if current_page >= total_pages - 1 {
        return;
    }
    viewer_content.meta.current_page += 1;
    viewer_content.update_page_num_text(ctx);
    if let Err(e) = viewer_content.render_page(ctx) {
        error!("Error rendering PDF page: {}", e);
        return;
    }
    ctx.request_repaint();
}

/// Helper function to navigate to the previous page in PDF
pub fn navigate_to_previous_page(viewer_content: &mut PdfViewerContent, ctx: &egui::Context) {
    let current_page = viewer_content.meta.current_page;
    if current_page <= 0 {
        return;
    }
    viewer_content.meta.current_page = (current_page - 1).max(0);
    viewer_content.update_page_num_text(ctx);
    if let Err(e) = viewer_content.render_page(ctx) {
        error!("Error rendering PDF page: {}", e);
        return;
    }
    ctx.request_repaint();
}

/// Helper function to render PDF page when navigation buttons are clicked
fn render_pdf_page_for_popup(
    ui: &mut egui::Ui,
    viewer_content: &mut PdfViewerContent,
) -> Result<(), String> {
    let ctx = ui.ctx();
    viewer_content.render_page(ctx)?;
    ctx.request_repaint();
    Ok(())
}

/// Handle key input events for the PDF preview popup
/// Returns true if the key was handled, false otherwise
pub fn handle_preview_popup_input_pdf(
    viewer_content: &mut PdfViewerContent,
    key: Key,
    modifiers: Modifiers,
    ctx: &egui::Context,
) {
    use crate::config::shortcuts::{self, ShortcutAction, ShortcutKey, TraverseResult};

    let shortcuts = shortcuts::get_default_shortcuts();
    let shortcut_key = ShortcutKey { key, modifiers };
    if let TraverseResult::Action(action) = shortcuts.traverse_tree(&[shortcut_key]) {
        match action {
            ShortcutAction::PageUp => {
                navigate_to_previous_page(viewer_content, ctx);
            }
            ShortcutAction::PageDown => {
                navigate_to_next_page(viewer_content, ctx);
            }
            _ => {
                // Other actions are not handled in preview popup
            }
        }
    }
}
