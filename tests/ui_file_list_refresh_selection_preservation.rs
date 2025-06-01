#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use std::{fs::File, thread, time::Duration};
use tempfile::tempdir;
use ui_test_helpers::create_harness;

// Helper function to find an entry by name in the current tab
fn find_entry_index(harness: &ui_test_helpers::TestHarness, name: &str) -> Option<usize> {
    harness
        .state()
        .tab_manager
        .current_tab_ref()
        .entries
        .iter()
        .position(|e| e.name == name)
}

// Helper function to wait for a condition to be met
fn wait_for_condition<F>(
    harness: &mut ui_test_helpers::TestHarness,
    condition: F,
    description: &str,
) where
    F: Fn(&ui_test_helpers::TestHarness) -> bool,
{
    let max_iterations = 300;
    let sleep_duration = Duration::from_millis(10);

    for _ in 0..max_iterations {
        harness.step();
        if condition(harness) {
            return;
        }
        // Sleep for a short interval before checking again
        thread::sleep(sleep_duration);
    }

    panic!(
        "Condition '{}' was not met after waiting for {} iterations of {}ms",
        description,
        max_iterations,
        sleep_duration.as_millis()
    );
}

/// Test that shows the expected behavior - selection should follow the file,
/// not stay at the same index when the list changes.
#[test]
fn test_file_list_refresh_should_preserve_selected_file() {
    let temp_dir = tempdir().unwrap();

    // Create initial test files
    let file1_path = temp_dir.path().join("file1.txt");
    let file2_path = temp_dir.path().join("file2.txt");
    let file3_path = temp_dir.path().join("file3.txt");

    File::create(&file1_path).expect("Failed to create file1.txt");
    File::create(&file2_path).expect("Failed to create file2.txt");
    File::create(&file3_path).expect("Failed to create file3.txt");

    let mut harness = create_harness(&temp_dir);

    // Move selection to file2.txt (index 1)
    harness.press_key(Key::J);
    harness.step();

    // Store the selected file name (not index)
    let selected_file_name = {
        let tab = harness.state().tab_manager.current_tab_ref();
        assert_eq!(tab.selected_index, 1, "Should have selected index 1");
        let selected_entry = &tab.entries[tab.selected_index];
        assert_eq!(
            selected_entry.name, "file2.txt",
            "Should have selected file2.txt"
        );
        selected_entry.name.clone()
    };

    // Create an external file that will change the list order
    let file0_path = temp_dir.path().join("file0.txt");
    File::create(&file0_path).expect("Failed to create file0.txt");

    // Wait for filesystem notification
    wait_for_condition(
        &mut harness,
        |h| find_entry_index(h, "file0.txt").is_some(),
        "file0.txt to appear in UI after creation",
    );

    // Check what should ideally happen (this test will pass once the bug is fixed)
    let tab = harness.state().tab_manager.current_tab_ref();
    let selected_entry = &tab.entries[tab.selected_index];

    assert_eq!(
        selected_entry.name, selected_file_name,
        "Selection should be preserved across file list refresh"
    );
}
