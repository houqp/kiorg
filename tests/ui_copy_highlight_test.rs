use egui::Key;

#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_copy_unmarked_file_clears_marked_files() {
    // Create a temporary directory with test files
    let temp_dir = tempfile::tempdir().unwrap();

    // Create test files and directories
    let test_files = create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("dir2"),
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
        temp_dir.path().join("test3.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Move down twice to select test1.txt (after dir1 and dir2)
    harness.key_press(Key::J);
    harness.step();
    harness.key_press(Key::J);
    harness.step();

    // Mark test1.txt
    harness.key_press(Key::Space);
    harness.step();

    // Move down to select test2.txt
    harness.key_press(Key::J);
    harness.step();

    // Mark test2.txt
    harness.key_press(Key::Space);
    harness.step();

    // Verify both files are marked
    {
        let app = harness.state();
        let tab = app.tab_manager.current_tab_ref();
        assert!(
            tab.marked_entries.contains(&test_files[2]),
            "test1.txt should be marked"
        );
        assert!(
            tab.marked_entries.contains(&test_files[3]),
            "test2.txt should be marked"
        );
    }

    // Copy the marked files
    harness.key_press(Key::Y);
    harness.step();

    // Verify both files are in the clipboard as a copy operation
    {
        let app = harness.state();
        assert!(
            app.clipboard.is_some(),
            "Clipboard should contain copied files"
        );
        if let Some(kiorg::app::Clipboard::Copy(paths)) = &app.clipboard {
            assert_eq!(paths.len(), 2, "Clipboard should contain exactly two files");
            assert!(
                paths.contains(&test_files[2]),
                "Clipboard should contain test1.txt"
            );
            assert!(
                paths.contains(&test_files[3]),
                "Clipboard should contain test2.txt"
            );
        } else {
            panic!("Clipboard should contain a Copy operation");
        }
    }

    // Move down to select test3.txt (which is not marked)
    harness.key_press(Key::J);
    harness.step();

    // Verify test3.txt is not marked
    {
        let app = harness.state();
        let tab = app.tab_manager.current_tab_ref();
        assert!(
            !tab.marked_entries.contains(&test_files[4]),
            "test3.txt should not be marked"
        );
    }

    // Copy test3.txt (which is not marked)
    harness.key_press(Key::Y);
    harness.step();

    // Verify:
    // 1. All previously marked files are now unmarked
    // 2. The clipboard now only contains test3.txt
    {
        let app = harness.state();
        let tab = app.tab_manager.current_tab_ref();

        // All previously marked files should be unmarked
        assert!(
            tab.marked_entries.is_empty(),
            "All files should be unmarked"
        );

        // Clipboard should only contain test3.txt
        assert!(
            app.clipboard.is_some(),
            "Clipboard should contain copied file"
        );
        if let Some(kiorg::app::Clipboard::Copy(paths)) = &app.clipboard {
            assert_eq!(paths.len(), 1, "Clipboard should contain exactly one file");
            assert_eq!(
                paths[0], test_files[4],
                "Clipboard should contain only test3.txt"
            );
        } else {
            panic!("Clipboard should contain a Copy operation");
        }
    }
}

#[test]
fn test_unmark_copied_file() {
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
    harness.key_press(Key::J);
    harness.step();
    harness.key_press(Key::J);
    harness.step();

    // Mark the file
    harness.key_press(Key::Space);
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

    // Copy the marked file
    harness.key_press(Key::Y);
    harness.step();

    // Verify the file is in the clipboard as a copy operation
    {
        let app = harness.state();
        assert!(
            app.clipboard.is_some(),
            "Clipboard should contain copied file"
        );
        if let Some(kiorg::app::Clipboard::Copy(paths)) = &app.clipboard {
            assert_eq!(paths.len(), 1, "Clipboard should contain exactly one file");
            assert_eq!(
                paths[0], test_files[2],
                "Clipboard should contain test1.txt"
            );
        } else {
            panic!("Clipboard should contain a Copy operation");
        }
    }

    // Unmark the file
    harness.key_press(Key::Space);
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
            Some(kiorg::app::Clipboard::Copy(paths) | kiorg::app::Clipboard::Cut(paths)) => {
                assert!(
                    !paths.contains(&test_files[2]),
                    "test1.txt should be removed from clipboard"
                );
            }
            None => {}
        }
    }
}
