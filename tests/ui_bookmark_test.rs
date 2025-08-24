#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use kiorg::ui::popup::PopupType;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files, shift_modifiers};

#[test]
fn test_bookmark_feature() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test directories and files
    create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("dir2"),
        temp_dir.path().join("test1.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Check initial state - no bookmarks
    harness.step();
    assert!(harness.state().bookmarks.is_empty());
    assert!(harness.state().show_popup.is_none());

    // Bookmark the directory with 'b'
    harness.key_press(Key::B);
    harness.step();

    // Verify bookmark was added
    {
        let app = harness.state();
        assert_eq!(app.bookmarks.len(), 1);
        assert!(app.bookmarks[0].ends_with("dir1"));
    }

    // Open bookmark popup with 'B' (shift+b)
    {
        harness.key_press_modifiers(shift_modifiers(), Key::B);
        harness.step();
    }

    // Verify bookmark popup is shown
    if let Some(PopupType::Bookmarks(_)) = harness.state().show_popup {
        // Bookmark popup is shown
    } else {
        panic!("Bookmark popup should be shown");
    }

    // Close bookmark popup with 'q'
    {
        harness.key_press(Key::Q);
        harness.step();
    }

    // Verify bookmark popup is closed
    assert!(harness.state().show_popup.is_none());

    // Select the second directory
    {
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = 1; // Select dir2
    }
    harness.step();

    // Bookmark the second directory
    harness.key_press(Key::B);
    harness.step();

    // Verify second bookmark was added
    {
        let app = harness.state();
        assert_eq!(app.bookmarks.len(), 2);
        assert!(app.bookmarks[1].ends_with("dir2"));
    }

    // Try to bookmark a file (should not work)
    {
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = 2; // Select test1.txt
    }
    harness.key_press(Key::B);
    harness.step();

    // Verify no new bookmark was added (still 2)
    let bookmark_count = harness.state().bookmarks.len();
    assert_eq!(
        bookmark_count, 2,
        "Expected 2 bookmarks, got {bookmark_count}"
    );

    // Open bookmark popup again
    {
        harness.key_press_modifiers(shift_modifiers(), Key::B);
        harness.step();
    }

    // Delete the first bookmark with 'd'
    harness.key_press(Key::D);
    harness.step();

    // Verify bookmark was removed
    {
        let app = harness.state();
        assert_eq!(app.bookmarks.len(), 1);
        assert!(app.bookmarks[0].ends_with("dir2")); // Only dir2 remains
    }

    // Close bookmark popup with 'q'
    harness.key_press(Key::Q);
    harness.step();

    // Verify bookmark popup is closed
    assert!(harness.state().show_popup.is_none());
}

#[test]
fn test_bookmark_popup_close_with_q_and_esc() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test directories
    create_test_files(&[temp_dir.path().join("dir1"), temp_dir.path().join("dir2")]);

    let mut harness = create_harness(&temp_dir);
    harness.step();

    // First bookmark a directory so we have something to show in the popup
    harness.key_press(Key::B);
    harness.step();

    // Verify bookmark was added
    {
        let app = harness.state();
        assert_eq!(app.bookmarks.len(), 1);
    }

    // Test 1: Open bookmark popup and close with 'q'
    {
        harness.key_press_modifiers(shift_modifiers(), Key::B);
        harness.step();
    }

    // Verify bookmark popup is shown
    if let Some(PopupType::Bookmarks(_)) = harness.state().show_popup {
        // Bookmark popup is shown
    } else {
        panic!("Bookmark popup should be shown");
    }

    // Close bookmark popup with 'q'
    harness.key_press(Key::Q);
    harness.step();

    // Verify bookmark popup is closed
    assert!(
        harness.state().show_popup.is_none(),
        "Bookmark popup should be closed after pressing 'q'"
    );

    // Test 2: Open bookmark popup and close with 'Esc'
    {
        harness.key_press_modifiers(shift_modifiers(), Key::B);
        harness.step();
    }

    // Verify bookmark popup is shown again
    if let Some(PopupType::Bookmarks(_)) = harness.state().show_popup {
        // Bookmark popup is shown
    } else {
        panic!("Bookmark popup should be shown again");
    }

    // Close bookmark popup with 'Esc'
    harness.key_press(Key::Escape);
    harness.step();

    // Verify bookmark popup is closed
    assert!(
        harness.state().show_popup.is_none(),
        "Bookmark popup should be closed after pressing 'Esc'"
    );
}
