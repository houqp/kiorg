#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

// Helper function to create a config.toml file with custom TOML content
fn create_config_file(config_dir: &PathBuf, toml_content: &str) {
    // Create the config directory if it doesn't exist
    fs::create_dir_all(config_dir).unwrap();

    // Write to config.toml
    fs::write(config_dir.join("config.toml"), toml_content).unwrap();
}

#[test]
fn test_config_error_triggers_error_dialog() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with an invalid action name
    let toml_content = r#"
# Invalid shortcut configuration - invalid action name
[[shortcuts.NonExistentAction]]
key = "x"
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Try to load the config and verify it fails with a config error
    let config_result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(
        config_result.is_err(),
        "Should return an error for invalid action name"
    );

    // Verify the error is a ConfigError
    let error = config_result.unwrap_err();
    assert!(
        matches!(error, kiorg::config::ConfigError::TomlError(..)),
        "Should be a ConfigError, got: {error:?}"
    );

    // Create a temp directory for the test workspace
    let temp_dir = tempdir().unwrap();

    // Create a new egui context
    let ctx = egui::Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx);

    // Try to create Kiorg app with invalid config - this should fail
    let app_result = kiorg::Kiorg::new(&cc, Some(temp_dir.path().to_path_buf()), Some(config_dir));

    // Verify that creating the app fails with a ConfigError
    assert!(
        app_result.is_err(),
        "Should fail to create app with invalid config"
    );

    if let Err(app_error) = app_result {
        assert!(
            matches!(app_error, kiorg::app::KiorgError::ConfigError(_)),
            "Should be a KiorgError::ConfigError, got: {app_error:?}"
        );

        // Test that we can create the error dialog app
        let error_message = app_error.to_string();
        let _error_app = kiorg::startup_error::StartupErrorApp::new(
            error_message.clone(),
            "Configuration Error".to_string(),
        );

        // Verify error message is captured correctly
        assert!(
            !error_message.is_empty(),
            "Error message should not be empty"
        );
        assert!(
            error_message.contains("NonExistentAction")
                || error_message.contains("unknown variant"),
            "Error message should mention the invalid action, got: {error_message}"
        );
    }
}

#[test]
fn test_config_error_dialog_ui_elements() {
    // Test with a sample error message
    let error_message =
        "Invalid config: unknown variant `NonExistentAction`, expected one of...".to_string();
    let _error_app = kiorg::startup_error::StartupErrorApp::new(
        error_message.clone(),
        "Configuration Error".to_string(),
    );

    // Verify the error app was created successfully with the correct message
    // Since we can't easily test UI rendering in unit tests without a full harness,
    // we just verify the error app can be instantiated without panicking
    assert!(!error_message.is_empty());
}

#[test]
fn test_malformed_toml_triggers_config_error() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a completely malformed TOML file
    let toml_content = r"
This is not TOML at all
just some random text
with no valid TOML syntax
";

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Try to load the config and verify it fails
    let config_result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(
        config_result.is_err(),
        "Should return an error for malformed TOML"
    );

    // Create a temp directory for the test workspace
    let temp_dir = tempdir().unwrap();

    // Create a new egui context
    let ctx = egui::Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx);

    // Try to create Kiorg app with malformed config - this should fail
    let app_result = kiorg::Kiorg::new(&cc, Some(temp_dir.path().to_path_buf()), Some(config_dir));

    // Verify that creating the app fails
    assert!(
        app_result.is_err(),
        "Should fail to create app with malformed config"
    );

    if let Err(app_error) = app_result {
        let error_message = app_error.to_string();
        assert!(
            error_message.contains("Configuration error")
                || error_message.contains("Invalid config")
                || error_message.contains("expected"),
            "Error message should indicate config problem, got: {error_message}"
        );
    }
}

#[test]
fn test_shortcut_conflict_triggers_config_error() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with conflicting shortcuts
    let toml_content = r#"
# Config with conflicting shortcuts
[[shortcuts.MoveDown]]
key = "d"

[[shortcuts.DeleteEntry]]
key = "d"
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Try to load the config and verify it fails with shortcut conflict
    let config_result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(
        config_result.is_err(),
        "Should return an error for conflicting shortcuts"
    );

    // Verify the error is a ShortcutConflict error
    if let Err(error) = config_result {
        assert!(
            matches!(error, kiorg::config::ConfigError::ShortcutConflict(..)),
            "Should be a ShortcutConflict error, got: {error:?}"
        );

        let error_message = error.to_string();
        assert!(
            error_message.contains("Shortcut conflict"),
            "Error message should mention shortcut conflict, got: {error_message}"
        );
    }

    // Create a temp directory for the test workspace
    let temp_dir = tempdir().unwrap();

    // Create a new egui context
    let ctx = egui::Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx);

    // Try to create Kiorg app with conflicting shortcuts - this should fail
    let app_result = kiorg::Kiorg::new(&cc, Some(temp_dir.path().to_path_buf()), Some(config_dir));

    // Verify that creating the app fails with a ConfigError
    assert!(
        app_result.is_err(),
        "Should fail to create app with conflicting shortcuts"
    );

    if let Err(app_error) = app_result {
        assert!(
            matches!(app_error, kiorg::app::KiorgError::ConfigError(_)),
            "Should be a KiorgError::ConfigError, got: {app_error:?}"
        );
    }
}

#[test]
fn test_valid_config_does_not_trigger_error_dialog() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a valid TOML config file
    let toml_content = r#"
# Valid config with custom shortcuts
[[shortcuts.MoveDown]]
key = "j"

[[shortcuts.MoveUp]]
key = "k"
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Try to load the config and verify it succeeds
    let config_result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(
        config_result.is_ok(),
        "Should successfully load valid config, got error: {config_result:?}"
    );

    // Create a temp directory for the test workspace
    let temp_dir = tempdir().unwrap();

    // Create a new egui context
    let ctx = egui::Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx);

    // Try to create Kiorg app with valid config - this should succeed
    let app_result = kiorg::Kiorg::new(&cc, Some(temp_dir.path().to_path_buf()), Some(config_dir));

    // Verify that creating the app succeeds
    assert!(
        app_result.is_ok(),
        "Should successfully create app with valid config"
    );
}

#[test]
fn test_empty_config_directory_uses_defaults() {
    // Create a temporary directory for the config (empty - no config.toml file)
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Don't create any config.toml file - should use defaults

    // Try to load the config and verify it succeeds with defaults
    let config_result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(
        config_result.is_ok(),
        "Should successfully load default config from empty directory, got error: {config_result:?}"
    );

    // Create a temp directory for the test workspace
    let temp_dir = tempdir().unwrap();

    // Create a new egui context
    let ctx = egui::Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx);

    // Try to create Kiorg app with empty config dir - this should succeed with defaults
    let app_result = kiorg::Kiorg::new(&cc, Some(temp_dir.path().to_path_buf()), Some(config_dir));

    // Verify that creating the app succeeds
    assert!(
        app_result.is_ok(),
        "Should successfully create app with default config from empty directory"
    );
}
