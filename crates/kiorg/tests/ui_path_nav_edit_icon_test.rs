#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use kiorg::ui::popup::PopupType;
use tempfile::tempdir;
use ui_test_helpers::create_harness;

#[test]
fn test_path_nav_edit_icon_triggers_goto_path_popup() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Verify no popup is open initially
    assert!(
        harness.state().show_popup.is_none(),
        "No popup should be open initially"
    );

    // Move the pointer into the path nav bar area to trigger hover.
    // The path nav is at the top of the window; y ~10 is well within the bar.
    let hover_pos = egui::pos2(200.0, 10.0);
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(hover_pos));
    harness.step();

    // After hovering, the edit icon (✎) should appear at the end of the path.
    use egui_kittest::kittest::Queryable;
    let edit_icon = harness.query_by_label("\u{270E}");
    assert!(
        edit_icon.is_some(),
        "Edit icon should be visible when hovering over the path nav bar"
    );

    // Get the center of the edit icon's bounding rect for clicking
    let edit_rect = edit_icon.unwrap().rect();
    let click_pos = edit_rect.center();

    // Move pointer to the icon (to maintain hover), then click
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(click_pos));
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: click_pos,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: click_pos,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::default(),
    });
    harness.step();

    // Verify the GoToPath popup is open
    assert!(
        matches!(harness.state().show_popup, Some(PopupType::GoToPath(_))),
        "GoToPath popup should be open after clicking the edit icon"
    );
}
