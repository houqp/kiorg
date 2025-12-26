#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use kiorg::config::shortcuts::ShortcutAction;
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
fn test_missing_key_field() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with a shortcut missing the required 'key' field
    let toml_content = r"
# Invalid shortcut configuration - missing key field
[[shortcuts.MoveDown]]
shift = true
";

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Try to load the config and expect an error
    let result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(
        result.is_err(),
        "Should return an error for missing key field"
    );

    // Check that the error message mentions the missing field
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("missing field") && error.contains("key"),
        "Error should mention missing 'key' field, got: {error}"
    );
}

#[test]
fn test_invalid_action_name() {
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

    // Try to load the config and expect an error
    let result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(
        result.is_err(),
        "Should return an error for invalid action name"
    );

    // Check that the error message mentions the unknown variant
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("unknown variant") && error.contains("NonExistentAction"),
        "Error should mention unknown variant 'NonExistentAction', got: {error}"
    );
}

#[test]
fn test_invalid_toml_syntax() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with invalid syntax
    let toml_content = r#"
# Invalid TOML syntax
[[shortcuts.MoveDown]
key = "j"
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Try to load the config and expect an error
    let result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(
        result.is_err(),
        "Should return an error for invalid TOML syntax"
    );

    // Check that the error message mentions syntax error
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("expected") || error.contains("syntax"),
        "Error should mention syntax error, got: {error}"
    );
}

#[test]
fn test_invalid_shortcut_format() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with wrong nesting structure
    let toml_content = r#"
# Invalid shortcut format - wrong nesting
[shortcuts.MoveDown]  # Using single brackets instead of double
key = "j"
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Try to load the config and expect an error
    let result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(
        result.is_err(),
        "Should return an error for invalid shortcut format"
    );

    // The error message will vary depending on how the deserializer handles this case
    let error = result.unwrap_err().to_string();
    assert!(
        !error.is_empty(),
        "Should have a non-empty error message, got: {error}"
    );
}

#[test]
fn test_empty_key_string() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with an empty key string
    let toml_content = r#"
# Invalid shortcut configuration - empty key string
[[shortcuts.MoveDown]]
key = ""
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Try to load the config
    // Note: This might not actually error since empty strings might be allowed by the deserializer,
    // but the shortcut would be non-functional since it wouldn't match any key
    let result = kiorg::config::load_config_with_override(Some(&config_dir));

    // If it doesn't error, the shortcut should at least have an empty key
    if let Ok(config) = result
        && let Some(shortcuts) = &config.shortcuts
        && let Some(move_down_shortcuts) =
            shortcuts.get(&kiorg::config::shortcuts::ShortcutAction::MoveDown)
        && !move_down_shortcuts.is_empty()
    {
        assert_eq!(move_down_shortcuts[0].key, "", "Key should be empty string");
    }
}

#[test]
fn test_invalid_boolean_value() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with an invalid boolean value
    let toml_content = r#"
# Invalid shortcut configuration - invalid boolean value
[[shortcuts.MoveDown]]
key = "j"
shift = "yes"  # Should be true/false, not a string
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Try to load the config and expect an error
    let result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(
        result.is_err(),
        "Should return an error for invalid boolean value"
    );

    // Check that the error message mentions the type mismatch
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("invalid type") || error.contains("expected a boolean"),
        "Error should mention type mismatch for boolean, got: {error}"
    );
}

#[test]
fn test_duplicate_shortcut_definition() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with duplicate shortcut definitions
    // This should not error as it's valid TOML, but it's a logical error in the config
    let toml_content = r#"
# Duplicate shortcut definitions - should be merged into a list
[[shortcuts.MoveDown]]
key = "j"

[[shortcuts.MoveDown]]
key = "j"
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Try to load the config - this should succeed but result in duplicates
    let result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(
        result.is_ok(),
        "Should not error for duplicate shortcut definitions"
    );

    // Check that there are two identical shortcuts for MoveDown
    let config = result.unwrap();
    if let Some(shortcuts) = &config.shortcuts {
        if let Some(move_down_shortcuts) =
            shortcuts.get(&kiorg::config::shortcuts::ShortcutAction::MoveDown)
        {
            assert_eq!(
                move_down_shortcuts.len(),
                2,
                "Should have two shortcuts for MoveDown"
            );
            assert_eq!(
                move_down_shortcuts[0].key, "j",
                "First shortcut key should be 'j'"
            );
            assert_eq!(
                move_down_shortcuts[1].key, "j",
                "Second shortcut key should be 'j'"
            );
        } else {
            panic!("MoveDown shortcuts not found");
        }
    } else {
        panic!("Shortcuts not found in config");
    }
}

#[test]
fn test_invalid_key_name() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with an invalid key name
    let toml_content = r#"
# Invalid shortcut configuration - invalid key name
[[shortcuts.MoveDown]]
key = "invalid_key_name"
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Try to load the config - this should succeed but the shortcut won't work
    // since to_egui_key() will return None for unknown keys
    let result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(result.is_ok(), "Should not error for invalid key name");

    // Check that the shortcut has the invalid key name
    let config = result.unwrap();
    if let Some(shortcuts) = &config.shortcuts {
        if let Some(move_down_shortcuts) = shortcuts.get(&ShortcutAction::MoveDown) {
            assert_eq!(
                move_down_shortcuts[0].key, "invalid_key_name",
                "Key should be 'invalid_key_name'"
            );
        } else {
            panic!("MoveDown shortcuts not found");
        }
    } else {
        panic!("Shortcuts not found in config");
    }
}

#[test]
fn test_mixed_valid_and_invalid_shortcuts() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with both valid and invalid shortcuts
    let toml_content = r#"
# Mix of valid and invalid shortcuts
[[shortcuts.MoveDown]]
key = "j"

[[shortcuts.NonExistentAction]]
key = "x"
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Try to load the config and expect an error due to the invalid action
    let result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(
        result.is_err(),
        "Should return an error for invalid action name"
    );

    // Check that the error message mentions the unknown variant
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("unknown variant") && error.contains("NonExistentAction"),
        "Error should mention unknown variant 'NonExistentAction', got: {error}"
    );
}

#[test]
fn test_invalid_nested_structure() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with an invalid nested structure
    let toml_content = r#"
# Invalid nested structure
[shortcuts]
MoveDown = { key = "j" }  # Should be [[shortcuts.MoveDown]] with key inside
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Try to load the config and expect an error
    let result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(
        result.is_err(),
        "Should return an error for invalid nested structure"
    );

    // The error message will vary depending on how the deserializer handles this case
    let error = result.unwrap_err().to_string();
    assert!(
        !error.is_empty(),
        "Should have a non-empty error message, got: {error}"
    );
}

#[test]
fn test_completely_malformed_toml() {
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

    // Try to load the config and expect an error
    let result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(
        result.is_err(),
        "Should return an error for completely malformed TOML"
    );

    // Check that the error message mentions parsing error
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("parse") || error.contains("expected") || error.contains("syntax"),
        "Error should mention parsing error, got: {error}"
    );
}
