use pdfium_bind::PdfDocument;
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
    /// Keep the texture handle alive to prevent GPU texture from being freed
    pub _texture_handle: Option<egui::TextureHandle>,
    /// Current page number (0-indexed)
    pub current_page: isize,
    /// Total number of pages in the PDF
    pub page_count: isize,
    /// Cached PDF file object to avoid reopening and parsing on every page navigation
    pub pdf_file: Arc<Mutex<PdfDocument>>,
}

impl std::fmt::Debug for PdfMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PdfMeta")
            .field("file_id", &self.file_id)
            .field("title", &self.title)
            .field("metadata", &self.metadata)
            .field("cover", &"<ImageSource>")
            .field(
                "_texture_handle",
                &self._texture_handle.as_ref().map(|_| "TextureHandle"),
            )
            .field("current_page", &self.current_page)
            .field("page_count", &self.page_count)
            .field("pdf_file", &"<PDF File>")
            .finish()
    }
}

impl PdfMeta {
    pub fn render_page(&mut self, ctx: &egui::Context) -> Result<(), String> {
        // Render the new page using cached file
        let (img_source, texture_handle) = crate::ui::preview::doc::render_pdf_page_high_dpi(
            &self.pdf_file.lock().expect("failed to lock pdf_file"),
            self.current_page,
            Some(&self.file_id),
            ctx,
        )?;

        self.cover = img_source;
        self._texture_handle = Some(texture_handle);
        Ok(())
    }

    #[inline]
    pub fn page_input_id(&self) -> egui::Id {
        egui::Id::new(format!("pdf_page_input_{}", &self.file_id))
    }

    pub fn update_page_num_text(&self, ctx: &egui::Context) {
        let input_id = self.page_input_id();
        // in the UI, we display the first page as 1 instead of 0
        let new_text = (self.current_page + 1).to_string();
        ctx.data_mut(|d| d.insert_temp(input_id, new_text));
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
    pub page_count: isize,
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
    /// Pre-constructed image widget ready for rendering
    pub image: egui::Image<'static>,
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
            .field("image", &"Image")
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
    /// Plugin-generated preview content
    PluginPreview { components: Vec<RenderedComponent> },
    /// Image content with metadata
    Image(ImageMeta),
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

fn load_into_texture(
    ctx: &egui::Context,
    dynamic_image: image::DynamicImage,
    name: String,
) -> (egui::Image<'static>, Option<egui::TextureHandle>) {
    let rgba8 = dynamic_image.to_rgba8();
    let size = [rgba8.width() as usize, rgba8.height() as usize];
    let color_image =
        egui::ColorImage::from_rgba_unmultiplied(size, rgba8.as_flat_samples().as_slice());
    let texture = ctx.load_texture(name, color_image, Default::default());
    let image = egui::Image::new(&texture);
    (image, Some(texture))
}

/// Rendered version of plugin components that can hold processed data like textures
#[derive(Clone, Debug)]
pub enum RenderedComponent {
    Title(kiorg_plugin::TitleComponent),
    Text(kiorg_plugin::TextComponent),
    Image(RenderedImageComponent),
    Table(kiorg_plugin::TableComponent),
}

#[derive(Clone)]
pub struct RenderedImageComponent {
    pub image: egui::Image<'static>,
    pub interactive: bool,
    pub _texture_handle: Option<egui::TextureHandle>,
}

impl std::fmt::Debug for RenderedImageComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderedImageComponent")
            .field("image", &"Image")
            .field("interactive", &self.interactive)
            .field(
                "_texture_handle",
                &self._texture_handle.as_ref().map(|_| "TextureHandle"),
            )
            .finish()
    }
}

