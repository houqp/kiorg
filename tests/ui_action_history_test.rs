#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use kiorg::ui::popup::PopupType;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files, ctrl_modifiers, wait_for_condition};

// Import required types for action history tests
// Note: Action history types are used internally by the application

#[test]
fn test_undo_redo_create_file() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    let test_file = temp_dir.path().join("test_file.txt");

    // Create a new file using 'a' shortcut
    harness.key_press(Key::A);
    harness.step();

    // Verify the add entry popup is shown
    if let Some(PopupType::AddEntry(_)) = harness.state().show_popup {
        // Add mode is active
    } else {
        panic!("Add mode should be active");
    }

    // Type the filename
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("test_file.txt".to_string()));
    harness.step();

    // Press Enter to confirm creation
    harness.key_press(Key::Enter);
    wait_for_condition(|| {
        harness.step();
        harness.state().show_popup.is_none()
    });

    // Verify file was created
    assert!(test_file.exists(), "File should be created");

    // Verify file appears in UI
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.entries.iter().any(|e| e.path == test_file),
            "File should appear in UI entries"
        );
    }

    // Test Undo - press 'u' to undo the creation
    harness.key_press(Key::U);
    harness.step();

    // Verify file was removed
    assert!(!test_file.exists(), "File should be deleted after undo");

    // Verify file is removed from UI
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            !tab.entries.iter().any(|e| e.path == test_file),
            "File should be removed from UI entries after undo"
        );
    }

    // Verify action was moved to rolled back
    {
        let history = &harness.state().tab_manager.current_tab_ref().action_history;
        assert!(
            history.has_rolled_back_actions(),
            "Should have rolled back actions"
        );
        assert!(
            history.get_last_rollbackable_action().is_none(),
            "Should have no undoable actions"
        );
    }

    // Test Redo - press Ctrl+r to redo the creation
    harness.key_press_modifiers(ctrl_modifiers(), Key::R);
    harness.step();

    // Verify file was recreated
    assert!(test_file.exists(), "File should be recreated after redo");

    // Verify file appears in UI again
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.entries.iter().any(|e| e.path == test_file),
            "File should appear in UI entries after redo"
        );
    }

    // Verify action was moved back to active
    {
        let history = &harness.state().tab_manager.current_tab_ref().action_history;
        assert!(
            !history.has_rolled_back_actions(),
            "Should have no rolled back actions"
        );
        assert!(
            history.get_last_rollbackable_action().is_some(),
            "Should have undoable actions"
        );
    }
}

#[test]
fn test_undo_redo_create_directory() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    let test_dir = temp_dir.path().join("test_dir");

    // Create a new directory using 'a' shortcut
    harness.key_press(Key::A);
    harness.step();

    // Verify the add entry popup is shown
    if let Some(PopupType::AddEntry(_)) = harness.state().show_popup {
        // Add mode is active
    } else {
        panic!("Add mode should be active");
    }

    // Type the directory name with trailing slash
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("test_dir/".to_string()));
    harness.step();

    // Press Enter to confirm creation
    harness.key_press(Key::Enter);
    wait_for_condition(|| {
        harness.step();
        harness.state().show_popup.is_none()
    });

    // Verify directory was created
    assert!(
        test_dir.exists() && test_dir.is_dir(),
        "Directory should be created"
    );

    // Verify directory appears in UI
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.entries.iter().any(|e| e.path == test_dir),
            "Directory should appear in UI entries"
        );
    }

    // Test Undo
    harness.key_press(Key::U);
    harness.step();

    // Verify directory was removed
    assert!(!test_dir.exists(), "Directory should be deleted after undo");

    // Verify directory is removed from UI
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            !tab.entries.iter().any(|e| e.path == test_dir),
            "Directory should be removed from UI entries after undo"
        );
    }

    // Test Redo
    harness.key_press_modifiers(ctrl_modifiers(), Key::R);
    harness.step();

    // Verify directory was recreated
    assert!(
        test_dir.exists() && test_dir.is_dir(),
        "Directory should be recreated after redo"
    );

    // Verify directory appears in UI again
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.entries.iter().any(|e| e.path == test_dir),
            "Directory should appear in UI entries after redo"
        );
    }
}

