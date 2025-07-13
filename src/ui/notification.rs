use std::sync::mpsc;

use crate::app::Kiorg;
use crate::ui::egui_notify::Toasts;
use crate::ui::popup::PopupType;
use crate::ui::update::Release;

/// Notification messages for background operations
#[derive(Debug, Clone)]
pub enum NotificationMessage {
    Error(String),
    UpdateAvailable(Release), // Version string
    UpdateSuccess,            // Version string
    UpdateFailed(String),     // Error message
}

/// Async notification system for handling background operation messages
pub struct AsyncNotification {
    pub sender: mpsc::Sender<NotificationMessage>,
    pub receiver: mpsc::Receiver<NotificationMessage>,
}

impl AsyncNotification {
    /// Get a clone of the sender for use in background threads
    pub fn get_sender(&self) -> mpsc::Sender<NotificationMessage> {
        self.sender.clone()
    }
}

impl Default for AsyncNotification {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel::<NotificationMessage>();
        Self { sender, receiver }
    }
}

/// Display an error notification with a consistent timeout
pub fn notify_error<T: ToString>(toasts: &mut Toasts, message: T) {
    toasts
        .error(message.to_string())
        .duration(Some(std::time::Duration::from_secs(10)));
}

/// Display an info notification with a consistent timeout
pub fn notify_info<T: ToString>(toasts: &mut Toasts, message: T) {
    toasts
        .info(message.to_string())
        .duration(Some(std::time::Duration::from_secs(5)));
}

/// Display a success notification with a consistent timeout
pub fn notify_success<T: ToString>(toasts: &mut Toasts, message: T) {
    toasts
        .success(message.to_string())
        .duration(Some(std::time::Duration::from_secs(5)));
}

/// Check and process notification messages from background operations
pub fn check_notifications(app: &mut Kiorg) {
    while let Ok(message) = app.notification_system.receiver.try_recv() {
        match message {
            NotificationMessage::UpdateAvailable(release) => {
                app.show_popup = Some(PopupType::UpdateConfirm(release));
            }
            NotificationMessage::UpdateSuccess => {
                app.show_popup = Some(PopupType::UpdateRestart);
            }
            NotificationMessage::UpdateFailed(error) => {
                notify_error(&mut app.toasts, &error);
                // Only clear popup if it's not currently showing progress
                if !matches!(app.show_popup, Some(PopupType::UpdateProgress(_))) {
                    app.show_popup = None;
                }
            }
            NotificationMessage::Error(error) => {
                notify_error(&mut app.toasts, &error);
            }
        }
    }
}
