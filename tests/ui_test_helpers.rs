#![allow(dead_code)] // Allow unused code for helpers

use egui_kittest::Harness;
use kiorg::Kiorg;
use std::fs::File;
use std::path::PathBuf;
use tempfile::tempdir;

/// Create files and directories from a list of paths.
/// Returns the created paths.
pub fn create_test_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    for path in paths {
        if path.extension().is_some() {
            File::create(path).unwrap();
        } else {
            std::fs::create_dir(path).unwrap();
        }
        assert!(path.exists());
    }
    paths.to_vec()
}

/// Create a test image file with minimal valid PNG content
pub fn create_test_image(path: &PathBuf) -> PathBuf {
    // Minimal valid PNG file (1x1 transparent pixel)
    let png_data: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    std::fs::write(path, png_data).unwrap();
    assert!(path.exists());
    path.clone()
}

/// Create a test zip file with some entries
pub fn create_test_zip(path: &PathBuf) -> PathBuf {
    use std::io::Write;
    use zip::write::FileOptions;

    let file = std::fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);

    // Use explicit type annotation to fix the compiler error
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755) as FileOptions<'_, ()>;

    // Add a few entries to the zip
    zip.start_file("file1.txt", options).unwrap();
    zip.write_all(b"Content of file1.txt").unwrap();

    zip.start_file("file2.txt", options).unwrap();
    zip.write_all(b"Content of file2.txt").unwrap();

    // Add a directory
    zip.add_directory("subdir", options).unwrap();

    // Add a file in the subdirectory
    zip.start_file("subdir/file3.txt", options).unwrap();
    zip.write_all(b"Content of file3.txt in subdir").unwrap();

    zip.finish().unwrap();
    assert!(path.exists());
    path.clone()
}

// Wrapper to hold both the harness and the config temp directory to prevent premature cleanup
pub struct TestHarness<'a> {
    pub harness: Harness<'a, Kiorg>,
    _config_temp_dir: tempfile::TempDir, // Prefixed with _ to indicate it's only kept for its Drop behavior
}

pub fn create_harness<'a>(temp_dir: &tempfile::TempDir) -> TestHarness<'a> {
    // Create a separate temporary directory for config files
    let config_temp_dir = tempdir().unwrap();
    let test_config_dir = config_temp_dir.path().to_path_buf();
    std::fs::create_dir_all(&test_config_dir).unwrap();

    // Create a new egui context
    let ctx = egui::Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx.clone());

    // Create the app with the test config directory override
    let app = Kiorg::new_with_config_dir(
        &cc,
        Some(temp_dir.path().to_path_buf()),
        Some(test_config_dir),
    );

    // Create a test harness with more steps to ensure all events are processed
    let harness = Harness::builder()
        .with_size(egui::Vec2::new(800.0, 600.0))
        .with_max_steps(20)
        .build_eframe(|_cc| app);

    // Run one step to initialize the app
    let mut harness = harness;
    harness.step();

    TestHarness {
        harness,
        _config_temp_dir: config_temp_dir,
    }
}

impl<'a> TestHarness<'a> {
    /// Ensures the current tab's entries are sorted by Name/Ascending.
    pub fn ensure_sorted_by_name_ascending(&mut self) {
        // Toggle twice on the TabManager to ensure Ascending order regardless of the initial state
        self.harness
            .state_mut()
            .tab_manager
            .toggle_sort(kiorg::models::tab::SortColumn::Name); // Sets Name/Descending or None
        self.harness
            .state_mut()
            .tab_manager
            .toggle_sort(kiorg::models::tab::SortColumn::Name); // Sets Name/Ascending
                                                                // sort_all_tabs is called implicitly by toggle_sort now, no need for explicit call
        self.harness.step(); // Allow sort to apply and UI to update
    }
}

// Add methods to TestHarness to delegate to the inner harness
impl<'a> std::ops::Deref for TestHarness<'a> {
    type Target = Harness<'a, Kiorg>;

    fn deref(&self) -> &Self::Target {
        &self.harness
    }
}

impl std::ops::DerefMut for TestHarness<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.harness
    }
}
