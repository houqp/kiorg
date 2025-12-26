#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use kiorg::ui::popup::PopupType;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_exit_saves_state() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test directories and files
    let test_dirs =
        create_test_files(&[temp_dir.path().join("dir1"), temp_dir.path().join("dir2")]);

    let mut harness = create_harness(&temp_dir);

    // Get the config directory path for later verification
    let config_dir = harness.state().config_dir_override.clone().unwrap();

    {
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = 0; // Select dir1
    }
    harness.step();

    // Bookmark the directory with 'b' shortcut
    harness.key_press(Key::B);
    harness.step();

    // Verify bookmark was added
    {
        let app = harness.state();
        assert_eq!(app.bookmarks.len(), 1, "Should have one bookmark");
        assert!(
            app.bookmarks[0].ends_with("dir1"),
            "Bookmark should be dir1"
        );
    }

    // Press 'q' to request exit (shows exit popup)
    harness.key_press(Key::Q);
    harness.step();

    // Verify exit popup is shown
    assert_eq!(
        harness.state().show_popup,
        Some(PopupType::Exit),
        "Exit popup should be shown after pressing 'q'"
    );

    // Press Enter to confirm exit
    harness.key_press(Key::Enter);
    harness.step();

    // Verify state.json was created
    let state_file_path = config_dir.join("state.json");
    assert!(
        state_file_path.exists(),
        "state.json should exist after exit"
    );

    // Verify bookmarks.txt was created
    let bookmarks_file_path = config_dir.join("bookmarks.txt");
    assert!(
        bookmarks_file_path.exists(),
        "bookmarks.txt should exist after exit"
    );

    // Verify the bookmarks file contains our bookmark
    let bookmarks_content = std::fs::read_to_string(&bookmarks_file_path).unwrap();
    let bookmark_path = test_dirs[0].clone(); // dir1 path
    let path_str = bookmark_path.to_string_lossy().to_string();
    assert!(
        bookmarks_content.contains(&path_str),
        "Bookmarks file should contain the path for dir1"
    );
}
