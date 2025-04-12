mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_g_shortcuts() {
    // Create test files and directories
    let temp_dir = tempdir().unwrap();
    create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("dir2"),
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);
    let mut harness = create_harness(&temp_dir);

    // Ensure consistent sort order for reliable selection
    harness.ensure_sorted_by_name_ascending();

    let tab = harness.state().tab_manager.current_tab_ref();
    assert_eq!(tab.selected_index, 0);

    // Test G shortcut (go to last entry)
    {
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        harness.press_key_modifiers(modifiers, Key::G);
        harness.step();
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, tab.entries.len() - 1);
    }

    // a single g press doesn't move selection
    {
        harness.press_key(Key::G);
        harness.step();
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, tab.entries.len() - 1);
    }

    // Test gg shortcut (go to first entry)
    {
        // First g press
        harness.press_key(Key::G);
        // Second g press should go back to the top
        harness.press_key(Key::G);
        harness.step();
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, 0);
    }
}

#[test]
fn test_g_shortcuts_empty_list() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Clear entries
    {
        let tab = harness.state_mut().tab_manager.current_tab();
        tab.entries.clear();
    }

    // Test G shortcut with empty list
    {
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        harness.press_key_modifiers(modifiers, Key::G);
        harness.step();
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, 0); // Should stay at 0
    }

    // Test gg shortcut with empty list
    {
        // First g press
        harness.press_key(Key::G);
        // Second g press
        harness.press_key(Key::G);
        harness.step();
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, 0); // Should stay at 0
    }
}

#[test]
fn test_parent_directory_selection() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files and directories
    let test_files = create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("dir2"),
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Ensure consistent sort order for reliable selection
    harness.ensure_sorted_by_name_ascending();

    // Move down to select dir2
    harness.press_key(Key::J);
    harness.step();

    // Navigate into dir2
    harness.press_key(Key::L);
    harness.step();

    // Create a file in dir2
    let dir2_file = test_files[1].join("dir2_file.txt");
    std::fs::File::create(&dir2_file).unwrap();

    // Move down to select dir2_file.txt
    harness.press_key(Key::J);
    harness.step();

    // Navigate to parent directory
    harness.press_key(Key::H);
    harness.step();

    // Verify that dir2 is still selected
    let tab = harness.state().tab_manager.current_tab_ref();
    assert_eq!(
        tab.entries[tab.selected_index].path, test_files[1],
        "dir2 should be selected after navigating to parent directory"
    );
}

#[test]
fn test_prev_path_selection_with_sort() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test directories (order matters for initial selection)
    let test_dirs = create_test_files(&[
        temp_dir.path().join("aaa"), // index 0
        temp_dir.path().join("bbb"), // index 1
        temp_dir.path().join("ccc"), // index 2
    ]);

    // Start the harness in the parent directory
    let mut harness = create_harness(&temp_dir);

    // Ensure consistent sort order for reliable selection
    harness.ensure_sorted_by_name_ascending();

    // Initial state: aaa, bbb, ccc (now explicitly sorted Name/Ascending)
    // Select ccc (index 2)
    harness.press_key(Key::J);
    harness.step();
    harness.press_key(Key::J);
    harness.step();
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        2
    );

    // Navigate into bbb
    harness.press_key(Key::L);
    harness.step();
    assert!(harness
        .state()
        .tab_manager
        .current_tab_ref()
        .current_path
        .ends_with("ccc"));

    // Manually set sort order to Descending Name *while inside bbb*
    // (Simulating header click is complex, direct state change is acceptable here)
    {
        let tab = harness.state_mut().tab_manager.current_tab();
        tab.toggle_sort(kiorg::models::tab::SortColumn::Name); // Sets to None
        tab.toggle_sort(kiorg::models::tab::SortColumn::Name); // Sets Name/Descending
        assert_eq!(tab.sort_column, kiorg::models::tab::SortColumn::Name);
        assert_eq!(tab.sort_order, kiorg::models::tab::SortOrder::Descending);
    }
    harness.step(); // Allow state update propagation if needed

    // Navigate back up to the parent directory
    harness.press_key(Key::H);
    harness.step();

    // Now in the parent directory, refresh_entries should have run:
    // 1. Entries read: [aaa, bbb, ccc]
    // 2. Sort applied (Name/Descending): [ccc, bbb, aaa]
    // 3. prev_path (bbb) searched in sorted list
    // 4. selected_index should be 1 (pointing to bbb)

    // Verify the state in the parent directory
    let tab = harness.state().tab_manager.current_tab_ref();
    assert_eq!(
        tab.current_path,
        temp_dir.path(),
        "Current path should be the parent"
    );
    assert_eq!(tab.entries.len(), 3, "Should have 3 entries");

    // Check sorted order
    assert_eq!(tab.entries[0].name, "ccc", "First entry should be ccc");
    assert_eq!(tab.entries[1].name, "bbb", "Second entry should be bbb");
    assert_eq!(tab.entries[2].name, "aaa", "Third entry should be aaa");

    // Check selected index based on prev_path (bbb)
    assert_eq!(tab.selected_index, 0, "Selected index should point to ccc");
    assert_eq!(
        tab.entries[tab.selected_index].path, test_dirs[2],
        "Selected entry should be ccc"
    );
}
