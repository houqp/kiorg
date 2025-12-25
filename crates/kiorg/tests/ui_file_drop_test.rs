#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use kiorg::ui::popup::PopupType;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files, wait_for_condition};

/// Test that the file drop popup allows copying files to the current directory
#[test]
fn test_file_drop_popup_copy() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files in the temp directory that will be "dropped"
    let source_files = create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("subdir"),
    ]);

    // Create a nested file in the subdirectory
    let nested_file = source_files[2].join("nested.txt");
    create_test_files(std::slice::from_ref(&nested_file));

    // Create a target subdirectory where we'll "drop" the files
    let target_dir = temp_dir.path().join("target");
    std::fs::create_dir(&target_dir).unwrap();

    // Create the harness and navigate to the target directory
    let mut harness = create_harness(&temp_dir);

    // Navigate to the target directory
    harness.state_mut().navigate_to_dir(target_dir.clone());
    harness.step();

    // Initially, target directory should be empty
    let initial_count = harness.state().tab_manager.current_tab_ref().entries.len();
    assert_eq!(
        initial_count, 0,
        "Target directory should be empty initially"
    );

    // Simulate file drop by directly setting the popup state
    // This simulates what handle_file_drop would do when files are dropped
    let dropped_files = vec![
        source_files[0].clone(), // file1.txt
        source_files[1].clone(), // file2.txt
        source_files[2].clone(), // subdir
    ];

    harness.state_mut().show_popup = Some(PopupType::FileDrop(dropped_files.clone()));
    harness.step();

    // Verify the file drop popup is shown
    match &harness.state().show_popup {
        Some(PopupType::FileDrop(files)) => {
            assert_eq!(files.len(), 3, "Should have 3 dropped files");
            assert_eq!(files[0], source_files[0]);
            assert_eq!(files[1], source_files[1]);
            assert_eq!(files[2], source_files[2]);
        }
        other => panic!("File drop popup should be shown, got {other:?}"),
    }

    // Test copy operation using the CopyEntry shortcut
    harness.key_press(Key::Y); // 'Y' for copy (CopyEntry shortcut)
    harness.step();

    // Wait for the clipboard operation to complete
    wait_for_condition(|| {
        harness.step();
        // Check if popup is closed (operation completed)
        harness.state().show_popup.is_none()
    });

    // Verify the popup is closed
    assert_eq!(
        harness.state().show_popup,
        None,
        "File drop popup should be closed after copy operation"
    );

    // Verify files were copied to the target directory
    assert!(
        target_dir.join("file1.txt").exists(),
        "file1.txt should be copied"
    );
    assert!(
        target_dir.join("file2.txt").exists(),
        "file2.txt should be copied"
    );
    assert!(
        target_dir.join("subdir").exists(),
        "subdir should be copied"
    );
    assert!(
        target_dir.join("subdir").join("nested.txt").exists(),
        "nested.txt should be copied recursively"
    );

    // Verify original files still exist (copy operation)
    assert!(
        source_files[0].exists(),
        "Original file1.txt should still exist"
    );
    assert!(
        source_files[1].exists(),
        "Original file2.txt should still exist"
    );
    assert!(
        source_files[2].exists(),
        "Original subdir should still exist"
    );
    assert!(
        nested_file.exists(),
        "Original nested.txt should still exist"
    );
}

