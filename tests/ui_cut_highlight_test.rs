use egui::Key;

mod ui_test_helpers;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_cut_file_highlight() {
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

    // Cut test1.txt
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

    // Verify the file is highlighted in red in the UI
    // This is a bit tricky to test directly, but we can check that the implementation
    // correctly identifies the file as being in the cut clipboard
    {
        let app = harness.state();
        let tab = app.tab_manager.current_tab_ref();
        let entry = &tab.entries[tab.selected_index];

        // Check if this entry is in the clipboard as a cut operation
        let is_in_cut_clipboard = match &app.clipboard {
            Some(kiorg::app::Clipboard::Cut(paths)) => paths.contains(&entry.path),
            _ => false,
        };

        assert!(
            is_in_cut_clipboard,
            "File should be identified as being in cut clipboard"
        );
    }
}
