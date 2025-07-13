#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_paste_copy_clears_marked_entries() {
    let temp_dir = tempdir().unwrap();
    let test_files = create_test_files(&[
        temp_dir.path().join("target_dir"),
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Mark multiple files (file1.txt and file2.txt)
    harness.key_press(Key::J);
    harness.step();
    // verify that file1.txt is selected
    {
        let app = harness.state();
        let tab = app.tab_manager.current_tab_ref();
        assert_eq!(
            tab.entries[tab.selected_index].path, test_files[1],
            "file1.txt should be selected"
        );
    }
    harness.key_press(Key::Space);
    harness.step();
    harness.key_press(Key::J);
    harness.step();
    harness.key_press(Key::Space);
    harness.step();

    // Verify both files are marked
    {
        let app = harness.state();
        let tab = app.tab_manager.current_tab_ref();
        assert!(
            tab.marked_entries.contains(&test_files[1]),
            "file1.txt should be marked"
        );
        assert!(
            tab.marked_entries.contains(&test_files[2]),
            "file2.txt should be marked"
        );
        assert_eq!(
            tab.marked_entries.len(),
            2,
            "Should have exactly 2 marked entries"
        );
    }

    // Copy the marked files
    harness.key_press(Key::Y);
    harness.step();

    // Verify the files are in the clipboard as a copy operation
    {
        let app = harness.state();
        assert!(
            app.clipboard.is_some(),
            "Clipboard should contain copied files"
        );
        if let Some(kiorg::app::Clipboard::Copy(paths)) = &app.clipboard {
            assert_eq!(paths.len(), 2, "Clipboard should contain exactly two files");
            assert!(
                paths.contains(&test_files[1]),
                "Clipboard should contain file1.txt"
            );
            assert!(
                paths.contains(&test_files[2]),
                "Clipboard should contain file2.txt"
            );
        } else {
            panic!("Clipboard should contain a Copy operation");
        }
    }

    // Navigate to target_dir
    harness.key_press(Key::G);
    harness.key_press(Key::G);
    harness.step();
    // Enter target_dir
    harness.key_press(Key::L);
    harness.step();

    // Paste the copied files
    harness.key_press(Key::P);
    harness.step();

    // Verify marked entries are cleared after paste operation
    {
        let app = harness.state();
        let tab = app.tab_manager.current_tab_ref();
        assert!(
            tab.marked_entries.is_empty(),
            "Marked entries should be cleared after paste operation"
        );
    }

    // Verify the files were copied to the target directory
    let target_dir = test_files[0].clone();
    let copied_file1 = target_dir.join("file1.txt");
    let copied_file2 = target_dir.join("file2.txt");
    assert!(
        copied_file1.exists(),
        "file1.txt should be copied to target_dir"
    );
    assert!(
        copied_file2.exists(),
        "file2.txt should be copied to target_dir"
    );

    // Verify original files still exist (copy operation)
    assert!(
        test_files[1].exists(),
        "Original file1.txt should still exist"
    );
    assert!(
        test_files[2].exists(),
        "Original file2.txt should still exist"
    );
}

#[test]
fn test_paste_cut_clears_marked_entries() {
    let temp_dir = tempdir().unwrap();
    let test_files = create_test_files(&[
        temp_dir.path().join("file1.txt"),
        temp_dir.path().join("file2.txt"),
        temp_dir.path().join("file3.txt"),
        temp_dir.path().join("target_dir"),
    ]);

    let mut harness = create_harness(&temp_dir);

    // Move to file3.txt
    harness.key_press(Key::J);
    harness.step();
    harness.key_press(Key::J);
    harness.step();
    harness.key_press(Key::J);
    harness.step();

    // Mark file3.txt
    harness.key_press(Key::Space);
    harness.step();

    // Verify file3.txt is marked
    {
        let app = harness.state();
        let tab = app.tab_manager.current_tab_ref();
        assert!(
            tab.marked_entries.contains(&test_files[2]),
            "file3.txt should be marked before paste"
        );
        assert_eq!(
            tab.marked_entries.len(),
            1,
            "Should have exactly 1 marked entry before paste"
        );
    }

    harness.key_press(Key::X);
    harness.step();

    // Enter target_dir
    harness.key_press(Key::G);
    harness.key_press(Key::G);
    harness.step();
    // confirm that we have target_dir selected
    {
        let app = harness.state();
        let tab = app.tab_manager.current_tab_ref();
        assert_eq!(
            tab.entries[tab.selected_index].path, test_files[3],
            "target_dir should be selected"
        );
    }
    harness.key_press(Key::L);
    harness.step();

    // Paste the cut files
    harness.key_press(Key::P);
    harness.step();
    harness.step();

    // Verify marked entries are cleared after paste operation
    {
        let app = harness.state();
        let tab = app.tab_manager.current_tab_ref();
        assert!(
            tab.marked_entries.is_empty(),
            "Marked entries should be cleared after paste operation"
        );
    }

    // Verify the files were moved to the target directory
    let target_dir = test_files[3].clone();
    let moved_file1 = target_dir.join("file1.txt");
    let moved_file2 = target_dir.join("file2.txt");
    let moved_file3 = target_dir.join("file3.txt");
    assert!(
        !moved_file1.exists(),
        "file1.txt should not be moved to target_dir"
    );
    assert!(
        !moved_file2.exists(),
        "file2.txt should not be moved to target_dir"
    );
    assert!(
        moved_file3.exists(),
        "file3.txt should be moved to target_dir"
    );

    // Verify original files no longer exist (cut operation)
    assert!(
        !test_files[2].exists(),
        "Original file3.txt should no longer exist after cut"
    );
}
