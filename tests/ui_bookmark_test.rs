mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

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
    assert!(!harness.state().show_bookmarks);

    // Bookmark the directory with 'b'
    harness.press_key(Key::B);
    harness.step();

    // Verify bookmark was added
    {
        let app = harness.state();
        assert_eq!(app.bookmarks.len(), 1);
        assert!(app.bookmarks[0].ends_with("dir1"));
    }

    // Open bookmark popup with 'B' (shift+b)
    {
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        harness.press_key_modifiers(modifiers, Key::B);
        harness.step();
    }

    // Verify bookmark popup is shown
    assert!(harness.state().show_bookmarks);

    // Close bookmark popup with 'q'
    {
        harness.press_key(Key::Q);
        harness.step();
    }

    // Verify bookmark popup is closed
    assert!(!harness.state().show_bookmarks);

    // Select the second directory
    {
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = 1; // Select dir2
    }
    harness.step();

    // Bookmark the second directory
    harness.press_key(Key::B);
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
    harness.press_key(Key::B);
    harness.step();

    // Verify no new bookmark was added (still 2)
    let bookmark_count = harness.state().bookmarks.len();
    assert_eq!(
        bookmark_count, 2,
        "Expected 2 bookmarks, got {}",
        bookmark_count
    );

    // Open bookmark popup again
    {
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        harness.press_key_modifiers(modifiers, Key::B);
        harness.step();
    }

    // Delete the first bookmark with 'd'
    harness.press_key(Key::D);
    harness.step();

    // Verify bookmark was removed
    {
        let app = harness.state();
        assert_eq!(app.bookmarks.len(), 1);
        assert!(app.bookmarks[0].ends_with("dir2")); // Only dir2 remains
    }

    // Close bookmark popup with 'q'
    harness.press_key(Key::Q);
    harness.step();

    // Verify bookmark popup is closed
    assert!(!harness.state().show_bookmarks);
}
