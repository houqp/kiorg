use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

/// Type alias for the async preview content receiver
pub type PreviewReceiver = Option<Arc<Mutex<Receiver<Result<PreviewContent, String>>>>>;

/// Represents different types of preview content that can be displayed in the right panel
#[derive(Clone, Debug)]
pub enum PreviewContent {
    /// Text content to be displayed
    Text(String),
    /// Image content with URI to the image file
    Image(String),
    /// Zip file content with a list of entries
    Zip(Vec<ZipEntry>),
    /// PDF content with rendered page images and metadata
    Pdf(egui::widgets::ImageSource<'static>),
    /// EPUB book metadata and optional cover image
    Epub(
        HashMap<String, Vec<String>>,
        Option<egui::widgets::ImageSource<'static>>,
    ),
    /// Loading state with path being loaded and optional receiver for async loading
    Loading(PathBuf, PreviewReceiver),
}

/// Represents an entry in a zip file
#[derive(Clone, Debug)]
pub struct ZipEntry {
    /// Name of the entry (file or directory)
    pub name: String,
    /// Size of the entry in bytes
    pub size: u64,
    /// Whether the entry is a directory
    pub is_dir: bool,
}

impl PreviewContent {
    /// Creates a new text preview content
    pub fn text(content: impl Into<String>) -> Self {
        PreviewContent::Text(content.into())
    }

    /// Creates a new image preview content from a path
    pub fn image(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        // Create a file URI for the image
        let file_uri = format!("file://{}", path.display());
        PreviewContent::Image(file_uri)
    }

    /// Creates a new image preview content directly from a URI
    pub fn image_uri(uri: impl Into<String>) -> Self {
        PreviewContent::Image(uri.into())
    }

    /// Creates a new zip preview content from a list of entries
    pub fn zip(entries: Vec<ZipEntry>) -> Self {
        PreviewContent::Zip(entries)
    }

    /// Creates a new loading preview content for a path
    pub fn loading(path: impl Into<PathBuf>) -> Self {
        PreviewContent::Loading(path.into(), None)
    }

    /// Creates a new loading preview content with a receiver for async updates
    pub fn loading_with_receiver(
        path: impl Into<PathBuf>,
        receiver: Receiver<Result<PreviewContent, String>>,
    ) -> Self {
        PreviewContent::Loading(path.into(), Some(Arc::new(Mutex::new(receiver))))
    }

    /// Creates a new PDF image preview content
    pub fn pdf(image: egui::widgets::ImageSource<'static>) -> Self {
        PreviewContent::Pdf(image)
    }

    /// Creates a new EPUB preview content with metadata and optional cover image
    pub fn epub(
        metadata: HashMap<String, Vec<String>>,
        cover_image: Option<egui::widgets::ImageSource<'static>>,
    ) -> Self {
        PreviewContent::Epub(metadata, cover_image)
    }
}
