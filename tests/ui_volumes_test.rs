#[cfg(target_os = "macos")]
#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

#[cfg(target_os = "macos")]
#[test]
fn test_show_volumes_popup() {
    use egui::Key;
    use kiorg::ui::popup::PopupType;
    use tempfile::tempdir;
    use ui_test_helpers::{create_harness, shift_modifiers};

    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Initially no popup should be shown
    assert!(harness.state().show_popup.is_none());

    // Simulate Shift+V to open volumes popup
    harness.key_press_modifiers(shift_modifiers(), Key::V);
    harness.step();

    // Check that volumes popup is now showing
    if let Some(PopupType::Volumes(_)) = harness.state().show_popup {
        // Popup is showing correctly
    } else {
        panic!("Expected Volumes popup to be showing");
    }
}
