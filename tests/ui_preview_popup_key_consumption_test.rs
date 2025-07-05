#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::{Key, Modifiers};
use kiorg::app::PopupType;
use kiorg::models::preview_content::PreviewContent;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_image, tab_num_modifiers};

/// Test that number keys don't trigger tab switches when preview popup is active
#[test]
fn test_preview_popup_consumes_number_keys() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a test image
    let image_path = temp_dir.path().join("test.png");
    create_test_image(&image_path);

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Create multiple tabs first to test tab switching
    harness.press_key(Key::T);
    harness.step();
    harness.press_key(Key::T);
    harness.step();
    harness.press_key(Key::T);
    harness.step();

    // Verify we have multiple tabs
    assert!(
        harness.state().tab_manager.get_tab_count() >= 3,
        "Should have at least 3 tabs for testing"
    );

    // Switch to tab 1 (index 0)
    let modifiers = tab_num_modifiers();
    harness.press_key_modifiers(modifiers, Key::Num1);
    harness.step();
    assert_eq!(
        harness.state().tab_manager.get_current_tab_index(),
        0,
        "Should be on tab 1 (index 0)"
    );

    // Switch to tab 2 (index 1)
    harness.press_key_modifiers(modifiers, Key::Num2);
    harness.step();
    assert_eq!(
        harness.state().tab_manager.get_current_tab_index(),
        1,
        "Should be on tab 2 (index 1)"
    );

    // Select the image file
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let image_index = tab
            .entries
            .iter()
            .position(|e| e.name == "test.png")
            .expect("Image file should be in the entries");
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = image_index;
    }
    harness.step();

    // Wait for the image preview to load
    for _ in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        match harness.state().preview_content.as_ref() {
            Some(PreviewContent::Image(_)) => break,
            Some(PreviewContent::Loading(..)) => harness.step(),
            _ => harness.step(),
        }
    }

    // Verify image preview is loaded
    match harness.state().preview_content.as_ref() {
        Some(PreviewContent::Image(_)) => {}
        other => panic!("Preview content should be Image, got {other:?}"),
    }

    // Open preview popup with Shift+K
    let modifiers = Modifiers {
        shift: true,
        ..Default::default()
    };
    harness.press_key_modifiers(modifiers, Key::K);
    harness.step();

    // Verify the preview popup is shown
    match &harness.state().show_popup {
        Some(PopupType::Preview) => {}
        other => panic!("Preview popup should be shown after pressing Shift+K, got {other:?}"),
    }

    // Store the current tab index before pressing number keys
    let tab_index_before = harness.state().tab_manager.get_current_tab_index();

    // Press number keys while the popup is active - these should NOT trigger tab switches
    harness.press_key(Key::Num1);
    harness.step();

    // Verify tab didn't change
    assert_eq!(
        harness.state().tab_manager.get_current_tab_index(),
        tab_index_before,
        "Tab should not change when pressing number keys in preview popup"
    );

    // Verify popup is still open
    match &harness.state().show_popup {
        Some(PopupType::Preview) => {}
        other => panic!("Preview popup should still be open, got {other:?}"),
    }

    // Try pressing more number keys
    harness.press_key(Key::Num3);
    harness.step();
    harness.press_key(Key::Num9);
    harness.step();

    // Verify tab still didn't change
    assert_eq!(
        harness.state().tab_manager.get_current_tab_index(),
        tab_index_before,
        "Tab should not change when pressing number keys in preview popup"
    );

    // Verify popup is still open
    match &harness.state().show_popup {
        Some(PopupType::Preview) => {}
        other => panic!("Preview popup should still be open, got {other:?}"),
    }

    // Close the popup with Escape
    harness.press_key(Key::Escape);
    harness.step();

    // Verify the popup is closed
    assert_eq!(
        harness.state().show_popup,
        None,
        "Preview popup should be closed after pressing Escape"
    );

    // Now test that tab switching with modifiers works normally after closing the popup
    let modifiers = tab_num_modifiers();
    harness.press_key_modifiers(modifiers, Key::Num1);
    harness.step();
    assert_eq!(
        harness.state().tab_manager.get_current_tab_index(),
        0,
        "Tab switching with modifiers should work normally after closing preview popup"
    );
}

/// Test that other keys are also consumed by the preview popup (except Escape and Q)
#[test]
fn test_preview_popup_consumes_other_keys() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a test image
    let image_path = temp_dir.path().join("test.png");
    create_test_image(&image_path);

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Select the image file
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let image_index = tab
            .entries
            .iter()
            .position(|e| e.name == "test.png")
            .expect("Image file should be in the entries");
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = image_index;
    }
    harness.step();

    // Wait for the image preview to load
    for _ in 0..10 {
        match harness.state().preview_content.as_ref() {
            Some(PreviewContent::Image(_)) => break,
            Some(PreviewContent::Loading(..)) => harness.step(),
            _ => harness.step(),
        }
    }

    // Open preview popup with Shift+K
    let modifiers = Modifiers {
        shift: true,
        ..Default::default()
    };
    harness.press_key_modifiers(modifiers, Key::K);
    harness.step();

    // Verify the preview popup is shown
    match &harness.state().show_popup {
        Some(PopupType::Preview) => {}
        other => panic!("Preview popup should be shown after pressing Shift+K, got {other:?}"),
    }

    // Test that various other keys don't trigger their normal actions
    let initial_selection = harness.state().tab_manager.current_tab_ref().selected_index;

    // Press J (normally moves down) - should be consumed
    harness.press_key(Key::J);
    harness.step();

    // Verify selection didn't change and popup is still open
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        initial_selection,
        "Selection should not change when pressing J in preview popup"
    );
    match &harness.state().show_popup {
        Some(PopupType::Preview) => {}
        other => panic!("Preview popup should still be open after pressing J, got {other:?}"),
    }

    // Press K (normally moves up) - should be consumed
    harness.press_key(Key::K);
    harness.step();

    // Verify selection didn't change and popup is still open
    assert_eq!(
        harness.state().tab_manager.current_tab_ref().selected_index,
        initial_selection,
        "Selection should not change when pressing K in preview popup"
    );
    match &harness.state().show_popup {
        Some(PopupType::Preview) => {}
        other => panic!("Preview popup should still be open after pressing K, got {other:?}"),
    }

    // Test that Escape still works to close the popup
    harness.press_key(Key::Escape);
    harness.step();

    // Verify the popup is closed
    assert_eq!(
        harness.state().show_popup,
        None,
        "Preview popup should be closed after pressing Escape"
    );
}
