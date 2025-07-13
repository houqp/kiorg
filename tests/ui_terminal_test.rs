#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::{Key, Modifiers};
use tempfile::tempdir;
use ui_test_helpers::create_harness;

#[test]
fn test_open_terminal_shortcut() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Initially, terminal should not be shown
    assert_eq!(
        harness.state().show_popup,
        None,
        "No popup should be shown initially"
    );

    // Press Shift+T to open terminal
    let modifiers = Modifiers {
        shift: true,
        ..Default::default()
    };
    harness.key_press_modifiers(modifiers, Key::T);
    harness.step();

    // Verify terminal is shown
    assert!(
        harness.state().terminal_ctx.is_some(),
        "Terminal should be open"
    );
}
