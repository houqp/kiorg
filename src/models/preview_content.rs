use std::path::PathBuf;

/// Represents different types of preview content that can be displayed in the right panel
#[derive(Clone, Debug)]
pub enum PreviewContent {
    /// Text content to be displayed
    Text(String),
    /// Image content with URI to the image file
    Image(String),
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
}
