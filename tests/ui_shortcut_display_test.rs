mod ui_test_helpers;

use kiorg::config::shortcuts::{shortcuts_helpers, KeyboardShortcut, ShortcutAction};
use std::collections::HashMap;

#[test]
fn test_arrow_key_display() {
    // Create a custom shortcuts map for testing
    let mut shortcuts = HashMap::new();

    // Add arrow key shortcuts
    shortcuts.insert(
        ShortcutAction::MoveUp,
        vec![
            KeyboardShortcut::new("up"),
            KeyboardShortcut::new("arrow_up"),
        ],
    );

    shortcuts.insert(
        ShortcutAction::MoveDown,
        vec![
            KeyboardShortcut::new("down"),
            KeyboardShortcut::new("arrow_down"),
        ],
    );

    shortcuts.insert(
        ShortcutAction::GoToParentDirectory,
        vec![
            KeyboardShortcut::new("left"),
            KeyboardShortcut::new("arrow_left"),
        ],
    );

    shortcuts.insert(
        ShortcutAction::OpenDirectory,
        vec![
            KeyboardShortcut::new("right"),
            KeyboardShortcut::new("arrow_right"),
        ],
    );

    // Test up arrow display
    let up_display = shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::MoveUp);
    assert!(
        up_display.contains("⬆"),
        "Up arrow should be displayed as ⬆ symbol, got: {}",
        up_display
    );

    // Test down arrow display
    let down_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::MoveDown);
    assert!(
        down_display.contains("⬇"),
        "Down arrow should be displayed as ⬇ symbol, got: {}",
        down_display
    );

    // Test left arrow display
    let left_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::GoToParentDirectory);
    assert!(
        left_display.contains("⬅"),
        "Left arrow should be displayed as ⬅ symbol, got: {}",
        left_display
    );

    // Test right arrow display
    let right_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::OpenDirectory);
    assert!(
        right_display.contains("➡"),
        "Right arrow should be displayed as ➡ symbol, got: {}",
        right_display
    );
}

#[test]
fn test_special_key_display() {
    // Create a custom shortcuts map for testing
    let mut shortcuts = HashMap::new();

    // Add special key shortcuts
    shortcuts.insert(
        ShortcutAction::OpenDirectoryOrFile,
        vec![KeyboardShortcut::new("enter")],
    );

    shortcuts.insert(
        ShortcutAction::SelectEntry,
        vec![KeyboardShortcut::new("space")],
    );

    // Test enter key display
    let enter_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::OpenDirectoryOrFile);
    assert!(
        enter_display.contains("Enter"),
        "Enter key should be displayed as 'Enter', got: {}",
        enter_display
    );

    // Test space key display
    let space_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::SelectEntry);
    assert!(
        space_display.contains("Space"),
        "Space key should be displayed as 'Space', got: {}",
        space_display
    );
}

#[test]
fn test_regular_key_display() {
    // Create a custom shortcuts map for testing
    let mut shortcuts = HashMap::new();

    // Add regular key shortcuts
    shortcuts.insert(ShortcutAction::AddEntry, vec![KeyboardShortcut::new("a")]);

    shortcuts.insert(
        ShortcutAction::DeleteEntry,
        vec![KeyboardShortcut::new("d")],
    );

    // Test regular key display
    let a_display = shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::AddEntry);
    assert_eq!(
        a_display, "a",
        "Regular key 'a' should be displayed as 'a', got: {}",
        a_display
    );

    let d_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::DeleteEntry);
    assert_eq!(
        d_display, "d",
        "Regular key 'd' should be displayed as 'd', got: {}",
        d_display
    );
}

#[test]
fn test_shortcut_with_modifiers() {
    // Create a custom shortcuts map for testing
    let mut shortcuts = HashMap::new();

    // Add shortcuts with modifiers
    shortcuts.insert(
        ShortcutAction::CloseCurrentTab,
        vec![KeyboardShortcut::new("c").with_ctrl()],
    );

    shortcuts.insert(
        ShortcutAction::ShowHelp,
        vec![KeyboardShortcut::new("?").with_shift()],
    );

    shortcuts.insert(
        ShortcutAction::OpenTerminal,
        vec![KeyboardShortcut::new("t").with_shift()],
    );

    // Add a shortcut with multiple modifiers
    shortcuts.insert(
        ShortcutAction::Exit,
        vec![KeyboardShortcut {
            key: "q".to_string(),
            shift: true,
            ctrl: true,
            alt: false,
            mac_cmd: false,
            namespace: false,
        }],
    );

    // Test Ctrl+C display
    let ctrl_c_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::CloseCurrentTab);
    assert_eq!(
        ctrl_c_display, "Ctrl+c",
        "Ctrl+C should be displayed as 'Ctrl+c', got: {}",
        ctrl_c_display
    );

    // Test Shift+? display
    let shift_question_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::ShowHelp);
    assert_eq!(
        shift_question_display, "Shift+?",
        "Shift+? should be displayed as 'Shift+?', got: {}",
        shift_question_display
    );

    // Test Shift+T display
    let shift_t_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::OpenTerminal);
    assert_eq!(
        shift_t_display, "Shift+t",
        "Shift+T should be displayed as 'Shift+t', got: {}",
        shift_t_display
    );

    // Test Ctrl+Shift+Q display
    let ctrl_shift_q_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::Exit);
    assert_eq!(
        ctrl_shift_q_display, "Ctrl+Shift+q",
        "Ctrl+Shift+Q should be displayed as 'Ctrl+Shift+q', got: {}",
        ctrl_shift_q_display
    );
}

#[test]
fn test_multiple_shortcuts_display() {
    // Create a custom shortcuts map for testing
    let mut shortcuts = HashMap::new();

    // Add multiple shortcuts for the same action
    shortcuts.insert(
        ShortcutAction::MoveDown,
        vec![KeyboardShortcut::new("j"), KeyboardShortcut::new("down")],
    );

    // Test multiple shortcuts display
    let move_down_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::MoveDown);
    assert!(
        move_down_display.contains("j") && move_down_display.contains("⬇"),
        "Multiple shortcuts should be displayed with separator, got: {}",
        move_down_display
    );

    // Check for separator
    assert!(
        move_down_display.contains(" or "),
        "Multiple shortcuts should be separated by ' or ', got: {}",
        move_down_display
    );
}

#[test]
fn test_no_shortcuts_display() {
    // Create an empty shortcuts map
    let shortcuts = HashMap::new();

    // Test display for action with no shortcuts
    let no_shortcut_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::AddEntry);
    assert_eq!(
        no_shortcut_display, "Not assigned",
        "Action with no shortcuts should display 'Not assigned', got: {}",
        no_shortcut_display
    );
}
