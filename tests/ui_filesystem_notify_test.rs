mod ui_test_helpers;

use std::{
    fs::{self, File},
    io::Write,
    sync::atomic::Ordering, // Import Ordering
    thread,
    time::Duration,
};
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

// Helper function to wait for FS events and update UI
fn wait_and_step(harness: &mut ui_test_helpers::TestHarness) {
    // Wait for filesystem events to propagate and be picked up by notify
    // Check in a loop with short intervals to avoid unnecessary waiting
    let max_iterations = 20;
    let sleep_duration = Duration::from_millis(50);
    let mut iterations = 0;

    while iterations < max_iterations {
        if harness.state().notify_fs_change.load(Ordering::Relaxed) {
            // Flag is set, we can proceed
            break;
        }
        // Sleep for a short interval before checking again
        thread::sleep(sleep_duration);
        iterations += 1;
    }

    // After the loop, assert that the flag was eventually set
    assert!(
        harness.state().notify_fs_change.load(Ordering::Relaxed),
        "notify_fs_change should be true after waiting for {} iterations of {}ms",
        max_iterations,
        sleep_duration.as_millis()
    );
    harness.step(); // Process events and update UI (should consume the flag)
    harness.step(); // Another step might be needed for async updates
}

#[test]
fn test_external_file_addition() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    let file_name = "external_file.txt";
    let file_path = temp_dir.path().join(file_name);

    assert!(
        find_entry_index(&harness, file_name).is_none(),
        "File should not exist in UI initially"
    );

    File::create(&file_path).expect("Failed to create external file");
    assert!(file_path.exists());

    wait_and_step(&mut harness);

    let file_index = find_entry_index(&harness, file_name)
        .expect("External file should appear in the UI after creation");
    let entry = &harness.state().tab_manager.current_tab_ref().entries[file_index];
    assert_eq!(entry.name, file_name);
    assert_eq!(entry.path, file_path);
    assert!(!entry.is_dir);
}

#[test]
fn test_external_directory_addition() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    let dir_name = "external_dir";
    let dir_path = temp_dir.path().join(dir_name);

    assert!(
        find_entry_index(&harness, dir_name).is_none(),
        "Directory should not exist in UI initially"
    );

    fs::create_dir(&dir_path).expect("Failed to create external directory");
    assert!(dir_path.exists());

    wait_and_step(&mut harness);

    let dir_index = find_entry_index(&harness, dir_name)
        .expect("External directory should appear in the UI after creation");
    let entry = &harness.state().tab_manager.current_tab_ref().entries[dir_index];
    assert_eq!(entry.name, dir_name);
    assert_eq!(entry.path, dir_path);
    assert!(entry.is_dir);
}

#[test]
fn test_external_file_modification() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    let mod_file_name = "modifiable_file.txt";
    let mod_file_path = temp_dir.path().join(mod_file_name);

    File::create(&mod_file_path).expect("Failed to create modifiable file");
    wait_and_step(&mut harness); // Ensure it's in the UI

    let initial_entry_index =
        find_entry_index(&harness, mod_file_name).expect("Modifiable file should be in UI");
    let initial_entry =
        harness.state().tab_manager.current_tab_ref().entries[initial_entry_index].clone();
    assert_eq!(initial_entry.size, 0, "Initial size should be 0");

    let mut file = File::options().write(true).open(&mod_file_path).unwrap();
    let content = "some content";
    file.write_all(content.as_bytes()).unwrap();
    file.sync_all().unwrap();
    drop(file);

    wait_and_step(&mut harness);

    let updated_entry_index = find_entry_index(&harness, mod_file_name)
        .expect("Modifiable file should still be in UI after modification");
    let updated_entry = &harness.state().tab_manager.current_tab_ref().entries[updated_entry_index];

    assert_eq!(updated_entry.name, mod_file_name);
    assert_eq!(updated_entry.path, mod_file_path);
    assert_ne!(
        updated_entry.size, initial_entry.size,
        "File size should have changed in UI"
    );
    assert_eq!(
        updated_entry.size,
        content.len() as u64,
        "File size should match content length"
    );
    assert!(
        updated_entry.modified > initial_entry.modified,
        "Modified time should have increased"
    );
}

#[test]
fn test_external_file_removal() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    let rem_file_name = "removable_file.txt";
    let rem_file_path = temp_dir.path().join(rem_file_name);

    File::create(&rem_file_path).expect("Failed to create removable file");
    wait_and_step(&mut harness); // Ensure it's in the UI

    assert!(
        find_entry_index(&harness, rem_file_name).is_some(),
        "Removable file should be in UI before removal"
    );

    fs::remove_file(&rem_file_path).expect("Failed to remove external file");
    assert!(!rem_file_path.exists());

    wait_and_step(&mut harness);

    assert!(
        find_entry_index(&harness, rem_file_name).is_none(),
        "External file should disappear from the UI after removal"
    );
}

#[test]
fn test_external_directory_removal() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    let rem_dir_name = "removable_dir";
    let rem_dir_path = temp_dir.path().join(rem_dir_name);

    fs::create_dir(&rem_dir_path).expect("Failed to create removable directory");
    File::create(rem_dir_path.join("inner.txt")).unwrap();
    wait_and_step(&mut harness); // Ensure it's in the UI

    assert!(
        find_entry_index(&harness, rem_dir_name).is_some(),
        "Removable directory should be in UI before removal"
    );

    fs::remove_dir_all(&rem_dir_path).expect("Failed to remove external directory");
    assert!(!rem_dir_path.exists());

    wait_and_step(&mut harness);

    assert!(
        find_entry_index(&harness, rem_dir_name).is_none(),
        "External directory should disappear from the UI after removal"
    );
}
