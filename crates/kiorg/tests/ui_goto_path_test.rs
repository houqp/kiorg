#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use kiorg::ui::popup::PopupType;
use std::path::MAIN_SEPARATOR;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files, wait_for_condition};

#[test]
fn test_ui_goto_path_navigation() {
    // Create test directories
    let temp_dir = tempdir().unwrap();
    let dir1 = temp_dir.path().join("dir1");
    let dir2 = temp_dir.path().join("dir2");
    create_test_files(&[dir1, dir2.clone()]);

    let mut harness = create_harness(&temp_dir);

    // 1. Open Go To Path popup with 'gl'
    // First 'g'
    harness.key_press(Key::G);
    harness.step();
    // Second 'l'
    harness.key_press(Key::L);
    harness.step();

    // Verify popup is open
    assert!(
        matches!(harness.state().show_popup, Some(PopupType::GoToPath(_))),
        "Go To Path popup should be open after 'gl'"
    );

    // 2. Type 'dir2' to filter suggestions
    // The input starts with current_path + "/"
    let input_to_type = "dir2";
    for ch in input_to_type.chars() {
        harness
            .input_mut()
            .events
            .push(egui::Event::Text(ch.to_string()));
    }
    harness.step();

    // Verify suggestions are updated
    if let Some(PopupType::GoToPath(state)) = &harness.state().show_popup {
        assert!(
            !state.suggestions.is_empty(),
            "Suggestions should not be empty after typing 'dir2'"
        );
        let has_dir2 = state.suggestions.iter().any(|p| p.ends_with("dir2"));
        assert!(
            has_dir2,
            "Suggestions should contain 'dir2', got: {:?}",
            state.suggestions
        );
    } else {
        panic!("Popup should be GoToPath");
    }

    // 3. Use Tab to select and fill the input (it should add /)
    harness.key_press(Key::Tab);
    harness.step();

    if let Some(PopupType::GoToPath(state)) = &harness.state().show_popup {
        let expected_suffix = format!("dir2{}", MAIN_SEPARATOR);
        assert!(
            state.input.ends_with(&expected_suffix),
            "Input should be updated to end with '{}', got: {}",
            expected_suffix,
            state.input
        );
    } else {
        panic!("Popup should be GoToPath");
    }

    // 4. Press Enter to navigate
    harness.key_press(Key::Enter);
    harness.step();

    // Verify we navigated to dir2
    assert!(
        harness.state().show_popup.is_none(),
        "Popup should be closed after navigation"
    );
    let current_path = harness
        .state()
        .tab_manager
        .current_tab_ref()
        .current_path
        .clone();
    assert_eq!(
        current_path.canonicalize().unwrap(),
        dir2.canonicalize().unwrap()
    );
}

#[test]
fn test_ui_goto_path_tab_completion() {
    let temp_dir = tempdir().unwrap();
    let sub_dir = temp_dir.path().join("sub_dir");
    create_test_files(&[sub_dir.clone()]);

    let mut harness = create_harness(&temp_dir);

    // Open popup
    harness.key_press(Key::G);
    harness.step();
    harness.key_press(Key::L);
    harness.step();

    // Type "sub"
    for ch in "sub".chars() {
        harness
            .input_mut()
            .events
            .push(egui::Event::Text(ch.to_string()));
    }
    harness.step();

    // Press Tab
    harness.key_press(Key::Tab);
    harness.step();

    // Verify input updated and suggestions refreshed immediately
    if let Some(PopupType::GoToPath(state)) = &harness.state().show_popup {
        let expected_suffix = format!("sub_dir{}", MAIN_SEPARATOR);
        assert!(
            state.input.ends_with(&expected_suffix),
            "Input should be completed with '{}' after Tab, got: {}",
            expected_suffix,
            state.input
        );
        // Since sub_dir is empty, suggestions should be empty now
        assert!(
            state.suggestions.is_empty(),
            "Suggestions should be empty after Tab into empty sub_dir, got: {:?}",
            state.suggestions
        );
    } else {
        panic!("Popup should be GoToPath");
    }
}

#[test]
fn test_ui_goto_path_empty_input_root() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Open popup
    harness.key_press(Key::G);
    harness.step();
    harness.key_press(Key::L);
    harness.step();

    // Manually clear input to simulate user clearing it
    if let Some(PopupType::GoToPath(state)) = &mut harness.state_mut().show_popup {
        state.input = "".to_string();
        state.update_suggestions();
    }
    harness.step();

    if let Some(PopupType::GoToPath(state)) = &harness.state().show_popup {
        assert!(state.input.is_empty());
        assert!(
            !state.suggestions.is_empty(),
            "Should show root suggestions when input is empty"
        );

        #[cfg(unix)]
        {
            // On Unix systems, root should contain some of these
            let common_root_dirs = [
                "/usr",
                "/bin",
                "/etc",
                "/Applications",
                "/Users",
                "/Library",
            ];
            let has_some = state.suggestions.iter().any(|p| {
                common_root_dirs
                    .iter()
                    .any(|common| p == std::path::Path::new(common))
            });
            assert!(
                has_some,
                "Suggestions should contain some common root directories, got: {:?}",
                state.suggestions
            );
        }
    } else {
        panic!("Popup should be GoToPath");
    }
}

