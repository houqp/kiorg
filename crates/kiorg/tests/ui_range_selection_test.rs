#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_range_selection_toggle() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_files = create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
        temp_dir.path().join("file4.txt"),
        temp_dir.path().join("file5.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Initially, range selection should be inactive
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            !tab.is_range_selection_active(),
            "Range selection should be inactive initially"
        );
    }

    // Press 'v' to enter range selection mode
    harness.key_press(Key::V);
    harness.step();

    // Range selection should now be active
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.is_range_selection_active(),
            "Range selection should be active after pressing 'v'"
        );

        // Should start at current selection (index 0)
        assert_eq!(
            tab.range_selection_start,
            Some(0),
            "Range selection should start at current selection index"
        );

        // Range should be (0, 0) since we haven't moved
        assert_eq!(
            tab.get_range_selection_range(),
            Some((0, 0)),
            "Range selection range should be (0, 0) initially"
        );
    }

    // Move down to expand the selection
    harness.key_press(Key::J);
    harness.step();
    harness.key_press(Key::J);
    harness.step();

    // Check the range selection range
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.is_range_selection_active(),
            "Range selection should still be active"
        );

        // Range should now be (0, 2)
        assert_eq!(
            tab.get_range_selection_range(),
            Some((0, 2)),
            "Range selection range should be (0, 2) after moving down twice"
        );

        // Check that the range selection includes the right entries
        let selected_entries = tab
            .get_range_selected_entries()
            .expect("Should have range selection");
        assert_eq!(
            selected_entries.len(),
            3,
            "Should have 3 entries in range selection"
        );
        assert_eq!(
            selected_entries[0].meta.path, test_files[0],
            "First entry should be file1.txt"
        );
        assert_eq!(
            selected_entries[1].meta.path, test_files[1],
            "Second entry should be file2.txt"
        );
        assert_eq!(
            selected_entries[2].meta.path, test_files[2],
            "Third entry should be file3.txt"
        );
    }

    // Press 'v' again to exit range selection mode
    harness.key_press(Key::V);
    harness.step();

    // Range selection should now be inactive
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            !tab.is_range_selection_active(),
            "Range selection should be inactive after pressing 'v' again"
        );

        assert_eq!(
            tab.range_selection_start, None,
            "Range selection start should be None when inactive"
        );

        assert_eq!(
            tab.get_range_selection_range(),
            None,
            "Range selection range should be None when inactive"
        );
    }
}

#[test]
fn test_range_selection_with_operations() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_files = create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
        temp_dir.path().join("file4.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Enter visual bulk selection mode and select files 1-3
    harness.key_press(Key::V);
    harness.step();
    harness.key_press(Key::J);
    harness.step();
    harness.key_press(Key::J);
    harness.step();

    // Copy the range selection
    harness.key_press(Key::Y);
    harness.step();

    // Check that range selection mode is exited and marked entries are set
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            !tab.is_range_selection_active(),
            "Range selection should be inactive after copy operation"
        );

        // Check that the files are now in marked entries
        assert_eq!(
            tab.marked_entries.len(),
            3,
            "Should have 3 marked entries after copy operation"
        );
        assert!(
            tab.marked_entries.contains(&test_files[0]),
            "file1.txt should be marked"
        );
        assert!(
            tab.marked_entries.contains(&test_files[1]),
            "file2.txt should be marked"
        );
        assert!(
            tab.marked_entries.contains(&test_files[2]),
            "file3.txt should be marked"
        );
    }

    // Check that the clipboard contains the right files
    {
        let clipboard = harness.state().clipboard.as_ref();
        assert!(clipboard.is_some(), "Clipboard should not be empty");

        if let Some(kiorg::app::Clipboard::Copy(paths)) = clipboard {
            assert_eq!(paths.len(), 3, "Clipboard should contain 3 files");
            assert!(
                paths.contains(&test_files[0]),
                "Clipboard should contain file1.txt"
            );
            assert!(
                paths.contains(&test_files[1]),
                "Clipboard should contain file2.txt"
            );
            assert!(
                paths.contains(&test_files[2]),
                "Clipboard should contain file3.txt"
            );
        } else {
            panic!("Clipboard should be a Copy operation");
        }
    }
}

#[test]
fn test_range_selection_backwards() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_files = create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
        temp_dir.path().join("file4.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Move to the third file (index 2)
    harness.key_press(Key::J);
    harness.step();
    harness.key_press(Key::J);
    harness.step();

    // Enter range selection mode
    harness.key_press(Key::V);
    harness.step();

    // Move up to create a backwards selection
    harness.key_press(Key::K);
    harness.step();

    // Check the range selection range (should be normalized)
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.is_range_selection_active(),
            "Range selection should be active"
        );

        // Range should be (1, 2) - the function should normalize start > end
        assert_eq!(
            tab.get_range_selection_range(),
            Some((1, 2)),
            "Range selection range should be normalized to (1, 2)"
        );

        // Check that the range selection includes the right entries
        let selected_entries = tab
            .get_range_selected_entries()
            .expect("Should have range selection");
        assert_eq!(
            selected_entries.len(),
            2,
            "Should have 2 entries in range selection"
        );
        assert_eq!(
            selected_entries[0].meta.path, test_files[1],
            "First entry should be file2.txt"
        );
        assert_eq!(
            selected_entries[1].meta.path, test_files[2],
            "Second entry should be file3.txt"
        );
    }
}

