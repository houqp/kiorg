#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui_kittest::kittest::Queryable;
use kiorg::models::tab::{SortColumn, SortOrder};
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_sort_by_name() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files with names that will have a different order when sorted
    create_test_files(&[
        temp_dir.path().join("c.txt"),
        temp_dir.path().join("a.txt"),
        temp_dir.path().join("b.txt"),
    ]);

    // Start the harness. `create_harness` ensures sorting is Name/Ascending.
    let mut harness = create_harness(&temp_dir);

    // Verify initial state
    {
        let state = harness.state();
        let tab_manager = &state.tab_manager;
        assert_eq!(tab_manager.sort_column, SortColumn::Name);
        assert_eq!(tab_manager.sort_order, SortOrder::Ascending);

        let tab = tab_manager.current_tab_ref();
        assert_eq!(tab.entries.len(), 3, "Should have 3 entries");
        assert_eq!(tab.entries[0].name, "a.txt", "First entry should be a.txt");
        assert_eq!(tab.entries[1].name, "b.txt", "Second entry should be b.txt");
        assert_eq!(tab.entries[2].name, "c.txt", "Third entry should be c.txt");
    }

    // Toggle sort to None by clicking the Name header (first click cycles from Ascending to None)
    harness.query_by_label("Name \u{2B89}").unwrap().click();
    // 2 steps needed to update column header label
    harness.step();
    harness.step();

    // Verify entries are now unsorted (SortColumn::None)
    {
        let state = harness.state();
        let tab_manager = &state.tab_manager;
        assert_eq!(tab_manager.sort_column, SortColumn::None);
        // sort_order is still Ascending, but it's ignored when sort_column is None
        assert_eq!(tab_manager.sort_order, SortOrder::Ascending);
    }

    // Toggle sort to Descending by clicking the Name header again
    harness.query_by_label("Name").unwrap().click();
    harness.step();
    harness.step();

    // Verify entries are now sorted in reverse alphabetical order
    {
        let state = harness.state();
        let tab_manager = &state.tab_manager;
        assert_eq!(tab_manager.sort_column, SortColumn::Name);
        assert_eq!(tab_manager.sort_order, SortOrder::Descending);

        let tab = tab_manager.current_tab_ref();
        assert_eq!(tab.entries[0].name, "c.txt", "First entry should be c.txt");
        assert_eq!(tab.entries[1].name, "b.txt", "Second entry should be b.txt");
        assert_eq!(tab.entries[2].name, "a.txt", "Third entry should be a.txt");
    }

    // Toggle sort to Ascending by clicking the Name header again
    harness.query_by_label("Name \u{2B8B}").unwrap().click();
    harness.step();

    // Verify entries are sorted alphabetically again
    {
        let state = harness.state();
        let tab_manager = &state.tab_manager;
        assert_eq!(tab_manager.sort_column, SortColumn::Name);
        assert_eq!(tab_manager.sort_order, SortOrder::Ascending);

        let tab = tab_manager.current_tab_ref();
        assert_eq!(tab.entries[0].name, "a.txt", "First entry should be a.txt");
        assert_eq!(tab.entries[1].name, "b.txt", "Second entry should be b.txt");
        assert_eq!(tab.entries[2].name, "c.txt", "Third entry should be c.txt");
    }
}
