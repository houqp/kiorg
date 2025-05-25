#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::create_harness;

/// Test that the exit popup sets shutdown_requested to true when confirmed with Enter key
#[test]
fn test_exit_popup_enter_key() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Initially, the app should not be in shutdown state
    assert!(
        !harness.state().shutdown_requested,
        "App should not be in shutdown state initially"
    );

    // Press 'q' to request exit (shows exit popup)
    harness.press_key(Key::Q);
    harness.step();

    // Verify exit popup is shown
    assert_eq!(
        harness.state().show_popup,
        Some(kiorg::app::PopupType::Exit),
        "Exit popup should be shown after pressing 'q'"
    );

    // Press Enter to confirm exit
    harness.press_key(Key::Enter);
    harness.step();

    // Verify shutdown was requested
    assert!(
        harness.state().shutdown_requested,
        "App should be in shutdown state after confirming exit"
    );
}

/// Test that the exit popup does not set shutdown_requested to true when canceled with Escape key
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
    harness.press_key(Key::Q);
    harness.step();

    // Verify exit popup is shown
    assert_eq!(
        harness.state().show_popup,
        Some(kiorg::app::PopupType::Exit),
        "Exit popup should be shown after pressing 'q'"
    );

    // Press Escape to cancel exit
    harness.press_key(Key::Escape);
    harness.step();

    // Verify popup is closed
    assert_eq!(
        harness.state().show_popup,
        None,
        "Exit popup should be closed after pressing Escape"
    );

    // Verify shutdown was not requested
    assert!(
        !harness.state().shutdown_requested,
        "App should not be in shutdown state after canceling exit"
    );
}

/// Test that the exit popup does not set shutdown_requested to true when canceled with 'q' key
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
    harness.press_key(Key::Q);
    harness.step();

    // Verify exit popup is shown
    assert_eq!(
        harness.state().show_popup,
        Some(kiorg::app::PopupType::Exit),
        "Exit popup should be shown after pressing 'q'"
    );

    // Press 'q' again to cancel exit
    harness.press_key(Key::Q);
    harness.step();

    // Verify popup is closed
    assert_eq!(
        harness.state().show_popup,
        None,
        "Exit popup should be closed after pressing 'q'"
    );

    // Verify shutdown was not requested
    assert!(
        !harness.state().shutdown_requested,
        "App should not be in shutdown state after canceling exit"
    );
}