#[test]
fn test_range_selection_with_goto_last_entry() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_files = create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
        temp_dir.path().join("file4.txt"),
        temp_dir.path().join("file5.txt"),
        temp_dir.path().join("file6.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Start at the first file (index 0)
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, 0, "Should start at index 0");
    }

    // Enter range selection mode
    harness.key_press(Key::V);
    harness.step();

    // Verify range selection is active and starts at index 0
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.is_range_selection_active(),
            "Range selection should be active"
        );
        assert_eq!(
            tab.range_selection_start,
            Some(0),
            "Range selection should start at index 0"
        );
        assert_eq!(
            tab.get_range_selection_range(),
            Some((0, 0)),
            "Range should initially be (0, 0)"
        );
    }

    // Use Shift+G to jump to the last entry (this should expand the range selection)
    harness.key_press_modifiers(egui::Modifiers::SHIFT, Key::G);
    harness.step();

    // Verify the selection jumped to the last entry and range expanded
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(
            tab.selected_index,
            test_files.len() - 1,
            "Should be at the last file index"
        );

        assert!(
            tab.is_range_selection_active(),
            "Range selection should still be active"
        );

        // Range should now span from 0 to the last index
        assert_eq!(
            tab.get_range_selection_range(),
            Some((0, test_files.len() - 1)),
            "Range should span from first to last entry"
        );

        // Check that all entries are in the range selection
        let selected_entries = tab
            .get_range_selected_entries()
            .expect("Should have range selection");
        assert_eq!(
            selected_entries.len(),
            test_files.len(),
            "All entries should be in range selection"
        );

        // Verify all files are included in order
        for (i, entry) in selected_entries.iter().enumerate() {
            assert_eq!(
                entry.meta.path, test_files[i],
                "Entry at index {i} should match expected file"
            );
        }
    }

    // Now test going in reverse: start from last entry and use gg to go to first
    // First, exit range selection mode
    harness.key_press(Key::V);
    harness.step();

    // Verify range selection is now inactive
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            !tab.is_range_selection_active(),
            "Range selection should be inactive"
        );
    }

    // Enter range selection mode again (should start at current position - last entry)
    harness.key_press(Key::V);
    harness.step();

    // Verify range selection starts at the last entry
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.is_range_selection_active(),
            "Range selection should be active again"
        );
        assert_eq!(
            tab.range_selection_start,
            Some(test_files.len() - 1),
            "Range selection should start at last index"
        );
    }

    // Use gg (double g) to jump to the first entry
    harness.key_press(Key::G);
    harness.step();
    harness.key_press(Key::G);
    harness.step();

    // Verify the selection jumped to the first entry and range expanded backwards
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(
            tab.selected_index, 0,
            "Should be back at the first file index"
        );

        assert!(
            tab.is_range_selection_active(),
            "Range selection should still be active"
        );

        // Range should span from first to last (normalized)
        assert_eq!(
            tab.get_range_selection_range(),
            Some((0, test_files.len() - 1)),
            "Range should span from first to last entry (normalized)"
        );

        // Check that all entries are still in the range selection
        let selected_entries = tab
            .get_range_selected_entries()
            .expect("Should have range selection");
        assert_eq!(
            selected_entries.len(),
            test_files.len(),
            "All entries should still be in range selection"
        );
    }
}

#[test]
fn test_range_selection_clears_marked_entries() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_files = create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
        temp_dir.path().join("file4.txt"),
        temp_dir.path().join("file5.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Mark some entries manually first
    {
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.marked_entries.insert(test_files[0].clone());
        tab.marked_entries.insert(test_files[2].clone());
        tab.marked_entries.insert(test_files[4].clone());

        // Verify entries are marked
        assert_eq!(
            tab.marked_entries.len(),
            3,
            "Should have 3 marked entries initially"
        );
        assert!(
            tab.marked_entries.contains(&test_files[0]),
            "file1.txt should be marked"
        );
        assert!(
            tab.marked_entries.contains(&test_files[2]),
            "file3.txt should be marked"
        );
        assert!(
            tab.marked_entries.contains(&test_files[4]),
            "file5.txt should be marked"
        );
    }

    // Enter range selection mode by pressing 'v'
    harness.key_press(Key::V);
    harness.step();

    // Verify that marked entries were cleared when entering range selection mode
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.is_range_selection_active(),
            "Range selection should be active after pressing 'v'"
        );

        assert_eq!(
            tab.marked_entries.len(),
            0,
            "Marked entries should be cleared when entering range selection mode"
        );

        assert!(
            tab.marked_entries.is_empty(),
            "Marked entries set should be empty"
        );
    }

    // Exit range selection mode
    harness.key_press(Key::V);
    harness.step();

    // Verify marked entries remain cleared after exiting range selection mode
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            !tab.is_range_selection_active(),
            "Range selection should be inactive after pressing 'v' again"
        );

        assert_eq!(
            tab.marked_entries.len(),
            0,
            "Marked entries should still be empty after exiting range selection mode"
        );
    }
}

