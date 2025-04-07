use egui::Key;
use egui_kittest::Harness;
use kiorg::Kiorg;
use std::fs::File;
use std::path::PathBuf;
use tempfile::tempdir;

/// Create files and directories from a list of paths.
/// Returns the created paths.
fn create_test_files(paths: &[PathBuf]) -> Vec<PathBuf> {
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
struct TestHarness<'a> {
    harness: Harness<'a, Kiorg>,
    _config_temp_dir: tempfile::TempDir, // Prefixed with _ to indicate it's only kept for its Drop behavior
}

fn create_harness<'a>(temp_dir: &tempfile::TempDir) -> TestHarness<'a> {
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

#[test]
fn test_delete_shortcut() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_files = create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("dir2"),
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);

    // Create a file inside dir2 to test non-empty directory deletion
    let nested_file = test_files[1].join("nested.txt");
    create_test_files(&[nested_file.clone()]);

    // Create files inside dir1 to test recursive deletion
    let dir1_files = create_test_files(&[
        test_files[0].join("file1.txt"),
        test_files[0].join("file2.txt"),
        test_files[0].join("subdir"),
    ]);

    // Create a file inside the subdirectory of dir1
    let subdir_file = dir1_files[2].join("subfile.txt");
    create_test_files(&[subdir_file.clone()]);

    let mut harness = create_harness(&temp_dir);

    // Test file deletion first
    // Move down twice to select test1.txt (after dir1 and dir2)
    harness.press_key(Key::J);
    harness.step();
    harness.press_key(Key::J);
    harness.step();

    // Simulate pressing 'd' key to delete test1.txt
    harness.press_key(Key::D);
    harness.step();

    // Simulate pressing Enter to confirm deletion
    harness.press_key(Key::Enter);
    harness.step();

    // Verify only test1.txt was deleted
    assert!(!test_files[2].exists(), "test1.txt should be deleted");
    assert!(test_files[0].exists(), "dir1 should still exist");
    assert!(test_files[1].exists(), "dir2 should still exist");
    assert!(test_files[3].exists(), "test2.txt should still exist");

    // Test recursive directory deletion
    // First entry should be dir1
    // Delete dir1 (directory with nested files and subdirectory)
    harness.press_key(Key::D);
    harness.step();
    harness.press_key(Key::Enter);
    harness.step();

    // Verify dir1 and all its contents were deleted recursively
    assert!(!test_files[0].exists(), "dir1 should be deleted");
    assert!(!dir1_files[0].exists(), "file1.txt should be deleted");
    assert!(!dir1_files[1].exists(), "file2.txt should be deleted");
    assert!(!dir1_files[2].exists(), "subdir should be deleted");
    assert!(!subdir_file.exists(), "subfile.txt should be deleted");
    assert!(test_files[1].exists(), "dir2 should still exist");
    assert!(test_files[3].exists(), "test2.txt should still exist");
}

#[test]
fn test_rename_shortcut() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files
    let test_files = create_test_files(&[
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // move down to test2.txt
    harness.press_key(Key::J);
    harness.step();

    // Press 'r' to start renaming
    harness.press_key(Key::R);
    harness.step();

    // Press 'delete' to clear any existing text
    for _ in 0..".txt".len() {
        harness.press_key(Key::Backspace);
        harness.step();
    }

    // Clear any existing text and simulate text input for the new name
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("_renamed.txt".to_string()));
    harness.step();

    // Press Enter to confirm rename
    harness.press_key(Key::Enter);
    harness.step();

    // Verify the file was renamed
    assert!(test_files[0].exists(), "test1.txt should still exist");
    assert!(!test_files[1].exists(), "test2.txt should no longer exist");
    assert!(
        temp_dir.path().join("test2_renamed.txt").exists(),
        "test2_renamed.txt should exist"
    );
}

#[test]
fn test_copy_paste_shortcuts() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files and directories
    let test_files = create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("dir2"),
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Move down twice to select test1.txt (after dir1 and dir2)
    harness.press_key(Key::J);
    harness.step();
    harness.press_key(Key::J);
    harness.step();

    // Copy test1.txt
    harness.press_key(Key::Y);
    harness.step();

    // Move up to select dir2
    harness.press_key(Key::K);
    harness.step();

    // Navigate into dir2
    harness.press_key(Key::L);
    harness.step();

    // Paste the file
    harness.press_key(Key::P);
    harness.step();

    // Verify the file was copied to dir2 while original remains
    assert!(
        test_files[2].exists(),
        "test1.txt should still exist in original location"
    );
    assert!(
        test_files[1].join("test1.txt").exists(),
        "test1.txt should be copied to dir2"
    );
}

