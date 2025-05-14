#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use kiorg::models::tab::SortColumn;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

/// Test to demonstrate the bug where toggle_sort doesn't update the path_to_index map
/// causing navigation with J/K shortcuts to select the wrong entries
#[test]
fn test_sort_navigation_bug() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files with names that will have a different order when sorted
    // The initial order will be alphabetical: a.txt, b.txt, c.txt
    let test_files = create_test_files(&[
        temp_dir.path().join("a.txt"),
        temp_dir.path().join("b.txt"),
        temp_dir.path().join("c.txt"),
    ]);

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Verify initial state - entries should be sorted alphabetically
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.entries.len(), 3, "Should have 3 entries");
        assert_eq!(tab.entries[0].name, "a.txt", "First entry should be a.txt");
        assert_eq!(tab.entries[1].name, "b.txt", "Second entry should be b.txt");
        assert_eq!(tab.entries[2].name, "c.txt", "Third entry should be c.txt");

        // Verify path_to_index mapping is correct initially
        assert_eq!(
            tab.get_index_by_path(&test_files[0]),
            Some(0),
            "a.txt should be at index 0"
        );
        assert_eq!(
            tab.get_index_by_path(&test_files[1]),
            Some(1),
            "b.txt should be at index 1"
        );
        assert_eq!(
            tab.get_index_by_path(&test_files[2]),
            Some(2),
            "c.txt should be at index 2"
        );
    }

    // Select the first entry (a.txt)
    harness
        .state_mut()
        .tab_manager
        .current_tab_mut()
        .update_selection(0);
    harness.step();

    // Toggle sort to reverse the order (Name/Descending)
    {
        let tab_manager = &mut harness.state_mut().tab_manager;
        tab_manager.toggle_sort(SortColumn::Name); // First toggle sets to None
        tab_manager.toggle_sort(SortColumn::Name); // First toggle sets to Name/Descending
        harness.step();
    }

    // Verify entries are now sorted in reverse alphabetical order
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.entries.len(), 3, "Should have 3 entries");
        assert_eq!(tab.entries[0].name, "c.txt", "First entry should be c.txt");
        assert_eq!(tab.entries[1].name, "b.txt", "Second entry should be b.txt");
        assert_eq!(tab.entries[2].name, "a.txt", "Third entry should be a.txt");
    }

    // Now let's demonstrate how this affects navigation
    // Press J to move down (should select b.txt which is now at index 1)
    harness.press_key(Key::J);
    harness.step();

    // Check what entry is selected
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let selected_index = tab.selected_index;
        let selected_name = &tab.entries[selected_index].name;

        // This assertion might fail due to the bug
        assert_eq!(
            selected_name, "b.txt",
            "After pressing J, b.txt should be selected, but got {} instead",
            selected_name
        );
    }

    // Press J again to move down (should select a.txt which is now at index 2)
    harness.press_key(Key::J);
    harness.step();

    // Check what entry is selected
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let selected_index = tab.selected_index;
        let selected_name = &tab.entries[selected_index].name;

        // This assertion might fail due to the bug
        assert_eq!(
            selected_name, "a.txt",
            "After pressing J twice, a.txt should be selected, but got {} instead",
            selected_name
        );
    }

    // Press K twice to move back up (should select c.txt which is at index 1)
    harness.press_key(Key::K);
    harness.step();
    harness.press_key(Key::K);
    harness.step();

    // Check what entry is selected
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let selected_index = tab.selected_index;
        let selected_name = &tab.entries[selected_index].name;

        // This assertion might fail due to the bug
        assert_eq!(
            selected_name, "c.txt",
            "After pressing K twice, c.txt should be selected, but got {} instead",
            selected_name
        );
    }
}