#[test]
fn test_range_selection_disabled_on_directory_change() {
    use std::fs;

    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a subdirectory
    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir_all(&sub_dir).unwrap();

    // Create test files in the main directory
    let _test_files = create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Enter range selection mode
    harness.key_press(Key::V);
    harness.step();

    // Move down to select multiple files
    harness.key_press(Key::J);
    harness.step();

    // Verify range selection is active
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.is_range_selection_active(),
            "Range selection should be active before directory change"
        );
        assert_eq!(
            tab.get_range_selection_range(),
            Some((0, 1)),
            "Range should be from 0 to 1"
        );
    }

    // Navigate to the subdirectory (simulating directory change)
    harness.state_mut().navigate_to_dir(sub_dir);
    harness.step();

    // Verify range selection is disabled after directory change
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            !tab.is_range_selection_active(),
            "Range selection should be disabled after directory change"
        );
        assert_eq!(
            tab.get_range_selection_range(),
            None,
            "Range should be None after directory change"
        );
    }
}

#[test]
fn test_marked_entries_cleared_on_range_selection_entry() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_files = create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
        temp_dir.path().join("file4.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Mark some entries first using space key
    harness.key_press(Key::Space); // Mark file1.txt
    harness.step();
    harness.key_press(Key::J); // Move to file2.txt
    harness.step();
    harness.key_press(Key::Space); // Mark file2.txt
    harness.step();

    // Verify that entries are marked
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(
            tab.marked_entries.len(),
            2,
            "Should have 2 marked entries before entering range selection"
        );
        assert!(
            tab.marked_entries.contains(&test_files[0]),
            "file1.txt should be marked"
        );
        assert!(
            tab.marked_entries.contains(&test_files[1]),
            "file2.txt should be marked"
        );
        assert!(
            !tab.is_range_selection_active(),
            "Range selection should not be active yet"
        );
    }

    // Enter range selection mode
    harness.key_press(Key::V);
    harness.step();

    // Verify that marked entries are cleared and range selection is active
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(
            tab.marked_entries.len(),
            0,
            "Marked entries should be cleared when entering range selection mode"
        );
        assert!(
            tab.is_range_selection_active(),
            "Range selection should be active after pressing V"
        );
        assert_eq!(
            tab.get_range_selection_range(),
            Some((1, 1)),
            "Range should start at current position (index 1)"
        );
    }
}

#[test]
fn test_select_entry_has_no_effect_during_range_selection() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_files = create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
        temp_dir.path().join("file4.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Move to the second file
    harness.key_press(Key::J);
    harness.step();

    // Enter range selection mode
    harness.key_press(Key::V);
    harness.step();

    // Verify range selection is active
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.is_range_selection_active(),
            "Range selection should be active after pressing V"
        );
        assert_eq!(
            tab.marked_entries.len(),
            0,
            "Marked entries should be empty in range selection mode"
        );
    }

    // Try to use SelectEntry action (Space key) - this should have no effect
    harness.key_press(Key::Space);
    harness.step();

    // Verify that marked_entries was not modified
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.is_range_selection_active(),
            "Range selection should still be active"
        );
        assert_eq!(
            tab.marked_entries.len(),
            0,
            "Marked entries should still be empty - Space key should have no effect during range selection"
        );
    }

    // Move down to extend the range selection
    harness.key_press(Key::J);
    harness.step();

    // Verify range selection range expanded but marked_entries is still empty
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.is_range_selection_active(),
            "Range selection should still be active"
        );
        assert_eq!(
            tab.get_range_selection_range(),
            Some((1, 2)),
            "Range should span from index 1 to 2"
        );
        assert_eq!(
            tab.marked_entries.len(),
            0,
            "Marked entries should still be empty"
        );
    }

    // Try Space key again while range selection is active
    harness.key_press(Key::Space);
    harness.step();

    // Verify it still has no effect
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.is_range_selection_active(),
            "Range selection should still be active"
        );
        assert_eq!(
            tab.marked_entries.len(),
            0,
            "Marked entries should still be empty - Space key should continue to have no effect"
        );
        assert_eq!(
            tab.get_range_selection_range(),
            Some((1, 2)),
            "Range should remain unchanged after Space key"
        );
    }

    // Exit range selection mode
    harness.key_press(Key::V);
    harness.step();

    // Verify range selection is now inactive and we can use Space key normally
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            !tab.is_range_selection_active(),
            "Range selection should be inactive after exiting"
        );
    }

    // Now Space key should work normally
    harness.key_press(Key::Space);
    harness.step();

    // Verify that Space key now works to mark entries
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(
            tab.marked_entries.len(),
            1,
            "Space key should work normally after exiting range selection mode"
        );
        assert!(
            tab.marked_entries.contains(&test_files[2]),
            "Current file should be marked after pressing Space"
        );
    }
}