#[test]
fn test_undo_redo_rename_file() {
    let temp_dir = tempdir().unwrap();
    let original_file = temp_dir.path().join("original.txt");
    let renamed_file = temp_dir.path().join("original_renamed.txt");

    // Create initial test file
    create_test_files(std::slice::from_ref(&original_file));

    let mut harness = create_harness(&temp_dir);

    // Select the file (should be the first entry)
    harness.key_press(Key::J); // Move to first file
    harness.step();

    // Rename the file using 'r' shortcut
    harness.key_press(Key::R);
    harness.step();

    // Verify the rename popup is shown
    if let Some(PopupType::Rename(name)) = &harness.state().show_popup {
        assert_eq!(
            name, "original.txt",
            "Rename popup should contain the current filename"
        );
    } else {
        panic!("Rename mode should be active");
    }

    // Clear existing text and type new name
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("_renamed".to_string()));
    harness.step();

    // Press Enter to confirm rename
    harness.key_press(Key::Enter);

    wait_for_condition(|| {
        harness.step();
        harness.state().show_popup.is_none()
    });

    // Verify the popup is closed
    assert_eq!(
        harness.state().show_popup,
        None,
        "Rename popup should be closed after confirming"
    );

    // Verify file was renamed
    assert!(!original_file.exists(), "Original file should not exist");
    assert!(renamed_file.exists(), "Renamed file should exist");

    // Verify UI reflects the rename
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            !tab.entries.iter().any(|e| e.path == original_file),
            "Original file should not appear in UI"
        );
        assert!(
            tab.entries.iter().any(|e| e.path == renamed_file),
            "Renamed file should appear in UI"
        );
    }

    // Test Undo
    harness.key_press(Key::U);
    harness.step();

    // Verify rename was undone
    assert!(
        original_file.exists(),
        "Original file should exist after undo"
    );
    assert!(
        !renamed_file.exists(),
        "Renamed file should not exist after undo"
    );

    // Verify UI reflects the undo
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.entries.iter().any(|e| e.path == original_file),
            "Original file should appear in UI after undo"
        );
        assert!(
            !tab.entries.iter().any(|e| e.path == renamed_file),
            "Renamed file should not appear in UI after undo"
        );
    }

    // Test Redo
    harness.key_press_modifiers(ctrl_modifiers(), Key::R);
    harness.step();

    // Verify rename was redone
    assert!(
        !original_file.exists(),
        "Original file should not exist after redo"
    );
    assert!(
        renamed_file.exists(),
        "Renamed file should exist after redo"
    );

    // Verify UI reflects the redo
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            !tab.entries.iter().any(|e| e.path == original_file),
            "Original file should not appear in UI after redo"
        );
        assert!(
            tab.entries.iter().any(|e| e.path == renamed_file),
            "Renamed file should appear in UI after redo"
        );
    }
}

#[test]
fn test_undo_redo_copy_file() {
    let temp_dir = tempdir().unwrap();
    let source_file = temp_dir.path().join("source.txt");
    let copied_file = temp_dir.path().join("source_1.txt");

    // Create initial test file
    create_test_files(std::slice::from_ref(&source_file));
    std::fs::write(&source_file, "test content").unwrap();

    let mut harness = create_harness(&temp_dir);

    // Select the file (should be the first entry)
    harness.key_press(Key::J); // Move to first file
    harness.step();

    // Copy the file using 'c' shortcut
    harness.key_press(Key::Y);
    harness.step();

    // Paste using 'p' shortcut
    harness.key_press(Key::P);
    harness.step();

    // Wait for paste operation to complete
    wait_for_condition(|| {
        harness.step();
        copied_file.exists()
    });

    // Verify file was copied
    assert!(source_file.exists(), "Source file should still exist");
    assert!(copied_file.exists(), "Copied file should exist");

    // Verify content is the same
    assert_eq!(
        std::fs::read_to_string(&source_file).unwrap(),
        std::fs::read_to_string(&copied_file).unwrap(),
        "File contents should match"
    );

    // Verify UI reflects the copy
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.entries.iter().any(|e| e.path == source_file),
            "Source file should appear in UI"
        );
        assert!(
            tab.entries.iter().any(|e| e.path == copied_file),
            "Copied file should appear in UI"
        );
    }

    // Test Undo
    harness.key_press(Key::U);
    harness.step();

    // Verify copy was undone
    assert!(
        source_file.exists(),
        "Source file should still exist after undo"
    );
    assert!(
        !copied_file.exists(),
        "Copied file should not exist after undo"
    );

    // Verify UI reflects the undo
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.entries.iter().any(|e| e.path == source_file),
            "Source file should still appear in UI after undo"
        );
        assert!(
            !tab.entries.iter().any(|e| e.path == copied_file),
            "Copied file should not appear in UI after undo"
        );
    }

    // Test Redo
    harness.key_press_modifiers(ctrl_modifiers(), Key::R);
    harness.step();

    // Verify copy was redone
    assert!(
        source_file.exists(),
        "Source file should still exist after redo"
    );
    assert!(copied_file.exists(), "Copied file should exist after redo");

    // Verify content is still the same
    assert_eq!(
        std::fs::read_to_string(&source_file).unwrap(),
        std::fs::read_to_string(&copied_file).unwrap(),
        "File contents should match after redo"
    );

    // Verify UI reflects the redo
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(
            tab.entries.iter().any(|e| e.path == source_file),
            "Source file should appear in UI after redo"
        );
        assert!(
            tab.entries.iter().any(|e| e.path == copied_file),
            "Copied file should appear in UI after redo"
        );
    }
}

