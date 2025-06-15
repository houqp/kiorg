#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use kiorg::config;
use std::fs;
use tempfile::tempdir;

/// Test that only user-provided shortcuts are serialized to config file,
/// not the full merged shortcuts (defaults + user overrides).
/// The config loading should only return user overrides, and merging with
/// defaults happens at runtime in the Kiorg struct.
#[test]
fn test_shortcut_serialization_only_saves_user_overrides() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with only user custom shortcuts
    let input_toml_content = r#"
# Custom shortcuts configuration - only user overrides
[[shortcuts.MoveDown]]
key = "s"

[[shortcuts.MoveUp]]
key = "w"
"#;

    // Write the initial config file
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.toml"), input_toml_content).unwrap();

    // Load the config
    let loaded_config = config::load_config_with_override(Some(&config_dir))
        .expect("Should load config successfully");

    // The loaded config should ONLY contain user overrides, not defaults
    let shortcuts = loaded_config
        .shortcuts
        .as_ref()
        .expect("Should have shortcuts");

    // User override: MoveDown should be 's'
    let move_down_shortcuts = shortcuts
        .get(&kiorg::config::shortcuts::ShortcutAction::MoveDown)
        .expect("Should have MoveDown shortcut");
    assert_eq!(move_down_shortcuts.len(), 1);
    assert_eq!(move_down_shortcuts[0].key, "s");

    // User override: MoveUp should be 'w'
    let move_up_shortcuts = shortcuts
        .get(&kiorg::config::shortcuts::ShortcutAction::MoveUp)
        .expect("Should have MoveUp shortcut");
    assert_eq!(move_up_shortcuts.len(), 1);
    assert_eq!(move_up_shortcuts[0].key, "w");

    // The loaded config should NOT have default shortcuts that weren't overridden
    // For example, DeleteEntry should not exist in the loaded config
    let delete_shortcuts = shortcuts.get(&kiorg::config::shortcuts::ShortcutAction::DeleteEntry);
    assert!(
        delete_shortcuts.is_none(),
        "Should not have default shortcuts in loaded config"
    );

    // Now save the config back to file
    config::save_config_with_override(&loaded_config, Some(&config_dir))
        .expect("Should save config successfully");

    // Read the saved config file content
    let saved_content =
        fs::read_to_string(config_dir.join("config.toml")).expect("Should read saved config file");

    println!("Saved config content:\n{}", saved_content);

    // The saved content should ONLY contain user overrides, not all the defaults
    // It should contain the user's custom shortcuts
    assert!(
        saved_content.contains("key = \"s\""),
        "Should save user override for MoveDown"
    );
    assert!(
        saved_content.contains("key = \"w\""),
        "Should save user override for MoveUp"
    );

    // It should NOT contain default shortcuts that weren't overridden
    // For example, it should not contain the default DeleteEntry shortcut
    assert!(
        !saved_content.contains("DeleteEntry"),
        "Should not save default shortcuts that weren't overridden"
    );
    assert!(
        !saved_content.contains("key = \"d\""),
        "Should not save default 'd' shortcut for DeleteEntry"
    );

    // It should not contain all the default shortcuts
    assert!(
        !saved_content.contains("AddEntry"),
        "Should not save default AddEntry shortcut"
    );
    assert!(
        !saved_content.contains("CopyEntry"),
        "Should not save default CopyEntry shortcut"
    );

    // The saved config should be much smaller than a config with all defaults
    let line_count = saved_content.lines().count();
    assert!(
        line_count < 20,
        "Saved config should be concise, got {} lines",
        line_count
    );
}

/// Test that when no user shortcuts are provided, saving the config
/// doesn't write out all the default shortcuts
#[test]
fn test_no_user_shortcuts_saves_empty_shortcuts() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with no shortcuts
    let input_toml_content = r#"
# Config with no custom shortcuts
theme = "dark"
"#;

    // Write the initial config file
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.toml"), input_toml_content).unwrap();

    // Load the config
    let loaded_config = config::load_config_with_override(Some(&config_dir))
        .expect("Should load config successfully");

    // Save the config back to file
    config::save_config_with_override(&loaded_config, Some(&config_dir))
        .expect("Should save config successfully");

    // Read the saved config file content
    let saved_content =
        fs::read_to_string(config_dir.join("config.toml")).expect("Should read saved config file");

    println!(
        "Saved config content (no user shortcuts):\n{}",
        saved_content
    );

    // The saved content should not contain any shortcuts section since no user overrides were provided
    assert!(
        !saved_content.contains("[shortcuts."),
        "Should not save shortcuts section when no user overrides"
    );
    assert!(
        !saved_content.contains("MoveDown"),
        "Should not save default MoveDown shortcut"
    );
    assert!(
        !saved_content.contains("key = \"j\""),
        "Should not save default 'j' shortcut"
    );

    // It should contain the theme setting
    assert!(
        saved_content.contains("theme"),
        "Should preserve other config settings"
    );
}

/// Test that modifying shortcuts in the app and then saving only saves user overrides
#[test]
fn test_runtime_shortcut_modification_serialization() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Start with an empty config
    let input_toml_content = r#"
# Empty config
"#;

    // Write the initial config file
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.toml"), input_toml_content).unwrap();

    // Load the config
    let mut loaded_config = config::load_config_with_override(Some(&config_dir))
        .expect("Should load config successfully");

    // Simulate adding a user shortcut override
    let mut user_shortcuts = kiorg::config::shortcuts::Shortcuts::new();
    user_shortcuts.add_shortcut(
        kiorg::config::shortcuts::KeyboardShortcut::new("m"),
        kiorg::config::shortcuts::ShortcutAction::MoveDown,
    );
    loaded_config.shortcuts = Some(user_shortcuts);

    // Save the modified config
    config::save_config_with_override(&loaded_config, Some(&config_dir))
        .expect("Should save config successfully");

    // Read the saved config file content
    let saved_content =
        fs::read_to_string(config_dir.join("config.toml")).expect("Should read saved config file");

    println!(
        "Saved config content (with runtime modification):\n{}",
        saved_content
    );

    // The saved content should contain the user's modification
    assert!(
        saved_content.contains("key = \"m\""),
        "Should save user's shortcut modification"
    );
    assert!(
        saved_content.contains("MoveDown"),
        "Should save the action that was modified"
    );

    // It should not contain other default shortcuts
    assert!(
        !saved_content.contains("DeleteEntry"),
        "Should not save unmodified default shortcuts"
    );
    assert!(
        !saved_content.contains("AddEntry"),
        "Should not save unmodified default shortcuts"
    );
}
