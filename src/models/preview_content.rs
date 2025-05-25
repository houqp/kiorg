use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

/// Type alias for the async preview content receiver
pub type PreviewReceiver = Option<Arc<Mutex<Receiver<Result<PreviewContent, String>>>>>;

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
    /// Texture handle for the image
    pub texture: egui::TextureHandle,
}

// Manual implementation of Debug for ImageMeta since TextureHandle doesn't implement Debug
impl std::fmt::Debug for ImageMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageMeta")
            .field("title", &self.title)
            .field("metadata", &self.metadata)
            .field("exif_data", &self.exif_data)
            .field("texture", &"TextureHandle")
            .finish()
    }
}

/// Represents different types of preview content that can be displayed in the right panel
#[derive(Clone, Debug)]
pub enum PreviewContent {
    /// Text content to be displayed
    Text(String),
    /// Image content with metadata
    Image(ImageMeta),
    /// Zip file content with a list of entries
    Zip(Vec<ZipEntry>),
    /// PDF document with page navigation support
    Pdf(PdfMeta),
    /// EPUB document without page navigation
    Epub(EpubMeta),
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

    /// Creates a new image preview content with a texture handle
    pub fn image(
        title: impl Into<String>,
        metadata: HashMap<String, String>,
        texture: egui::TextureHandle,
        exif_data: Option<HashMap<String, String>>,
    ) -> Self {
        PreviewContent::Image(ImageMeta {
            title: title.into(),
            metadata,
            exif_data,
            texture,
        })
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

    /// Creates a new PDF document preview content with cached PDF file
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

        PreviewContent::Pdf(PdfMeta {
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

        PreviewContent::Epub(EpubMeta {
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

        for key in title_keys.iter() {
            if let Some(values) = metadata.get(*key) {
                if !values.is_empty() {
                    return values[0].clone();
                }
            }
        }
        // If no title found, return a default
        "__Untitled__".to_string()
    }
}
