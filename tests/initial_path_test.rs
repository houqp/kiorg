use egui::Context;
use kiorg::Kiorg;
use tempfile::tempdir;

#[test]
fn test_fallback_to_current_dir_when_saved_path_nonexistent() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_dir_path = temp_dir.path().to_path_buf();

    // Create a config directory
    let config_dir = test_dir_path.join("config");
    std::fs::create_dir_all(&config_dir).unwrap();

    // Create a state.json file with a non-existent path
    let state_json = r#"{
        "tab_manager": {
            "tab_states": [
                {
                    "current_path": "/path/that/does/not/exist"
                }
            ],
            "current_tab_index": 0,
            "sort_column": "Name",
            "sort_order": "Ascending"
        }
    }"#;
    std::fs::write(config_dir.join("state.json"), state_json).unwrap();

    // Create a new egui context
    let ctx = Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx.clone());

    // Create the app with the test config directory override
    // We don't provide an initial directory, so it should try to load from state.json
    let app = Kiorg::new_with_config_dir(&cc, None, Some(config_dir.clone()));

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
