#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_exit_shortcut() {
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
    assert_eq!(
        harness.state().show_popup,
        Some(kiorg::app::PopupType::Exit),
        "Exit popup should be shown after pressing 'q'"
    );

    // Press Enter to confirm exit
    harness.key_press(Key::Enter);
    harness.step();

    // Verify shutdown was requested
    assert!(
        harness.state().shutdown_requested,
        "App should be in shutdown state after confirming exit"
    );
}

#[test]
fn test_exit_with_unsaved_changes() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Create a file to be cut (simulating unsaved changes)
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "test content").unwrap();
    harness.state_mut().refresh_entries();
    harness.step();

    // Select the file
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let file_index = tab
            .entries
            .iter()
            .position(|e| e.path == file_path)
            .expect("File should be in the entries");

        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = file_index;
    }
    harness.step();

    // Cut the file (creating unsaved changes)
    harness.key_press(Key::X);
    harness.step();

    // Verify the file is marked as cut
    {
        let app = harness.state();
        assert!(app.clipboard.is_some(), "Clipboard should contain cut file");
        if let Some(kiorg::app::Clipboard::Cut(paths)) = &app.clipboard {
            assert!(paths.contains(&file_path), "File should be marked as cut");
        } else {
            panic!("Clipboard should contain a Cut operation");
        }
    }

    // Press 'q' to request exit (shows exit popup)
    harness.key_press(Key::Q);
    harness.step();

    // Verify exit popup is shown
    assert_eq!(
        harness.state().show_popup,
        Some(kiorg::app::PopupType::Exit),
        "Exit popup should be shown after pressing 'q'"
    );

    // Press Enter to confirm exit
    harness.key_press(Key::Enter);
    harness.step();

    // Verify shutdown was requested despite unsaved changes
    assert!(
        harness.state().shutdown_requested,
        "App should be in shutdown state after confirming exit"
    );

    // Verify the file still exists (cut operation doesn't delete until paste)
    assert!(
        file_path.exists(),
        "File should still exist after cut and exit"
    );
}

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
        Some(kiorg::app::PopupType::Exit),
        "Exit popup should be shown after pressing 'q'"
    );

    // Press Enter to confirm exit
    harness.key_press(Key::Enter);
    harness.step();

    // Verify shutdown was requested
    assert!(
        harness.state().shutdown_requested,
        "App should be in shutdown state after confirming exit"
    );

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
