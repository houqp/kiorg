#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use kiorg::config::shortcuts::{KeyboardShortcut, ShortcutAction, Shortcuts, shortcuts_helpers};

#[test]
fn test_arrow_key_display() {
    // Create a custom shortcuts map for testing
    let mut shortcuts = Shortcuts::new();

    // Add arrow key shortcuts
    shortcuts
        .set_shortcuts(ShortcutAction::MoveUp, vec![KeyboardShortcut::new("up")])
        .unwrap();

    shortcuts
        .set_shortcuts(
            ShortcutAction::MoveDown,
            vec![KeyboardShortcut::new("down")],
        )
        .unwrap();

    shortcuts
        .set_shortcuts(
            ShortcutAction::GoToParentDirectory,
            vec![KeyboardShortcut::new("left")],
        )
        .unwrap();

    shortcuts
        .set_shortcuts(
            ShortcutAction::OpenDirectory,
            vec![KeyboardShortcut::new("right")],
        )
        .unwrap();

    // Test up arrow display
    let up_display = shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::MoveUp);
    assert!(
        up_display.contains("⬆"),
        "Up arrow should be displayed as ⬆ symbol, got: {up_display}"
    );

    // Test down arrow display
    let down_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::MoveDown);
    assert!(
        down_display.contains("⬇"),
        "Down arrow should be displayed as ⬇ symbol, got: {down_display}"
    );

    // Test left arrow display
    let left_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::GoToParentDirectory);
    assert!(
        left_display.contains("⬅"),
        "Left arrow should be displayed as ⬅ symbol, got: {left_display}"
    );

    // Test right arrow display
    let right_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::OpenDirectory);
    assert!(
        right_display.contains("➡"),
        "Right arrow should be displayed as ➡ symbol, got: {right_display}"
    );
}

#[test]
fn test_special_key_display() {
    // Create a custom shortcuts map for testing
    let mut shortcuts = Shortcuts::new();

    // Add special key shortcuts
    let enter_shortcuts = vec![KeyboardShortcut::new("enter")];
    shortcuts
        .set_shortcuts(ShortcutAction::OpenDirectoryOrFile, enter_shortcuts)
        .unwrap();

    let space_shortcuts = vec![KeyboardShortcut::new("space")];
    shortcuts
        .set_shortcuts(ShortcutAction::SelectEntry, space_shortcuts)
        .unwrap();

    // Test enter key display
    let enter_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::OpenDirectoryOrFile);
    assert!(
        enter_display.contains("Enter"),
        "Enter key should be displayed as 'Enter', got: {enter_display}"
    );

    // Test space key display
    let space_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::SelectEntry);
    assert!(
        space_display.contains("Space"),
        "Space key should be displayed as 'Space', got: {space_display}"
    );
}

#[test]
fn test_regular_key_display() {
    // Create a custom shortcuts map for testing
    let mut shortcuts = Shortcuts::new();

    // Add regular key shortcuts
    let a_shortcuts = vec![KeyboardShortcut::new("a")];
    shortcuts
        .set_shortcuts(ShortcutAction::AddEntry, a_shortcuts)
        .unwrap();

    let d_shortcuts = vec![KeyboardShortcut::new("d")];
    shortcuts
        .set_shortcuts(ShortcutAction::DeleteEntry, d_shortcuts)
        .unwrap();

    // Test regular key display
    let a_display = shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::AddEntry);
    assert_eq!(
        a_display, "a",
        "Regular key 'a' should be displayed as 'a', got: {a_display}"
    );

    let d_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::DeleteEntry);
    assert_eq!(
        d_display, "d",
        "Regular key 'd' should be displayed as 'd', got: {d_display}"
    );
}

#[test]
fn test_shortcut_with_modifiers() {
    // Create a custom shortcuts map for testing
    let mut shortcuts = Shortcuts::new();

    // Add shortcuts with modifiers
    let ctrl_c_shortcuts = vec![KeyboardShortcut::new("c").with_ctrl()];
    shortcuts
        .set_shortcuts(ShortcutAction::CloseCurrentTab, ctrl_c_shortcuts)
        .unwrap();

    let shift_question_shortcuts = vec![KeyboardShortcut::new("?").with_shift()];
    shortcuts
        .set_shortcuts(ShortcutAction::ShowHelp, shift_question_shortcuts)
        .unwrap();

    let shift_t_shortcuts = vec![KeyboardShortcut::new("t").with_shift()];
    shortcuts
        .set_shortcuts(ShortcutAction::OpenTerminal, shift_t_shortcuts)
        .unwrap();

    // Add a shortcut with multiple modifiers
    let ctrl_shift_q_shortcuts = vec![KeyboardShortcut {
        key: "q".to_string(),
        shift: true,
        ctrl: true,
        alt: false,
        #[cfg(target_os = "macos")]
        command: false,
    }];
    shortcuts
        .set_shortcuts(ShortcutAction::Exit, ctrl_shift_q_shortcuts)
        .unwrap();

    // Test Ctrl+C display
    let ctrl_c_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::CloseCurrentTab);
    assert_eq!(
        ctrl_c_display, "Ctrl+c",
        "Ctrl+C should be displayed as 'Ctrl+c', got: {ctrl_c_display}"
    );

    // Test Shift+? display
    let shift_question_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::ShowHelp);
    assert_eq!(
        shift_question_display, "Shift+?",
        "Shift+? should be displayed as 'Shift+?', got: {shift_question_display}"
    );

    // Test Shift+T display
    let shift_t_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::OpenTerminal);
    assert_eq!(
        shift_t_display, "Shift+t",
        "Shift+T should be displayed as 'Shift+t', got: {shift_t_display}"
    );

    // Test Ctrl+Shift+Q display
    let ctrl_shift_q_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::Exit);
    assert_eq!(
        ctrl_shift_q_display, "Ctrl+Shift+q",
        "Ctrl+Shift+Q should be displayed as 'Ctrl+Shift+q', got: {ctrl_shift_q_display}"
    );
}

#[test]
fn test_multiple_shortcuts_display() {
    // Create a custom shortcuts map for testing
    let mut shortcuts = Shortcuts::new();

    // Add multiple shortcuts for the same action
    let move_down_shortcuts = vec![KeyboardShortcut::new("j"), KeyboardShortcut::new("down")];
    shortcuts
        .set_shortcuts(ShortcutAction::MoveDown, move_down_shortcuts)
        .unwrap();

    // Test multiple shortcuts display
    let move_down_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::MoveDown);
    assert!(
        move_down_display.contains('j') && move_down_display.contains("⬇"),
        "Multiple shortcuts should be displayed with separator, got: {move_down_display}"
    );

    // Check for separator
    assert!(
        move_down_display.contains(" or "),
        "Multiple shortcuts should be separated by ' or ', got: {move_down_display}"
    );
}

#[test]
fn test_no_shortcuts_display() {
    // Create an empty shortcuts map
    let shortcuts = Shortcuts::new();

    // Test display for action with no shortcuts
    let no_shortcut_display =
        shortcuts_helpers::get_shortcut_display(&shortcuts, ShortcutAction::AddEntry);
    assert_eq!(
        no_shortcut_display, "Not assigned",
        "Action with no shortcuts should display 'Not assigned', got: {no_shortcut_display}"
    );
}
