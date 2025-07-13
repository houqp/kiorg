#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use kiorg::ui::popup::PopupType;
use tempfile::tempdir;
use ui_test_helpers::create_harness;

#[test]
fn test_teleport_nonexistent_directory_shows_error_and_removes_from_history() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let mut harness = create_harness(&temp_dir);

    // Create a directory that we'll add to history and then delete
    let test_dir = temp_dir.path().join("test_directory");
    std::fs::create_dir(&test_dir).unwrap();

    // Navigate to the test directory to add it to the visit history
    harness.state_mut().navigate_to_dir(test_dir.clone());
    harness.step();

    // Navigate back to the parent directory
    harness
        .state_mut()
        .navigate_to_dir(temp_dir.path().to_path_buf());
    harness.step();

    // Verify the test directory is in the visit history
    assert!(
        harness.state().visit_history.contains_key(&test_dir),
        "Test directory should be in visit history"
    );

    // Now delete the directory to simulate it being removed externally
    std::fs::remove_dir(&test_dir).unwrap();
    assert!(!test_dir.exists(), "Test directory should be deleted");

    // Try to navigate to the deleted directory
    harness.state_mut().navigate_to_dir(test_dir.clone());
    harness.step();

    // Check that we stayed in the original directory (navigation should have failed)
    let current_path = harness
        .state()
        .tab_manager
        .current_tab_ref()
        .current_path
        .clone();
    assert_eq!(
        current_path,
        temp_dir.path(),
        "Should remain in original directory when navigation fails"
    );

    // Verify that the non-existent directory was removed from visit history
    assert!(
        !harness.state().visit_history.contains_key(&test_dir),
        "Non-existent directory should be removed from visit history"
    );

    // Test teleport popup behavior with the cleaned up history
    harness.state_mut().show_popup = Some(PopupType::Teleport(
        kiorg::ui::popup::teleport::TeleportState::default(),
    ));
    harness.step();

    // Verify the teleport popup is open
    assert!(
        matches!(harness.state().show_popup, Some(PopupType::Teleport(_))),
        "Teleport popup should be open"
    );

    // The search results should not include the deleted directory since it was removed
    // from the visit history and get_search_results filters out non-existent paths
    let search_results =
        kiorg::ui::popup::teleport::get_search_results("", &harness.state().visit_history);

    // The deleted directory should not appear in search results
    let contains_deleted_dir = search_results.iter().any(|result| result.path == test_dir);
    assert!(
        !contains_deleted_dir,
        "Deleted directory should not appear in teleport search results"
    );
}
