#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_bulk_delete_with_space_key() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_files = create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
        temp_dir.path().join("file4.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Initially, no files should be selected (marked)
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.marked_entries.is_empty(),
            "No entries should be selected initially"
        );
    }

    // Select the first file using space
    harness.press_key(Key::Space);
    harness.step();

    // Move to the second file
    harness.press_key(Key::J);
    harness.step();

    // Select the second file
    harness.press_key(Key::Space);
    harness.step();

    // Verify both first and second files are selected
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.marked_entries.contains(&test_files[0]),
            "First entry should be selected"
        );
        assert!(
            tab.marked_entries.contains(&test_files[1]),
            "Second entry should be selected"
        );
    }

    // Press delete key to trigger bulk deletion
    harness.press_key(Key::D);
    harness.step();

    // Verify the delete popup is shown with multiple entries
    {
        let app = harness.state();
        assert_eq!(
            app.show_popup,
            Some(kiorg::app::PopupType::Delete),
            "Delete popup should be shown"
        );
        assert_eq!(
            app.entries_to_delete.len(),
            2,
            "Two entries should be marked for deletion"
        );
    }

    // Press Enter for first confirmation
    harness.press_key(Key::Enter);
    harness.step();

    // Verify we're now in the recursive confirmation state (second confirmation required)
    {
        let app = harness.state();
        assert_eq!(
            app.delete_popup_state,
            kiorg::ui::delete_popup::DeleteConfirmState::RecursiveConfirm,
            "Should be in initial confirmation state after first Enter"
        );

        // Verify files still exist after first confirmation
        assert!(
            test_files[0].exists(),
            "file1.txt should still exist after first confirmation"
        );
        assert!(
            test_files[1].exists(),
            "file2.txt should still exist after first confirmation"
        );
    }

    // Press Enter for second confirmation
    harness.press_key(Key::Enter);
    harness.step();

    // Verify the files are deleted after second confirmation
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.marked_entries.is_empty(),
            "Marked entries should be cleared after deletion"
        );

        // Check that the files are actually deleted from the file system
        assert!(!test_files[0].exists(), "file1.txt should be deleted");
        assert!(!test_files[1].exists(), "file2.txt should be deleted");
        assert!(test_files[2].exists(), "file3.txt should still exist");
        assert!(test_files[3].exists(), "file4.txt should still exist");
    }
}
