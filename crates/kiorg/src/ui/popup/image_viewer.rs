use crate::config::colors::AppColors;
use crate::models::preview_content::ImageMeta;
use crate::ui::file_list::truncate_text;
use crate::ui::popup::window_utils::new_center_popup_window;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};

/// Type alias for image meta receiver
pub type ImageMetaReceiver = Arc<Mutex<mpsc::Receiver<Result<ImageMeta, String>>>>;

/// Dedicated state for the Image viewer app
#[derive(Debug)]
pub enum ImageViewer {
    Loading(PathBuf, ImageMetaReceiver, std::sync::mpsc::Sender<()>),
    Loaded(Box<ImageMeta>),
    Error(String),
}

impl crate::ui::popup::PopupApp for ImageViewer {
    type Content = ImageMeta;

    fn loading(
        path: PathBuf,
        receiver: Arc<Mutex<mpsc::Receiver<Result<Self::Content, String>>>>,
        cancel_sender: mpsc::Sender<()>,
    ) -> Self {
        Self::Loading(path, receiver, cancel_sender)
    }

    fn loaded(content: Self::Content) -> Self {
        Self::Loaded(Box::new(content))
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
        "Image Viewer".to_string()
    }
}

impl ImageViewer {
    pub fn draw(&mut self, ctx: &egui::Context, colors: &AppColors) -> bool {
        let mut keep_open = true;
        let screen_size = ctx.content_rect().size();
        let popup_size = egui::vec2(screen_size.x * 0.9, screen_size.y * 0.9);
        let popup_content_width = popup_size.x * 0.9;

        new_center_popup_window(&truncate_text("Image Viewer", popup_content_width))
            .max_size(popup_size)
            .min_size(popup_size)
            .open(&mut keep_open)
            .show(ctx, |ui| {
                let available_width = ui.available_width();
                let available_height = ui.available_height();

                match self {
                    Self::Loaded(image_meta) => {
                        render_popup(ui, image_meta, available_width, available_height);
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

/// Render image content optimized for popup view
///
/// This version focuses on displaying the image at a large size without metadata tables
pub fn render_popup(
    ui: &mut egui::Ui,
    image_meta: &ImageMeta,
    available_width: f32,
    available_height: f32,
) {
    crate::ui::preview::image::render_interactive(
        ui,
        &image_meta.image,
        available_width,
        available_height,
    );
}