#[test]
fn test_copy_paste_same_directory() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files
    let test_files = create_test_files(&[
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Make sure we're selecting the first file (test1.txt)
    let tab = harness.state_mut().tab_manager.current_tab();
    tab.selected_index = 0;

    // Copy test1.txt
    harness.press_key(Key::Y);
    harness.step();

    // Paste in the same directory
    harness.press_key(Key::P);
    harness.step();

    // Verify the file was copied with a new suffix
    assert!(test_files[0].exists(), "test1.txt should still exist");
    assert!(test_files[1].exists(), "test2.txt should still exist");

    // Check for the copied file with a new suffix
    let copied_file = temp_dir.path().join("test1_1.txt");
    assert!(
        copied_file.exists(),
        "test1.txt should be copied with suffix `_1`"
    );
}

#[test]
fn test_cut_paste_shortcuts() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files and directories
    let test_files = create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("dir2"),
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Move down twice to select test1.txt (after dir1 and dir2)
    harness.press_key(Key::J);
    harness.step();
    harness.press_key(Key::J);
    harness.step();

    // Cut test1.txt
    harness.press_key(Key::X);
    harness.step();

    // Verify the file still exists in the original location
    assert!(
        test_files[2].exists(),
        "test1.txt should still exist after cutting"
    );

    // Move up to select dir2
    harness.press_key(Key::K);
    harness.step();

    // Navigate into dir2
    harness.press_key(Key::L);
    harness.step();

    // Paste the file
    harness.press_key(Key::P);
    harness.step();

    // Verify the file was moved to dir2
    assert!(
        !test_files[2].exists(),
        "test1.txt should be moved from original location"
    );
    assert!(
        test_files[1].join("test1.txt").exists(),
        "test1.txt should exist in dir2"
    );
}

#[test]
fn test_g_shortcuts() {
    // Create test files and directories
    let temp_dir = tempdir().unwrap();
    create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("dir2"),
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);
    let mut harness = create_harness(&temp_dir);

    let tab = harness.state().tab_manager.current_tab_ref();
    assert_eq!(tab.selected_index, 0);

    // Test G shortcut (go to last entry)
    {
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        harness.press_key_modifiers(modifiers, Key::G);
        harness.step();
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, tab.entries.len() - 1);
    }

    // a single g press doesn't move selection
    {
        harness.press_key(Key::G);
        harness.step();
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, tab.entries.len() - 1);
    }

    // Test gg shortcut (go to first entry)
    {
        // First g press
        harness.press_key(Key::G);
        // Second g press should go back to the top
        harness.press_key(Key::G);
        harness.step();
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, 0);
    }
}

#[test]
fn test_g_shortcuts_empty_list() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Clear entries
    {
        let tab = harness.state_mut().tab_manager.current_tab();
        tab.entries.clear();
    }

    // Test G shortcut with empty list
    {
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        harness.press_key_modifiers(modifiers, Key::G);
        harness.step();
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, 0); // Should stay at 0
    }

    // Test gg shortcut with empty list
    {
        // First g press
        harness.press_key(Key::G);
        // Second g press
        harness.press_key(Key::G);
        harness.step();
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, 0); // Should stay at 0
    }
}

#[test]
fn test_bookmark_feature() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test directories and files
    create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("dir2"),
        temp_dir.path().join("test1.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Check initial state - no bookmarks
    harness.step();
    assert!(harness.state().bookmarks.is_empty());
    assert!(!harness.state().show_bookmarks);

    // Select the first directory
    {
        let tab = harness.state_mut().tab_manager.current_tab();
        tab.selected_index = 0; // Select dir1
    }
    harness.step();

    // Bookmark the directory with 'b'
    harness.press_key(Key::B);
    harness.step();

    // Verify bookmark was added
    {
        let app = harness.state();
        assert_eq!(app.bookmarks.len(), 1);
        assert!(app.bookmarks[0].ends_with("dir1"));
    }

    // Open bookmark popup with 'B' (shift+b)
    {
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        harness.press_key_modifiers(modifiers, Key::B);
        harness.step();
    }

    // Verify bookmark popup is shown
    assert!(harness.state().show_bookmarks);

    // Close bookmark popup with 'q'
    {
        harness.press_key(Key::Q);
        harness.step();
    }

    // Verify bookmark popup is closed
    assert!(!harness.state().show_bookmarks);

    // Select the second directory
    {
        let tab = harness.state_mut().tab_manager.current_tab();
        tab.selected_index = 1; // Select dir2
    }
    harness.step();

    // Bookmark the second directory
    harness.press_key(Key::B);
    harness.step();

    // Verify second bookmark was added
    {
        let app = harness.state();
        assert_eq!(app.bookmarks.len(), 2);
        assert!(app.bookmarks[1].ends_with("dir2"));
    }

    // Try to bookmark a file (should not work)
    {
        let tab = harness.state_mut().tab_manager.current_tab();
        tab.selected_index = 2; // Select test1.txt
    }
    harness.press_key(Key::B);
    harness.step();

    // Verify no new bookmark was added (still 2)
    assert_eq!(harness.state().bookmarks.len(), 2);

    // Open bookmark popup again
    {
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        harness.press_key_modifiers(modifiers, Key::B);
        harness.step();
    }

    // Delete the first bookmark with 'd'
    harness.press_key(Key::D);
    harness.step();

    // Verify bookmark was removed
    {
        let app = harness.state();
        assert_eq!(app.bookmarks.len(), 1);
        assert!(app.bookmarks[0].ends_with("dir2")); // Only dir2 remains
    }

    // Close bookmark popup with 'q'
    harness.press_key(Key::Q);
    harness.step();

    // Verify bookmark popup is closed
    assert!(!harness.state().show_bookmarks);
}

