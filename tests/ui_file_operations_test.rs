mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_delete_shortcut() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_files = create_test_files(&[
        temp_dir.path().join("dir1"),
        temp_dir.path().join("dir2"),
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);

    // Create a file inside dir2 to test non-empty directory deletion
    let nested_file = test_files[1].join("nested.txt");
    create_test_files(&[nested_file.clone()]);

    // Create files inside dir1 to test recursive deletion
    let dir1_files = create_test_files(&[
        test_files[0].join("file1.txt"),
        test_files[0].join("file2.txt"),
        test_files[0].join("subdir"),
    ]);

    // Create a file inside the subdirectory of dir1
    let subdir_file = dir1_files[2].join("subfile.txt");
    create_test_files(&[subdir_file.clone()]);

    let mut harness = create_harness(&temp_dir);

    // Test file deletion first
    // Move down twice to select test1.txt (after dir1 and dir2)
    harness.press_key(Key::J);
    harness.step();
    harness.press_key(Key::J);
    harness.step();

    // Simulate pressing 'd' key to delete test1.txt
    harness.press_key(Key::D);
    harness.step();

    // Simulate pressing Enter to confirm deletion
    harness.press_key(Key::Enter);
    harness.step();

    // Verify only test1.txt was deleted
    assert!(!test_files[2].exists(), "test1.txt should be deleted");
    assert!(test_files[0].exists(), "dir1 should still exist");
    assert!(test_files[1].exists(), "dir2 should still exist");
    assert!(test_files[3].exists(), "test2.txt should still exist");

    // Verify UI list is updated (test1.txt removed)
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            !tab.entries.iter().any(|e| e.path == test_files[2]),
            "UI entry list should not contain test1.txt after deletion"
        );
    }

    // Test recursive directory deletion
    // First entry should be dir1, move 2 entries up
    harness.press_key(Key::K);
    harness.step();
    harness.press_key(Key::K);
    harness.step();
    // Delete dir1 (directory with nested files and subdirectory)
    harness.press_key(Key::D);
    harness.step();
    harness.press_key(Key::Enter);
    harness.step();
    // confirm twice
    harness.press_key(Key::Enter);
    harness.step();

    // Verify dir1 and all its contents were deleted recursively
    assert!(!test_files[0].exists(), "dir1 should be deleted");
    assert!(!dir1_files[0].exists(), "file1.txt should be deleted");
    assert!(!dir1_files[1].exists(), "file2.txt should be deleted");
    assert!(!dir1_files[2].exists(), "subdir should be deleted");
    assert!(!subdir_file.exists(), "subfile.txt should be deleted");
    assert!(test_files[1].exists(), "dir2 should still exist");
    assert!(test_files[3].exists(), "test2.txt should still exist");

    // Verify UI list is updated (dir1 removed)
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            !tab.entries.iter().any(|e| e.path == test_files[0]),
            "UI entry list should not contain dir1 after deletion"
        );
    }
}

#[test]
fn test_rename_shortcut() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files
    let test_files = create_test_files(&[
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // move down to test2.txt
    harness.press_key(Key::J);
    harness.step();

    // Press 'r' to start renaming
    harness.press_key(Key::R);
    harness.step();

    // verify we are in rename mode
    assert_eq!(
        harness.state().show_popup,
        Some(kiorg::app::PopupType::Rename),
        "Rename popup should be open"
    );

    // Clear any existing text and simulate text input for the new name
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("_renamed".to_string()));
    harness.step();

    // Press Enter to confirm rename
    harness.press_key(Key::Enter);
    harness.step();

    // Verify the file was renamed
    assert!(test_files[0].exists(), "test1.txt should still exist");
    assert!(!test_files[1].exists(), "test2.txt should no longer exist");
    assert!(
        temp_dir.path().join("test2_renamed.txt").exists(),
        "test2_renamed.txt should exist"
    );

    // Verify UI list is updated
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            !tab.entries.iter().any(|e| e.name == "test2.txt"),
            "UI entry list should not contain test2.txt after rename"
        );
        assert!(
            tab.entries.iter().any(|e| e.name == "test2_renamed.txt"),
            "UI entry list should contain test2_renamed.txt after rename"
        );
    }
}

#[test]
fn test_copy_paste_shortcuts() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

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

    // Copy test1.txt
    harness.press_key(Key::Y);
    harness.step();

    // Move up to select dir2
    harness.press_key(Key::K);
    harness.step();

    // Navigate into dir2
    harness.press_key(Key::L);
    harness.step();

    // Paste the file
    harness.press_key(Key::P);
    harness.step();

    // Verify the file was copied to dir2 while original remains
    assert!(
        test_files[2].exists(),
        "test1.txt should still exist in original location"
    );
    assert!(
        test_files[1].join("test1.txt").exists(),
        "test1.txt should be copied to dir2"
    );

    // Verify UI list in dir2 is updated
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.entries.iter().any(|e| e.name == "test1.txt"),
            "UI entry list in dir2 should contain test1.txt after paste"
        );
    }
}

#[test]
fn test_copy_paste_same_directory() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files
    let test_files = create_test_files(&[
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Make sure we're selecting the first file (test1.txt)
    let tab = harness.state_mut().tab_manager.current_tab_mut();
    tab.selected_index = 0;

    // Copy test1.txt
    harness.press_key(Key::Y);
    harness.step();

    // Paste in the same directory
    harness.press_key(Key::P);
    harness.step();

    // Verify the file was copied with a new suffix
    assert!(test_files[0].exists(), "test1.txt should still exist");
    assert!(test_files[1].exists(), "test2.txt should still exist");

    // Check for the copied file with a new suffix
    let copied_file = temp_dir.path().join("test1_1.txt");
    assert!(
        copied_file.exists(),
        "test1.txt should be copied with suffix `_1`"
    );
}

#[test]
fn test_cut_paste_shortcuts() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

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

    // Verify the file still exists in the original location
    assert!(
        test_files[2].exists(),
        "test1.txt should still exist after cutting"
    );

    // Move up to select dir2
    harness.press_key(Key::K);
    harness.step();

    // Navigate into dir2
    harness.press_key(Key::L);
    harness.step();

    // Paste the file
    harness.press_key(Key::P);
    harness.step();

    // Verify the file was moved to dir2
    assert!(
        !test_files[2].exists(),
        "test1.txt should be moved from original location"
    );
    assert!(
        test_files[1].join("test1.txt").exists(),
        "test1.txt should exist in dir2"
    );

    // Verify UI list in dir2 is updated
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.entries.iter().any(|e| e.name == "test1.txt"),
            "UI entry list in dir2 should contain test1.txt after paste"
        );
    }

    // Navigate back to parent to verify original file is removed from UI list
    harness.press_key(Key::H);
    harness.step();

    // Verify UI list in parent directory is updated
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            !tab.entries.iter().any(|e| e.path == test_files[2]),
            "UI entry list in parent dir should not contain test1.txt after cut/paste"
        );
    }
}
