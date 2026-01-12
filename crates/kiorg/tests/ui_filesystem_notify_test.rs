#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use std::{
    fs::{self, File},
    io::Write,
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

    for _i in 0..max_iterations {
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

#[test]
fn test_external_file_addition() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    let file_name = "external_file.txt";
    assert!(
        find_entry_index(&harness, file_name).is_none(),
        "File should not exist in UI initially"
    );

    let file_path = temp_dir.path().join(file_name);
    File::create(&file_path).expect("Failed to create external file");
    assert!(file_path.exists());

    wait_for_condition(
        &mut harness,
        |h| find_entry_index(h, file_name).is_some(),
        "external file to appear in UI",
    );

    let file_index = find_entry_index(&harness, file_name)
        .expect("External file should appear in the UI after creation");
    let entry = &harness.state().tab_manager.current_tab_ref().entries[file_index];
    assert_eq!(entry.name, file_name);
    assert_eq!(entry.meta.path, file_path);
    assert!(!entry.is_dir);
}

#[test]
fn test_external_directory_addition() {
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    let dir_name = "external_dir";
    assert!(
        find_entry_index(&harness, dir_name).is_none(),
        "Directory should not exist in UI initially"
    );

    let dir_path = temp_dir.path().join(dir_name);
    fs::create_dir(&dir_path).expect("Failed to create external directory");
    assert!(dir_path.exists());

    wait_for_condition(
        &mut harness,
        |h| find_entry_index(h, dir_name).is_some(),
        "external directory to appear in UI",
    );

    let dir_index = find_entry_index(&harness, dir_name)
        .expect("External directory should appear in the UI after creation");
    let entry = &harness.state().tab_manager.current_tab_ref().entries[dir_index];
    assert_eq!(entry.name, dir_name);
    assert_eq!(entry.meta.path, dir_path);
    assert!(entry.is_dir);
}

#[test]
fn test_external_file_modification() {
    let temp_dir = tempdir().unwrap();

    let mod_file_name = "modifiable_file.txt";
    let mod_file_path = temp_dir.path().join(mod_file_name);

    File::create(&mod_file_path).expect("Failed to create modifiable file");
    let mut harness = create_harness(&temp_dir);

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

    wait_for_condition(
        &mut harness,
        |h| {
            if let Some(index) = find_entry_index(h, mod_file_name) {
                let entry = &h.state().tab_manager.current_tab_ref().entries[index];
                entry.size != initial_entry.size
            } else {
                false
            }
        },
        "file size to be updated in UI",
    );

    let updated_entry_index = find_entry_index(&harness, mod_file_name)
        .expect("Modifiable file should still be in UI after modification");
    let updated_entry = &harness.state().tab_manager.current_tab_ref().entries[updated_entry_index];

    assert_eq!(updated_entry.name, mod_file_name);
    assert_eq!(updated_entry.meta.path, mod_file_path);
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
        updated_entry.meta.modified > initial_entry.meta.modified,
        "File modification time should have increased: new {:?} > old {:?}",
        updated_entry.meta.modified,
        initial_entry.meta.modified
    );
}

#[test]
fn test_external_file_removal() {
    let temp_dir = tempdir().unwrap();

    let rem_file_name = "removable_file.txt";
    let rem_file_path = temp_dir.path().join(rem_file_name);
    File::create(&rem_file_path).expect("Failed to create removable file");
    let mut harness = create_harness(&temp_dir);

    assert!(
        find_entry_index(&harness, rem_file_name).is_some(),
        "Removable file should be in UI before removal"
    );

    fs::remove_file(&rem_file_path).expect("Failed to remove external file");
    assert!(!rem_file_path.exists());

    wait_for_condition(
        &mut harness,
        |h| find_entry_index(h, rem_file_name).is_none(),
        "external file to disappear from UI",
    );
    assert!(
        find_entry_index(&harness, rem_file_name).is_none(),
        "External file should disappear from the UI after removal"
    );
}

#[test]
fn test_external_directory_removal() {
    let temp_dir = tempdir().unwrap();

    let rem_dir_name = "removable_dir";
    let rem_dir_path = temp_dir.path().join(rem_dir_name);

    fs::create_dir(&rem_dir_path).expect("Failed to create removable directory");
    File::create(rem_dir_path.join("inner.txt")).unwrap();
    let mut harness = create_harness(&temp_dir);

    assert!(
        find_entry_index(&harness, rem_dir_name).is_some(),
        "Removable directory should be in UI before removal"
    );

    fs::remove_dir_all(&rem_dir_path).expect("Failed to remove external directory");
    assert!(!rem_dir_path.exists());

    wait_for_condition(
        &mut harness,
        |h| find_entry_index(h, rem_dir_name).is_none(),
        "external directory to disappear from UI",
    );

    assert!(
        find_entry_index(&harness, rem_dir_name).is_none(),
        "External directory should disappear from the UI after removal"
    );
}
