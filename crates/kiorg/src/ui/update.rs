use crate::app::Kiorg;
use egui::Context;
use humansize::{BINARY, format_size};
use self_update::cargo_crate_version;
use semver::Version;
use std::env::consts::ARCH;
#[cfg(not(target_os = "macos"))]
use std::env::consts::OS;
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
                // No update available - we're already on the latest or newer version
                let _ = notification_sender.send(NotificationMessage::Info(
                    "You're already using the latest version.".to_string(),
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

    std::thread::spawn(move || {
        match perform_self_update(&ctx_clone, to_release, progress_tx.clone()) {
            Ok(_to_release) => {
                let _ = notification_sender.send(NotificationMessage::UpdateSuccess);
                ctx_clone.request_repaint();
            }
            Err(e) => {
                let _ =
                    progress_tx.send(UpdateProgressUpdate::Error(format!("Update failed: {e}")));
                ctx_clone.request_repaint();
            }
        }
    });
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
    #[cfg(target_os = "windows")]
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
            app.graceful_shutdown();
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
    let current_version_str = cargo_crate_version!();

    // Parse versions for proper comparison
    let current_version = Version::parse(current_version_str)?;
    let latest_version = Version::parse(&latest_release.version)?;

    // Only offer update if latest version is actually newer than current version
    if latest_version > current_version {
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
    let archive_name = archive_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Invalid archive filename")?;

    if archive_name.ends_with(".zip") {
        extract_zip(archive_path, destination)
    } else if archive_name.ends_with(".tar.gz") || archive_name.ends_with(".tgz") {
        extract_tar_gz(archive_path, destination)
    } else {
        Err(format!("Unsupported archive format: {:?}", archive_path).into())
    }
}

/// Extract zip archive contents to a directory
fn extract_zip(
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

/// Extract tar.gz archive contents to a directory
fn extract_tar_gz(
    archive_path: &std::path::Path,
    destination: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let archive_file = std::fs::File::open(archive_path)?;
    let decompressor = flate2::read::GzDecoder::new(archive_file);
    let mut archive = tar::Archive::new(decompressor);

    archive.unpack(destination)?;

    Ok(())
}

/// method to copy the complete directory `src` to `dest` but skipping the binary `binary_name`
/// since we have to use `self-replace` for that.
#[cfg(target_os = "macos")]
fn copy_bundle_without_binary(
    src: &std::path::Path,
    dest: &std::path::Path,
    binary_name: &str,
) -> std::io::Result<()> {
    // return error if source directory does not exist
    if !dest.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Destination directory does not exist: {}", dest.display()),
        ));
    }

    // Iterate through entries in the source directory
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if path.is_dir() {
            if !dest_path.exists() {
                std::fs::create_dir(&dest_path)?;
            }
            // Recursively copy subdirectories
            copy_bundle_without_binary(&path, &dest_path, binary_name)?;
        } else if let Some(file_name) = path.file_name()
            && file_name != binary_name
        {
            // Copy files except for the binary
            std::fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}

/// custom update function for use with bundles
/// taken from: https://github.com/jaemk/self_update/pull/147/files
pub fn perform_self_update(
    ctx: &Context,
    to_release: Release,
    progress_tx: mpsc::Sender<UpdateProgressUpdate>,
) -> Result<Release, Box<dyn std::error::Error>> {
    let binary_path = std::env::current_exe()?;
    let binary = binary_path
        .file_name()
        .ok_or("failed to extract current binary name")?
        .to_str()
        .ok_or("failed to convert OsStr")?;

    // get the first available release
    #[cfg(target_os = "macos")]
    let (asset, bundle_contents_dir) = {
        // get the parent directory of the `Application.app` bundle
        let contents_dir = binary_path
            .parent() // MacOS
            .ok_or("Failed to derive app bundle OS directory")?
            .parent() // Contents
            .ok_or("Failed to derive app bundle contents directory")?;
        let plist = contents_dir.join("Info.plist");

        if plist.exists() {
            (
                to_release.assets.iter().find(|asset| {
                    asset.name.contains(ARCH) && asset.name.ends_with("app.bundle.tar.gz")
                }),
                Some(contents_dir),
            )
        } else {
            (
                to_release
                    .assets
                    .iter()
                    .find(|asset| asset.name.contains(ARCH) && asset.name.ends_with("macos.zip")),
                None,
            )
        }
    };
    #[cfg(not(target_os = "macos"))]
    let asset = {
        to_release.assets.iter().find(|asset| {
            asset.name.contains(OS) && asset.name.contains(ARCH) && asset.name.ends_with(".zip")
        })
    };
    let asset = asset.ok_or("No compatible release found for the current platform")?;

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
            if let Some(contents_dir) = bundle_contents_dir {
                let app_dir = contents_dir
                    .parent() // Kiorg.app dir
                    .ok_or("Failed to derive app bundle root directory")?
                    .to_path_buf();
                let app_name = app_dir
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or("Failed to derive app name")?;

                copy_bundle_without_binary(
                    &tmp_archive_dir.path().join(app_name),
                    &app_dir,
                    binary,
                )?;
                tmp_archive_dir
                    .path()
                    .join(format!("{app_name}/Contents/MacOS/{binary}"))
            } else {
                // not an app bundle, just swap the binary
                tmp_archive_dir.path().join(binary)
            }
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
    fn test_extract_into_zip() {
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
    fn test_extract_into_tar_gz() {
        use flate2::write::GzEncoder;
        use tar::{Builder, Header};

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_tar_gz_path = temp_dir.path().join("test.tar.gz");

        // Create a test tar.gz file
        {
            let tar_gz_file = fs::File::create(&test_tar_gz_path).unwrap();
            let encoder = GzEncoder::new(tar_gz_file, flate2::Compression::default());
            let mut tar_builder = Builder::new(encoder);

            // Add a file to the tar
            let mut header = Header::new_gnu();
            header.set_path("test_file.txt").unwrap();
            header.set_size(13);
            header.set_cksum();
            tar_builder
                .append(&header, b"Hello, world!" as &[u8])
                .unwrap();

            // Add a directory and nested file
            let mut dir_header = Header::new_gnu();
            dir_header.set_path("test_dir/").unwrap();
            dir_header.set_entry_type(tar::EntryType::Directory);
            dir_header.set_size(0);
            dir_header.set_cksum();
            tar_builder.append(&dir_header, std::io::empty()).unwrap();

            let mut nested_header = Header::new_gnu();
            nested_header.set_path("test_dir/nested_file.txt").unwrap();
            nested_header.set_size(14);
            nested_header.set_cksum();
            tar_builder
                .append(&nested_header, b"Nested content" as &[u8])
                .unwrap();

            // Finish the tar builder and encoder
            let encoder = tar_builder.into_inner().unwrap();
            encoder.finish().unwrap();
        } // Drop scope to ensure file is closed

        // Create extraction destination
        let extract_dir = TempDir::new().unwrap();

        // Test the extraction
        let result = extract_into(&test_tar_gz_path, extract_dir.path());
        assert!(
            result.is_ok(),
            "Extraction should succeed, got: {:?}",
            result
        );

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
    fn test_extract_into_unsupported_format() {
        let temp_dir = TempDir::new().unwrap();
        let unsupported_file = temp_dir.path().join("test.rar");
        fs::write(&unsupported_file, b"not supported").unwrap();

        let extract_dir = TempDir::new().unwrap();
        let result = extract_into(&unsupported_file, extract_dir.path());
        assert!(result.is_err(), "Should fail for unsupported format");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported archive format")
        );
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

    #[test]
    #[cfg(target_os = "macos")]
    fn test_copy_bundle_without_binary_nested_dirs() {
        // Create source directory structure
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("source");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create nested directory structure in source
        let nested_dir = src_dir.join("nested").join("deeply").join("nested");
        std::fs::create_dir_all(&nested_dir).unwrap();

        // Create some test files
        std::fs::write(src_dir.join("root_file.txt"), "root content").unwrap();
        std::fs::write(src_dir.join("binary_to_skip"), "binary content").unwrap();
        std::fs::write(
            src_dir.join("nested").join("mid_file.txt"),
            "middle content",
        )
        .unwrap();
        std::fs::write(nested_dir.join("deep_file.txt"), "deep content").unwrap();

        // Create empty destination directory (target is empty)
        let dest_dir = temp_dir.path().join("destination");
        std::fs::create_dir_all(&dest_dir).unwrap();
        // Note: We intentionally don't create dest_dir to test directory creation

        // Test copying bundle without the binary
        let result = copy_bundle_without_binary(&src_dir, &dest_dir, "binary_to_skip");
        assert!(result.is_ok(), "Copy should succeed: {:?}", result);

        // Verify destination directory was created
        assert!(dest_dir.is_dir(), "Destination should be a directory");

        // Verify files were copied correctly
        let root_file = dest_dir.join("root_file.txt");
        assert!(root_file.exists(), "Root file should be copied");
        assert_eq!(std::fs::read_to_string(&root_file).unwrap(), "root content");

        // Verify binary was skipped
        let binary_file = dest_dir.join("binary_to_skip");
        assert!(!binary_file.exists(), "Binary file should be skipped");

        // Verify nested directories were created
        let nested_dir_dest = dest_dir.join("nested");
        assert!(
            nested_dir_dest.is_dir(),
            "Nested path should be a directory"
        );

        let deeply_nested_dest = dest_dir.join("nested").join("deeply").join("nested");
        assert!(
            deeply_nested_dest.is_dir(),
            "Deeply nested path should be a directory"
        );

        // Verify nested files were copied
        let mid_file = dest_dir.join("nested").join("mid_file.txt");
        assert!(mid_file.exists(), "Middle file should be copied");
        assert_eq!(
            std::fs::read_to_string(&mid_file).unwrap(),
            "middle content"
        );

        let deep_file = deeply_nested_dest.join("deep_file.txt");
        assert!(deep_file.exists(), "Deep file should be copied");
        assert_eq!(std::fs::read_to_string(&deep_file).unwrap(), "deep content");
    }
}