#[test]
fn test_undo_redo_move_file() {
    let temp_dir = tempdir().unwrap();
    let subdir = temp_dir.path().join("subdir");
    let source_file = temp_dir.path().join("moveme.txt");
    let moved_file = subdir.join("moveme.txt");

    // Create initial test files and directory
    std::fs::create_dir(&subdir).unwrap();
    create_test_files(std::slice::from_ref(&source_file));
    std::fs::write(&source_file, "move test content").unwrap();

    let mut harness = create_harness(&temp_dir);

    // Select the file to move (moveme.txt should be after subdir)
    harness.key_press(Key::J); // Move to first entry (subdir)
    harness.step();
    harness.key_press(Key::J); // Move to second entry (moveme.txt)
    harness.step();

    // Cut the file using 'x' shortcut
    harness.key_press(Key::X);
    harness.step();

    // Navigate into subdirectory
    harness.key_press(Key::K); // Move back to subdir
    harness.step();
    harness.key_press(Key::Enter); // Enter subdirectory
    harness.step();

    // Paste using 'p' shortcut
    harness.key_press(Key::P);
    harness.step();

    // Wait for move operation to complete
    wait_for_condition(|| {
        harness.step();
        moved_file.exists()
    });

    // Verify file was moved
    assert!(
        !source_file.exists(),
        "Source file should not exist after move"
    );
    assert!(moved_file.exists(), "Moved file should exist");

    // Verify content is preserved
    assert_eq!(
        std::fs::read_to_string(&moved_file).unwrap(),
        "move test content",
        "File content should be preserved"
    );

    // Test Undo
    harness.key_press(Key::U);
    harness.step();

    // Verify move was undone
    assert!(source_file.exists(), "Source file should exist after undo");
    assert!(
        !moved_file.exists(),
        "Moved file should not exist after undo"
    );

    // Verify content is preserved
    assert_eq!(
        std::fs::read_to_string(&source_file).unwrap(),
        "move test content",
        "File content should be preserved after undo"
    );

    // Test Redo
    harness.key_press_modifiers(ctrl_modifiers(), Key::R);
    harness.step();

    // Verify move was redone
    assert!(
        !source_file.exists(),
        "Source file should not exist after redo"
    );
    assert!(moved_file.exists(), "Moved file should exist after redo");

    // Verify content is still preserved
    assert_eq!(
        std::fs::read_to_string(&moved_file).unwrap(),
        "move test content",
        "File content should be preserved after redo"
    );
}

