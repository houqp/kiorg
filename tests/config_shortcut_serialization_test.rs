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

    println!("Saved config content:\n{saved_content}");

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
        "Saved config should be concise, got {line_count} lines"
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

    println!("Saved config content (no user shortcuts):\n{saved_content}");

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
    user_shortcuts
        .add_shortcut(
            kiorg::config::shortcuts::KeyboardShortcut::new("m"),
            kiorg::config::shortcuts::ShortcutAction::MoveDown,
        )
        .unwrap();
    loaded_config.shortcuts = Some(user_shortcuts);

    // Save the modified config
    config::save_config_with_override(&loaded_config, Some(&config_dir))
        .expect("Should save config successfully");

    // Read the saved config file content
    let saved_content =
        fs::read_to_string(config_dir.join("config.toml")).expect("Should read saved config file");

    println!("Saved config content (with runtime modification):\n{saved_content}");

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

/// Test serialization of complex shortcuts with multiple keys and modifiers
#[test]
fn test_complex_shortcut_serialization_with_modifiers() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with complex shortcuts containing modifiers and multiple keys
    let input_toml_content = r#"
# Complex shortcuts with modifiers
[[shortcuts.AddEntry]]
key = "n"
ctrl = true

[[shortcuts.AddEntry]]
key = "insert"

[[shortcuts.DeleteEntry]]
key = "d"
shift = true
ctrl = true

[[shortcuts.CopyEntry]]
key = "c"
ctrl = true

[[shortcuts.CopyEntry]]
key = "y"

[[shortcuts.PasteEntry]]
key = "v"
ctrl = true

#[cfg(target_os = "macos")]
[[shortcuts.PasteEntry]]
key = "v"
command = true

[[shortcuts.MoveDown]]
key = "j"
alt = true
"#;

    // Write the initial config file
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.toml"), input_toml_content).unwrap();

    // Load the config
    let loaded_config = config::load_config_with_override(Some(&config_dir))
        .expect("Should load config successfully");

    // Verify the complex shortcuts were loaded correctly
    let shortcuts = loaded_config
        .shortcuts
        .as_ref()
        .expect("Should have shortcuts");

    // Test AddEntry shortcuts (Ctrl+n and Insert)
    let add_entry_shortcuts = shortcuts
        .get(&kiorg::config::shortcuts::ShortcutAction::AddEntry)
        .expect("Should have AddEntry shortcuts");
    assert_eq!(add_entry_shortcuts.len(), 2);

    // Find the Ctrl+n shortcut
    let ctrl_n = add_entry_shortcuts
        .iter()
        .find(|s| s.key == "n" && s.ctrl)
        .expect("Should have Ctrl+n shortcut");
    assert!(ctrl_n.ctrl);
    assert!(!ctrl_n.shift);
    assert!(!ctrl_n.alt);

    // Find the Insert shortcut
    let insert = add_entry_shortcuts
        .iter()
        .find(|s| s.key == "insert")
        .expect("Should have Insert shortcut");
    assert!(!insert.ctrl);
    assert!(!insert.shift);
    assert!(!insert.alt);

    // Test DeleteEntry shortcut (Ctrl+Shift+d)
    let delete_shortcuts = shortcuts
        .get(&kiorg::config::shortcuts::ShortcutAction::DeleteEntry)
        .expect("Should have DeleteEntry shortcut");
    assert_eq!(delete_shortcuts.len(), 1);
    let ctrl_shift_d = &delete_shortcuts[0];
    assert_eq!(ctrl_shift_d.key, "d");
    assert!(ctrl_shift_d.ctrl);
    assert!(ctrl_shift_d.shift);
    assert!(!ctrl_shift_d.alt);

    // Test CopyEntry shortcuts (Ctrl+c and y)
    let copy_shortcuts = shortcuts
        .get(&kiorg::config::shortcuts::ShortcutAction::CopyEntry)
        .expect("Should have CopyEntry shortcuts");
    assert_eq!(copy_shortcuts.len(), 2);

    // Test MoveDown with Alt+j
    let move_down_shortcuts = shortcuts
        .get(&kiorg::config::shortcuts::ShortcutAction::MoveDown)
        .expect("Should have MoveDown shortcut");
    assert_eq!(move_down_shortcuts.len(), 1);
    let alt_j = &move_down_shortcuts[0];
    assert_eq!(alt_j.key, "j");
    assert!(!alt_j.ctrl);
    assert!(!alt_j.shift);
    assert!(alt_j.alt);

    // Save the config back to file
    config::save_config_with_override(&loaded_config, Some(&config_dir))
        .expect("Should save config successfully");

    // Read the saved config file content for debugging
    let saved_content =
        fs::read_to_string(config_dir.join("config.toml")).expect("Should read saved config file");

    println!("Saved complex shortcuts config:\n{saved_content}");

    // Test round-trip: reload the saved config
    let reloaded_config = config::load_config_with_override(Some(&config_dir))
        .expect("Should reload config successfully");

    let reloaded_shortcuts = reloaded_config
        .shortcuts
        .as_ref()
        .expect("Should have shortcuts after reload");

    // Verify the reloaded shortcuts match the original
    let reloaded_add_entry = reloaded_shortcuts
        .get(&kiorg::config::shortcuts::ShortcutAction::AddEntry)
        .expect("Should have AddEntry shortcuts after reload");
    assert_eq!(
        reloaded_add_entry.len(),
        2,
        "Should maintain multiple AddEntry shortcuts"
    );

    // Verify Ctrl+n shortcut is preserved
    let reloaded_ctrl_n = reloaded_add_entry
        .iter()
        .find(|s| s.key == "n" && s.ctrl)
        .expect("Should have Ctrl+n shortcut after reload");
    assert!(reloaded_ctrl_n.ctrl && !reloaded_ctrl_n.shift && !reloaded_ctrl_n.alt);

    // Verify Insert shortcut is preserved
    let reloaded_insert = reloaded_add_entry
        .iter()
        .find(|s| s.key == "insert")
        .expect("Should have Insert shortcut after reload");
    assert!(!reloaded_insert.ctrl && !reloaded_insert.shift && !reloaded_insert.alt);

    let reloaded_delete = reloaded_shortcuts
        .get(&kiorg::config::shortcuts::ShortcutAction::DeleteEntry)
        .expect("Should have DeleteEntry shortcut after reload");
    assert_eq!(reloaded_delete.len(), 1);
    assert!(
        reloaded_delete[0].ctrl && reloaded_delete[0].shift,
        "Should maintain Ctrl+Shift modifiers"
    );

    let reloaded_copy = reloaded_shortcuts
        .get(&kiorg::config::shortcuts::ShortcutAction::CopyEntry)
        .expect("Should have CopyEntry shortcuts after reload");
    assert_eq!(
        reloaded_copy.len(),
        2,
        "Should maintain multiple CopyEntry shortcuts"
    );

    let reloaded_move_down = reloaded_shortcuts
        .get(&kiorg::config::shortcuts::ShortcutAction::MoveDown)
        .expect("Should have MoveDown shortcut after reload");
    assert_eq!(reloaded_move_down.len(), 1);
    let reloaded_alt_j = &reloaded_move_down[0];
    assert_eq!(reloaded_alt_j.key, "j");
    assert!(
        !reloaded_alt_j.ctrl && !reloaded_alt_j.shift && reloaded_alt_j.alt,
        "Should maintain Alt modifier"
    );
}
