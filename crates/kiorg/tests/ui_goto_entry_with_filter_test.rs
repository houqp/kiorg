#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use kiorg::models::dir_entry::DirEntry;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files, shift_modifiers};

#[test]
fn test_goto_first_entry_with_filter() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    create_test_files(&[
        temp_dir.path().join("apple.txt"),
        temp_dir.path().join("banana.txt"),
        temp_dir.path().join("cherry.txt"),
        temp_dir.path().join("date.txt"),
        temp_dir.path().join("elderberry.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Activate search
    harness.key_press(Key::Slash);
    harness.step();

    // Input search query "berry" (should only match "elderberry.txt")
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("berry".to_string()));
    harness.step();
    harness.key_press(Key::Enter);
    harness.step();

    // Verify search is active and filtering works
    assert_eq!(
        harness.state().search_bar.query.as_deref(),
        Some("berry"),
        "Search query should be 'berry'"
    );

    // Move selection to the middle of the list
    harness.key_press(Key::J);
    harness.step();
    harness.key_press(Key::J);
    harness.step();

    // Press 'gg' to go to first entry
    harness.key_press(Key::G);
    harness.step();
    harness.key_press(Key::G);
    harness.step();

    // Verify selection is at the first filtered entry
    let query = harness.state().search_bar.query.clone();
    let tab = harness.state_mut().tab_manager.current_tab_mut();
    tab.update_filtered_cache(&query, true, false);
    let filtered_entries: Vec<&DirEntry> = tab
        .get_cached_filtered_entries()
        .iter()
        .map(|&index| &tab.entries[index])
        .collect();

    // Get the selected entry
    let selected_entry = &tab.entries[tab.selected_index];

    // Verify the selected entry is in the filtered list
    assert!(
        filtered_entries
            .iter()
            .any(|&entry| entry.meta.path == selected_entry.meta.path),
        "Selected entry should be in the filtered list"
    );

    // Verify the selected entry is the first one in the filtered list
    assert_eq!(
        selected_entry.meta.path, filtered_entries[0].meta.path,
        "Selected entry should be the first entry in the filtered list"
    );
}

#[test]
fn test_goto_last_entry_with_filter() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    create_test_files(&[
        temp_dir.path().join("apple.txt"),
        temp_dir.path().join("banana.txt"),
        temp_dir.path().join("cherry.txt"),
        temp_dir.path().join("date.txt"),
        temp_dir.path().join("elderberry.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Activate search
    harness.key_press(Key::Slash);
    harness.step();

    // Input search query "a" (should match "apple.txt", "banana.txt", "date.txt")
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("a".to_string()));
    harness.step();
    harness.key_press(Key::Enter);
    harness.step();

    // Verify search is active and filtering works
    assert_eq!(
        harness.state().search_bar.query.as_deref(),
        Some("a"),
        "Search query should be 'a'"
    );

    // Press Shift+G to go to last entry
    harness.key_press_modifiers(shift_modifiers(), Key::G);
    harness.step();

    // Verify selection is at the last filtered entry
    let query = harness.state().search_bar.query.clone();
    let tab = harness.state_mut().tab_manager.current_tab_mut();
    tab.update_filtered_cache(&query, true, false);
    let filtered_entries: Vec<&DirEntry> = tab
        .get_cached_filtered_entries()
        .iter()
        .map(|&index| &tab.entries[index])
        .collect();

    // Get the selected entry
    let selected_entry = &tab.entries[tab.selected_index];

    // Verify the selected entry is in the filtered list
    assert!(
        filtered_entries
            .iter()
            .any(|&entry| entry.meta.path == selected_entry.meta.path),
        "Selected entry should be in the filtered list"
    );

    // Verify the selected entry is the last one in the filtered list
    assert_eq!(
        selected_entry.meta.path,
        filtered_entries[filtered_entries.len() - 1].meta.path,
        "Selected entry should be the last entry in the filtered list"
    );
}