#[test]
fn test_ui_goto_path_auto_prepend_root() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Open popup
    harness.key_press(Key::G);
    harness.step();
    harness.key_press(Key::L);
    harness.step();

    // 1. Manually clear input
    if let Some(PopupType::GoToPath(state)) = &mut harness.state_mut().show_popup {
        state.input = "".to_string();
        state.update_suggestions();
    }
    harness.step();

    // 2. Type "etc"
    for ch in "etc".chars() {
        harness
            .input_mut()
            .events
            .push(egui::Event::Text(ch.to_string()));
    }
    harness.step();

    // Verify input became root-relative and suggestions are updated
    if let Some(PopupType::GoToPath(state)) = &harness.state().show_popup {
        #[cfg(unix)]
        assert_eq!(state.input, "/etc");
        #[cfg(windows)]
        assert_eq!(state.input, "\\etc");
    } else {
        panic!("Popup should be GoToPath");
    }
}

#[test]
fn test_ui_goto_path_cursor_preservation() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Open popup
    harness.key_press(Key::G);
    harness.step();
    harness.key_press(Key::L);
    harness.step();

    // 1. Manually clear input
    if let Some(PopupType::GoToPath(state)) = &mut harness.state_mut().show_popup {
        state.input = "".to_string();
        state.update_suggestions();
    }
    harness.step();

    // 2. Type "h"
    harness
        .input_mut()
        .events
        .push(egui::Event::Text("h".to_string()));
    harness.step();

    // Verify input became root-relative
    if let Some(PopupType::GoToPath(state)) = &harness.state().show_popup {
        #[cfg(unix)]
        assert_eq!(state.input, "/h");
        #[cfg(windows)]
        assert_eq!(state.input, "\\h");
    }

    // 3. Verify cursor is at the end
    let ctx = &harness.ctx;
    // The ID is explicitly set in goto_path.rs
    let input_id = egui::Id::new("goto_path_input");

    if let Some(text_state) = egui::TextEdit::load_state(ctx, input_id) {
        let cursor_pos = text_state.cursor.char_range().unwrap().primary.index;
        #[cfg(unix)]
        assert_eq!(
            cursor_pos, 2,
            "Cursor should be at the end after auto-prepending root (expected 2 for '/h')"
        );
        #[cfg(windows)]
        assert_eq!(
            cursor_pos, 2,
            "Cursor should be at the end after auto-prepending root (expected 2 for '\\h')"
        );
    } else {
        panic!(
            "Could not find TextEdit state for Go To Path input with ID {:?}",
            input_id
        );
    }
}

#[test]
fn test_ui_goto_path_tab_focus() {
    let temp_dir = tempdir().unwrap();
    let sub_dir = temp_dir.path().join("sub_dir");
    create_test_files(&[sub_dir.clone()]);

    let mut harness = create_harness(&temp_dir);

    // Open popup
    harness.key_press(Key::G);
    harness.step();
    harness.key_press(Key::L);
    harness.step();
    harness.step();

    let input_id = egui::Id::new("goto_path_input");

    // Type "sub"
    for ch in "sub".chars() {
        harness
            .input_mut()
            .events
            .push(egui::Event::Text(ch.to_string()));
    }
    harness.step();
    harness.step();

    // Press Tab
    harness.key_press(Key::Tab);

    // Wait for focus to return to input (might take a few frames)
    let focus_returned = wait_for_condition(|| {
        harness.step();
        harness.ctx.memory(|m| m.focused()) == Some(input_id)
    });
    assert!(
        focus_returned,
        "Focus should return to input after Tab completion."
    );

    // Verify input updated after Tab
    let expected_suffix = format!("sub_dir{}", MAIN_SEPARATOR);
    let input_after_tab = if let Some(PopupType::GoToPath(state)) = &harness.state().show_popup {
        state.input.clone()
    } else {
        panic!("Popup should still be open");
    };
    assert!(
        input_after_tab.ends_with(&expected_suffix),
        "Input should be updated after Tab, got: {}",
        input_after_tab
    );

    // Now try to type more to verify focus was maintained
    for ch in "foo".chars() {
        harness
            .input_mut()
            .events
            .push(egui::Event::Text(ch.to_string()));
        harness.step();
    }
    harness.step();

    // Verify input contains the typed text
    if let Some(PopupType::GoToPath(state)) = &harness.state().show_popup {
        assert!(
            state.input.ends_with("foo"),
            "Input should contain 'foo' after typing. Got: {}",
            state.input
        );
    } else {
        panic!("Popup should still be open");
    }
}
