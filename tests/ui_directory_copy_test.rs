mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_copy_directory() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test directories and files with a nested structure
    let test_files = create_test_files(&[
        temp_dir.path().join("source_dir"),
        temp_dir.path().join("target_dir"),
    ]);

    // Create files inside source_dir
    let source_files = create_test_files(&[
        test_files[0].join("file1.txt"),
        test_files[0].join("file2.txt"),
        test_files[0].join("subdir"),
    ]);

    // Create files inside the subdirectory
    let subdir_files = create_test_files(&[
        source_files[2].join("subfile1.txt"),
        source_files[2].join("subfile2.txt"),
        source_files[2].join("nested_subdir"),
    ]);

    // Create a file in the nested subdirectory
    let _nested_subdir_file = create_test_files(&[subdir_files[2].join("nested_file.txt")]);

    let mut harness = create_harness(&temp_dir);

    // Select source_dir (should be the first entry)
    let tab = harness.state_mut().tab_manager.current_tab_mut();
    tab.selected_index = 0;

    // Copy source_dir
    harness.press_key(Key::Y);
    harness.step();

    // Verify the directory is in the clipboard as a copy operation
    {
        let app = harness.state();
        assert!(
            app.clipboard.is_some(),
            "Clipboard should contain copied directory"
        );
        if let Some(kiorg::app::Clipboard::Copy(paths)) = &app.clipboard {
            assert_eq!(
                paths.len(),
                1,
                "Clipboard should contain exactly one directory"
            );
            assert_eq!(
                paths[0], test_files[0],
                "Clipboard should contain source_dir"
            );
        } else {
            panic!("Clipboard should contain a Copy operation");
        }
    }

    // Move down to select target_dir
    harness.press_key(Key::J);
    harness.step();

    // Navigate into target_dir
    harness.press_key(Key::L);
    harness.step();

    // Paste the directory
    harness.press_key(Key::P);
    harness.step();

    // Verify the directory was copied with all its contents
    let copied_dir = test_files[1].join("source_dir");
    assert!(
        copied_dir.exists(),
        "source_dir should be copied to target_dir"
    );

    // Verify original directory still exists
    assert!(
        test_files[0].exists(),
        "Original source_dir should still exist"
    );

    // Verify all files and subdirectories were copied
    assert!(
        copied_dir.join("file1.txt").exists(),
        "file1.txt should be copied"
    );
    assert!(
        copied_dir.join("file2.txt").exists(),
        "file2.txt should be copied"
    );
    assert!(
        copied_dir.join("subdir").exists(),
        "subdir should be copied"
    );

    // Verify nested files were copied
    assert!(
        copied_dir.join("subdir/subfile1.txt").exists(),
        "subfile1.txt should be copied"
    );
    assert!(
        copied_dir.join("subdir/subfile2.txt").exists(),
        "subfile2.txt should be copied"
    );
    assert!(
        copied_dir.join("subdir/nested_subdir").exists(),
        "nested_subdir should be copied"
    );
    assert!(
        copied_dir
            .join("subdir/nested_subdir/nested_file.txt")
            .exists(),
        "nested_file.txt should be copied"
    );

    // Verify UI list in target_dir is updated
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.entries
                .iter()
                .any(|e| e.name == "source_dir" && e.is_dir),
            "UI entry list in target_dir should contain source_dir after paste"
        );
    }
}

#[test]
fn test_copy_directory_to_same_parent() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test directory with contents
    let test_files = create_test_files(&[temp_dir.path().join("source_dir")]);

    // Create files inside source_dir
    let _source_files = create_test_files(&[
        test_files[0].join("file1.txt"),
        test_files[0].join("file2.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Select source_dir (should be the first entry)
    let tab = harness.state_mut().tab_manager.current_tab_mut();
    tab.selected_index = 0;

    // Copy source_dir
    harness.press_key(Key::Y);
    harness.step();

    // Paste in the same directory
    harness.press_key(Key::P);
    harness.step();

    // Verify the directory was copied with a new suffix
    let copied_dir = temp_dir.path().join("source_dir_1");
    assert!(copied_dir.exists(), "source_dir_1 should exist after copy");

    // Verify original directory still exists
    assert!(
        test_files[0].exists(),
        "Original source_dir should still exist"
    );

    // Verify all files were copied
    assert!(
        copied_dir.join("file1.txt").exists(),
        "file1.txt should be copied"
    );
    assert!(
        copied_dir.join("file2.txt").exists(),
        "file2.txt should be copied"
    );

    // Verify UI list is updated
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.entries
                .iter()
                .any(|e| e.name == "source_dir_1" && e.is_dir),
            "UI entry list should contain source_dir_1 after paste"
        );
    }
}
