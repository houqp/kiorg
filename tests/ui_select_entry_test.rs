mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_select_entry_shortcut() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_files = create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Ensure consistent sort order for reliable selection
    harness.ensure_sorted_by_name_ascending();

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

    // Verify the first file is now selected
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.marked_entries.contains(&test_files[0]),
            "First entry should be selected after pressing Space"
        );
        assert_eq!(
            tab.entries[0].path, test_files[0],
            "Selected entry should be file1.txt"
        );
    }

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
            "First entry should still be selected"
        );
        assert!(
            tab.marked_entries.contains(&test_files[1]),
            "Second entry should now be selected"
        );
    }

    // Deselect the second file by pressing space again
    harness.press_key(Key::Space);
    harness.step();

    // Verify only the first file is selected
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.marked_entries.contains(&test_files[0]),
            "First entry should still be selected"
        );
        assert!(
            !tab.marked_entries.contains(&test_files[1]),
            "Second entry should be deselected"
        );
    }

    // Move to the third file
    harness.press_key(Key::J);
    harness.step();

    // Select the third file
    harness.press_key(Key::Space);
    harness.step();

    // Verify first and third files are selected
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.marked_entries.contains(&test_files[0]),
            "First entry should still be selected"
        );
        assert!(
            !tab.marked_entries.contains(&test_files[1]),
            "Second entry should not be selected"
        );
        assert!(
            tab.marked_entries.contains(&test_files[2]),
            "Third entry should be selected"
        );
    }
}

#[test]
fn test_select_entry_with_operations() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_files = create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Ensure consistent sort order for reliable selection
    harness.ensure_sorted_by_name_ascending();

    // Select both files
    {
        // Select first file (dir1)
        harness.press_key(Key::Space);
        harness.step();

        // Move to second file (file1.txt)
        harness.press_key(Key::J);
        harness.step();

        // Select second file
        harness.press_key(Key::Space);
        harness.step();

        // Move to third file (file2.txt)
        harness.press_key(Key::J);
        harness.step();

        // Select third file
        harness.press_key(Key::Space);
        harness.step();
    }

    // Verify all three entries are selected
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.entries
                .iter()
                .all(|e| tab.marked_entries.contains(&e.path)),
            "All entries should be selected"
        );
    }

    // Copy the selected entries
    harness.press_key(Key::Y);
    harness.step();

    // Create a subdirectory to paste into
    let subdir_path = temp_dir.path().join("subdir");
    std::fs::create_dir(&subdir_path).unwrap();
    harness.state_mut().refresh_entries();
    harness.step();

    // Navigate to the subdirectory
    {
        // Find the index of the subdirectory
        let tab = harness.state().tab_manager.current_tab_ref();
        let subdir_index = tab
            .entries
            .iter()
            .position(|e| e.path == subdir_path)
            .expect("Subdirectory should be in the entries");

        // Select the subdirectory
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = subdir_index;
    }
    harness.step();

    // Navigate into the subdirectory
    harness.press_key(Key::L);
    harness.step();

    // Paste the selected entries
    harness.press_key(Key::P);
    harness.step();

    // Verify all three entries were copied to the subdirectory
    assert!(
        subdir_path.join("dir1").exists(),
        "dir1 should be copied to subdirectory"
    );
    assert!(
        subdir_path.join("file1.txt").exists(),
        "file1.txt should be copied to subdirectory"
    );
    assert!(
        subdir_path.join("file2.txt").exists(),
        "file2.txt should be copied to subdirectory"
    );

    // Verify the original entries still exist
    assert!(test_files[0].exists(), "Original dir1 should still exist");
    assert!(
        test_files[1].exists(),
        "Original file1.txt should still exist"
    );
    assert!(
        test_files[2].exists(),
        "Original file2.txt should still exist"
    );
}
