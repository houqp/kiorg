mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_folder_delete_double_confirmation() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    
    // Create a test directory with some files inside to test recursive deletion
    let test_files = create_test_files(&[
        temp_dir.path().join("test_dir"),
        temp_dir.path().join("test_file.txt"),
    ]);
    
    // Create files inside the test directory to ensure it's not empty
    create_test_files(&[
        test_files[0].join("file1.txt"),
        test_files[0].join("file2.txt"),
    ]);
    
    let mut harness = create_harness(&temp_dir);
    
    // Ensure consistent sort order for reliable selection
    harness.ensure_sorted_by_name_ascending();
    
    // Verify initial state
    assert!(test_files[0].exists(), "Test directory should exist initially");
    assert!(test_files[1].exists(), "Test file should exist initially");
    
    // Select the directory (should be the first entry)
    let tab = harness.state_mut().tab_manager.current_tab_mut();
    tab.selected_index = 0;
    harness.step();
    
    // Press 'd' to initiate deletion
    harness.press_key(Key::D);
    harness.step();
    
    // Verify delete popup is shown
    assert_eq!(
        harness.state().show_popup,
        Some(kiorg::app::PopupType::Delete),
        "Delete popup should be open"
    );
    
    // Verify we're in the initial confirmation state
    assert_eq!(
        harness.state().delete_popup_state,
        kiorg::ui::delete_popup::DeleteConfirmState::Initial,
        "Should be in initial confirmation state"
    );
    
    // Press Enter for first confirmation
    harness.press_key(Key::Enter);
    harness.step();
    
    // Verify we're now in the recursive confirmation state
    assert_eq!(
        harness.state().delete_popup_state,
        kiorg::ui::delete_popup::DeleteConfirmState::RecursiveConfirm,
        "Should be in recursive confirmation state after first Enter"
    );
    
    // Verify directory still exists after first confirmation
    assert!(test_files[0].exists(), "Directory should still exist after first confirmation");
    
    // Press Enter for second confirmation
    harness.press_key(Key::Enter);
    harness.step();
    
    // Verify popup is closed
    assert_eq!(
        harness.state().show_popup,
        None,
        "Delete popup should be closed after second confirmation"
    );
    
    // Verify directory is deleted
    assert!(!test_files[0].exists(), "Directory should be deleted after second confirmation");
    assert!(test_files[1].exists(), "Test file should still exist");
}

#[test]
fn test_folder_delete_cancel_first_confirmation() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    
    // Create a test directory
    let test_files = create_test_files(&[
        temp_dir.path().join("test_dir"),
    ]);
    
    let mut harness = create_harness(&temp_dir);
    
    // Ensure consistent sort order for reliable selection
    harness.ensure_sorted_by_name_ascending();
    
    // Select the directory (should be the first entry)
    let tab = harness.state_mut().tab_manager.current_tab_mut();
    tab.selected_index = 0;
    harness.step();
    
    // Press 'd' to initiate deletion
    harness.press_key(Key::D);
    harness.step();
    
    // Verify delete popup is shown
    assert_eq!(
        harness.state().show_popup,
        Some(kiorg::app::PopupType::Delete),
        "Delete popup should be open"
    );
    
    // Press Escape to cancel
    harness.press_key(Key::Escape);
    harness.step();
    
    // Verify popup is closed
    assert_eq!(
        harness.state().show_popup,
        None,
        "Delete popup should be closed after Escape"
    );
    
    // Verify directory still exists
    assert!(test_files[0].exists(), "Directory should still exist after cancellation");
}

#[test]
fn test_folder_delete_cancel_second_confirmation() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    
    // Create a test directory
    let test_files = create_test_files(&[
        temp_dir.path().join("test_dir"),
    ]);
    
    let mut harness = create_harness(&temp_dir);
    
    // Ensure consistent sort order for reliable selection
    harness.ensure_sorted_by_name_ascending();
    
    // Select the directory (should be the first entry)
    let tab = harness.state_mut().tab_manager.current_tab_mut();
    tab.selected_index = 0;
    harness.step();
    
    // Press 'd' to initiate deletion
    harness.press_key(Key::D);
    harness.step();
    
    // Press Enter for first confirmation
    harness.press_key(Key::Enter);
    harness.step();
    
    // Verify we're in the recursive confirmation state
    assert_eq!(
        harness.state().delete_popup_state,
        kiorg::ui::delete_popup::DeleteConfirmState::RecursiveConfirm,
        "Should be in recursive confirmation state after first Enter"
    );
    
    // Press Escape to cancel
    harness.press_key(Key::Escape);
    harness.step();
    
    // Verify popup is closed
    assert_eq!(
        harness.state().show_popup,
        None,
        "Delete popup should be closed after Escape"
    );
    
    // Verify directory still exists
    assert!(test_files[0].exists(), "Directory should still exist after cancellation");
}
