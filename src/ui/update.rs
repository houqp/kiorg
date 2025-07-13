use crate::app::Kiorg;
use egui::Context;
use humansize::{BINARY, format_size};
use self_update::cargo_crate_version;
use std::io::Write;
use std::sync::mpsc;

use crate::ui::notification::NotificationMessage;
use crate::ui::popup::{PopupType, utils};

#[derive(Debug, Clone)]
pub struct Release {
    _release: self_update::update::Release,
}

/// Progress update message sent from background thread
#[derive(Debug, Clone)]
pub enum UpdateProgressUpdate {
    /// Progress update with current and total bytes
    Progress {
        downloaded_bytes: u64,
        total_bytes: i64,
    },
    /// Operation completed successfully
    Completed,
    /// Operation failed with error
    Error(String),
}

/// Progress state for update operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateProgressState {
    pub downloaded_bytes: u64,
    pub total_bytes: i64,
}

/// Progress data containing state and receiver
pub struct UpdateProgressData {
    pub state: UpdateProgressState,
    pub receiver: mpsc::Receiver<UpdateProgressUpdate>,
}

impl std::fmt::Debug for UpdateProgressData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdateProgressData")
            .field("state", &self.state)
            .field("receiver", &"<mpsc::Receiver>")
            .finish()
    }
}

impl PartialEq for UpdateProgressData {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

impl Eq for UpdateProgressData {}

impl Release {
    pub fn new(release: self_update::update::Release) -> Self {
        Self { _release: release }
    }
}

impl std::ops::Deref for Release {
    type Target = self_update::update::Release;

    fn deref(&self) -> &Self::Target {
        &self._release
    }
}

impl PartialEq for Release {
    fn eq(&self, other: &Self) -> bool {
        self._release.version == other._release.version
    }
}

impl Eq for Release {}

/// Check for updates and show confirmation if available
pub fn check_for_updates(app: &mut Kiorg) {
    // TODO: disable checking for updates once the API call completes
    app.notify_info("Checking for updates...");

    let notification_sender = app.notification_system.get_sender();

    std::thread::spawn(move || {
        match check_for_latest_version() {
            Ok(Some(release)) => {
                // Send update available message
                let _ = notification_sender.send(NotificationMessage::UpdateAvailable(release));
            }
            Ok(None) => {
                // No update available
                let _ = notification_sender.send(NotificationMessage::Error(
                    "No updates available. You're using the latest version.".to_string(),
                ));
            }
            Err(e) => {
                // Send error message
                let _ = notification_sender.send(NotificationMessage::Error(format!(
                    "Failed to check for updates: {e}"
                )));
            }
        }
    });
}

/// Start the actual update process after user confirmation
pub fn perform_update_async(ctx: &Context, app: &mut Kiorg, to_release: Release) {
    // Create a channel for progress updates
    let (progress_tx, progress_rx) = mpsc::channel();

    // Initialize progress popup
    let progress_data = UpdateProgressData {
        state: UpdateProgressState {
            downloaded_bytes: 0,
            total_bytes: 0, // Will be updated with actual size from header
        },
        receiver: progress_rx,
    };

    app.show_popup = Some(PopupType::UpdateProgress(progress_data));

    let notification_sender = app.notification_system.get_sender();
    let ctx_clone = ctx.clone(); // Clone the context so it can be moved into the thread

    std::thread::spawn(
        move || match perform_self_update(&ctx_clone, to_release, progress_tx) {
            Ok(_to_release) => {
                let _ = notification_sender.send(NotificationMessage::UpdateSuccess);
                ctx_clone.request_repaint();
            }
            Err(e) => {
                let _ = notification_sender.send(NotificationMessage::UpdateFailed(format!(
                    "Update failed: {e}"
                )));
                ctx_clone.request_repaint();
            }
        },
    );
}

/// Show update confirmation popup
pub fn show_update_confirm_popup(ctx: &Context, app: &mut Kiorg) {
    let release = if let Some(PopupType::UpdateConfirm(release)) = &app.show_popup {
        release
    } else {
        return;
    };

    let mut show_popup = true;

    let result = utils::show_confirm_popup(
        ctx,
        "Update Available",
        &mut show_popup,
        |ui| {
            ui.label(format!("A new version {} is available!", release.version));
            ui.separator();
            ui.label("Would you like to download and install the update?");
        },
        "Update Now",
        "Later",
    );

    match result {
        utils::ConfirmResult::Confirm => {
            // perform update will set popup to the update progress popup
            perform_update_async(ctx, app, release.clone());
        }
        utils::ConfirmResult::Cancel => {
            app.show_popup = None;
        }
        utils::ConfirmResult::None => {
            // Keep popup open
        }
    }
}

fn restart_app() {
    let current_exe = std::env::current_exe().unwrap_or_else(|_| {
        eprintln!("Failed to get current executable path");
        std::process::exit(1);
    });
    let args: Vec<String> = std::env::args().skip(1).collect();

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let e = std::process::Command::new(current_exe).args(&args).exec();

        // Exit the current process with error
        eprintln!("Failed to restart application: {e}");
        std::process::exit(-1);
    }
    #[cfg(windows)]
    {
        std::process::Command::new(current_exe)
            .args(args)
            .spawn()
            .expect("Failed to spawn new process");
        // Now, exit the current process.
        std::process::exit(0);
    }
}

