mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::create_harness;

#[test]
fn test_help_menu_close_behavior() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Open help menu with shift+?
    {
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        harness.press_key_modifiers(modifiers, Key::Questionmark);
        harness.step();
    }
    assert_eq!(
        harness.state().show_popup,
        Some(kiorg::app::PopupType::Help),
        "Help menu should be open"
    );

    // Test closing with Escape
    harness.press_key(Key::Escape);
    harness.step();
    assert_eq!(
        harness.state().show_popup,
        None,
        "Help menu should close with Escape"
    );

    // Reopen help menu
    {
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        harness.press_key_modifiers(modifiers, Key::Questionmark);
        harness.step();
    }

    // Test closing with Q
    harness.press_key(Key::Q);
    harness.step();
    assert_eq!(
        harness.state().show_popup,
        None,
        "Help menu should close with Q"
    );

    // Reopen help menu
    {
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        harness.press_key_modifiers(modifiers, Key::Questionmark);
        harness.step();
    }

    // Test closing with Enter
    harness.press_key(Key::Enter);
    harness.step();
    assert_eq!(
        harness.state().show_popup,
        None,
        "Help menu should close with Enter"
    );

    // Reopen help menu
    {
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        harness.press_key_modifiers(modifiers, Key::Questionmark);
        harness.step();
    }

    // Test closing with ? (Questionmark)
    harness.press_key(Key::Questionmark);
    harness.step();
    assert_eq!(
        harness.state().show_popup,
        None,
        "Help menu should close with ?"
    );
}
