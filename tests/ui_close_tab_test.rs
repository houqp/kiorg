#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::{Key, Modifiers};
use tempfile::tempdir;
use ui_test_helpers::{create_harness, tab_num_modifiers};

#[test]
fn test_close_current_tab_shortcut() {
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

    // Verify we have three tabs
    assert_eq!(
        harness.state().tab_manager.tab_indexes().len(),
        3,
        "Should have three tabs after creating two more"
    );

    // Verify we're on the third tab (index 2)
    {
        let tab_indexes = harness.state().tab_manager.tab_indexes();
        let current_tab_index = tab_indexes
            .iter()
            .position(|(_, is_current)| *is_current)
            .unwrap();
        assert_eq!(
            current_tab_index, 2,
            "Current tab should be the third tab (index 2)"
        );
    }

    // Close the current tab using Ctrl+C
    let modifiers = Modifiers {
        ctrl: true,
        ..Default::default()
    };
    harness.press_key_modifiers(modifiers, Key::Q);
    harness.step();

    // Verify we now have two tabs
    assert_eq!(
        harness.state().tab_manager.tab_indexes().len(),
        2,
        "Should have two tabs after closing one"
    );

    // Verify we're now on the second tab (index 1)
    {
        let tab_indexes = harness.state().tab_manager.tab_indexes();
        let current_tab_index = tab_indexes
            .iter()
            .position(|(_, is_current)| *is_current)
            .unwrap();
        assert_eq!(
            current_tab_index, 1,
            "Current tab should be the second tab (index 1)"
        );
    }

    // Close the current tab again
    harness.press_key_modifiers(modifiers, Key::Q);
    harness.step();

    // Verify we now have one tab
    assert_eq!(
        harness.state().tab_manager.tab_indexes().len(),
        1,
        "Should have one tab after closing another"
    );

    // Verify we're now on the first tab (index 0)
    {
        let tab_indexes = harness.state().tab_manager.tab_indexes();
        let current_tab_index = tab_indexes
            .iter()
            .position(|(_, is_current)| *is_current)
            .unwrap();
        assert_eq!(
            current_tab_index, 0,
            "Current tab should be the first tab (index 0)"
        );
    }

    // Try to close the last tab (should not close)
    harness.press_key_modifiers(modifiers, Key::Q);
    harness.step();

    // Verify we still have one tab (can't close the last tab)
    assert_eq!(
        harness.state().tab_manager.tab_indexes().len(),
        1,
        "Should still have one tab (can't close the last tab)"
    );
}

#[test]
fn test_close_tab_preserves_other_tabs() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Create a second tab
    harness.press_key(Key::T);
    harness.step();

    // Create a third tab
    harness.press_key(Key::T);
    harness.step();

    // Verify we have three tabs
    assert_eq!(
        harness.state().tab_manager.tab_indexes().len(),
        3,
        "Should have three tabs"
    );

    // Switch to the second tab with platform-specific modifier
    let modifiers = tab_num_modifiers();
    harness.press_key_modifiers(modifiers, Key::Num2);
    harness.step();
    {
        let tab_indexes = harness.state().tab_manager.tab_indexes();
        let current_tab_index = tab_indexes
            .iter()
            .position(|(_, is_current)| *is_current)
            .unwrap();
        assert_eq!(
            current_tab_index, 1,
            "Current tab should be the second tab (index 1)"
        );
    }

    // Navigate to a different directory in the second tab
    // (We'll use the parent directory of the temp directory)
    let parent_dir = temp_dir.path().parent().unwrap().to_path_buf();
    {
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.current_path = parent_dir.clone();
    }
    harness.state_mut().refresh_entries();
    harness.step();

    // Switch to the third tab with platform-specific modifier
    harness.press_key_modifiers(tab_num_modifiers(), Key::Num3);
    harness.step();
    {
        let tab_indexes = harness.state().tab_manager.tab_indexes();
        let current_tab_index = tab_indexes
            .iter()
            .position(|(_, is_current)| *is_current)
            .unwrap();
        assert_eq!(
            current_tab_index, 2,
            "Current tab should be the third tab (index 2)"
        );
    }

    // Close the third tab
    let modifiers = Modifiers {
        ctrl: true,
        ..Default::default()
    };
    harness.press_key_modifiers(modifiers, Key::Q);
    harness.step();

    // Verify we're now on the second tab
    {
        let tab_indexes = harness.state().tab_manager.tab_indexes();
        let current_tab_index = tab_indexes
            .iter()
            .position(|(_, is_current)| *is_current)
            .unwrap();
        assert_eq!(
            current_tab_index, 1,
            "Current tab should be the second tab (index 1)"
        );
    }

    // Verify the second tab still has its custom path
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().current_path,
        parent_dir,
        "Second tab should still have its custom path"
    );

    // Switch to the first tab with platform-specific modifier
    harness.press_key_modifiers(tab_num_modifiers(), Key::Num1);
    harness.step();

    // Verify the first tab has the original path
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().current_path,
        temp_dir.path(),
        "First tab should have the original path"
    );
}
