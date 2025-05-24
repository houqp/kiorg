#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

/// Test that renaming a file entry doesn't reset the selected index
#[test]
fn test_rename_preserves_selected_index() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files
    let test_files = create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Move down to select file2.txt (index 1)
    harness.press_key(Key::J);
    harness.step();

    // Verify initial selection
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        1,
        "Initial selection should be at index 1 (file2.txt)"
    );

    // Press 'r' to start renaming
    harness.press_key(Key::R);
    harness.step();

    // Simulate text input for the new name
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("_renamed".to_string()));
    harness.step();

    // Press Enter to confirm rename
    harness.press_key(Key::Enter);
    harness.step();

    // Verify the file was renamed
    assert!(test_files[0].exists(), "file1.txt should still exist");
    assert!(!test_files[1].exists(), "file2.txt should no longer exist");
    assert!(
        temp_dir.path().join("file2_renamed.txt").exists(),
        "file2_renamed.txt should exist"
    );

    // Verify UI list is updated
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            !tab.entries.iter().any(|e| e.name == "file2.txt"),
            "UI entry list should not contain file2.txt after rename"
        );
        assert!(
            tab.entries.iter().any(|e| e.name == "file2_renamed.txt"),
            "UI entry list should contain file2_renamed.txt after rename"
        );
    }

    // Verify that the selected index is still 1 after renaming
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        1,
        "Selected index should still be 1 after renaming"
    );

    // Move selection up and down to ensure navigation still works properly
    harness.press_key(Key::K);
    harness.step();
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0,
        "Should be able to move selection up after rename"
    );

    harness.press_key(Key::J);
    harness.step();
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        1,
        "Should be able to move selection back down after rename"
    );
}
