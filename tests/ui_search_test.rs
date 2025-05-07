mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_search_edit_existing_query() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    create_test_files(&[
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
        temp_dir.path().join("another.log"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Activate search
    harness.press_key(Key::Slash);
    harness.step();

    // Input search query "test"
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("test".to_string()));
    harness.step();

    // Verify search bar has the query
    assert!(
        harness.state().search_bar.query.is_some(),
        "Search bar should have query after input"
    );
    assert_eq!(
        harness.state().search_bar.query.as_deref(),
        Some("test"),
        "Search query should be 'test'"
    );

    // Press '/' again while search is active
    harness.press_key(Key::Slash);
    harness.step();

    // Verify search bar query is preserved
    assert!(
        harness.state().search_bar.query.is_some(),
        "Search query should still be Some"
    );
    assert_eq!(
        harness.state().search_bar.query.as_deref(),
        Some("test"),
        "Search query should not be reset"
    );
}

#[test]
fn test_search_resets_selection() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    create_test_files(&[
        temp_dir.path().join("apple.txt"),
        temp_dir.path().join("apricot.txt"),
        temp_dir.path().join("banana.txt"), // index 2
    ]);

    let mut harness = create_harness(&temp_dir);

    // Select the third entry (banana.txt)
    harness.press_key(Key::J);
    harness.step();
    harness.press_key(Key::J);
    harness.step();
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        2
    );
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().entries[2].name,
        "banana.txt"
    );

    // Activate search
    harness.press_key(Key::Slash);
    harness.step();

    // Input search query "ap" (matches apple and apricot)
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("ap".to_string()));
    harness.step();
    harness.press_key(Key::Enter);
    harness.step();

    // Verify selection is reset to the first matching entry (apple.txt)
    let tab = harness.state().tab_manager.current_tab_ref();
    assert_eq!(
        tab.selected_index, 0,
        "Selection should reset to the first filtered item"
    );
}

#[test]
fn test_search_cleared_on_directory_change() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_files = create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Activate search
    harness.press_key(Key::Slash);
    harness.step();

    // Input search query "test"
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("dir".to_string()));
    harness.step();
    harness.press_key(Key::Enter);
    harness.step();

    // Verify search bar has the query
    assert_eq!(
        harness.state().search_bar.query.as_deref(),
        Some("dir"),
        "Search query should be 'dir'"
    );

    // Select dir1 (index 0) - already selected by default
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().entries[0].path,
        test_files[0]
    );

    // Navigate into dir1
    harness.press_key(Key::L);
    harness.step();

    // Verify search query is cleared (is None) after directory change
    assert!(
        harness.state().search_bar.query.is_none(),
        "Search query should be None after entering a directory. Actual: {:?}",
        harness.state().search_bar.query
    );
}

#[test]
fn test_search_cleared_on_escape() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    create_test_files(&[
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
        temp_dir.path().join("another.log"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Activate search
    harness.press_key(Key::Slash);
    harness.step();

    // Input search query "test"
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("test".to_string()));
    harness.step();

    // Verify search bar has the query
    assert!(
        harness.state().search_bar.query.is_some(),
        "Search bar should have query after input"
    );
    assert_eq!(
        harness.state().search_bar.query.as_deref(),
        Some("test"),
        "Search query should be 'test'"
    );

    // Press Enter to apply the filter
    harness.press_key(Key::Enter);
    harness.step();

    // Verify search query is still active after pressing Enter
    assert!(
        harness.state().search_bar.query.is_some(),
        "Search query should still be active after pressing Enter"
    );
    assert_eq!(
        harness.state().search_bar.query.as_deref(),
        Some("test"),
        "Search query should still be 'test' after pressing Enter"
    );

    // Verify that the selection is updated to the first matching entry
    let tab = harness.state().tab_manager.current_tab_ref();
    let selected_entry = &tab.entries[tab.selected_index];
    assert!(
        selected_entry.name.contains("test"),
        "Selected entry should match the search query. Selected: {}",
        selected_entry.name
    );

    // Press Escape to clear the search
    harness.press_key(Key::Escape);
    harness.step();

    // Verify search query is cleared (is None) after pressing Escape
    assert!(
        harness.state().search_bar.query.is_none(),
        "Search query should be None after pressing Escape. Actual: {:?}",
        harness.state().search_bar.query
    );
}