#[test]
fn test_parent_directory_selection() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files and directories
    let test_files = create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("dir2"),
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Move down to select dir2
    harness.press_key(Key::J);
    harness.step();

    // Navigate into dir2
    harness.press_key(Key::L);
    harness.step();

    // Create a file in dir2
    let dir2_file = test_files[1].join("dir2_file.txt");
    std::fs::File::create(&dir2_file).unwrap();

    // Move down to select dir2_file.txt
    harness.press_key(Key::J);
    harness.step();

    // Navigate to parent directory
    harness.press_key(Key::H);
    harness.step();

    // Verify that dir2 is still selected
    let tab = harness.state().tab_manager.current_tab_ref();
    assert_eq!(
        tab.entries[tab.selected_index].path, test_files[1],
        "dir2 should be selected after navigating to parent directory"
    );
}

#[test]
fn test_prev_path_selection_with_sort() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test directories (order matters for initial selection)
    let test_dirs = create_test_files(&[
        temp_dir.path().join("aaa"), // index 0
        temp_dir.path().join("bbb"), // index 1
        temp_dir.path().join("ccc"), // index 2
    ]);

    // Start the harness in the parent directory
    let mut harness = create_harness(&temp_dir);

    // Initial state: aaa, bbb, ccc (default sort)
    // Select ccc (index 2)
    harness.press_key(Key::J);
    harness.step();
    harness.press_key(Key::J);
    harness.step();
    assert_eq!(harness.state().tab_manager.current_tab_ref().selected_index, 2);

    // Navigate into bbb
    harness.press_key(Key::L);
    harness.step();
    assert!(harness.state().tab_manager.current_tab_ref().current_path.ends_with("ccc"));

    // Manually set sort order to Descending Name *while inside bbb*
    // (Simulating header click is complex, direct state change is acceptable here)
    {
        let tab = harness.state_mut().tab_manager.current_tab();
        tab.toggle_sort(kiorg::models::tab::SortColumn::Name); // Sets Name/Descending
        assert_eq!(tab.sort_column, kiorg::models::tab::SortColumn::Name);
        assert_eq!(tab.sort_order, kiorg::models::tab::SortOrder::Descending);
    }
    harness.step(); // Allow state update propagation if needed

    // Navigate back up to the parent directory
    harness.press_key(Key::H);
    harness.step();

    // Now in the parent directory, refresh_entries should have run:
    // 1. Entries read: [aaa, bbb, ccc]
    // 2. Sort applied (Name/Descending): [ccc, bbb, aaa]
    // 3. prev_path (bbb) searched in sorted list
    // 4. selected_index should be 1 (pointing to bbb)

    // Verify the state in the parent directory
    let tab = harness.state().tab_manager.current_tab_ref();
    assert_eq!(tab.current_path, temp_dir.path(), "Current path should be the parent");
    assert_eq!(tab.entries.len(), 3, "Should have 3 entries");

    // Check sorted order
    assert_eq!(tab.entries[0].name, "ccc", "First entry should be ccc");
    assert_eq!(tab.entries[1].name, "bbb", "Second entry should be bbb");
    assert_eq!(tab.entries[2].name, "aaa", "Third entry should be aaa");

    // Check selected index based on prev_path (bbb)
    assert_eq!(tab.selected_index, 0, "Selected index should point to ccc");
    assert_eq!(tab.entries[tab.selected_index].path, test_dirs[2], "Selected entry should be ccc");
}
