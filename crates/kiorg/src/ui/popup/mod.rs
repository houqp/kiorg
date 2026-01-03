use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};

use crate::ui::update::Release;

/// Type alias for the async content receiver used in popup loading states
type ContentReceiver<T> = Arc<Mutex<mpsc::Receiver<Result<T, String>>>>;

/// Trait for popup viewers that support async loading with Loading, Loaded, and Error states
pub trait PopupApp: Sized {
    /// The type that is loaded asynchronously
    type Content;

    /// Create a new instance in the Loading state
    fn loading(
        path: PathBuf,
        receiver: ContentReceiver<Self::Content>,
        cancel_sender: mpsc::Sender<()>,
    ) -> Self;

    /// Create a new instance in the Loaded state
    fn loaded(content: Self::Content) -> Self;

    /// Create a new instance in the Error state
    fn error(message: String) -> Self;

    /// Check if the viewer is in the Loading state and return its receiver if so
    fn as_loading(&self) -> Option<&ContentReceiver<Self::Content>>;

    /// Get the title for the viewer window
    fn title(&self) -> String;
}

pub mod about;
pub mod action_history;
pub mod add_entry;
pub mod bookmark;
pub mod delete;
pub mod ebook_viewer;
pub mod exit;
pub mod file_drop;
pub mod fuzzy_search_popup;
pub mod generic_message;
pub mod image_viewer;
pub mod open_with;
pub mod pdf_viewer;
pub mod plugin;
pub mod plugin_viewer;
pub mod preview;
pub mod rename;
pub mod sort_toggle;
pub mod teleport;
pub mod text_input_popup;
pub mod theme;
pub mod utils;
pub mod video_viewer;
#[cfg(target_os = "macos")]
pub mod volumes;
pub mod window_utils;
#[cfg(target_os = "windows")]
pub mod windows_drives;

/// Popup types that can be shown in the application
#[derive(Debug)]
pub enum PopupType {
    About,
    Help,
    Exit,
    GenericMessage(String, String), // Title and message for generic popup
    Delete(crate::ui::popup::delete::DeleteConfirmState, Vec<PathBuf>),
    DeleteProgress(crate::ui::popup::delete::DeleteProgressData),
    Rename(String),   // New name for the file/directory being renamed
    OpenWith,         // Open file with custom command popup
    AddEntry(String), // Name for the new file/directory being added
    Bookmarks(usize), // Selected index in the bookmarks list
    #[cfg(target_os = "windows")]
    WindowsDrives(usize), // Selected index in the drives list (Windows only)
    #[cfg(target_os = "macos")]
    Volumes(usize), // Selected index in the volumes list (macOS only)
    Preview,          // Show file preview in a popup window
    Pdf(Box<crate::ui::popup::pdf_viewer::PdfViewer>), // PDF app
    Ebook(Box<crate::ui::popup::ebook_viewer::EbookViewer>), // Ebook app
    Image(Box<crate::ui::popup::image_viewer::ImageViewer>), // Image app
    Video(Box<crate::ui::popup::video_viewer::VideoViewer>), // Video app
    Plugin(Box<crate::ui::popup::plugin_viewer::PluginViewer>), // Plugin app
    Themes(String),   // Selected theme key in the themes list
    Plugins,          // Show plugins list
    FileDrop(Vec<PathBuf>), // List of dropped files
    Teleport(crate::ui::popup::teleport::TeleportState), // Teleport through visit history
    UpdateConfirm(Release), // Show update confirmation with version info
    UpdateProgress(crate::ui::update::UpdateProgressData), // Show update progress during download
    UpdateRestart,    // Show restart confirmation with version info
    SortToggle,       // Show sort toggle popup for column sorting
    ActionHistory,    // Show action history with rollback options
}