#[test]
fn test_undo_redo_multiple_operations() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    let dir1 = temp_dir.path().join("testdir");

    // Create multiple operations to test sequential undo/redo

    // Operation 1: Create file1.txt
    harness.key_press(Key::A);
    harness.step();
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("file1.txt".to_string()));
    harness.step();
    harness.key_press(Key::Enter);
    wait_for_condition(|| {
        harness.step();
        harness.state().show_popup.is_none()
    });

    // Operation 2: Create file2.txt
    harness.key_press(Key::A);
    harness.step();
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("file2.txt".to_string()));
    harness.step();
    harness.key_press(Key::Enter);
    wait_for_condition(|| {
        harness.step();
        harness.state().show_popup.is_none()
    });

    // Operation 3: Create directory
    harness.key_press(Key::A);
    harness.step();
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("testdir/".to_string()));
    harness.step();
    harness.key_press(Key::Enter);
    wait_for_condition(|| {
        harness.step();
        harness.state().show_popup.is_none()
    });

    // Verify all were created
    assert!(file1.exists(), "file1.txt should exist");
    assert!(file2.exists(), "file2.txt should exist");
    assert!(dir1.exists() && dir1.is_dir(), "testdir should exist");

    // Test sequential undo

    // Undo operation 3 (directory creation)
    harness.key_press(Key::U);
    harness.step();
    assert!(file1.exists(), "file1.txt should still exist");
    assert!(file2.exists(), "file2.txt should still exist");
    assert!(!dir1.exists(), "testdir should be removed");

    // Undo operation 2 (file2.txt creation)
    harness.key_press(Key::U);
    harness.step();
    assert!(file1.exists(), "file1.txt should still exist");
    assert!(!file2.exists(), "file2.txt should be removed");
    assert!(!dir1.exists(), "testdir should still be removed");

    // Undo operation 1 (file1.txt creation)
    harness.key_press(Key::U);
    harness.step();
    assert!(!file1.exists(), "file1.txt should be removed");
    assert!(!file2.exists(), "file2.txt should still be removed");
    assert!(!dir1.exists(), "testdir should still be removed");

    // Verify we have no more undoable actions
    {
        let history = &harness.state().tab_manager.current_tab_ref().action_history;
        assert!(
            history.get_last_rollbackable_action().is_none(),
            "Should have no more undoable actions"
        );
    }

    // Test sequential redo

    // Redo operation 1 (file1.txt creation)
    harness.key_press_modifiers(ctrl_modifiers(), Key::R);
    harness.step();
    assert!(file1.exists(), "file1.txt should be recreated");
    assert!(!file2.exists(), "file2.txt should still not exist");
    assert!(!dir1.exists(), "testdir should still not exist");

    // Redo operation 2 (file2.txt creation)
    harness.key_press_modifiers(ctrl_modifiers(), Key::R);
    harness.step();
    assert!(file1.exists(), "file1.txt should still exist");
    assert!(file2.exists(), "file2.txt should be recreated");
    assert!(!dir1.exists(), "testdir should still not exist");

    // Redo operation 3 (directory creation)
    harness.key_press_modifiers(ctrl_modifiers(), Key::R);
    harness.step();
    assert!(file1.exists(), "file1.txt should still exist");
    assert!(file2.exists(), "file2.txt should still exist");
    assert!(
        dir1.exists() && dir1.is_dir(),
        "testdir should be recreated"
    );

    // Verify we have no more redoable actions
    {
        let history = &harness.state().tab_manager.current_tab_ref().action_history;
        assert!(
            history.get_last_redoable_action().is_none(),
            "Should have no more redoable actions"
        );
    }
}

#[test]
fn test_undo_redo_invalidates_redo_stack() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");

    // Create file1.txt
    harness.key_press(Key::A);
    harness.step();
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("file1.txt".to_string()));
    harness.step();
    harness.key_press(Key::Enter);
    wait_for_condition(|| {
        harness.step();
        harness.state().show_popup.is_none()
    });

    // Undo the creation
    harness.key_press(Key::U);
    harness.step();
    assert!(!file1.exists(), "file1.txt should be removed");

    // Verify we can redo
    {
        let history = &harness.state().tab_manager.current_tab_ref().action_history;
        assert!(
            history.get_last_redoable_action().is_some(),
            "Should have redoable action"
        );
    }

    // Perform a new operation (should clear redo stack)
    harness.key_press(Key::A);
    harness.step();
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("file2.txt".to_string()));
    harness.step();
    harness.key_press(Key::Enter);
    wait_for_condition(|| {
        harness.step();
        harness.state().show_popup.is_none()
    });

    assert!(file2.exists(), "file2.txt should be created");

    // Verify redo stack is cleared
    {
        let history = &harness.state().tab_manager.current_tab_ref().action_history;
        assert!(
            history.get_last_redoable_action().is_none(),
            "Should have no redoable actions after new operation"
        );
    }

    // Verify we can't redo the file1 creation anymore
    harness.key_press_modifiers(ctrl_modifiers(), Key::R);
    harness.step();
    assert!(!file1.exists(), "file1.txt should still not exist");
    assert!(file2.exists(), "file2.txt should still exist");
}
