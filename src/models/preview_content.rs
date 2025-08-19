use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

/// Type alias for the async preview content receiver
// TODO: can we delete the option wrapper?
pub type PreviewReceiver = Arc<Mutex<Receiver<Result<PreviewContent, String>>>>;

/// Metadata for PDF documents
#[derive(Clone)]
pub struct PdfMeta {
    /// Unique identifier for this PDF file (based on file path)
    pub file_id: String,
    /// Document title
    pub title: String,
    /// Document metadata (key-value pairs)
    pub metadata: HashMap<String, String>,
    /// Cover image or first page
    pub cover: egui::widgets::ImageSource<'static>,
    /// Current page number (0-indexed)
    pub current_page: usize,
    /// Total number of pages in the PDF
    pub page_count: usize,
    /// Cached PDF file object to avoid reopening and parsing on every page navigation
    pub pdf_file:
        Arc<pdf::file::File<Vec<u8>, pdf::file::NoCache, pdf::file::NoCache, pdf::file::NoLog>>,
}

impl std::fmt::Debug for PdfMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PdfMeta")
            .field("file_id", &self.file_id)
            .field("title", &self.title)
            .field("metadata", &self.metadata)
            .field("cover", &"<ImageSource>")
            .field("current_page", &self.current_page)
            .field("page_count", &self.page_count)
            .field("pdf_file", &"<PDF File>")
            .finish()
    }
}

/// Metadata for EPUB documents
#[derive(Clone, Debug)]
pub struct EpubMeta {
    /// Document title
    pub title: String,
    /// Document metadata (key-value pairs)
    pub metadata: HashMap<String, String>,
    /// Cover image or first page
    pub cover: egui::widgets::ImageSource<'static>,
    /// Total number of pages in the EPUB
    pub page_count: usize,
}

/// Metadata for image files
#[derive(Clone)]
pub struct ImageMeta {
    /// Image title (usually filename)
    pub title: String,
    /// Image metadata (key-value pairs)
    pub metadata: HashMap<String, String>,
    /// EXIF metadata (key-value pairs), stored separately from regular metadata
    pub exif_data: Option<HashMap<String, String>>,
    /// Image source (can be texture handle or URI for animated images)
    pub image_source: egui::widgets::ImageSource<'static>,
    /// Keep the texture handle alive to prevent GPU texture from being freed
    pub _texture_handle: Option<egui::TextureHandle>,
}

// Manual implementation of Debug for ImageMeta since TextureHandle doesn't implement Debug
impl std::fmt::Debug for ImageMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageMeta")
            .field("title", &self.title)
            .field("metadata", &self.metadata)
            .field("exif_data", &self.exif_data)
            .field("image_source", &"ImageSource")
            .field(
                "_texture_handle",
                &self._texture_handle.as_ref().map(|_| "TextureHandle"),
            )
            .finish()
    }
}

/// Metadata for video files
#[derive(Clone)]
pub struct VideoMeta {
    /// Video title (usually filename)
    pub title: String,
    /// Video metadata (key-value pairs)
    pub metadata: HashMap<String, String>,
    /// Video thumbnail image
    pub thumbnail: egui::widgets::ImageSource<'static>,
    /// Keep the texture handle alive to prevent GPU texture from being freed
    pub _texture_handle: Option<egui::TextureHandle>,
}

// Manual implementation of Debug for VideoMeta since TextureHandle doesn't implement Debug
impl std::fmt::Debug for VideoMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoMeta")
            .field("title", &self.title)
            .field("metadata", &self.metadata)
            .field("thumbnail", &"ImageSource")
            .field(
                "_texture_handle",
                &self._texture_handle.as_ref().map(|_| "TextureHandle"),
            )
            .finish()
    }
}

/// Represents different types of preview content that can be displayed in the right panel
#[derive(Clone, Debug)]
pub enum PreviewContent {
    /// Text content to be displayed
    Text(String),
    /// Syntax highlighted text content with language specification
    HighlightedCode {
        content: String,
        language: &'static str,
    },
    /// Image content with metadata
    Image(ImageMeta),
    /// Video content with metadata and thumbnail
    Video(VideoMeta),
    /// Zip file content with a list of entries
    Zip(Vec<ZipEntry>),
    /// Tar file content with a list of entries (supports both compressed and uncompressed)
    Tar(Vec<TarEntry>),
    /// PDF document with page navigation support
    Pdf(PdfMeta),
    /// EPUB document without page navigation
    Epub(EpubMeta),
    /// Directory content with a list of entries
    Directory(Vec<DirectoryEntry>),
    /// Loading state with path being loaded and optional receiver for async loading
    Loading(PathBuf, PreviewReceiver, std::sync::mpsc::Sender<()>),
}

