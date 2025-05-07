mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::create_harness;

#[test]
fn test_exit_shortcut() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Initially, the app should not be in shutdown state
    assert_eq!(
        harness.state().shutdown_requested,
        false,
        "App should not be in shutdown state initially"
    );

    // Press 'q' to request exit (shows exit popup)
    harness.press_key(Key::Q);
    harness.step();

    // Verify exit popup is shown
    assert_eq!(
        harness.state().show_popup,
        Some(kiorg::app::PopupType::Exit),
        "Exit popup should be shown after pressing 'q'"
    );

    // Press Enter to confirm exit
    harness.press_key(Key::Enter);
    harness.step();

    // Verify shutdown was requested
    assert_eq!(
        harness.state().shutdown_requested,
        true,
        "App should be in shutdown state after confirming exit"
    );
}

#[test]
fn test_exit_with_unsaved_changes() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Create a file to be cut (simulating unsaved changes)
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "test content").unwrap();
    harness.state_mut().refresh_entries();
    harness.step();

    // Select the file
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let file_index = tab
            .entries
            .iter()
            .position(|e| e.path == file_path)
            .expect("File should be in the entries");

        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = file_index;
    }
    harness.step();

    // Cut the file (creating unsaved changes)
    harness.press_key(Key::X);
    harness.step();

    // Verify the file is marked as cut
    {
        let app = harness.state();
        assert!(app.clipboard.is_some(), "Clipboard should contain cut file");
        if let Some(kiorg::app::Clipboard::Cut(paths)) = &app.clipboard {
            assert!(paths.contains(&file_path), "File should be marked as cut");
        } else {
            panic!("Clipboard should contain a Cut operation");
        }
    }

    // Press 'q' to request exit (shows exit popup)
    harness.press_key(Key::Q);
    harness.step();

    // Verify exit popup is shown
    assert_eq!(
        harness.state().show_popup,
        Some(kiorg::app::PopupType::Exit),
        "Exit popup should be shown after pressing 'q'"
    );

    // Press Enter to confirm exit
    harness.press_key(Key::Enter);
    harness.step();

    // Verify shutdown was requested despite unsaved changes
    assert_eq!(
        harness.state().shutdown_requested,
        true,
        "App should be in shutdown state after confirming exit"
    );

    // Verify the file still exists (cut operation doesn't delete until paste)
    assert!(
        file_path.exists(),
        "File should still exist after cut and exit"
    );
}

#[test]
fn test_exit_saves_state() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a temporary directory for config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a test harness with the custom config directory
    let ctx = egui::Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx.clone());
    let app = kiorg::Kiorg::new_with_config_dir(
        &cc,
        Some(temp_dir.path().to_path_buf()),
        Some(config_dir.clone()),
    );
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::Vec2::new(800.0, 600.0))
        .with_max_steps(20)
        .build_eframe(|_cc| app);

    // Run one step to initialize
    harness.step();

    // Create a bookmark to have some state to save
    let bookmark_path = temp_dir.path().to_path_buf();
    harness.state_mut().bookmarks.push(bookmark_path.clone());

    // Press 'q' to request exit (shows exit popup)
    harness.press_key(Key::Q);
    harness.step();

    // Verify exit popup is shown
    assert_eq!(
        harness.state().show_popup,
        Some(kiorg::app::PopupType::Exit),
        "Exit popup should be shown after pressing 'q'"
    );

    // Press Enter to confirm exit
    harness.press_key(Key::Enter);
    harness.step();

    // Verify shutdown was requested
    assert_eq!(
        harness.state().shutdown_requested,
        true,
        "App should be in shutdown state after confirming exit"
    );

    // Verify state.json was created
    let state_file_path = config_dir.join("state.json");
    assert!(
        state_file_path.exists(),
        "state.json should exist after exit"
    );

    // Verify the state file contains our bookmark
    let state_content = std::fs::read_to_string(&state_file_path).unwrap();
    assert!(
        state_content.contains(&bookmark_path.to_string_lossy().to_string()),
        "state.json should contain the bookmark path"
    );
}
