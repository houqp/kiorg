use egui::{Key, Modifiers};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

use crate::ui_test_helpers::create_harness;

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
    let toml_content = r##"
# Custom shortcuts configuration
[[shortcuts.MoveDown]]
key = "n"
"##;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Create the test harness
    let mut harness = create_harness(&temp_dir);

    // Override the config directory to use our custom one
    harness.state_mut().config_dir_override = Some(config_dir.clone());

    // Reload the config from the custom directory
    harness.state_mut().config =
        kiorg::config::load_config_with_override(Some(&config_dir)).expect("Failed to load config");
    harness.state_mut().refresh_entries();
    harness.step();

    // Verify initial selection is at index 0
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );

    // Press the new shortcut key 'n' to move down
    harness.press_key(Key::N);
    harness.step();

    // Verify selection moved down to index 1
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        1
    );

    // The original 'j' key should no longer work for moving down
    harness.press_key(Key::J);
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
    let toml_content = r##"
# Empty config file - will use default shortcuts
"##;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Create the test harness
    let mut harness = create_harness(&temp_dir);

    // Override the config directory to use our custom one
    harness.state_mut().config_dir_override = Some(config_dir.clone());

    // Reload the config from the custom directory
    harness.state_mut().config =
        kiorg::config::load_config_with_override(Some(&config_dir)).expect("Failed to load config");

    // Refresh entries to make sure the files are loaded
    harness.state_mut().refresh_entries();
    harness.step();

    // Verify initial selection is at index 0
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );

    // Press the default shortcut key 'j' to move down
    harness.press_key(Key::J);
    harness.step();

    // Verify selection moved down to index 1
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        1
    );

    // Press 'k' to move back up
    harness.press_key(Key::K);
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
    let toml_content = r##"
# Custom shortcuts configuration with multiple shortcuts
[[shortcuts.MoveDown]]
key = "m"

[[shortcuts.MoveUp]]
key = "i"

[[shortcuts.DeleteEntry]]
key = "d"
shift = true
"##;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Create the test harness
    let mut harness = create_harness(&temp_dir);

    // Override the config directory to use our custom one
    harness.state_mut().config_dir_override = Some(config_dir.clone());

    // Reload the config from the custom directory
    harness.state_mut().config =
        kiorg::config::load_config_with_override(Some(&config_dir)).expect("Failed to load config");

    // Refresh entries to make sure the files are loaded
    harness.state_mut().refresh_entries();
    harness.step();

    // Verify initial selection is at index 0
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );

    // Press the custom shortcut key 'm' to move down
    harness.press_key(Key::M);
    harness.step();

    // Verify selection moved down to index 1
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        1
    );

    // Press 'i' to move back up
    harness.press_key(Key::I);
    harness.step();

    // Selection should be back at index 0
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );

    // The original 'j' key should no longer work for moving down
    harness.press_key(Key::J);
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
    let toml_content = r##"
# Custom shortcuts configuration with modifiers
[[shortcuts.MoveDown]]
key = "d"
ctrl = true
"##;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Create the test harness
    let mut harness = create_harness(&temp_dir);

    // Override the config directory to use our custom one
    harness.state_mut().config_dir_override = Some(config_dir.clone());

    // Reload the config from the custom directory
    harness.state_mut().config =
        kiorg::config::load_config_with_override(Some(&config_dir)).expect("Failed to load config");
    harness.state_mut().refresh_entries();
    harness.step();

    // Verify initial selection is at index 0
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );

    // Press 'd' without Ctrl - should not move
    harness.press_key(Key::D);
    harness.step();
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );

    // Press Ctrl+D - should move down
    let modifiers = Modifiers {
        ctrl: true,
        ..Default::default()
    };
    harness.press_key_modifiers(modifiers, Key::D);
    harness.step();

    // Verify selection moved down to index 1
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        1
    );
}