/// Represents an entry in a directory listing for preview
#[derive(Clone, Debug)]
pub struct DirectoryEntry {
    /// Name of the entry (file or directory)
    pub name: String,
    /// Whether the entry is a directory
    pub is_dir: bool,
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

/// Represents an entry in a tar file
#[derive(Clone, Debug)]
pub struct TarEntry {
    /// Name of the entry (file or directory)
    pub name: String,
    /// Size of the entry in bytes
    pub size: u64,
    /// Whether the entry is a directory
    pub is_dir: bool,
    /// Unix permissions in octal format (e.g., "755", "644")
    pub permissions: String,
}

impl PreviewContent {
    /// Creates a new text preview content
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text(content.into())
    }

    /// Creates a new image preview content with a texture handle
    pub fn image(
        title: impl Into<String>,
        metadata: HashMap<String, String>,
        texture: egui::TextureHandle,
        exif_data: Option<HashMap<String, String>>,
    ) -> Self {
        Self::Image(ImageMeta {
            title: title.into(),
            metadata,
            exif_data,
            image_source: egui::widgets::ImageSource::from(&texture),
            _texture_handle: Some(texture),
        })
    }

    /// Creates a new image preview content with a URI (for animated images like GIFs)
    pub fn image_from_uri(
        title: impl Into<String>,
        metadata: HashMap<String, String>,
        uri: String,
        exif_data: Option<HashMap<String, String>>,
    ) -> Self {
        Self::Image(ImageMeta {
            title: title.into(),
            metadata,
            exif_data,
            image_source: egui::widgets::ImageSource::Uri(uri.into()),
            _texture_handle: None, // No texture handle for URI-based images//
        })
    }

    /// Creates a new video preview content with a texture handle for thumbnail
    pub fn video(
        title: impl Into<String>,
        metadata: HashMap<String, String>,
        texture: egui::TextureHandle,
    ) -> Self {
        Self::Video(VideoMeta {
            title: title.into(),
            metadata,
            thumbnail: egui::widgets::ImageSource::from(&texture),
            _texture_handle: Some(texture),
        })
    }

    /// Creates a new zip preview content from a list of entries
    #[must_use]
    pub const fn zip(entries: Vec<ZipEntry>) -> Self {
        Self::Zip(entries)
    }

    /// Creates a new tar preview content from a list of entries
    #[must_use]
    pub const fn tar(entries: Vec<TarEntry>) -> Self {
        Self::Tar(entries)
    }

    /// Creates a new directory preview content from a list of entries
    #[must_use]
    pub const fn directory(entries: Vec<DirectoryEntry>) -> Self {
        Self::Directory(entries)
    }

    /// Creates a new loading preview content with a receiver for async updates
    pub fn loading_with_receiver(
        path: impl Into<PathBuf>,
        receiver: Receiver<Result<Self, String>>,
        cancel_sender: std::sync::mpsc::Sender<()>,
    ) -> Self {
        Self::Loading(path.into(), Arc::new(Mutex::new(receiver)), cancel_sender)
    }

    /// Creates a new PDF document preview content with cached PDF file
    #[must_use]
    pub fn pdf_with_file(
        image: egui::widgets::ImageSource<'static>,
        metadata: HashMap<String, String>,
        title: Option<String>,
        page_count: usize,
        pdf_file: Arc<
            pdf::file::File<Vec<u8>, pdf::file::NoCache, pdf::file::NoCache, pdf::file::NoLog>,
        >,
        file_path: &std::path::Path,
    ) -> Self {
        // Use provided title or default
        let title = title.unwrap_or_else(|| "__Untitled__".to_string());
        // Generate unique file ID from path
        let file_id = file_path.to_string_lossy().to_string();

        Self::Pdf(PdfMeta {
            file_id,
            title,
            metadata,
            cover: image,
            current_page: 0,
            page_count,
            pdf_file,
        })
    }

    /// Creates a new EPUB preview content with metadata and optional cover image
    #[must_use]
    pub fn epub(
        mut metadata: HashMap<String, Vec<String>>,
        cover_image: egui::widgets::ImageSource<'static>,
        page_count: usize,
    ) -> Self {
        // Extract title from metadata
        let title = Self::extract_epub_book_title(&metadata);

        // Remove title keys from metadata since we've extracted the title
        metadata.remove("title");
        metadata.remove("dc:title");

        // Convert multi-value metadata to single-value by joining with commas
        let single_metadata = metadata
            .into_iter()
            .filter(|(key, _)| key != "cover") // Filter out cover as it's handled separately
            .map(|(key, values)| {
                let value = if values.len() > 1 {
                    values.join(", ")
                } else if !values.is_empty() {
                    values[0].clone()
                } else {
                    "N/A".to_string()
                };
                (key, value)
            })
            .collect();

        Self::Epub(EpubMeta {
            title,
            metadata: single_metadata,
            cover: cover_image,
            page_count,
        })
    }

    /// Helper function to extract book title from EPUB metadata
    fn extract_epub_book_title(metadata: &HashMap<String, Vec<String>>) -> String {
        // Check for title in various possible metadata keys
        let title_keys = ["title", "dc:title"];

        for key in &title_keys {
            if let Some(values) = metadata.get(*key)
                && !values.is_empty()
            {
                return values[0].clone();
            }
        }
        // If no title found, return a default
        "__Untitled__".to_string()
    }
}
