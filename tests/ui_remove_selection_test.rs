//! UI test that reproduces a crash scenario related to filtered entry selection and deletion.
//!
//! This test implements the crash reproduction steps described in the issue:
//! 1. Apply a filter to show only a subset of entries
//! 2. Select all the filtered entries  
//! 3. Delete all selected entries
//! 4. Observe the app crash

#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::{Key, Modifiers};
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_crash_reproduction_filtered_deletion() {
    // This is a focused test that reproduces the specific crash scenario
    // described in the issue: apply filter, select all filtered entries, delete them

    let temp_dir = tempdir().unwrap();
    create_test_files(&[
        temp_dir.path().join("match1.txt"),
        temp_dir.path().join("match2.txt"),
        temp_dir.path().join("nomatch.png"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Step 1: Apply filter
    harness.press_key(Key::Slash);
    harness.step();
    harness
        .input_mut()
        .events
        .push(egui::Event::Text(".txt".to_string()));
    harness.step();
    harness.press_key(Key::Enter);
    harness.step();

    // Verify filter is applied
    assert_eq!(
        harness.state().search_bar.query.as_deref(),
        Some(".txt"),
        "Filter should be applied"
    );

    // Step 2: Select all filtered entries using Ctrl+A
    let modifiers = Modifiers {
        ctrl: true,
        ..Default::default()
    };
    harness.press_key_modifiers(modifiers, Key::A);
    harness.step();

    // Verify entries are selected
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.marked_entries.len(), 2, "Should have 2 marked entries");
    }

    // Step 3: Delete all selected entries - this triggers the bug
    harness.press_key(Key::D);
    harness.step();

    // Verify we're in the initial confirmation state
    if let Some(kiorg::app::PopupType::Delete(state, _)) = &harness.state().show_popup {
        assert_eq!(
            *state,
            kiorg::ui::delete_popup::DeleteConfirmState::Initial,
            "Should be in initial confirmation state"
        );
    } else {
        panic!("Expected Delete popup to be open");
    }

    // Press Enter for first confirmation
    harness.press_key(Key::Enter);
    harness.step();

    // Verify we're now in the recursive confirmation state
    if let Some(kiorg::app::PopupType::Delete(state, _)) = &harness.state().show_popup {
        assert_eq!(
            *state,
            kiorg::ui::delete_popup::DeleteConfirmState::RecursiveConfirm,
            "Should be in recursive confirmation state after first Enter"
        );
    } else {
        panic!("Expected Delete popup to be open");
    }

    // Press Enter for 2nd confirmation
    harness.press_key(Key::Enter);
    harness.step();

    for _ in 0..100 {
        harness.step();
        if harness.state().show_popup.is_none() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Verify popup is closed
    assert_eq!(
        harness.state().show_popup,
        None,
        "Delete popup should be closed after second confirmation"
    );
}
