#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_files, ctrl_modifiers};

use crate::ui_test_helpers::wait_for_condition;

#[cfg(windows)]
fn set_hidden_attribute_on_paths(paths: &[std::path::PathBuf]) {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{FILE_ATTRIBUTE_HIDDEN, SetFileAttributesW};

    for path in paths {
        let wide_path: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let result = unsafe { SetFileAttributesW(wide_path.as_ptr(), FILE_ATTRIBUTE_HIDDEN) };
        if result == 0 {
            panic!("Failed to set hidden attribute on {:?}", path);
        }
    }
}

#[test]
fn test_ctrl_h_toggle_hidden_files() {
    // Create a temporary directory with visible and hidden files/dirs
    let temp_dir = tempdir().unwrap();
    #[cfg(not(windows))]
    {
        create_test_files(&[
            temp_dir.path().join("visible_file.txt"),
            temp_dir.path().join(".hidden_file.txt"),
            temp_dir.path().join(".hidden_dir"),
        ]);
    }
    #[cfg(windows)]
    {
        let paths_to_hide = [
            temp_dir.path().join("hidden_file.txt"),
            temp_dir.path().join("hidden_dir"),
        ];
        create_test_files(&[temp_dir.path().join("visible_file.txt")]);
        create_test_files(&paths_to_hide);

        set_hidden_attribute_on_paths(&paths_to_hide);
    }

    let mut harness = create_harness(&temp_dir);

    // Initial state check
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(!harness.state().tab_manager.show_hidden);
        assert_eq!(tab.entries.len(), 1);
        assert_eq!(tab.entries[0].name, "visible_file.txt");
    }

    // Press Ctrl+H to show hidden files
    harness.key_press_modifiers(ctrl_modifiers(), Key::H);
    harness.step();

    // Verify hidden files are shown
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert!(harness.state().tab_manager.show_hidden);
        assert_eq!(tab.entries.len(), 3);
    }

    // Press Ctrl+H again to hide the files
    harness.key_press_modifiers(ctrl_modifiers(), Key::H);
    wait_for_condition(|| {
        harness.step();
        !harness.state().tab_manager.show_hidden
    });

    // Verify hidden files are hidden again
    {
        assert!(!harness.state().tab_manager.show_hidden);
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.entries.len(), 1);
    }
}
