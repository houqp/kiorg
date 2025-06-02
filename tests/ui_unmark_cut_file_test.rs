use egui::Key;

#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_unmark_cut_file() {
    // Create a temporary directory with test files
    let temp_dir = tempfile::tempdir().unwrap();

    // Create test files and directories
    let test_files = create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("dir2"),
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Move down twice to select test1.txt (after dir1 and dir2)
    harness.press_key(Key::J);
    harness.step();
    harness.press_key(Key::J);
    harness.step();

    // Mark the file
    harness.press_key(Key::Space);
    harness.step();

    // Verify the file is marked
    {
        let app = harness.state();
        let tab = app.tab_manager.current_tab_ref();
        assert!(
            tab.marked_entries.contains(&test_files[2]),
            "test1.txt should be marked"
        );
    }

    // Cut the marked file
    harness.press_key(Key::X);
    harness.step();

    // Verify the file is in the clipboard as a cut operation
    {
        let app = harness.state();
        assert!(app.clipboard.is_some(), "Clipboard should contain cut file");
        if let Some(kiorg::app::Clipboard::Cut(paths)) = &app.clipboard {
            assert_eq!(paths.len(), 1, "Clipboard should contain exactly one file");
            assert_eq!(
                paths[0], test_files[2],
                "Clipboard should contain test1.txt"
            );
        } else {
            panic!("Clipboard should contain a Cut operation");
        }
    }

    // Unmark the file
    harness.press_key(Key::Space);
    harness.step();

    // Verify the file is unmarked and removed from the clipboard
    {
        let app = harness.state();
        let tab = app.tab_manager.current_tab_ref();

        // File should be unmarked
        assert!(
            !tab.marked_entries.contains(&test_files[2]),
            "test1.txt should be unmarked"
        );

        // Clipboard should be empty or the file should be removed from it
        match &app.clipboard {
            Some(kiorg::app::Clipboard::Cut(paths) | kiorg::app::Clipboard::Copy(paths)) => {
                assert!(
                    !paths.contains(&test_files[2]),
                    "test1.txt should be removed from clipboard"
                );
            }
            None => {}
        }
    }
}
