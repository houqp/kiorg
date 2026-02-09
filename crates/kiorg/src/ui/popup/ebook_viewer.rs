use crate::config::colors::AppColors;
use crate::models::preview_content::EbookMeta;
use crate::ui::file_list::truncate_text;
use crate::ui::popup::window_utils::new_center_popup_window;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};

/// Type alias for Ebook meta receiver
pub type EbookMetaReceiver = Arc<Mutex<mpsc::Receiver<Result<EbookMeta, String>>>>;

/// Dedicated state for the Ebook viewer app (EPUB, etc)
#[derive(Debug)]
pub enum EbookViewer {
    Loading(PathBuf, EbookMetaReceiver, std::sync::mpsc::Sender<()>),
    Loaded(EbookMeta),
    Error(String),
}

impl crate::ui::popup::PopupApp for EbookViewer {
    type Content = EbookMeta;

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
        "Ebook Viewer".to_string()
    }
}

impl EbookViewer {
    pub fn draw(&mut self, ctx: &egui::Context, colors: &AppColors) -> bool {
        let mut keep_open = true;
        let screen_size = ctx.content_rect().size();
        let popup_size = egui::vec2(screen_size.x * 0.9, screen_size.y * 0.9);
        let popup_content_width = popup_size.x * 0.9;

        new_center_popup_window(&truncate_text("Ebook Viewer", popup_content_width))
            .max_size(popup_size)
            .min_size(popup_size)
            .open(&mut keep_open)
            .show(ctx, |ui| {
                let available_width = ui.available_width();
                let available_height = ui.available_height();

                match self {
                    Self::Loaded(epub_meta) => {
                        render_popup(ui, epub_meta, colors, available_width, available_height);
                    }
                    Self::Loading(path, _, _cancel_sender) => {
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

/// Render Ebook in popup without page navigation
pub fn render_popup(
    ui: &mut egui::Ui,
    ebook_meta: &EbookMeta,
    _colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    ui.vertical_centered(|ui| {
        // Add a small space at the top
        ui.add_space(10.0);

        // For Ebook, just display the cover without any page navigation controls or page count
        // Use maximum available space since there's no navigation bar or page count display
        let max_height = available_height * 0.95;
        let max_width = available_width * 0.95;

        // Add the ebook preview with maximum possible size
        ui.add_sized(
            egui::vec2(max_width, max_height),
            egui::Image::new(ebook_meta.cover.clone()).maintain_aspect_ratio(true),
        );
    });
}
