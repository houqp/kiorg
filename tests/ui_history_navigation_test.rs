#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files, tab_num_modifiers};

/// Test for directory history navigation with keyboard shortcuts
#[test]
fn test_directory_history_navigation() {
    // Create a temporary directory structure for testing
    let temp_dir = tempdir().unwrap();

    // Create test directories with a nested structure
    let test_files = create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("dir2"),
        temp_dir.path().join("dir3"),
    ]);

    // Create nested directories inside dir1
    create_test_files(&[test_files[0].join("nested1"), test_files[0].join("nested2")]);

    // Create nested directory inside dir2
    create_test_files(&[test_files[1].join("nested3")]);

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Verify initial state
    let initial_path = harness
        .state()
        .tab_manager
        .current_tab_ref()
        .current_path
        .clone();
    assert_eq!(
        initial_path,
        temp_dir.path(),
        "Initial path should be the temp directory"
    );

    // Navigate to dir1
    {
        // Select dir1 (index 0)
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = 0;
        harness.step();

        // Navigate into dir1
        harness.key_press(Key::L);
        harness.step();

        // Verify we're in dir1
        assert_eq!(
            harness.state().tab_manager.current_tab_ref().current_path,
            test_files[0],
            "Should have navigated to dir1"
        );

        // Verify history state
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.history.len(), 2, "History should have 2 entries");
        assert_eq!(tab.history_position, 2, "History position should be 2");
    }

    // Navigate to nested1
    {
        // Select nested1 (index 0)
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = 0;
        harness.step();

        // Navigate into nested1
        harness.key_press(Key::L);
        harness.step();

        // Verify we're in nested1
        let nested1_path = test_files[0].join("nested1");
        assert_eq!(
            harness.state().tab_manager.current_tab_ref().current_path,
            nested1_path,
            "Should have navigated to nested1"
        );

        // Verify history state
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.history.len(), 3, "History should have 3 entries");
        assert_eq!(tab.history_position, 3, "History position should be 3");
    }

    // Go back in history (Ctrl+O)
    {
        // Press Ctrl+O to go back
        harness.key_press_modifiers(egui::Modifiers::CTRL, Key::O);
        harness.step();

        // Verify we're back in dir1
        assert_eq!(
            harness.state().tab_manager.current_tab_ref().current_path,
            test_files[0],
            "Should have navigated back to dir1"
        );

        // Verify history state
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.history.len(), 3, "History should still have 3 entries");
        assert_eq!(tab.history_position, 2, "History position should be 2");
    }

    // Go back again to root
    {
        // Press Ctrl+O to go back
        harness.key_press_modifiers(egui::Modifiers::CTRL, Key::O);
        harness.step();

        // Verify we're back at the root
        assert_eq!(
            harness.state().tab_manager.current_tab_ref().current_path,
            temp_dir.path(),
            "Should have navigated back to root"
        );

        // Verify history state
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.history.len(), 3, "History should still have 3 entries");
        assert_eq!(tab.history_position, 1, "History position should be 1");
    }

    // Go forward in history (Ctrl+I)
    {
        // Press Ctrl+I to go forward
        harness.key_press_modifiers(egui::Modifiers::CTRL, Key::I);
        harness.step();

        // Verify we're back in dir1
        assert_eq!(
            harness.state().tab_manager.current_tab_ref().current_path,
            test_files[0],
            "Should have navigated forward to dir1"
        );

        // Verify history state
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.history.len(), 3, "History should still have 3 entries");
        assert_eq!(tab.history_position, 2, "History position should be 2");
    }

    // Go forward again to nested1
    {
        // Press Ctrl+I to go forward
        harness.key_press_modifiers(egui::Modifiers::CTRL, Key::I);
        harness.step();

        // Verify we're back in nested1
        let nested1_path = test_files[0].join("nested1");
        assert_eq!(
            harness.state().tab_manager.current_tab_ref().current_path,
            nested1_path,
            "Should have navigated forward to nested1"
        );

        // Verify history state
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.history.len(), 3, "History should still have 3 entries");
        assert_eq!(tab.history_position, 3, "History position should be 3");
    }

    // Test history truncation when navigating to a new path after going back
    {
        // First go back to dir1
        harness.key_press_modifiers(egui::Modifiers::CTRL, Key::O);
        harness.step();

        // Verify we're back in dir1
        assert_eq!(
            harness.state().tab_manager.current_tab_ref().current_path,
            test_files[0],
            "Should have navigated back to dir1"
        );

        // Now navigate to nested2 instead of nested1
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = 1; // Select nested2
        harness.step();

        // Navigate into nested2
        harness.key_press(Key::L);
        harness.step();

        // Verify we're in nested2
        let nested2_path = test_files[0].join("nested2");
        assert_eq!(
            harness.state().tab_manager.current_tab_ref().current_path,
            nested2_path,
            "Should have navigated to nested2"
        );

        // Verify history state - should have truncated and added new entry
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.history.len(), 3, "History should have 3 entries");
        assert_eq!(tab.history_position, 3, "History position should be 3");
        assert_eq!(
            tab.history[2], nested2_path,
            "Last history entry should be nested2"
        );
    }

    // Test that history is maintained per tab
    {
        // Create a new tab
        harness.key_press(Key::T);
        harness.step();

        // Verify we have two tabs
        assert_eq!(harness.state().tab_manager.tab_indexes().len(), 2);

        // Verify the new tab has only one history entry (the initial path)
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.history.len(), 1, "New tab should have 1 history entry");
        assert_eq!(
            tab.history_position, 1,
            "New tab history position should be 1"
        );

        // Switch back to first tab
        harness.key_press_modifiers(tab_num_modifiers(), Key::Num1);
        harness.step();

        // Verify first tab still has its history
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(
            tab.history.len(),
            3,
            "First tab should still have 3 history entries"
        );
        assert_eq!(
            tab.history_position, 3,
            "First tab history position should be 3"
        );
    }
}
