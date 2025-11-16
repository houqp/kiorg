#[cfg(target_os = "windows")]
#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

#[cfg(target_os = "windows")]
#[test]
fn test_show_windows_drives_popup() {
    use egui::Key;
    use kiorg::ui::popup::PopupType;
    use tempfile::tempdir;
    use ui_test_helpers::{create_harness, ctrl_shift_modifiers, wait_for_condition};

    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Initially no popup should be shown
    assert!(harness.state().show_popup.is_none());

    // Simulate Ctrl+Shift+V to open drives popup
    harness.key_press_modifiers(ctrl_shift_modifiers(), Key::D);

    wait_for_condition(|| {
        harness.step();
        harness.state().show_popup.is_some()
    });

    // Check that drives popup is now showing
    if let Some(PopupType::WindowsDrives(_)) = harness.state().show_popup {
        // Popup is showing correctly
    } else {
        panic!("Expected WindowsDrives popup to be showing");
    }
}
