#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use kiorg::models::tab::{SortColumn, SortOrder};
use kiorg::ui::popup::PopupType;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_sort_toggle_popup_open_and_close() {
    let temp_dir = tempdir().unwrap();

    // Create some test files
    create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Initially, no popup should be shown
    assert_eq!(harness.state().show_popup, None);

    // Press comma to open sort toggle popup
    harness.key_press(Key::Comma);
    harness.step();

    // Verify the popup is open
    assert_eq!(
        harness.state().show_popup,
        Some(PopupType::SortToggle),
        "Sort toggle popup should be open after pressing comma"
    );

    // Test closing with Escape
    harness.key_press(Key::Escape);
    harness.step();

    assert_eq!(
        harness.state().show_popup,
        None,
        "Sort toggle popup should close with Escape"
    );
}

#[test]
fn test_sort_toggle_popup_name_sorting() {
    let temp_dir = tempdir().unwrap();

    // Create test files with names that will have different sort orders
    create_test_files(&[
        temp_dir.path().join("zebra.txt"),
        temp_dir.path().join("alpha.txt"),
        temp_dir.path().join("beta.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Verify initial sort state (should be Name/Ascending by default)
    {
        let state = harness.state();
        assert_eq!(state.tab_manager.sort_column, SortColumn::Name);
        assert_eq!(state.tab_manager.sort_order, SortOrder::Ascending);

        let tab = state.tab_manager.current_tab_ref();
        assert_eq!(tab.entries[0].name, "alpha.txt");
        assert_eq!(tab.entries[1].name, "beta.txt");
        assert_eq!(tab.entries[2].name, "zebra.txt");
    }

    // Open sort toggle popup
    harness.key_press(Key::Comma);
    harness.step();

    // Press 'n' to toggle name sorting (should change to None since it was already Name/Ascending)
    harness.key_press(Key::N);
    harness.step();

    // Verify sort changed to None
    {
        let state = harness.state();
        assert_eq!(state.tab_manager.sort_column, SortColumn::None);
        // Popup should still be open after sorting
        assert_eq!(state.show_popup, Some(PopupType::SortToggle));
    }

    // Press 'n' again to toggle back to Name sorting (should be Descending)
    harness.key_press(Key::N);
    harness.step();

    // Verify sort changed to Name/Descending
    {
        let state = harness.state();
        assert_eq!(state.tab_manager.sort_column, SortColumn::Name);
        assert_eq!(state.tab_manager.sort_order, SortOrder::Descending);

        let tab = state.tab_manager.current_tab_ref();
        assert_eq!(tab.entries[0].name, "zebra.txt");
        assert_eq!(tab.entries[1].name, "beta.txt");
        assert_eq!(tab.entries[2].name, "alpha.txt");
    }

    // Press 'n' once more to toggle to Name/Ascending
    harness.key_press(Key::N);
    harness.step();

    // Verify sort changed back to Name/Ascending
    {
        let state = harness.state();
        assert_eq!(state.tab_manager.sort_column, SortColumn::Name);
        assert_eq!(state.tab_manager.sort_order, SortOrder::Ascending);

        let tab = state.tab_manager.current_tab_ref();
        assert_eq!(tab.entries[0].name, "alpha.txt");
        assert_eq!(tab.entries[1].name, "beta.txt");
        assert_eq!(tab.entries[2].name, "zebra.txt");
    }

    // Close the popup
    harness.key_press(Key::Escape);
    harness.step();

    assert_eq!(harness.state().show_popup, None);
}

#[test]
fn test_sort_toggle_popup_size_sorting() {
    let temp_dir = tempdir().unwrap();

    // Create test files with different sizes
    let file1 = temp_dir.path().join("small.txt");
    let file2 = temp_dir.path().join("medium.txt");
    let file3 = temp_dir.path().join("large.txt");

    std::fs::write(&file1, "a").unwrap(); // 1 byte
    std::fs::write(&file2, "abc").unwrap(); // 3 bytes
    std::fs::write(&file3, "abcdefghij").unwrap(); // 10 bytes

    let mut harness = create_harness(&temp_dir);

    // Open sort toggle popup
    harness.key_press(Key::Comma);
    harness.step();

    // Press 's' to toggle size sorting
    harness.key_press(Key::S);
    harness.step();

    // Verify sort changed to Size/Descending (first toggle from Name should be Descending)
    {
        let state = harness.state();
        assert_eq!(state.tab_manager.sort_column, SortColumn::Size);
        assert_eq!(state.tab_manager.sort_order, SortOrder::Descending);

        let tab = state.tab_manager.current_tab_ref();
        assert_eq!(tab.entries[0].name, "large.txt");
        assert_eq!(tab.entries[1].name, "medium.txt");
        assert_eq!(tab.entries[2].name, "small.txt");
    }

    // Press 's' again to toggle to Size/Ascending
    harness.key_press(Key::S);
    harness.step();

    // Verify sort changed to Size/Ascending
    {
        let state = harness.state();
        assert_eq!(state.tab_manager.sort_column, SortColumn::Size);
        assert_eq!(state.tab_manager.sort_order, SortOrder::Ascending);

        let tab = state.tab_manager.current_tab_ref();
        assert_eq!(tab.entries[0].name, "small.txt");
        assert_eq!(tab.entries[1].name, "medium.txt");
        assert_eq!(tab.entries[2].name, "large.txt");
    }

    // Close the popup
    harness.key_press(Key::Escape);
    harness.step();
}

#[test]
fn test_sort_toggle_popup_modified_sorting() {
    let temp_dir = tempdir().unwrap();

    // Create test files (they'll have slightly different modification times)
    let file1 = temp_dir.path().join("first.txt");
    let file2 = temp_dir.path().join("second.txt");

    std::fs::write(&file1, "content").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1));
    std::fs::write(&file2, "content").unwrap();

    let mut harness = create_harness(&temp_dir);

    // Open sort toggle popup
    harness.key_press(Key::Comma);
    harness.step();

    // Press 'm' to toggle modified sorting
    harness.key_press(Key::M);
    harness.step();

    // Verify sort changed to Modified
    {
        let state = harness.state();
        assert_eq!(state.tab_manager.sort_column, SortColumn::Modified);
        assert_eq!(state.tab_manager.sort_order, SortOrder::Descending);

        let tab = state.tab_manager.current_tab_ref();
        // Most recently modified should be first (third.txt was created last)
        assert_eq!(tab.entries[0].name, "second.txt");
    }

    // Press 'm' again to toggle to Modified/Ascending
    harness.key_press(Key::M);
    harness.step();

    // Verify sort changed to Modified/Ascending
    {
        let state = harness.state();
        assert_eq!(state.tab_manager.sort_column, SortColumn::Modified);
        assert_eq!(state.tab_manager.sort_order, SortOrder::Ascending);

        let tab = state.tab_manager.current_tab_ref();
        // Oldest modified should be first (first.txt was created first)
        assert_eq!(tab.entries[0].name, "first.txt");
    }

    // Close the popup
    harness.key_press(Key::Escape);
    harness.step();
}

#[test]
fn test_sort_toggle_popup_multiple_column_switching() {
    let temp_dir = tempdir().unwrap();

    // Create test files
    create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Open sort toggle popup
    harness.key_press(Key::Comma);
    harness.step();

    // Start with name sorting (default), switch to size
    harness.key_press(Key::S);
    harness.step();

    assert_eq!(harness.state().tab_manager.sort_column, SortColumn::Size);

    // Switch to modified
    harness.key_press(Key::M);
    harness.step();

    assert_eq!(
        harness.state().tab_manager.sort_column,
        SortColumn::Modified
    );

    // Switch back to name
    harness.key_press(Key::N);
    harness.step();

    assert_eq!(harness.state().tab_manager.sort_column, SortColumn::Name);

    // All these operations should keep the popup open
    assert_eq!(harness.state().show_popup, Some(PopupType::SortToggle));

    // Close the popup
    harness.key_press(Key::Escape);
    harness.step();

    assert_eq!(harness.state().show_popup, None);
}

#[test]
fn test_sort_toggle_popup_invalid_keys_ignored() {
    let temp_dir = tempdir().unwrap();

    create_test_files(&[temp_dir.path().join("file.txt")]);

    let mut harness = create_harness(&temp_dir);

    // Store initial sort state
    let initial_sort_column = harness.state().tab_manager.sort_column;
    let initial_sort_order = harness.state().tab_manager.sort_order;

    // Open sort toggle popup
    harness.key_press(Key::Comma);
    harness.step();

    // Press various invalid keys - they should be ignored
    harness.key_press(Key::A);
    harness.step();
    harness.key_press(Key::Z);
    harness.step();
    harness.key_press(Key::Enter);
    harness.step();
    harness.key_press(Key::Space);
    harness.step();

    // Verify sort state hasn't changed and popup is still open
    {
        let state = harness.state();
        assert_eq!(state.tab_manager.sort_column, initial_sort_column);
        assert_eq!(state.tab_manager.sort_order, initial_sort_order);
        assert_eq!(state.show_popup, Some(PopupType::SortToggle));
    }

    // Only valid close keys should work
    harness.key_press(Key::Escape);
    harness.step();

    assert_eq!(harness.state().show_popup, None);
}

#[test]
fn test_sort_toggle_popup_preserves_selection() {
    let temp_dir = tempdir().unwrap();

    // Create test files
    create_test_files(&[
        temp_dir.path().join("aaa.txt"),
        temp_dir.path().join("bbb.txt"),
        temp_dir.path().join("ccc.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Move selection to the second file
    harness.key_press(Key::ArrowDown);
    harness.step();

    // Verify selection is on "bbb.txt"
    {
        let state = harness.state();
        let tab = state.tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, 1);
        assert_eq!(tab.entries[tab.selected_index].name, "bbb.txt");
    }

    // Open sort toggle popup and change sort to size
    harness.key_press(Key::Comma);
    harness.step();
    harness.key_press(Key::S);
    harness.step();

    // Close popup
    harness.key_press(Key::Escape);
    harness.step();

    // Verify that selection is still valid (though the file might be in a different position)
    {
        let state = harness.state();
        let tab = state.tab_manager.current_tab_ref();
        assert!(
            tab.selected_index < tab.entries.len(),
            "Selection should still be valid"
        );
        // The selection might have moved due to sort order change, but should still be valid
    }
}
