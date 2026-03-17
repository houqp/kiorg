use egui::Context;
use kiorg::Kiorg;
use tempfile::tempdir;

/// Helper to build a state.json string with a given path and restore_session value
fn make_state_json(path: &str, restore_session: bool) -> String {
    format!(
        r#"{{
        "tab_manager": {{
            "tab_states": [
                {{
                    "current_path": "{path}"
                }}
            ],
            "current_tab_index": 0,
            "sort_column": "Name",
            "sort_order": "Ascending"
        }},
        "restore_session": {restore_session}
    }}"#
    )
}

#[test]
fn test_fallback_to_current_dir_when_saved_path_nonexistent() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_dir_path = temp_dir.path().to_path_buf();

    // Create a config directory
    let config_dir = test_dir_path.join("config");
    std::fs::create_dir_all(&config_dir).unwrap();

    // Create a state.json file with restore_session true but a non-existent path
    let state_json = make_state_json("/path/that/does/not/exist", true);
    std::fs::write(config_dir.join("state.json"), state_json).unwrap();

    // Create a new egui context
    let ctx = Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx);

    // Create the app with the test config directory override
    // We don't provide an initial directory, so it should try to load from state.json
    let app = Kiorg::new(&cc, None, Some(config_dir)).expect("Failed to create Kiorg app");

    // Check that the current path is not the non-existent path from state.json
    let current_path = app.tab_manager.current_tab_ref().current_path.clone();
    assert_ne!(
        current_path.to_str().unwrap(),
        "/path/that/does/not/exist",
        "App should not use non-existent path from state.json"
    );

    // The app should have fallen back to the current directory
    let expected_path = dirs::home_dir().unwrap();
    assert_eq!(
        current_path, expected_path,
        "App should fall back to current directory when saved path doesn't exist"
    );
}

#[test]
fn test_restore_session_true_restores_valid_saved_path() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_dir_path = temp_dir.path().to_path_buf();

    // Create a directory to use as the saved session path
    let saved_dir = test_dir_path.join("saved_session");
    std::fs::create_dir_all(&saved_dir).unwrap();

    // Create a config directory
    let config_dir = test_dir_path.join("config");
    std::fs::create_dir_all(&config_dir).unwrap();

    // Create a state.json file with restore_session true and a valid path
    let state_json = make_state_json(saved_dir.to_str().unwrap(), true);
    std::fs::write(config_dir.join("state.json"), state_json).unwrap();

    // Create a new egui context
    let ctx = Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx);

    // Create the app with the test config directory override
    // We don't provide an initial directory, so it should restore from state.json
    let app = Kiorg::new(&cc, None, Some(config_dir)).expect("Failed to create Kiorg app");

    // The app should have restored the saved session path
    let current_path = app.tab_manager.current_tab_ref().current_path.clone();
    assert_eq!(
        current_path, saved_dir,
        "App should restore the saved session path when restore_session is true"
    );
}

#[test]
fn test_restore_session_false_ignores_saved_path() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_dir_path = temp_dir.path().to_path_buf();

    // Create a directory to use as the saved session path
    let saved_dir = test_dir_path.join("saved_session");
    std::fs::create_dir_all(&saved_dir).unwrap();

    // Create a config directory
    let config_dir = test_dir_path.join("config");
    std::fs::create_dir_all(&config_dir).unwrap();

    // Create a state.json file with restore_session false and a valid path
    let state_json = make_state_json(saved_dir.to_str().unwrap(), false);
    std::fs::write(config_dir.join("state.json"), state_json).unwrap();

    // Create a new egui context
    let ctx = Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx);

    // Create the app with the test config directory override
    // restore_session is false, so it should NOT restore the saved path
    let app = Kiorg::new(&cc, None, Some(config_dir)).expect("Failed to create Kiorg app");

    // Check that the current path is NOT the saved session path
    let current_path = app.tab_manager.current_tab_ref().current_path.clone();
    assert_ne!(
        current_path, saved_dir,
        "App should not restore saved path when restore_session is false"
    );

    // The app should have fallen back to the home directory
    let expected_path = dirs::home_dir().unwrap();
    assert_eq!(
        current_path, expected_path,
        "App should fall back to home directory when restore_session is false"
    );
}

#[test]
fn test_restore_session_flag_reset_to_false_after_load() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_dir_path = temp_dir.path().to_path_buf();

    // Create a directory to use as the saved session path
    let saved_dir = test_dir_path.join("saved_session");
    std::fs::create_dir_all(&saved_dir).unwrap();

    // Create a config directory
    let config_dir = test_dir_path.join("config");
    std::fs::create_dir_all(&config_dir).unwrap();

    // Create a state.json file with restore_session true
    let state_json = make_state_json(saved_dir.to_str().unwrap(), true);
    std::fs::write(config_dir.join("state.json"), &state_json).unwrap();

    // Create a new egui context
    let ctx = Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx);

    // Create the app, load state and then reset the flag
    let _app = Kiorg::new(&cc, None, Some(config_dir.clone())).expect("Failed to create Kiorg app");

    // After loading, the state file should have restore_session reset to false
    let updated_json = std::fs::read_to_string(config_dir.join("state.json")).unwrap();
    let updated_state: serde_json::Value = serde_json::from_str(&updated_json).unwrap();
    assert_eq!(
        updated_state["restore_session"], false,
        "restore_session should be reset to false after loading"
    );
}

#[test]
fn test_no_restore_session_uses_default_directory() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_dir_path = temp_dir.path().to_path_buf();

    // Create a directory to use as the default directory
    let default_dir = test_dir_path.join("my_default");
    std::fs::create_dir_all(&default_dir).unwrap();

    // Create a config directory
    let config_dir = test_dir_path.join("config");
    std::fs::create_dir_all(&config_dir).unwrap();

    // Create a config.toml with default_directory set
    let config_toml = format!("default_directory = \"{}\"", default_dir.to_str().unwrap());
    std::fs::write(config_dir.join("config.toml"), config_toml).unwrap();

    // Create a new egui context
    let ctx = Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx);

    // Create the app with the test config directory override
    // Initial directory and state.json are not provided, so it should use default_directory
    let app = Kiorg::new(&cc, None, Some(config_dir)).expect("Failed to create Kiorg app");

    // The app should have used the default_directory from config
    let current_path = app.tab_manager.current_tab_ref().current_path.clone();
    assert_eq!(
        current_path, default_dir,
        "App should use default_directory from config when no session to restore"
    );
}
