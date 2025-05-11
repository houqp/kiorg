use egui::Key;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

use ui_test_helpers::{create_harness, create_test_files};

#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

/// Test that tab selection is preserved when switching between tabs at runtime
#[test]
fn test_tab_selection_preserved_at_runtime() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files
    create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
    ]);

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Create a second tab
    harness.press_key(Key::T);
    harness.step();

    // Verify we have two tabs
    assert_eq!(harness.state().tab_manager.tab_indexes().len(), 2);

    // In the first tab, select the second file (index 1)
    {
        harness.press_key(Key::Num1); // Switch to first tab
        harness.step();
        harness.press_key(Key::J); // Move down to select second file
        harness.step();
        assert_eq!(
            harness.state().tab_manager.current_tab_ref().selected_index,
            1,
            "First tab should have index 1 selected"
        );
    }

    // In the second tab, select the third file (index 2)
    {
        harness.press_key(Key::Num2); // Switch to second tab
        harness.step();
        harness.press_key(Key::J); // Move down
        harness.step();
        harness.press_key(Key::J); // Move down again to select third file
        harness.step();
        assert_eq!(
            harness.state().tab_manager.current_tab_ref().selected_index,
            2,
            "Second tab should have index 2 selected"
        );
    }

    // Switch back to first tab and verify selection is preserved
    harness.press_key(Key::Num1);
    harness.step();
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        1,
        "First tab should still have index 1 selected after switching back"
    );

    // Switch to second tab again and verify selection is preserved
    harness.press_key(Key::Num2);
    harness.step();
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        2,
        "Second tab should still have index 2 selected after switching back"
    );
}

/// Test that tab selection is not persisted when restarting the application
#[test]
fn test_tab_selection_not_persisted() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_dir_path = temp_dir.path().to_path_buf();

    // Create test files
    create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
    ]);

    // Create a config directory that will be shared between app instances
    let config_temp_dir = tempdir().unwrap();
    let config_dir_path = config_temp_dir.path().to_path_buf();

    // First app instance - set up tabs and selections
    {
        // Create a new egui context
        let ctx = egui::Context::default();
        let cc = eframe::CreationContext::_new_kittest(ctx.clone());

        // Create the app with the test config directory override
        let app = kiorg::Kiorg::new_with_config_dir(
            &cc,
            Some(test_dir_path.clone()),
            Some(config_dir_path.clone()),
        );

        // Create a test harness
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::Vec2::new(800.0, 600.0))
            .with_max_steps(20)
            .build_eframe(|_cc| app);

        // Run one step to initialize the app
        harness.step();

        // Create a second tab
        harness.press_key(Key::T);
        harness.step();

        // In the first tab, select the second file (index 1)
        harness.press_key(Key::Num1); // Switch to first tab
        harness.step();
        harness.press_key(Key::J); // Move down to select second file
        harness.step();
        assert_eq!(
            harness.state().tab_manager.current_tab_ref().selected_index,
            1,
            "First tab should have index 1 selected"
        );

        // In the second tab, select the third file (index 2)
        harness.press_key(Key::Num2); // Switch to second tab
        harness.step();
        harness.press_key(Key::J); // Move down
        harness.step();
        harness.press_key(Key::J); // Move down again to select third file
        harness.step();
        assert_eq!(
            harness.state().tab_manager.current_tab_ref().selected_index,
            2,
            "Second tab should have index 2 selected"
        );

        // Trigger graceful shutdown to save state
        harness.state_mut().shutdown_requested = true;
        harness.step();
    }

    // Check the content of the persisted state.json file
    {
        // Get the path to the state.json file
        let state_file_path = config_dir_path.join("state.json");
        assert!(
            state_file_path.exists(),
            "state.json file should exist after graceful shutdown"
        );

        // Read and parse the state.json file
        let state_json_content =
            fs::read_to_string(&state_file_path).expect("Failed to read state.json");
        let state_json: Value =
            serde_json::from_str(&state_json_content).expect("Failed to parse state.json");

        // Verify the structure of the state.json file
        assert!(
            state_json.is_object(),
            "state.json should contain a JSON object"
        );

        // Check that tab_manager exists and is an object
        assert!(
            state_json.get("tab_manager").is_some(),
            "state.json should contain a tab_manager field"
        );

        let tab_manager = state_json.get("tab_manager").unwrap();

        // Check that tab_states exists and is an array with 2 elements
        assert!(
            tab_manager.get("tab_states").is_some(),
            "tab_manager should contain a tab_states field"
        );

        let tab_states = tab_manager.get("tab_states").unwrap().as_array().unwrap();
        assert_eq!(tab_states.len(), 2, "tab_states should contain 2 tabs");

        // Check that current_tab_index exists
        assert!(
            tab_manager.get("current_tab_index").is_some(),
            "tab_manager should contain a current_tab_index field"
        );

        // Verify that the tab states only contain paths and not selected_index
        for (i, tab_state) in tab_states.iter().enumerate() {
            assert!(
                tab_state.get("current_path").is_some(),
                "Tab state {} should contain a current_path field",
                i
            );

            // Verify that selected_index is NOT present in the persisted state
            assert!(
                tab_state.get("selected_index").is_none(),
                "Tab state {} should NOT contain a selected_index field",
                i
            );
        }
    }

    // Second app instance - verify selections are reset
    {
        // Create a new egui context
        let ctx = egui::Context::default();
        let cc = eframe::CreationContext::_new_kittest(ctx.clone());

        // Create the app with the same config directory to load the saved state
        // Pass None as initial_dir to force loading from saved state
        let app = kiorg::Kiorg::new_with_config_dir(
            &cc,
            None, // Use None to load from saved state
            Some(config_dir_path.clone()),
        );

        // Create a test harness
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::Vec2::new(800.0, 600.0))
            .with_max_steps(20)
            .build_eframe(|_cc| app);

        // Run one step to initialize the app
        harness.step();

        // Verify we have two tabs (loaded from state)
        assert_eq!(
            harness.state().tab_manager.tab_indexes().len(),
            2,
            "Should have loaded two tabs from saved state"
        );

        // Verify first tab has selection reset to 0
        harness.press_key(Key::Num1); // Switch to first tab
        harness.step();
        assert_eq!(
            harness.state().tab_manager.current_tab_ref().selected_index,
            0,
            "First tab should have selection reset to index 0"
        );

        // Verify second tab has selection reset to 0
        harness.press_key(Key::Num2); // Switch to second tab
        harness.step();
        assert_eq!(
            harness.state().tab_manager.current_tab_ref().selected_index,
            0,
            "Second tab should have selection reset to index 0"
        );
    }
}
