#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files};

#[test]
fn test_add_file_and_directory() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // --- Test 1: Add a file ---
    let file_name = "new_file.txt";
    let expected_file_path = temp_dir.path().join(file_name);

    // Press 'a' to activate add mode
    harness.key_press(Key::A);
    harness.step();

    // Check if add mode is active
    if let Some(kiorg::app::PopupType::AddEntry(_)) = harness.state().show_popup {
        // Add mode is active
    } else {
        panic!("Add mode should be active");
    }

    // Input the file name
    harness
        .input_mut()
        .events
        .push(egui::Event::Text(file_name.to_string()));
    harness.step();

    // Check if the input field contains the file name
    if let Some(kiorg::app::PopupType::AddEntry(name)) = &harness.state().show_popup {
        assert_eq!(name, file_name, "Input field should contain the file name");
    } else {
        panic!("Add mode should be active with the file name");
    }

    // Press Enter to confirm creation
    harness.key_press(Key::Enter);
    harness.step(); // Step to process creation and refresh

    // Verify file exists on filesystem
    assert!(
        expected_file_path.exists(),
        "File '{file_name}' should exist on filesystem"
    );
    assert!(
        expected_file_path.is_file(),
        "'{file_name}' should be a file"
    );

    // Verify file appears in UI list and is selected
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let entry_index = tab
            .entries
            .iter()
            .position(|e| e.name == file_name)
            .expect("New file should be in the UI list");
        assert_eq!(
            tab.selected_index, entry_index,
            "Newly created file should be selected"
        );
        assert_eq!(
            tab.entries[entry_index].path, expected_file_path,
            "UI entry path should match expected path"
        );
        assert!(
            !tab.entries[entry_index].is_dir,
            "UI entry should be marked as a file"
        );
    }
    assert!(
        harness.state().show_popup.is_none(),
        "Add mode should be inactive"
    );

    // --- Test 2: Add a directory ---
    let dir_name_input = "new_dir/"; // Input includes trailing slash
    let dir_name_actual = "new_dir"; // Actual directory name doesn't include slash
    let expected_dir_path = temp_dir.path().join(dir_name_actual);

    // Press 'a' to activate add mode
    harness.key_press(Key::A);
    harness.step();

    // Check if add mode is active
    if let Some(kiorg::app::PopupType::AddEntry(_)) = harness.state().show_popup {
        // Add mode is active
    } else {
        panic!("Add mode should be active");
    }

    // Input the directory name
    harness
        .input_mut()
        .events
        .push(egui::Event::Text(dir_name_input.to_string()));
    harness.step();

    // Check if the input field contains the directory name
    if let Some(kiorg::app::PopupType::AddEntry(name)) = &harness.state().show_popup {
        assert_eq!(
            name, dir_name_input,
            "Input field should contain the directory name"
        );
    } else {
        panic!("Add mode should be active with the directory name");
    }

    // Press Enter to confirm creation
    harness.key_press(Key::Enter);
    harness.step(); // Step to process creation and refresh

    // Verify directory exists on filesystem
    assert!(
        expected_dir_path.exists(),
        "Directory '{dir_name_actual}' should exist on filesystem"
    );
    assert!(
        expected_dir_path.is_dir(),
        "'{dir_name_actual}' should be a directory"
    );

    // Verify directory appears in UI list and is selected
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let entry_index = tab
            .entries
            .iter()
            .position(|e| e.name == dir_name_actual)
            .expect("New directory should be in the UI list");
        assert_eq!(
            tab.selected_index, entry_index,
            "Newly created directory should be selected"
        );
        assert_eq!(
            tab.entries[entry_index].path, expected_dir_path,
            "UI entry path should match expected path"
        );
        assert!(
            tab.entries[entry_index].is_dir,
            "UI entry should be marked as a directory"
        );
    }
    assert!(
        harness.state().show_popup.is_none(),
        "Add mode should be inactive"
    );

    // --- Test 3: Add a file with 'q' ---
    let file_name_q = "quick_file.txt";
    let expected_file_q_path = temp_dir.path().join(file_name_q);

    // Press 'a'
    harness.key_press(Key::A);
    harness.step();
    // Input name
    harness
        .input_mut()
        .events
        .push(egui::Event::Text(file_name_q.to_string()));
    harness.step();
    // Press Enter
    harness.key_press(Key::Enter);
    harness.step();

    // Verify file exists
    assert!(
        expected_file_q_path.exists(),
        "File '{file_name_q}' should exist"
    );
    assert!(
        expected_file_q_path.is_file(),
        "'{file_name_q}' should be a file"
    );

    // Verify file appears in UI and is selected
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let entry_index = tab
            .entries
            .iter()
            .position(|e| e.name == file_name_q)
            .expect("File with 'q' should be in the UI list");
        assert_eq!(
            tab.selected_index, entry_index,
            "Newly created file with 'q' should be selected"
        );
    }

    // --- Test 4: Add a directory with 'q' ---
    let dir_name_q_input = "quirky_dir/";
    let dir_name_q_actual = "quirky_dir";
    let expected_dir_q_path = temp_dir.path().join(dir_name_q_actual);

    // Press 'a'
    harness.key_press(Key::A);
    harness.step();
    // Input name
    harness
        .input_mut()
        .events
        .push(egui::Event::Text(dir_name_q_input.to_string()));
    harness.step();
    // Press Enter
    harness.key_press(Key::Enter);
    harness.step();

    // Verify directory exists
    assert!(
        expected_dir_q_path.exists(),
        "Directory '{dir_name_q_actual}' should exist"
    );
    assert!(
        expected_dir_q_path.is_dir(),
        "'{dir_name_q_actual}' should be a directory"
    );

    // Verify directory appears in UI and is selected
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let entry_index = tab
            .entries
            .iter()
            .position(|e| e.name == dir_name_q_actual)
            .expect("Directory with 'q' should be in the UI list");
        assert_eq!(
            tab.selected_index, entry_index,
            "Newly created directory with 'q' should be selected"
        );
    }
}

