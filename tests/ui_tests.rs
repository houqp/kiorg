use std::fs::File;
use std::path::PathBuf;
use tempfile::tempdir;
use egui_kittest::Harness;
use kiorg::Kiorg;
use eframe::CreationContext;
use egui::Key;

/// Create files and directories from a list of paths.
/// Returns the created paths.
fn create_test_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    for path in paths {
        if path.extension().is_some() {
            File::create(path).unwrap();
        } else {
            std::fs::create_dir(path).unwrap();
        }
        assert!(path.exists());
    }
    paths.to_vec()
}

fn create_harness<'a>(temp_dir: &tempfile::TempDir) -> Harness<'a, Kiorg> {
    // Create a new egui context
    let ctx = egui::Context::default();
    let cc = CreationContext::_new_kittest(ctx.clone());
    let app = Kiorg::new(&cc, temp_dir.path().to_path_buf());

    // Create a test harness with more steps to ensure all events are processed
    let harness = Harness::builder()
        .with_size(egui::Vec2::new(800.0, 600.0))
        .with_max_steps(20)
        .build_eframe(|_cc| app);

    harness
}

#[test]
fn test_delete_shortcut() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create multiple test files and directories
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

    // Test recursive directory deletion
    // First entry should be dir1
    // Delete dir1 (directory with nested files and subdirectory)
    harness.press_key(Key::D);
    harness.step();
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

    // Press 'delete' to clear any existing text
    for _ in 0..".text".len() {
        harness.press_key(Key::Backspace);
        harness.step();
    }

    // Clear any existing text and simulate text input for the new name
    harness.input_mut().events.push(egui::Event::Text("_renamed.txt".to_string()));
    harness.step();

    // Press Enter to confirm rename
    harness.press_key(Key::Enter);
    harness.step();

    // Verify the file was renamed
    assert!(test_files[0].exists(), "test1.txt should still exist");
    assert!(!test_files[1].exists(), "test2.txt should no longer exist");
    assert!(temp_dir.path().join("test2_renamed.txt").exists(), "renamed.txt should exist");
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
    assert!(test_files[2].exists(), "test1.txt should still exist in original location");
    assert!(test_files[1].join("test1.txt").exists(), "test1.txt should be copied to dir2");
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

    // Copy test1.txt
    harness.press_key(Key::Y);
    harness.step(); // Run 5 steps to handle repainting

    // Paste in the same directory
    harness.press_key(Key::P);
    harness.step(); // Run 5 steps to handle repainting

    // Verify the file was copied with a new suffix
    assert!(test_files[0].exists(), "test1.txt should still exist");
    assert!(test_files[1].exists(), "test2.txt should still exist");

    // Check for the copied file with a new suffix
    let copied_file = temp_dir.path().join("test1_1.txt");
    assert!(copied_file.exists(), "test1.txt should be copied with suffix `_1`");
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
    assert!(test_files[2].exists(), "test1.txt should still exist after cutting");

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
    assert!(!test_files[2].exists(), "test1.txt should be moved from original location");
    assert!(test_files[1].join("test1.txt").exists(), "test1.txt should exist in dir2");
}