use egui::Key;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use ui_test_helpers::{create_harness_with_config_dir, ctrl_modifiers};

#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

// Helper function to create a config.toml file with custom TOML content
fn create_config_file(config_dir: &PathBuf, toml_content: &str) {
    // Create the config directory if it doesn't exist
    fs::create_dir_all(config_dir).unwrap();

    // Write to config.toml
    fs::write(config_dir.join("config.toml"), toml_content).unwrap();
}

#[test]
fn test_custom_shortcuts() {
    // Create a temporary directory for the test files
    let temp_dir = tempdir().unwrap();

    // Create some test files in the temp directory
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    let file3 = temp_dir.path().join("file3.txt");

    std::fs::write(&file1, "test content").unwrap();
    std::fs::write(&file2, "test content").unwrap();
    std::fs::write(&file3, "test content").unwrap();

    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with custom shortcuts
    let toml_content = r#"
# Custom shortcuts configuration
[[shortcuts.MoveDown]]
key = "n"
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Create the test harness with the custom config directory
    let mut harness = create_harness_with_config_dir(&temp_dir, config_temp_dir);

    // Refresh entries to make sure the files are loaded
    harness.state_mut().refresh_entries();
    harness.step();

    // Verify initial selection is at index 0
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );

    // Press the new shortcut key 'n' to move down
    harness.key_press(Key::N);
    harness.step();

    // Verify selection moved down to index 1
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        1
    );

    // The original 'j' key should no longer work for moving down
    harness.key_press(Key::J);
    harness.step();

    // Selection should still be at index 1
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        1
    );
}

#[test]
fn test_default_shortcuts() {
    // Create a temporary directory for the test files
    let temp_dir = tempdir().unwrap();

    // Create some test files in the temp directory
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    let file3 = temp_dir.path().join("file3.txt");

    std::fs::write(&file1, "test content").unwrap();
    std::fs::write(&file2, "test content").unwrap();
    std::fs::write(&file3, "test content").unwrap();

    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create an empty config file to use default shortcuts
    let toml_content = r"
# Empty config file - will use default shortcuts
";

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Create the test harness with the custom config directory
    let mut harness = create_harness_with_config_dir(&temp_dir, config_temp_dir);

    // Refresh entries to make sure the files are loaded
    harness.state_mut().refresh_entries();
    harness.step();

    // Verify initial selection is at index 0
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );

    // Press the default shortcut key 'j' to move down
    harness.key_press(Key::J);
    harness.step();

    // Verify selection moved down to index 1
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        1
    );

    // Press 'k' to move back up
    harness.key_press(Key::K);
    harness.step();

    // Selection should be back at index 0
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );
}

#[test]
fn test_toml_config_file() {
    // Create a temporary directory for the test files
    let temp_dir = tempdir().unwrap();

    // Create some test files in the temp directory
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    let file3 = temp_dir.path().join("file3.txt");

    std::fs::write(&file1, "test content").unwrap();
    std::fs::write(&file2, "test content").unwrap();
    std::fs::write(&file3, "test content").unwrap();

    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with multiple custom shortcuts
    let toml_content = r#"
# Custom shortcuts configuration with multiple shortcuts
[[shortcuts.MoveDown]]
key = "m"

[[shortcuts.MoveUp]]
key = "i"

[[shortcuts.DeleteEntry]]
key = "d"
shift = true
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Create the test harness with the custom config directory
    let mut harness = create_harness_with_config_dir(&temp_dir, config_temp_dir);

    // Refresh entries to make sure the files are loaded
    harness.state_mut().refresh_entries();
    harness.step();

    // Verify initial selection is at index 0
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );

    // Press the custom shortcut key 'm' to move down
    harness.key_press(Key::M);
    harness.step();

    // Verify selection moved down to index 1
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        1
    );

    // Press 'i' to move back up
    harness.key_press(Key::I);
    harness.step();

    // Selection should be back at index 0
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );

    // The original 'j' key should no longer work for moving down
    harness.key_press(Key::J);
    harness.step();

    // Selection should still be at index 0
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );
}

