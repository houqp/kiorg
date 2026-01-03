use crate::config::colors::AppColors;
use crate::models::preview_content::RenderedComponent;
use crate::ui::file_list::truncate_text;
use crate::ui::popup::window_utils::new_center_popup_window;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};

/// Type alias for plugin components receiver
pub type PluginComponentsReceiver = Arc<Mutex<mpsc::Receiver<Result<PluginContent, String>>>>;

/// Content loaded by the plugin viewer
#[derive(Debug)]
pub struct PluginContent {
    pub filename: String,
    pub components: Vec<RenderedComponent>,
}

/// Dedicated state for the Plugin viewer app
#[derive(Debug)]
pub enum PluginViewer {
    Loading(
        String,
        PathBuf,
        PluginComponentsReceiver,
        std::sync::mpsc::Sender<()>,
    ),
    Loaded(PluginContent),
    Error(String, String),
}

impl crate::ui::popup::PopupApp for PluginViewer {
    type Content = PluginContent;

    fn loading(
        path: PathBuf,
        receiver: Arc<Mutex<mpsc::Receiver<Result<Self::Content, String>>>>,
        cancel_sender: mpsc::Sender<()>,
    ) -> Self {
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Plugin".to_string());
        Self::Loading(filename, path, receiver, cancel_sender)
    }

    fn loaded(content: Self::Content) -> Self {
        Self::Loaded(content)
    }

    fn error(message: String) -> Self {
        Self::Error("Plugin".to_string(), message)
    }

    fn as_loading(&self) -> Option<&Arc<Mutex<mpsc::Receiver<Result<Self::Content, String>>>>> {
        match self {
            Self::Loading(_, _, receiver, _) => Some(receiver),
            _ => None,
        }
    }

    fn title(&self) -> String {
        match self {
            Self::Loaded(content) => content.filename.clone(),
            Self::Loading(filename, _, _, _) | Self::Error(filename, _) => filename.clone(),
        }
    }
}

impl PluginViewer {
    pub fn draw(&mut self, ctx: &egui::Context, colors: &AppColors) -> bool {
        let mut keep_open = true;
        let screen_size = ctx.content_rect().size();
        let popup_size = egui::vec2(screen_size.x * 0.9, screen_size.y * 0.9);
        let popup_content_width = popup_size.x * 0.9;

        let window_title = crate::ui::popup::PopupApp::title(self);

        new_center_popup_window(&truncate_text(&window_title, popup_content_width))
            .max_size(popup_size)
            .min_size(popup_size)
            .open(&mut keep_open)
            .show(ctx, |ui| {
                let available_width = ui.available_width();
                let available_height = ui.available_height();

                match self {
                    Self::Loaded(content) => {
                        crate::ui::preview::plugin::render(
                            ui,
                            &content.components,
                            colors,
                            available_width,
                            available_height,
                        );
                    }
                    Self::Loading(_, path, _, _cancel_sender) => {
                        crate::ui::popup::preview::render_loading(ui, path, colors);
                    }
                    Self::Error(_, e) => {
                        crate::ui::popup::preview::render_error(ui, e, colors);
                    }
                }
            });

        keep_open
    }
}
