use egui::Key;

mod ui_test_helpers;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_cut_unmarked_file_clears_marked_files() {
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

    // Ensure consistent sort order for reliable selection
    harness.ensure_sorted_by_name_ascending();

    // Move down twice to select test1.txt (after dir1 and dir2)
    harness.press_key(Key::J);
    harness.step();
    harness.press_key(Key::J);
    harness.step();

    // Mark test1.txt
    harness.press_key(Key::Space);
    harness.step();

    // Move down to select test2.txt
    harness.press_key(Key::J);
    harness.step();

    // Mark test2.txt
    harness.press_key(Key::Space);
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

    // Cut the marked files
    harness.press_key(Key::X);
    harness.step();

    // Verify both files are in the clipboard as a cut operation
    {
        let app = harness.state();
        assert!(
            app.clipboard.is_some(),
            "Clipboard should contain cut files"
        );
        if let Some(kiorg::app::Clipboard::Cut(paths)) = &app.clipboard {
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
            panic!("Clipboard should contain a Cut operation");
        }
    }

    // Move down to select test3.txt (which is not marked)
    harness.press_key(Key::J);
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

    // Cut test3.txt (which is not marked)
    harness.press_key(Key::X);
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
        assert!(app.clipboard.is_some(), "Clipboard should contain cut file");
        if let Some(kiorg::app::Clipboard::Cut(paths)) = &app.clipboard {
            assert_eq!(paths.len(), 1, "Clipboard should contain exactly one file");
            assert_eq!(
                paths[0], test_files[4],
                "Clipboard should contain only test3.txt"
            );
        } else {
            panic!("Clipboard should contain a Cut operation");
        }
    }
}
