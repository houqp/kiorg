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

    // On Windows, we should show a popup message instead of opening terminal
    #[cfg(target_os = "windows")]
    {
        use kiorg::ui::popup::PopupType;

        match &harness.state().show_popup {
            Some(popup) => {
                // Verify it's a generic message popup (not terminal or other types)
                match popup {
                    PopupType::GenericMessage { .. } => {
                        // This is the expected popup type for Windows
                    }
                    _ => panic!("Expected GenericMessage popup on Windows, got: {:?}", popup),
                }
            }
            None => panic!("A popup message should be shown on Windows"),
        }
        assert!(
            harness.state().terminal_ctx.is_none(),
            "Terminal should not be opened on Windows"
        );
    }

    // On non-Windows platforms, terminal should be opened
    #[cfg(not(target_os = "windows"))]
    {
        assert!(
            harness.state().terminal_ctx.is_some(),
            "Terminal should be open on non-Windows platforms"
        );
    }
}
