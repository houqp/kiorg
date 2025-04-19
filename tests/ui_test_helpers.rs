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
    let app = Kiorg::new_with_config_dir(&cc, temp_dir.path().to_path_buf(), Some(test_config_dir));

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
