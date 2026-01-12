use crate::config::colors::AppColors;
use crate::models::preview_content::VideoMeta;
use crate::ui::file_list::truncate_text;
use crate::ui::popup::window_utils::new_center_popup_window;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};

/// Type alias for video meta receiver
pub type VideoMetaReceiver = Arc<Mutex<mpsc::Receiver<Result<VideoMeta, String>>>>;

/// Dedicated state for the Video viewer app
#[derive(Debug)]
pub enum VideoViewer {
    Loading(PathBuf, VideoMetaReceiver, std::sync::mpsc::Sender<()>),
    Loaded(Box<VideoMeta>),
    Error(String),
}

impl crate::ui::popup::PopupApp for VideoViewer {
    type Content = VideoMeta;

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
        "Video Viewer".to_string()
    }
}

impl VideoViewer {
    pub fn draw(&mut self, ctx: &egui::Context, colors: &AppColors) -> bool {
        let mut keep_open = true;
        let screen_size = ctx.content_rect().size();
        let popup_size = egui::vec2(screen_size.x * 0.9, screen_size.y * 0.9);
        let popup_content_width = popup_size.x * 0.9;

        new_center_popup_window(&truncate_text("Video Viewer", popup_content_width))
            .max_size(popup_size)
            .min_size(popup_size)
            .open(&mut keep_open)
            .show(ctx, |ui| {
                let available_width = ui.available_width();
                let available_height = ui.available_height();

                match self {
                    Self::Loaded(video_meta) => {
                        render_popup(ui, video_meta, available_width, available_height);
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

/// Render video content optimized for popup view
pub fn render_popup(
    ui: &mut egui::Ui,
    video_meta: &VideoMeta,
    available_width: f32,
    available_height: f32,
) {
    crate::ui::preview::image::render_interactive(
        ui,
        &video_meta.thumbnail_image,
        available_width,
        available_height,
    );
}
