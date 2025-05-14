#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::create_harness;

#[test]
fn test_tab_navigation_shortcuts() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Initially, there should be one tab
    assert_eq!(
        harness.state().tab_manager.tab_indexes().len(),
        1,
        "Should start with one tab"
    );

    // Create a second tab
    harness.press_key(Key::T);
    harness.step();

    // Create a third tab
    harness.press_key(Key::T);
    harness.step();

    // Create a fourth tab
    harness.press_key(Key::T);
    harness.step();

    // Verify we have four tabs
    assert_eq!(
        harness.state().tab_manager.tab_indexes().len(),
        4,
        "Should have four tabs after creating three more"
    );

    // Initially we should be on the fourth tab (index 3)
    assert_eq!(
        harness.state().tab_manager.get_current_tab_index(),
        3,
        "Should be on the fourth tab initially"
    );

    // Test switching to the next tab (wrapping from last to first)
    harness.press_key(Key::CloseBracket); // ']' key for next tab
    harness.step();

    // Should now be on the first tab (index 0)
    assert_eq!(
        harness.state().tab_manager.get_current_tab_index(),
        0,
        "Should wrap to the first tab after pressing ']' on the last tab"
    );

    // Test switching to the next tab again
    harness.press_key(Key::CloseBracket);
    harness.step();

    // Should now be on the second tab (index 1)
    assert_eq!(
        harness.state().tab_manager.get_current_tab_index(),
        1,
        "Should be on the second tab after pressing ']' again"
    );

    // Test switching to the previous tab
    harness.press_key(Key::OpenBracket);
    harness.step();

    // Should now be back on the first tab (index 0)
    assert_eq!(
        harness.state().tab_manager.get_current_tab_index(),
        0,
        "Should be back on the first tab after pressing ["
    );

    // Test switching to the previous tab again (wrapping from first to last)
    harness.press_key(Key::OpenBracket);
    harness.step();

    // Should now be on the fourth tab (index 3)
    assert_eq!(
        harness.state().tab_manager.get_current_tab_index(),
        3,
        "Should wrap to the last tab after pressing [ on the first tab"
    );

    // Test multiple tab navigation in sequence
    // Go forward two tabs
    harness.press_key(Key::CloseBracket);
    harness.step();
    harness.press_key(Key::CloseBracket);
    harness.step();

    // Should now be on the second tab (index 1)
    assert_eq!(
        harness.state().tab_manager.get_current_tab_index(),
        1,
        "Should be on the second tab after pressing ']' twice from the last tab"
    );

    // Go back three tabs (wrapping around)
    harness.press_key(Key::OpenBracket);
    harness.step();
    harness.press_key(Key::OpenBracket);
    harness.step();
    harness.press_key(Key::OpenBracket);
    harness.step();
    harness.press_key(Key::OpenBracket);
    harness.step();

    // Should now be on the second tab again (index 1)
    assert_eq!(
        harness.state().tab_manager.get_current_tab_index(),
        1,
        "Should be back on the second tab after pressing [ three times"
    );
}