impl PreviewContent {
    /// Creates a new text preview content
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text(content.into())
    }

    /// Creates a new plugin preview content by processing plugin components
    pub fn plugin_preview_from_components(
        components: Vec<kiorg_plugin::Component>,
        ctx: &egui::Context,
    ) -> Self {
        let mut rendered_components = Vec::with_capacity(components.len());

        for component in components {
            match component {
                kiorg_plugin::Component::Title(t) => {
                    rendered_components.push(RenderedComponent::Title(t))
                }
                kiorg_plugin::Component::Text(t) => {
                    rendered_components.push(RenderedComponent::Text(t))
                }
                kiorg_plugin::Component::Table(t) => {
                    rendered_components.push(RenderedComponent::Table(t))
                }
                kiorg_plugin::Component::Image(img) => match img.source {
                    kiorg_plugin::ImageSource::Path(path) => match image::open(&path) {
                        Ok(dynamic_image) => {
                            let (image, texture_handle) = load_into_texture(
                                ctx,
                                dynamic_image,
                                format!("plugin_preview_path_{}", path),
                            );
                            rendered_components.push(RenderedComponent::Image(
                                RenderedImageComponent {
                                    image,
                                    interactive: img.interactive,
                                    _texture_handle: texture_handle,
                                },
                            ));
                        }
                        Err(e) => {
                            rendered_components.push(RenderedComponent::Text(
                                kiorg_plugin::TextComponent {
                                    text: format!(
                                        "Failed to load image from path: {}\nError: {}",
                                        path, e
                                    ),
                                },
                            ));
                        }
                    },
                    kiorg_plugin::ImageSource::Bytes { format, data, uid } => {
                        match image::load_from_memory_with_format(&data, format) {
                            Ok(dynamic_image) => {
                                let (image, texture_handle) = load_into_texture(
                                    ctx,
                                    dynamic_image,
                                    format!("plugin_preview_bytes_{}", uid),
                                );
                                rendered_components.push(RenderedComponent::Image(
                                    RenderedImageComponent {
                                        image,
                                        interactive: img.interactive,
                                        _texture_handle: texture_handle,
                                    },
                                ));
                            }
                            Err(e) => {
                                rendered_components.push(RenderedComponent::Text(
                                        kiorg_plugin::TextComponent {
                                            text: format!(
                                                "Failed to decode image (format: {:?}, uid: {}\nError: {}",
                                                format, uid, e
                                            ),
                                        },
                                    ));
                            }
                        }
                    }
                },
            }
        }
        Self::PluginPreview {
            components: rendered_components,
        }
    }

    /// Creates a new image preview content with a texture handle
    pub fn image(
        title: impl Into<String>,
        metadata: HashMap<String, String>,
        texture: egui::TextureHandle,
        exif_data: Option<HashMap<String, String>>,
    ) -> Self {
        let image = egui::Image::new(&texture);
        Self::Image(ImageMeta {
            title: title.into(),
            metadata,
            exif_data,
            image,
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
        let image = egui::Image::new(egui::widgets::ImageSource::Uri(uri.into()));
        Self::Image(ImageMeta {
            title: title.into(),
            metadata,
            exif_data,
            image,
            _texture_handle: None, // No texture handle for URI-based images
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
        texture_handle: Option<egui::TextureHandle>,
        metadata: HashMap<String, String>,
        title: Option<String>,
        page_count: isize,
        pdf_file: Arc<Mutex<PdfDocument>>,
        file_path: &std::path::Path,
    ) -> Self {
        // Generate unique file ID from path
        let file_id = file_path.to_string_lossy().to_string();
        // Use provided title or file name
        let title = title.unwrap_or_else(|| {
            let file_name = file_path
                .file_name()
                .map(|f| f.to_string_lossy().to_string());
            file_name.unwrap_or_else(|| file_id.clone())
        });
        Self::Pdf(PdfMeta {
            file_id,
            title,
            metadata,
            cover: image,
            _texture_handle: texture_handle,
            current_page: 0,
            page_count,
            pdf_file,
        })
    }

    /// Creates a new EPUB preview content with metadata and optional cover image
    #[must_use]
    pub fn epub_with_file(
        mut metadata: HashMap<String, String>,
        cover_image: egui::widgets::ImageSource<'static>,
        page_count: isize,
        file_path: &std::path::Path,
    ) -> Self {
        fn pop_title(
            metadata: &mut HashMap<String, String>,
            file_path: &std::path::Path,
        ) -> String {
            let title_keys = ["title", "dc:title"];
            for key in title_keys {
                if let Some(v) = metadata.remove(key)
                    && !v.is_empty()
                {
                    return v;
                }
            }
            // no title key found, use file name/path as title
            let file_name = file_path.file_name().map(|f| f.to_string_lossy());
            file_name
                .unwrap_or_else(|| file_path.to_string_lossy())
                .to_string()
        }
        let title = pop_title(&mut metadata, file_path);
        Self::Epub(EpubMeta {
            title,
            metadata,
            cover: cover_image,
            page_count,
        })
    }
}