#[test]
fn test_add_entry_cancel() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Press 'a' to activate add mode
    harness.key_press(Key::A);
    harness.step();

    // Check if add mode is active
    if let Some(kiorg::app::PopupType::AddEntry(_)) = harness.state().show_popup {
        // Add mode is active
    } else {
        panic!("Add mode should be active");
    }

    // Input some text
    let partial_name = "partial_name";
    harness
        .input_mut()
        .events
        .push(egui::Event::Text(partial_name.to_string()));
    harness.step();

    // Check if the input field contains the partial name
    if let Some(kiorg::app::PopupType::AddEntry(name)) = &harness.state().show_popup {
        assert_eq!(
            name, partial_name,
            "Input field should contain partial name"
        );
    } else {
        panic!("Add mode should be active with the partial name");
    }

    // Press Escape to cancel
    harness.key_press(Key::Escape);
    harness.step();

    // Verify add mode is inactive and input is cleared
    assert!(
        harness.state().show_popup.is_none(),
        "Add mode should be inactive"
    );

    // Verify no file/directory was created
    assert!(
        !temp_dir.path().join(partial_name).exists(),
        "No entry should have been created after cancelling"
    );
}

#[test]
fn test_add_entry_name_conflict() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files that will cause conflicts
    let existing_file = "existing_file.txt";
    let existing_dir = "existing_dir";
    create_test_files(&[
        temp_dir.path().join(existing_file),
        temp_dir.path().join(existing_dir),
    ]);

    let mut harness = create_harness(&temp_dir);

    // --- Test 1: Try to create a file with a name that already exists ---

    // Press 'a' to activate add mode
    harness.key_press(Key::A);
    harness.step();

    // Check if add mode is active
    if let Some(kiorg::app::PopupType::AddEntry(_)) = harness.state().show_popup {
        // Add mode is active
    } else {
        panic!("Add mode should be active");
    }

    // Input the existing file name
    harness
        .input_mut()
        .events
        .push(egui::Event::Text(existing_file.to_string()));
    harness.step();

    // Press Enter to attempt creation
    harness.key_press(Key::Enter);
    harness.step();

    // Verify add mode is still active (popup remains open)
    if let Some(kiorg::app::PopupType::AddEntry(_)) = harness.state().show_popup {
        // Add mode is still active
    } else {
        panic!("Add mode should still be active after name conflict");
    }

    // Verify the error message was shown (we can't directly check toast content in tests)
    // But we can verify the popup is still open with the same content
    if let Some(kiorg::app::PopupType::AddEntry(name)) = &harness.state().show_popup {
        assert_eq!(
            name, existing_file,
            "Input field should still contain the conflicting name"
        );
    } else {
        panic!("Add mode should be active with the conflicting name");
    }

    // Press Escape to cancel
    harness.key_press(Key::Escape);
    harness.step();

    // --- Test 2: Try to create a directory with a name that already exists ---

    // Press 'a' to activate add mode
    harness.key_press(Key::A);
    harness.step();

    // Input the existing directory name with trailing slash
    let dir_input = format!("{existing_dir}/");
    harness
        .input_mut()
        .events
        .push(egui::Event::Text(dir_input.clone()));
    harness.step();

    // Press Enter to attempt creation
    harness.key_press(Key::Enter);
    harness.step();

    // Verify add mode is still active (popup remains open)
    if let Some(kiorg::app::PopupType::AddEntry(_)) = harness.state().show_popup {
        // Add mode is still active
    } else {
        panic!("Add mode should still be active after directory name conflict");
    }

    // Verify the input field still contains the conflicting name
    if let Some(kiorg::app::PopupType::AddEntry(name)) = &harness.state().show_popup {
        assert_eq!(
            name, &dir_input,
            "Input field should still contain the conflicting directory name"
        );
    } else {
        panic!("Add mode should be active with the conflicting directory name");
    }

    // Press Escape to cancel
    harness.key_press(Key::Escape);
    harness.step();

    // Verify add mode is now inactive
    assert!(
        harness.state().show_popup.is_none(),
        "Add mode should be inactive after cancellation"
    );
}
