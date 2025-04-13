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
        let modifiers = egui::Modifiers {
            shift: false,
            ..Default::default()
        };
        harness.press_key_modifiers(modifiers, Key::G);
        harness.step();
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, tab.entries.len() - 1);
    }

    // Test gg shortcut (go to first entry)
    {
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

#[test]
fn test_mouse_click_selects_and_previews() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files
    let test_files = create_test_files(&[
        temp_dir.path().join("a.txt"), // index 0
        temp_dir.path().join("b.txt"), // index 1
        temp_dir.path().join("c.txt"), // index 2
    ]);

    // Create some content for b.txt to ensure preview is generated
    std::fs::write(&test_files[1], "Content of b.txt").unwrap();

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Ensure consistent sort order for reliable selection
    harness.ensure_sorted_by_name_ascending();

    // Initially, index 0 ("a.txt") should be selected
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );
    // Preview cache should be empty or contain preview for a.txt initially
    // (Depending on initial load behavior, let's ensure it doesn't contain b.txt yet)
    assert!(
        harness.state().cached_preview_path != Some(test_files[1].clone()),
        "Preview path should not be b.txt yet"
    );

    // --- Simulate Click on the second entry ("b.txt") ---
    // Calculate the bounding box for the second row (index 1)
    // all the heights and widths are emprically determined from actual UI run
    let header_height = kiorg::ui::style::HEADER_ROW_HEIGHT; // Header row
    let row_height = kiorg::ui::file_list::ROW_HEIGHT; // Entry row
    let banner_height = 27.0;
    let target_y = banner_height + header_height + (2.0 * row_height) + (row_height / 2.0); // Click in the middle of the second entry row
    let target_pos = egui::pos2(200.0, target_y); // Click somewhere within the row horizontally

    // Simulate a primary mouse button click (press and release)
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: target_pos,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: target_pos,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::default(),
    });

    harness.step(); // Process the click and update state
    harness.step(); // One more step to process the preview content

    // --- Assertions ---
    // 1. Check if the selected index is updated to 1 ("b.txt")
    let tab = harness.state().tab_manager.current_tab_ref();
    assert_eq!(
        tab.selected_index, 1,
        "Selected index should be 1 after clicking the second entry"
    );
    assert_eq!(
        tab.entries[tab.selected_index].path, test_files[1],
        "Selected entry should be b.txt"
    );

    // 2. Check if the preview cache now contains the preview for "b.txt"
    // The preview update happens in the right_panel draw phase, which runs during step()
    assert_eq!(
        harness.state().cached_preview_path,
        Some(test_files[1].clone()),
        "Cached preview path should be b.txt after selection"
    );
    assert!(
        harness.state().preview_content.contains("Content of b.txt"),
        "Preview content should contain 'Content of b.txt'"
    );
}
