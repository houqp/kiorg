//! Loading state module for preview content

use crate::app::Kiorg;
use crate::config::colors::AppColors;
use crate::models::preview_content::{PreviewContent, PreviewReceiver};
use egui::RichText;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

/// Render loading state
pub fn render(
    app: &mut Kiorg,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    path: &Path,
    receiver: PreviewReceiver,
    colors: &AppColors,
) {
    // Display loading indicator
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        ui.spinner();
        ui.add_space(10.0);
        ui.label(
            RichText::new(format!(
                "Loading preview contents for {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ))
            .color(colors.fg),
        );
    });

    // Try to get a lock on the receiver
    let receiver = receiver.lock().expect("failed to obtain lock");
    // Try to receive the result without blocking
    if let Ok(result) = receiver.try_recv() {
        // Request a repaint to update the UI with the result
        ctx.request_repaint();
        // Update the preview content with the result
        match result {
            Ok(content) => {
                // Set the preview content directly with the received content
                app.preview_content = Some(content);
            }
            Err(e) => {
                app.preview_content =
                    Some(PreviewContent::text(format!("Error loading file: {e}")));
            }
        }
    }
}

/// Helper function to load preview content asynchronously
///
/// This function handles the common pattern of:
/// - Creating a channel for communication
/// - Setting up the loading state with receiver
/// - Spawning a thread to process the file
/// - Sending the result back through the channel
///
/// # Arguments
/// * `app` - The application state
/// * `path` - The path to the file to load
/// * `processor` - A closure that processes the file and returns a Result<`PreviewContent`, String>
pub fn load_preview_async<F>(app: &mut Kiorg, path: PathBuf, processor: F)
where
    F: FnOnce(PathBuf) -> Result<PreviewContent, String> + Send + 'static,
{
    // Check for existing loading content and trigger cancel signal
    if let Some(PreviewContent::Loading(_, _, existing_cancel_sender)) = &app.preview_content {
        let _ = existing_cancel_sender.send(());
    }

    let (receiver, cancel_sender) = create_preview_task(path.clone(), processor);

    // Set the initial loading state with the receiver
    app.preview_content = Some(PreviewContent::Loading(path, receiver, cancel_sender));
}

/// Create an async preview content loading task
pub fn create_preview_task<F>(path: PathBuf, processor: F) -> (PreviewReceiver, mpsc::Sender<()>)
where
    F: FnOnce(PathBuf) -> Result<PreviewContent, String> + Send + 'static,
{
    // Create a channel for process result communication
    let (sender, receiver) = std::sync::mpsc::channel();
    // Create a channel for cancel signaling
    let (cancel_sender, cancel_receiver) = mpsc::channel();

    // Spawn a thread to process the file
    std::thread::spawn(move || {
        // Wait for debounce treshold
        match cancel_receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                // Cancel signal received or dropped, terminate early
                return;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Timeout reached, proceed with processing
            }
        }
        let preview_result = processor(path);
        let _ = sender.send(preview_result);
    });

    (Arc::new(Mutex::new(receiver)), cancel_sender)
}

/// Type alias for the return type of popup loading tasks
pub type PopupLoadTask<T> = (
    Arc<Mutex<mpsc::Receiver<Result<T, String>>>>,
    mpsc::Sender<()>,
);

/// Generic function to create an async loading task for popup viewers
/// No debouncing since popups are explicitly triggered by user action
pub fn create_load_popup_meta_task<T, F>(path: PathBuf, processor: F) -> PopupLoadTask<T>
where
    T: Send + 'static,
    F: FnOnce(PathBuf) -> Result<T, String> + Send + 'static,
{
    let (sender, receiver) = std::sync::mpsc::channel();
    let (cancel_sender, cancel_receiver) = mpsc::channel();

    std::thread::spawn(move || {
        // Check for cancellation before processing
        if cancel_receiver.try_recv().is_ok() {
            return;
        }
        let _ = sender.send(processor(path));
    });

    (Arc::new(Mutex::new(receiver)), cancel_sender)
}