/// Show update restart confirmation popup
pub fn show_update_restart_popup(ctx: &Context, app: &mut Kiorg) {
    if app.show_popup != Some(PopupType::UpdateRestart) {
        return;
    };

    let mut show_popup = true;

    let result = utils::show_confirm_popup(
        ctx,
        "Update Complete",
        &mut show_popup,
        |ui| {
            ui.label("The application needs to be restarted to use the new version.");
        },
        "Restart Now",
        "Later",
    );

    match result {
        utils::ConfirmResult::Confirm => {
            app.show_popup = None;
            app.shutdown_requested = true;
            app.persist_app_state();
            restart_app();
        }
        utils::ConfirmResult::Cancel => {
            app.show_popup = None;
        }
        utils::ConfirmResult::None => {
            // Keep popup open
        }
    }
}

/// Handle update progress popup UI
pub fn show_update_progress(ctx: &Context, app: &mut Kiorg) {
    let progress_data =
        if let Some(PopupType::UpdateProgress(ref mut progress_data)) = app.show_popup {
            progress_data
        } else {
            return;
        };

    while let Ok(update) = progress_data.receiver.try_recv() {
        match update {
            UpdateProgressUpdate::Progress {
                downloaded_bytes,
                total_bytes,
            } => {
                progress_data.state.downloaded_bytes = downloaded_bytes;
                progress_data.state.total_bytes = total_bytes;
            }
            UpdateProgressUpdate::Completed => {
                app.show_popup = Some(PopupType::UpdateRestart);
                return;
            }
            UpdateProgressUpdate::Error(error) => {
                app.show_popup = None;
                app.notify_error(error);
                return;
            }
        }
    }

    let state = &progress_data.state;
    crate::ui::popup::window_utils::new_center_popup_window("Update Progress").show(ctx, |ui| {
        ui.set_min_width(400.0);

        ui.vertical_centered(|ui| {
            ui.add_space(10.0);

            // Progress bar
            let progress = if state.total_bytes > 0 {
                state.downloaded_bytes as f32 / state.total_bytes as f32
            } else {
                0.0
            };

            ui.add(egui::ProgressBar::new(progress).desired_width(350.0));

            ui.add_space(10.0);

            // Status text
            ui.label(format!(
                "{} / {} ({:.1}%)",
                format_size(state.downloaded_bytes, BINARY),
                format_size(state.total_bytes as u64, BINARY),
                progress * 100.0
            ));

            ui.add_space(5.0);

            // Operation description
            ui.label("Downloading update...");

            ui.add_space(10.0);
        });
    });
}

/// Helper function to create a base updater configuration
fn create_base_updater() -> self_update::backends::github::UpdateBuilder {
    let mut updater = self_update::backends::github::Update::configure();
    updater
        .repo_owner("houqp")
        .repo_name("kiorg")
        .bin_name("kiorg")
        .no_confirm(true)
        .current_version(cargo_crate_version!());
    updater
}

/// Check for the latest version without downloading
fn check_for_latest_version() -> Result<Option<Release>, Box<dyn std::error::Error>> {
    let updater = create_base_updater().build()?;
    let latest_release = updater.get_latest_release()?;
    let current_version = cargo_crate_version!();

    // Compare versions
    if latest_release.version != current_version {
        Ok(Some(Release::new(latest_release)))
    } else {
        Ok(None)
    }
}