/// Test that the file drop popup allows moving files to the current directory
#[test]
fn test_file_drop_popup_move() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a source directory with test files
    let source_dir = temp_dir.path().join("source");
    std::fs::create_dir(&source_dir).unwrap();

    let source_files = create_test_files(&[
        source_dir.join("move_file1.txt"),
        source_dir.join("move_file2.txt"),
    ]);

    // Create a target directory
    let target_dir = temp_dir.path().join("target");
    std::fs::create_dir(&target_dir).unwrap();

    // Create the harness and navigate to the target directory
    let mut harness = create_harness(&temp_dir);

    // Navigate to the target directory
    harness.state_mut().navigate_to_dir(target_dir.clone());
    harness.step();

    // Initially, target directory should be empty
    let initial_count = harness.state().tab_manager.current_tab_ref().entries.len();
    assert_eq!(
        initial_count, 0,
        "Target directory should be empty initially"
    );

    // Simulate file drop by directly setting the popup state
    let dropped_files = vec![
        source_files[0].clone(), // move_file1.txt
        source_files[1].clone(), // move_file2.txt
    ];

    harness.state_mut().show_popup = Some(PopupType::FileDrop(dropped_files.clone()));
    harness.step();

    // Verify the file drop popup is shown
    match &harness.state().show_popup {
        Some(PopupType::FileDrop(files)) => {
            assert_eq!(files.len(), 2, "Should have 2 dropped files");
        }
        other => panic!("File drop popup should be shown, got {other:?}"),
    }

    // Test move operation by using the keyboard shortcut
    harness.key_press(Key::X); // 'X' for move/cut (as per our shortcut mapping)
    harness.step();

    // Wait for the clipboard operation to complete
    wait_for_condition(|| {
        harness.step();
        // Check if popup is closed (operation completed)
        harness.state().show_popup.is_none()
    });

    // Verify the popup is closed
    assert_eq!(
        harness.state().show_popup,
        None,
        "File drop popup should be closed after move operation"
    );

    // Refresh entries to ensure we see the moved files
    harness.state_mut().refresh_entries();
    harness.step();

    // Verify files were moved to the target directory
    let final_count = harness.state().tab_manager.current_tab_ref().entries.len();
    assert_eq!(
        final_count,
        initial_count + 2,
        "Should have 2 more entries after moving"
    );

    // Verify the actual files exist in the target directory
    assert!(
        target_dir.join("move_file1.txt").exists(),
        "move_file1.txt should be moved"
    );
    assert!(
        target_dir.join("move_file2.txt").exists(),
        "move_file2.txt should be moved"
    );

    // Verify original files no longer exist (move operation)
    assert!(
        !source_files[0].exists(),
        "Original move_file1.txt should no longer exist"
    );
    assert!(
        !source_files[1].exists(),
        "Original move_file2.txt should no longer exist"
    );
}

/// Test that the file drop popup can be canceled
#[test]
fn test_file_drop_popup_cancel() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a source directory with test files
    let source_dir = temp_dir.path().join("source");
    std::fs::create_dir(&source_dir).unwrap();

    let source_files = create_test_files(&[source_dir.join("cancel_file.txt")]);

    // Create a target directory
    let target_dir = temp_dir.path().join("target");
    std::fs::create_dir(&target_dir).unwrap();

    // Create the harness and navigate to the target directory
    let mut harness = create_harness(&temp_dir);

    // Navigate to the target directory
    harness.state_mut().navigate_to_dir(target_dir.clone());
    harness.step();

    // Initially, target directory should be empty
    let initial_count = harness.state().tab_manager.current_tab_ref().entries.len();
    assert_eq!(
        initial_count, 0,
        "Target directory should be empty initially"
    );

    // Simulate file drop by directly setting the popup state
    let dropped_files = vec![source_files[0].clone()];

    harness.state_mut().show_popup = Some(PopupType::FileDrop(dropped_files.clone()));
    harness.step();

    // Verify the file drop popup is shown
    match &harness.state().show_popup {
        Some(PopupType::FileDrop(_)) => {}
        other => panic!("File drop popup should be shown, got {other:?}"),
    }

    // Test cancel operation by pressing Escape
    harness.key_press(Key::Escape);
    harness.step();

    // Verify the popup is closed
    assert_eq!(
        harness.state().show_popup,
        None,
        "File drop popup should be closed after cancel"
    );

    // Verify no files were copied or moved
    let final_count = harness.state().tab_manager.current_tab_ref().entries.len();
    assert_eq!(
        final_count, initial_count,
        "Should have same number of entries after cancel"
    );

    // Verify the target directory is unchanged
    assert!(
        !target_dir.join("cancel_file.txt").exists(),
        "File should not be copied after cancel"
    );

    // Verify original file still exists
    assert!(
        source_files[0].exists(),
        "Original file should still exist after cancel"
    );
}
