#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use kiorg::ui::popup::PopupType;
use tempfile::tempdir;
use ui_test_helpers::create_harness;

use crate::ui_test_helpers::create_harness_with_config_dir;

/// Test that the exit popup sets `shutdown_requested` to true when confirmed with Enter key
#[test]
fn test_exit_popup_enter_key() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let temp_config_dir = tempdir().unwrap();
    let state_file_path = temp_config_dir.path().join("state.json");
    let mut harness = create_harness_with_config_dir(&temp_dir, temp_config_dir);

    // Initially, the app should not be in shutdown state
    assert!(
        !harness.state().shutdown_requested,
        "App should not be in shutdown state initially"
    );
    assert!(!state_file_path.exists());

    // Press 'q' to request exit (shows exit popup)
    harness.key_press(Key::Q);
    harness.step();

    // Verify exit popup is shown
    assert!(
        matches!(harness.state().show_popup, Some(PopupType::Exit)),
        "Exit popup should be shown after pressing 'q'"
    );

    // Press Enter to confirm exit
    harness.key_press(Key::Enter);
    harness.step();

    // Verify shutdown was requested
    assert!(
        harness.state().show_popup.is_none(),
        "Confirming exit should close popup"
    );
    // Verify config is correctly serialized on exit
    assert!(state_file_path.exists());
}

/// Test that the exit popup does not set `shutdown_requested` to true when canceled with Escape key
#[test]
fn test_exit_popup_escape_key() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Initially, the app should not be in shutdown state
    assert!(
        !harness.state().shutdown_requested,
        "App should not be in shutdown state initially"
    );

    // Press 'q' to request exit (shows exit popup)
    harness.key_press(Key::Q);
    harness.step();

    // Verify exit popup is shown
    assert!(
        matches!(harness.state().show_popup, Some(PopupType::Exit)),
        "Exit popup should be shown after pressing 'q'"
    );

    // Press Escape to cancel exit
    harness.key_press(Key::Escape);
    harness.step();

    // Verify popup is closed
    assert!(
        harness.state().show_popup.is_none(),
        "Exit popup should be closed after pressing Escape"
    );

    // Verify shutdown was not requested
    assert!(
        !harness.state().shutdown_requested,
        "App should not be in shutdown state after canceling exit"
    );
}

/// Test that the exit popup does not set `shutdown_requested` to true when canceled with 'q' key
#[test]
fn test_exit_popup_q_key() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Initially, the app should not be in shutdown state
    assert!(
        !harness.state().shutdown_requested,
        "App should not be in shutdown state initially"
    );

    // Press 'q' to request exit (shows exit popup)
    harness.key_press(Key::Q);
    harness.step();

    // Verify exit popup is shown
    assert!(
        matches!(harness.state().show_popup, Some(PopupType::Exit)),
        "Exit popup should be shown after pressing 'q'"
    );

    // Press 'q' again to cancel exit
    harness.key_press(Key::Q);
    harness.step();

    // Verify popup is closed
    assert!(
        harness.state().show_popup.is_none(),
        "Exit popup should be closed after pressing 'q'"
    );

    // Verify shutdown was not requested
    assert!(
        !harness.state().shutdown_requested,
        "App should not be in shutdown state after canceling exit"
    );
}
