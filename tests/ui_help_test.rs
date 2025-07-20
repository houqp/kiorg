#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use kiorg::ui::popup::PopupType;
use tempfile::tempdir;
use ui_test_helpers::TestHarnessBuilder;

#[test]
fn test_help_menu_close_behavior() {
    let temp_dir = tempdir().unwrap();

    // Create the test harness with default config (only built-in themes)
    let mut harness = TestHarnessBuilder::new()
        .with_temp_dir(&temp_dir)
        .with_window_size(egui::Vec2::new(800.0, 800.0))
        .build();

    // Open help menu with shift+?
    {
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        harness.key_press_modifiers(modifiers, Key::Questionmark);
        harness.step();
    }
    assert_eq!(
        harness.state().show_popup,
        Some(PopupType::Help),
        "Help menu should be open"
    );
    #[cfg(feature = "snapshot")]
    {
        // multiple steps to ensure the menu animation completes
        harness.step();
        harness.step();
        harness.snapshot("help_menu");
    }

    // Test closing with Escape
    harness.key_press(Key::Escape);
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
        harness.key_press_modifiers(modifiers, Key::Questionmark);
        harness.step();
    }

    // Test closing with Q
    harness.key_press(Key::Q);
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
        harness.key_press_modifiers(modifiers, Key::Questionmark);
        harness.step();
    }

    // Test closing with Enter
    harness.key_press(Key::Enter);
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
        harness.key_press_modifiers(modifiers, Key::Questionmark);
        harness.step();
    }

    // Test closing with ? (Questionmark)
    harness.key_press(Key::Questionmark);
    harness.step();
    assert_eq!(
        harness.state().show_popup,
        None,
        "Help menu should close with ?"
    );
}
