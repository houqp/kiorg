use std::path::PathBuf;

use crate::ui::update::Release;

pub mod about;
pub mod action_history;
pub mod add_entry;
pub mod bookmark;
pub mod delete;
pub mod exit;
pub mod file_drop;
pub mod generic_message;
pub mod open_with;
pub mod preview;
pub mod rename;
pub mod sort_toggle;
pub mod teleport;
pub mod theme;
pub mod utils;
pub mod window_utils;

/// Popup types that can be shown in the application
#[derive(Debug, PartialEq, Eq)]
pub enum PopupType {
    About,
    Help,
    Exit,
    GenericMessage(String, String), // Title and message for generic popup
    Delete(crate::ui::popup::delete::DeleteConfirmState, Vec<PathBuf>),
    DeleteProgress(crate::ui::popup::delete::DeleteProgressData),
    Rename(String),         // New name for the file/directory being renamed
    OpenWith(String),       // Command to use when opening a file with a custom command
    AddEntry(String),       // Name for the new file/directory being added
    Bookmarks(usize),       // Selected index in the bookmarks list
    Preview,                // Show file preview in a popup window
    Themes(String),         // Selected theme key in the themes list
    FileDrop(Vec<PathBuf>), // List of dropped files
    Teleport(crate::ui::popup::teleport::TeleportState), // Teleport through visit history
    UpdateConfirm(Release), // Show update confirmation with version info
    UpdateProgress(crate::ui::update::UpdateProgressData), // Show update progress during download
    UpdateRestart,          // Show restart confirmation with version info
    SortToggle,             // Show sort toggle popup for column sorting
    ActionHistory,          // Show action history with rollback options
}
