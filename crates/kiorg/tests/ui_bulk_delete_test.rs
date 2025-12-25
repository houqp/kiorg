#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use kiorg::ui::popup::PopupType;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files, wait_for_condition};

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
    harness.key_press(Key::Space);
    harness.step();

    // Move to the second file
    harness.key_press(Key::J);
    harness.step();

    // Select the second file
    harness.key_press(Key::Space);
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
    harness.key_press(Key::D);
    harness.step();

    // Verify the delete popup is shown with multiple entries
    {
        let app = harness.state();
        assert!(
            matches!(app.show_popup, Some(PopupType::Delete(_, _))),
            "Delete popup should be shown"
        );
        if let Some(PopupType::Delete(_, entries)) = &app.show_popup {
            assert_eq!(
                entries.len(),
                2,
                "Two entries should be marked for deletion"
            );
        }
    }

    // Press Enter for first confirmation
    harness.key_press(Key::Enter);
    harness.step();

    // Verify we're now in the recursive confirmation state (second confirmation required)
    {
        let app = harness.state();
        if let Some(PopupType::Delete(state, _)) = &app.show_popup {
            assert_eq!(
                *state,
                kiorg::ui::popup::delete::DeleteConfirmState::RecursiveConfirm,
                "Should be in recursive confirmation state after first Enter"
            );
        } else {
            panic!("Expected Delete popup to be open");
        }

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
    harness.key_press(Key::Enter);

    wait_for_condition(|| {
        harness.step();
        harness.state().show_popup.is_none()
    });

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
