#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use kiorg::ui::popup::PopupType;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use ui_test_helpers::{create_harness_with_config_dir, ctrl_modifiers, shift_modifiers};

// Helper function to create a config.toml file with custom TOML content
fn create_config_file(config_dir: &PathBuf, toml_content: &str) {
    // Create the config directory if it doesn't exist
    fs::create_dir_all(config_dir).unwrap();

    // Write to config.toml
    fs::write(config_dir.join("config.toml"), toml_content).unwrap();
}

#[test]
fn test_custom_shortcut_tree_building() {
    let temp_dir = tempdir().unwrap();

    // Create test files
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    std::fs::write(&file1, "test content").unwrap();
    std::fs::write(&file2, "test content").unwrap();

    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Test single character first - this should work
    let toml_content = r#"
[[shortcuts.GoToLastEntry]]
key = "z"
"#;
    create_config_file(&config_dir, toml_content);

    let mut harness = create_harness_with_config_dir(&temp_dir, config_temp_dir);
    harness.state_mut().refresh_entries();

    // Verify z works for GoToLastEntry
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );
    harness.key_press(Key::Z);
    harness.step();
    let entries_count = harness.state().tab_manager.current_tab_ref().entries.len();
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        entries_count - 1,
        "Single character custom shortcut should work"
    );
}

#[test]
fn test_xy_shortcut_only() {
    let temp_dir = tempdir().unwrap();

    // Create test files
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    std::fs::write(&file1, "test content").unwrap();
    std::fs::write(&file2, "test content").unwrap();

    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Test with mn only - use an action that doesn't have built-in multi-character shortcuts
    let toml_content = r#"
[[shortcuts.GoToLastEntry]]
key = "mn"
"#;
    create_config_file(&config_dir, toml_content);

    let mut harness = create_harness_with_config_dir(&temp_dir, config_temp_dir);
    harness.state_mut().refresh_entries();

    // Test the mn shortcut for GoToLastEntry
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        0
    );
    harness.key_press(Key::M);
    harness.step();
    harness.key_press(Key::N);
    harness.step();

    let entries_count = harness.state().tab_manager.current_tab_ref().entries.len();
    let current_selection = harness.state().tab_manager.current_tab_ref().selected_index;
    if current_selection != entries_count - 1 {
        panic!(
            "mn shortcut for GoToLastEntry not working. Expected {}, got {}",
            entries_count - 1,
            current_selection
        );
    }
}

#[test]
fn test_four_character_shortcut_sequence() {
    let temp_dir = tempdir().unwrap();

    // Create some test files in the temp directory
    let file1 = temp_dir.path().join("file1.txt");
    std::fs::write(&file1, "test content").unwrap();

    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with custom 4-character shortcuts
    let toml_content = r#"
# Custom shortcuts configuration for multi-character sequences
[[shortcuts.ShowTeleport]]
key = "mnws"
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Create the test harness with custom config
    let mut harness = create_harness_with_config_dir(&temp_dir, config_temp_dir);

    // Refresh entries to make sure the files are loaded
    harness.state_mut().refresh_entries();

    // Initially no popup should be showing
    assert_eq!(harness.state().show_popup, None);

    // Press the 4-character sequence: m, n, w, s
    let keys = [Key::M, Key::N, Key::W, Key::S];

    // Press first 3 keys - should not trigger action
    for &key in keys.iter().take(3) {
        harness.key_press(key);
        harness.step();

        // Should still have no popup
        assert_eq!(harness.state().show_popup, None);
    }

    // Press the final key to complete sequence
    harness.key_press(keys[3]);
    harness.step();

    // Should trigger Teleport popup
    assert!(matches!(
        harness.state().show_popup,
        Some(PopupType::Teleport(_))
    ));
}

#[test]
fn test_invalid_sequence_clears_buffer() {
    let temp_dir = tempdir().unwrap();

    // Create some test files in the temp directory
    let file1 = temp_dir.path().join("file1.txt");
    std::fs::write(&file1, "test content").unwrap();

    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with custom shortcuts
    let toml_content = r#"
# Custom shortcuts configuration for multi-character sequences
[[shortcuts.ShowHelp]]
key = "mne"
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Create the test harness with custom config
    let mut harness = create_harness_with_config_dir(&temp_dir, config_temp_dir);

    // Refresh entries to make sure the files are loaded
    harness.state_mut().refresh_entries();

    // Press 'm' and 'n' to start building sequence
    harness.key_press(Key::M);
    harness.step();
    harness.key_press(Key::N);
    harness.step();

    // Should have no popup but buffer should be building
    assert_eq!(harness.state().show_popup, None);

    // Press 's' instead of 'e' - this should clear the buffer and not trigger action
    harness.key_press(Key::S); // This doesn't match the expected 'e'
    harness.step();

    // Should clear the buffer and not show any popup
    assert_eq!(harness.state().show_popup, None);
}

