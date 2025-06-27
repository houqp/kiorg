#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::{Key, Modifiers};
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

/// Integration test that uses Ctrl+A to select all current files and then deletes them
#[test]
fn test_ctrl_a_select_all_and_delete() {
    // Create a temporary directory with test files
    let temp_dir = tempdir().unwrap();
    let test_files = create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
        temp_dir.path().join("directory1"),
        temp_dir.path().join("file4.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Initially, no files should be selected
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.marked_entries.is_empty(),
            "No entries should be selected initially"
        );
    }

    // Press Ctrl+A to select all entries
    let modifiers = Modifiers {
        ctrl: true,
        ..Default::default()
    };
    harness.press_key_modifiers(modifiers, Key::A);
    harness.step();

    // Verify all entries are now selected
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(
            tab.marked_entries.len(),
            5,
            "All 5 entries should be selected after Ctrl+A"
        );

        // Check that all test files are marked
        for test_file in &test_files {
            assert!(
                tab.marked_entries.contains(test_file),
                "File {test_file:?} should be selected"
            );
        }
    }

    // Press 'd' to trigger delete operation
    harness.press_key(Key::D);
    harness.step();

    // Verify the delete popup is shown with all entries
    {
        let app = harness.state();
        assert!(
            matches!(app.show_popup, Some(kiorg::app::PopupType::Delete(_))),
            "Delete popup should be shown"
        );
        assert_eq!(
            app.entries_to_delete.len(),
            5,
            "All 5 entries should be marked for deletion"
        );
    }

    // Confirm the first deletion prompt
    harness.press_key(Key::Enter);
    harness.step();

    // Verify we're in the recursive confirmation state
    {
        let app = harness.state();
        if let Some(kiorg::app::PopupType::Delete(state)) = &app.show_popup {
            assert_eq!(
                *state,
                kiorg::ui::delete_popup::DeleteConfirmState::RecursiveConfirm,
                "Should be in recursive confirmation state after first Enter"
            );
        } else {
            panic!("Expected Delete popup to be open");
        }

        // Verify files still exist after first confirmation
        for test_file in &test_files {
            assert!(
                test_file.exists(),
                "File {test_file:?} should still exist after first confirmation"
            );
        }
    }

    // Confirm the second deletion prompt
    harness.press_key(Key::Enter);
    harness.step();

    // Give time for deletion to process (deletion happens asynchronously)
    for _ in 0..10 {
        harness.step();
        let app = harness.state();
        if app.show_popup.is_none() {
            break;
        }
    }

    // Verify the delete popup is closed
    {
        let app = harness.state();
        assert!(
            app.show_popup.is_none(),
            "Delete popup should be closed after deletion completes"
        );
    }

    // Verify all files have been deleted
    for test_file in &test_files {
        assert!(
            !test_file.exists(),
            "File {test_file:?} should be deleted after confirmation"
        );
    }

    // Verify no entries are selected after deletion
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.marked_entries.is_empty(),
            "No entries should be selected after deletion"
        );
    }

    // Verify the directory is now empty (except for potential hidden files)
    let remaining_entries: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            // Filter out hidden files that might be created by the system
            if name.starts_with('.') {
                None
            } else {
                Some(name)
            }
        })
        .collect();

    assert!(
        remaining_entries.is_empty(),
        "Directory should be empty after deleting all files, but found: {remaining_entries:?}"
    );
}
