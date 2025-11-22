#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files, ctrl_modifiers};

/// Test for page down shortcut with small file list
/// This test verifies that page down works correctly in small lists (3 entries)
/// and moves from entry 1 (second entry) to entry 2 (last entry)
/// Tests both PageDown key and Ctrl+D shortcuts
#[test]
fn test_page_down_on_a_partial_page() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create exactly 3 test files as described in the bug report
    let test_files = vec![
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
    ];
    create_test_files(&test_files);

    let mut harness = create_harness(&temp_dir);

    // Verify we have exactly 3 entries
    let tab = harness.state().tab_manager.current_tab_ref();
    assert_eq!(
        tab.entries.len(),
        3,
        "Should have exactly 3 entries for this test"
    );

    // Test 1: PageDown key
    {
        // Initially should be at first entry (index 0)
        assert_eq!(
            tab.selected_index, 0,
            "Should start at first entry (index 0)"
        );

        // Move down to select the 2nd entry (index 1)
        harness.key_press(Key::J); // 'j' is the default move down shortcut
        harness.step();

        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(
            tab.selected_index, 1,
            "Should be at second entry (index 1) after moving down"
        );

        // Now press page down - this should move to the last entry (index 2)
        harness.key_press(Key::PageDown);
        harness.step();

        let tab = harness.state().tab_manager.current_tab_ref();

        // This assertion now passes - page down should move to index 2
        assert_eq!(
            tab.selected_index, 2,
            "PageDown should move from index 1 to index 2 (last entry)"
        );
    }

    // Test 2: Ctrl+D shortcut (reset to test position first)
    {
        // Reset to first entry
        harness.key_press(Key::K); // Move up to index 1
        harness.step();
        harness.key_press(Key::K); // Move up to index 0
        harness.step();

        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(
            tab.selected_index, 0,
            "Should be back at first entry (index 0) for second test"
        );

        // Move down to select the 2nd entry (index 1) again
        harness.key_press(Key::J);
        harness.step();

        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(
            tab.selected_index, 1,
            "Should be at second entry (index 1) for Ctrl+D test"
        );

        // Now press Ctrl+D (alternative page down shortcut) - this should move to the last entry (index 2)
        // But the bug is that it does nothing instead
        harness.key_press_modifiers(ctrl_modifiers(), Key::D);
        harness.step();

        let tab = harness.state().tab_manager.current_tab_ref();

        // This assertion now passes - Ctrl+D should move to index 2
        assert_eq!(
            tab.selected_index, 2,
            "Ctrl+D (page down) should move from index 1 to index 2 (last entry)"
        );
    }
}