#[test]
fn test_nested_sequence_support() {
    let temp_dir = tempdir().unwrap();

    // Create some test files in the temp directory
    let file1 = temp_dir.path().join("file1.txt");
    std::fs::write(&file1, "test content").unwrap();

    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with custom nested shortcuts
    let toml_content = r#"
# Custom shortcuts configuration with nested sequences
[[shortcuts.ShowTeleport]]
key = "mn"

[[shortcuts.ShowHelp]]  
key = "ws"
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Create the test harness with custom config
    let mut harness = create_harness_with_config_dir(&temp_dir, config_temp_dir);

    // Refresh entries to make sure the files are loaded
    harness.state_mut().refresh_entries();

    // First test: trigger "mn" sequence (Teleport)
    harness.key_press(Key::M);
    harness.step();

    // Should be building sequence
    assert_eq!(harness.state().show_popup, None);

    harness.key_press(Key::N);
    harness.step();

    // Should trigger Teleport popup
    assert!(matches!(
        harness.state().show_popup,
        Some(PopupType::Teleport(_))
    ));

    // Clear the popup for next test
    harness.state_mut().show_popup = None;

    // Second test: trigger "ws" sequence (Help)
    harness.key_press(Key::W);
    harness.step();
    harness.key_press(Key::S);
    harness.step();

    // Should trigger Help popup
    assert_eq!(harness.state().show_popup, Some(PopupType::Help));
}

#[test]
fn test_multi_key_sequence_with_modifiers() {
    let temp_dir = tempdir().unwrap();

    // Create some test files in the temp directory
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    std::fs::write(&file1, "test content").unwrap();
    std::fs::write(&file2, "test content").unwrap();

    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a TOML config file with multi-character shortcuts that have modifiers
    let toml_content = r#"
# Multi-character shortcuts with Ctrl modifier
[[shortcuts.ShowHelp]]
key = "mn"
ctrl = true

# Multi-character shortcuts with Shift modifier  
[[shortcuts.ShowTeleport]]
key = "ws"
shift = true
"#;

    // Write the config file
    create_config_file(&config_dir, toml_content);

    // Create the test harness with custom config
    let mut harness = create_harness_with_config_dir(&temp_dir, config_temp_dir);

    // Refresh entries to make sure the files are loaded
    harness.state_mut().refresh_entries();

    // Test 1: Ctrl+m, Ctrl+n should trigger Help popup
    assert_eq!(harness.state().show_popup, None);

    // Press Ctrl+m (first key with modifier)
    harness.key_press_modifiers(ctrl_modifiers(), Key::M);
    harness.step();

    // Should be building sequence, no popup yet
    assert_eq!(harness.state().show_popup, None);

    // Press Ctrl+n (second key with modifier to complete sequence)
    harness.key_press_modifiers(ctrl_modifiers(), Key::N);
    harness.step();

    // Should trigger Help popup
    assert_eq!(harness.state().show_popup, Some(PopupType::Help));

    // Clear popup for next test
    harness.state_mut().show_popup = None;

    // Test 2: Verify that pressing m,n without Ctrl does NOT trigger the action
    harness.key_press(Key::M);
    harness.step();
    harness.key_press(Key::N);
    harness.step();

    // Should NOT trigger Help popup (no modifier)
    assert_eq!(harness.state().show_popup, None);

    // Test 3: Shift+w, Shift+s should trigger Teleport popup

    // Press Shift+w
    harness.key_press_modifiers(shift_modifiers(), Key::W);
    harness.step();

    // Should be building sequence, no popup yet
    assert_eq!(harness.state().show_popup, None);

    // Press Shift+s to complete sequence
    harness.key_press_modifiers(shift_modifiers(), Key::S);
    harness.step();

    // Should trigger Teleport popup
    assert!(matches!(
        harness.state().show_popup,
        Some(PopupType::Teleport(_))
    ));

    // Clear popup for next test
    harness.state_mut().show_popup = None;

    // Test 4: Verify that pressing w,s without Shift does NOT trigger the action
    harness.key_press(Key::W);
    harness.step();
    harness.key_press(Key::S);
    harness.step();

    // Should NOT trigger Teleport popup (no modifier)
    assert_eq!(harness.state().show_popup, None);

    // Test 5: Verify that mixing modifiers doesn't work (Ctrl+m, Shift+n)
    harness.key_press_modifiers(ctrl_modifiers(), Key::M);
    harness.step();
    harness.key_press_modifiers(shift_modifiers(), Key::N);
    harness.step();

    // Should NOT trigger Help popup (inconsistent modifiers)
    assert_eq!(harness.state().show_popup, None);
}