/// Extract zip archive contents to a directory
fn extract_into(
    archive_path: &std::path::Path,
    destination: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let archive_file = std::fs::File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(archive_file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => destination.join(path),
            None => continue,
        };

        if file.is_dir() {
            // Directory
            std::fs::create_dir_all(&outpath)?;
        } else {
            // File
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }

        // Set permissions on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

/// custom update function for use with bundles
/// takeen from: https://github.com/jaemk/self_update/pull/147/files
pub fn perform_self_update(
    ctx: &Context,
    to_release: Release,
    progress_tx: mpsc::Sender<UpdateProgressUpdate>,
) -> Result<Release, Box<dyn std::error::Error>> {
    // get the first available release
    let asset = to_release
        .asset_for(self_update::get_target(), None)
        .ok_or("no compatible binary target found for release")?;

    let tmp_archive_dir = tempfile::TempDir::new()?;
    let tmp_archive_path = tmp_archive_dir.path().join(&asset.name);
    let mut tmp_archive = std::fs::File::create(&tmp_archive_path)?;

    let response = ureq::get(&asset.download_url)
        .set("Accept", "application/octet-stream")
        .call()?;

    let total_size: i64 = response
        .header("Content-Length")
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or("Content-Length header not found or invalid")?;

    // Send initial progress update
    let _ = progress_tx.send(UpdateProgressUpdate::Progress {
        downloaded_bytes: 0,
        total_bytes: total_size,
    });

    let mut reader = response.into_reader();
    let mut downloaded_bytes: u64 = 0;
    let mut buffer = [0; 8192]; // 8KB buffer
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => {
                // EOF reached
                break;
            }
            Ok(bytes_read) => {
                downloaded_bytes += bytes_read as u64;
                tmp_archive.write_all(&buffer[..bytes_read])?;

                // Send progress update
                let _ = progress_tx.send(UpdateProgressUpdate::Progress {
                    downloaded_bytes,
                    total_bytes: total_size,
                });
                ctx.request_repaint();
            }
            Err(e) => {
                let _ = progress_tx.send(UpdateProgressUpdate::Error(format!(
                    "Error reading response: {e}"
                )));
                ctx.request_repaint();
                return Err(e.into());
            }
        }
    }
    tmp_archive.flush()?;

    // Extract the zip archive
    extract_into(&tmp_archive_path, tmp_archive_dir.path())?;

    let binary_path = std::env::current_exe()?;
    let binary = binary_path
        .file_name()
        .ok_or("failed to extract current binary name")?
        .to_str()
        .ok_or("failed to convert OsStr")?;

    let new_exe = {
        #[cfg(target_os = "windows")]
        {
            tmp_archive_dir.path().join(binary)
        }
        #[cfg(target_os = "linux")]
        {
            tmp_archive_dir.path().join(binary)
        }
        #[cfg(target_os = "macos")]
        {
            // TODO: support app bundle atomic swap, see:
            // https://github.com/jaemk/self_update/pull/147/files
            tmp_archive_dir.path().join(binary)
        }
    };

    self_replace::self_replace(new_exe)?;

    // Send completion update
    let _ = progress_tx.send(UpdateProgressUpdate::Completed);
    ctx.request_repaint();

    Ok(to_release)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_extract_into() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_zip_path = temp_dir.path().join("test.zip");

        // Create a test zip file
        let zip_file = fs::File::create(&test_zip_path).unwrap();
        let mut zip_writer = zip::ZipWriter::new(zip_file);

        // Add a directory
        zip_writer
            .add_directory("test_dir/", zip::write::FileOptions::<()>::default())
            .unwrap();

        // Add a file in the root
        zip_writer
            .start_file("test_file.txt", zip::write::FileOptions::<()>::default())
            .unwrap();
        zip_writer.write_all(b"Hello, world!").unwrap();

        // Add a file in a subdirectory
        zip_writer
            .start_file(
                "test_dir/nested_file.txt",
                zip::write::FileOptions::<()>::default(),
            )
            .unwrap();
        zip_writer.write_all(b"Nested content").unwrap();

        zip_writer.finish().unwrap();

        // Create extraction destination
        let extract_dir = TempDir::new().unwrap();

        // Test the extraction
        let result = extract_into(&test_zip_path, extract_dir.path());
        assert!(result.is_ok(), "Extraction should succeed");

        // Verify extracted contents
        let root_file = extract_dir.path().join("test_file.txt");
        assert!(root_file.exists(), "Root file should exist");
        let content = fs::read_to_string(&root_file).unwrap();
        assert_eq!(content, "Hello, world!");

        let nested_file = extract_dir.path().join("test_dir/nested_file.txt");
        assert!(nested_file.exists(), "Nested file should exist");
        let nested_content = fs::read_to_string(&nested_file).unwrap();
        assert_eq!(nested_content, "Nested content");

        let dir_path = extract_dir.path().join("test_dir");
        assert!(dir_path.exists(), "Directory should exist");
        assert!(dir_path.is_dir(), "Should be a directory");
    }

    #[test]
    fn test_extract_into_nonexistent_archive() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_zip = temp_dir.path().join("nonexistent.zip");
        let extract_dir = TempDir::new().unwrap();

        let result = extract_into(&nonexistent_zip, extract_dir.path());
        assert!(result.is_err(), "Should fail for nonexistent archive");
    }

    #[test]
    fn test_extract_into_invalid_zip() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_zip = temp_dir.path().join("invalid.zip");

        // Create a file that's not a valid zip
        fs::write(&invalid_zip, b"not a zip file").unwrap();

        let extract_dir = TempDir::new().unwrap();
        let result = extract_into(&invalid_zip, extract_dir.path());
        assert!(result.is_err(), "Should fail for invalid zip file");
    }
}
