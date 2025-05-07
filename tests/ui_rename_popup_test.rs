mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

/// Test that the rename popup works correctly
#[test]
fn test_rename_popup() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files
    let test_files = create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Ensure consistent sort order for reliable selection
    harness.ensure_sorted_by_name_ascending();

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

    // Verify the rename popup is shown
    assert_eq!(
        harness.state().show_popup,
        Some(kiorg::app::PopupType::Rename),
        "Rename popup should be open"
    );

    // Verify the new_name field is initialized with the current filename
    assert_eq!(
        harness.state().new_name,
        "file2.txt",
        "new_name should be initialized with the current filename"
    );

    // Clear existing text first
    for _ in 0..".txt".len() {
        harness.press_key(Key::Backspace);
        harness.step();
    }

    // Simulate text input for the new name
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("_renamed.txt".to_string()));
    harness.step();

    // Press Enter to confirm rename
    harness.press_key(Key::Enter);
    harness.step();

    // Add extra step to ensure rename operation completes
    harness.step();

    // Verify the popup is closed
    assert_eq!(
        harness.state().show_popup,
        None,
        "Rename popup should be closed after confirming"
    );

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

    // Test canceling the rename popup
    // Select file3.txt
    harness.press_key(Key::J);
    harness.step();

    // Press 'r' to start renaming
    harness.press_key(Key::R);
    harness.step();

    // Verify the rename popup is shown
    assert_eq!(
        harness.state().show_popup,
        Some(kiorg::app::PopupType::Rename),
        "Rename popup should be open"
    );

    // Verify the new_name field is initialized with the current filename
    assert_eq!(
        harness.state().new_name,
        "file3.txt",
        "new_name should be initialized with the current filename"
    );

    // Simulate text input for the new name
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("_should_not_rename.txt".to_string()));
    harness.step();

    // Press Escape to cancel rename
    harness.press_key(Key::Escape);
    harness.step();

    // Verify the popup is closed
    assert_eq!(
        harness.state().show_popup,
        None,
        "Rename popup should be closed after canceling"
    );

    // Verify the file was NOT renamed
    assert!(test_files[2].exists(), "file3.txt should still exist");
    assert!(
        !temp_dir.path().join("file3_should_not_rename.txt").exists(),
        "file3_should_not_rename.txt should NOT exist"
    );

    // Verify UI list is not changed
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.entries.iter().any(|e| e.name == "file3.txt"),
            "UI entry list should still contain file3.txt after canceled rename"
        );
    }
}