#[test]
fn test_modifier_shortcuts() {
    // Create a temporary directory for the test files
    let temp_dir = tempdir().unwrap();

    // Create some test files in the temp directory
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    let file3 = temp_dir.path().join("file3.txt");

    std::fs::write(&file1, "test content").unwrap();
    std::fs::write(&file2, "test content").unwrap();
    std::fs::write(&file3, "test content").unwrap();

    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with custom shortcuts using modifiers
    // Include both the default MoveUp shortcut and a custom MoveDown shortcut
    let toml_content = r#"
# Custom shortcuts configuration with modifiers
[[shortcuts.MoveDown]]
key = "z"
ctrl = true

[[shortcuts.MoveUp]]
key = "m"
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Create the test harness
    let mut harness = create_harness_with_config_dir(&temp_dir, config_temp_dir);
    harness.step();

    // Verify initial selection is at index 0
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );

    // Press 'z' without Ctrl - should not move
    harness.key_press(Key::Z);
    harness.step();
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );

    // Press Ctrl+z - should move down
    harness.key_press_modifiers(ctrl_modifiers(), Key::Z);
    harness.step();

    // Verify selection moved down to index 1
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        1
    );

    // Move back to index 0 using the explicitly defined MoveUp shortcut
    harness.key_press(Key::M);
    harness.step();

    // Verify selection is back at index 0
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );
}

#[test]
fn test_shortcut_merging() {
    // Create a temporary directory for the test files
    let temp_dir = tempdir().unwrap();

    // Create some test files in the temp directory
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    let file3 = temp_dir.path().join("file3.txt");

    std::fs::write(&file1, "test content").unwrap();
    std::fs::write(&file2, "test content").unwrap();
    std::fs::write(&file3, "test content").unwrap();

    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with only one custom shortcut
    // This should override just the MoveDown action while keeping all other default shortcuts
    let toml_content = r#"
# Custom shortcuts configuration with only one action
[[shortcuts.MoveDown]]
key = "n"
"#;
    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Create the test harness
    let mut harness = create_harness_with_config_dir(&temp_dir, config_temp_dir);

    harness.step();
    // Verify initial selection is at index 0
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );

    // Test that the custom shortcut for MoveDown works
    harness.key_press(Key::N);
    harness.step();

    // Verify selection moved down to index 1
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        1
    );

    // Test that the default shortcut for MoveUp still works
    // This verifies that default shortcuts for actions not overridden by the user are preserved
    harness.key_press(Key::K);
    harness.step();

    // Verify selection moved back up to index 0
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );

    // The original 'j' key should no longer work for moving down
    // since it was replaced by the custom 'n' shortcut
    harness.key_press(Key::J);
    harness.step();

    // Selection should still be at index 0
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );
}

#[test]
fn test_shortcut_conflict_detection() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with conflicting shortcuts
    // Both MoveDown and DeleteEntry are assigned to the same shortcut (d key)
    let toml_content = r#"
# Config with conflicting shortcuts
[[shortcuts.MoveDown]]
key = "d"

[[shortcuts.DeleteEntry]]
key = "d"
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Try to load the config and expect a ShortcutConflictError
    let result = kiorg::config::load_config_with_override(Some(&config_dir));

    // Verify that we got a ShortcutConflictError
    assert!(
        result.is_err(),
        "Should return an error for conflicting shortcuts"
    );

    // Check that the error is a ShortcutConflictError
    if let Err(kiorg::config::ConfigError::ShortcutConflict(conflict, _)) = result {
        // Verify the conflicting shortcut details
        assert_eq!(conflict.shortcut.key, "d");
        assert!(
            (conflict.action1 == kiorg::config::shortcuts::ShortcutAction::MoveDown
                && conflict.action2 == kiorg::config::shortcuts::ShortcutAction::DeleteEntry)
                || (conflict.action1 == kiorg::config::shortcuts::ShortcutAction::DeleteEntry
                    && conflict.action2 == kiorg::config::shortcuts::ShortcutAction::MoveDown),
            "Error should identify the correct conflicting actions"
        );
    } else {
        panic!("Expected ShortcutConflictError, got: {result:?}");
    }
}
#[test]
#[cfg(target_os = "windows")]
fn test_disallow_ctrl_shift_v_config() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with Ctrl+Shift+V
    let toml_content = r#"
# Config with reserved Windows shortcut
[[shortcuts.MoveDown]]
key = "v"
ctrl = true
shift = true
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Try to load the config
    let result = kiorg::config::load_config_with_override(Some(&config_dir));

    // Verify that we got an error and it's a value error mentioning the reservation
    assert!(
        result.is_err(),
        "Should return an error for reserved Windows shortcut"
    );

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("reserved on Windows"),
        "Error message should mention it's reserved on Windows, got: {error_msg}"
    );
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_allow_ctrl_shift_v_config_non_windows() {
    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with Ctrl+Shift+V
    let toml_content = r#"
# Config with Ctrl+Shift+V which is allowed on non-Windows
[[shortcuts.MoveDown]]
key = "v"
ctrl = true
shift = true
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Try to load the config
    let result = kiorg::config::load_config_with_override(Some(&config_dir));

    // Verify that it works on non-Windows
    assert!(
        result.is_ok(),
        "Should NOT return an error for Ctrl+Shift+V on non-Windows platforms"
    );
}
