use kiorg::app::Kiorg;
use kiorg::visit_history::{
    VisitHistoryEntry, load_visit_history, save_visit_history, update_visit_history,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::tempdir;

#[test]
fn test_load_visit_history_from_empty_directory() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();

    let history = load_visit_history(Some(&config_dir)).unwrap();
    assert!(history.is_empty());
}

#[test]
fn test_load_visit_history_with_valid_csv() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();
    fs::create_dir_all(&config_dir).unwrap();

    let csv_content = r#"path,accessed_ts,count
/home/user/documents,1640995200,5
/home/user/downloads,1640995300,2
/var/log,1640995400,1
"#;

    let history_file_path = config_dir.join("history.csv");
    fs::write(&history_file_path, csv_content).unwrap();

    let history = load_visit_history(Some(&config_dir)).unwrap();

    assert_eq!(history.len(), 3);

    let documents_entry = history.get(&PathBuf::from("/home/user/documents"));
    assert!(documents_entry.is_some());
    let entry = documents_entry.unwrap();
    assert_eq!(entry.path, PathBuf::from("/home/user/documents"));
    assert_eq!(entry.accessed_ts, 1640995200);
    assert_eq!(entry.count, 5);

    let downloads_entry = history.get(&PathBuf::from("/home/user/downloads"));
    assert!(downloads_entry.is_some());
    let entry = downloads_entry.unwrap();
    assert_eq!(entry.path, PathBuf::from("/home/user/downloads"));
    assert_eq!(entry.accessed_ts, 1640995300);
    assert_eq!(entry.count, 2);

    let log_entry = history.get(&PathBuf::from("/var/log"));
    assert!(log_entry.is_some());
    let entry = log_entry.unwrap();
    assert_eq!(entry.path, PathBuf::from("/var/log"));
    assert_eq!(entry.accessed_ts, 1640995400);
    assert_eq!(entry.count, 1);
}

#[test]
fn test_load_visit_history_with_malformed_csv() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();
    fs::create_dir_all(&config_dir).unwrap();

    // Test invalid field count (less than 3 fields)
    let csv_content = r#"path,accessed_ts,count
/home/user/documents,1640995200,5
invalid_line_with_only_two,fields
"#;

    let history_file_path = config_dir.join("history.csv");
    fs::write(&history_file_path, csv_content).unwrap();

    let result = load_visit_history(Some(&config_dir));
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid CSV format at line 2"));
    assert!(error_msg.contains("expected at least 3 fields, found 2"));
}

#[test]
fn test_load_visit_history_with_invalid_timestamp() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();
    fs::create_dir_all(&config_dir).unwrap();

    let csv_content = r#"path,accessed_ts,count
/home/user/documents,not_a_number,5
"#;

    let history_file_path = config_dir.join("history.csv");
    fs::write(&history_file_path, csv_content).unwrap();

    let result = load_visit_history(Some(&config_dir));
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid timestamp at line 1"));
    assert!(error_msg.contains("not_a_number"));
}

#[test]
fn test_load_visit_history_with_invalid_count() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();
    fs::create_dir_all(&config_dir).unwrap();

    let csv_content = r#"path,accessed_ts,count
/home/user/documents,1640995200,not_a_number
"#;

    let history_file_path = config_dir.join("history.csv");
    fs::write(&history_file_path, csv_content).unwrap();

    let result = load_visit_history(Some(&config_dir));
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid count at line 1"));
    assert!(error_msg.contains("not_a_number"));
}

#[test]
fn test_load_visit_history_with_empty_csv() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();
    fs::create_dir_all(&config_dir).unwrap();

    let csv_content = "path,accessed_ts,count\n";
    let history_file_path = config_dir.join("history.csv");
    fs::write(&history_file_path, csv_content).unwrap();

    let history = load_visit_history(Some(&config_dir)).unwrap();
    assert!(history.is_empty());
}

#[test]
fn test_load_visit_history_with_empty_lines() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();
    fs::create_dir_all(&config_dir).unwrap();

    let csv_content = r#"path,accessed_ts,count
/home/user/documents,1640995200,5

/home/user/downloads,1640995300,2
"#;

    let history_file_path = config_dir.join("history.csv");
    fs::write(&history_file_path, csv_content).unwrap();

    let history = load_visit_history(Some(&config_dir)).unwrap();
    assert_eq!(history.len(), 2);
    assert!(history.contains_key(&PathBuf::from("/home/user/documents")));
    assert!(history.contains_key(&PathBuf::from("/home/user/downloads")));
}

#[test]
fn test_save_visit_history_creates_directory() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join("nonexistent");

    let mut history = HashMap::new();
    history.insert(
        PathBuf::from("/test/path"),
        VisitHistoryEntry {
            path: PathBuf::from("/test/path"),
            accessed_ts: 1640995200,
            count: 1,
        },
    );

    let result = save_visit_history(&history, Some(&config_dir));
    assert!(result.is_ok());
    assert!(config_dir.exists());
    assert!(config_dir.join("history.csv").exists());
}

#[test]
fn test_save_visit_history_writes_correct_format() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();

    let mut history = HashMap::new();
    history.insert(
        PathBuf::from("/home/user/documents"),
        VisitHistoryEntry {
            path: PathBuf::from("/home/user/documents"),
            accessed_ts: 1640995200,
            count: 5,
        },
    );
    history.insert(
        PathBuf::from("/home/user/downloads"),
        VisitHistoryEntry {
            path: PathBuf::from("/home/user/downloads"),
            accessed_ts: 1640995300,
            count: 2,
        },
    );

    let result = save_visit_history(&history, Some(&config_dir));
    assert!(result.is_ok());

    let history_file_path = config_dir.join("history.csv");
    let content = fs::read_to_string(history_file_path).unwrap();

    assert!(content.starts_with("path,accessed_ts,count\n"));
    assert!(content.contains("/home/user/documents,1640995200,5"));
    assert!(content.contains("/home/user/downloads,1640995300,2"));
}

#[test]
fn test_save_empty_visit_history() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();

    let history = HashMap::new();
    let result = save_visit_history(&history, Some(&config_dir));
    assert!(result.is_ok());

    let history_file_path = config_dir.join("history.csv");
    let content = fs::read_to_string(history_file_path).unwrap();
    assert_eq!(content, "path,accessed_ts,count\n");
}

#[test]
fn test_update_visit_history_new_entry() {
    let mut history = HashMap::new();
    let test_path = PathBuf::from("/test/new/path");

    let before_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    update_visit_history(&mut history, &test_path);

    let after_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    assert_eq!(history.len(), 1);
    let entry = history.get(&test_path).unwrap();
    assert_eq!(entry.path, test_path);
    assert_eq!(entry.count, 1);
    assert!(entry.accessed_ts >= before_time);
    assert!(entry.accessed_ts <= after_time);
}

#[test]
fn test_update_visit_history_existing_entry() {
    let mut history = HashMap::new();
    let test_path = PathBuf::from("/test/existing/path");

    // Insert initial entry
    history.insert(
        test_path.clone(),
        VisitHistoryEntry {
            path: test_path.clone(),
            accessed_ts: 1640995200,
            count: 3,
        },
    );

    let before_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    update_visit_history(&mut history, &test_path);

    let after_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    assert_eq!(history.len(), 1);
    let entry = history.get(&test_path).unwrap();
    assert_eq!(entry.path, test_path);
    assert_eq!(entry.count, 4); // Incremented
    assert!(entry.accessed_ts >= before_time); // Updated timestamp
    assert!(entry.accessed_ts <= after_time);
    assert!(entry.accessed_ts > 1640995200); // Timestamp changed
}

#[test]
fn test_update_visit_history_multiple_updates() {
    let mut history = HashMap::new();
    let test_path = PathBuf::from("/test/multi/path");

    // Update multiple times
    update_visit_history(&mut history, &test_path);
    update_visit_history(&mut history, &test_path);
    update_visit_history(&mut history, &test_path);

    assert_eq!(history.len(), 1);
    let entry = history.get(&test_path).unwrap();
    assert_eq!(entry.count, 3);
}

#[test]
fn test_roundtrip_save_and_load() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();

    // Create original history
    let mut original_history = HashMap::new();
    original_history.insert(
        PathBuf::from("/home/user/documents"),
        VisitHistoryEntry {
            path: PathBuf::from("/home/user/documents"),
            accessed_ts: 1640995200,
            count: 5,
        },
    );
    original_history.insert(
        PathBuf::from("/home/user/downloads"),
        VisitHistoryEntry {
            path: PathBuf::from("/home/user/downloads"),
            accessed_ts: 1640995300,
            count: 2,
        },
    );

    // Save to file
    let save_result = save_visit_history(&original_history, Some(&config_dir));
    assert!(save_result.is_ok());

    // Load from file
    let loaded_history = load_visit_history(Some(&config_dir)).unwrap();

    // Compare
    assert_eq!(loaded_history.len(), original_history.len());
    for (path, original_entry) in &original_history {
        let loaded_entry = loaded_history.get(path).unwrap();
        assert_eq!(loaded_entry.path, original_entry.path);
        assert_eq!(loaded_entry.accessed_ts, original_entry.accessed_ts);
        assert_eq!(loaded_entry.count, original_entry.count);
    }
}

#[test]
fn test_visit_history_entry_serialization() {
    let entry = VisitHistoryEntry {
        path: PathBuf::from("/test/path"),
        accessed_ts: 1640995200,
        count: 42,
    };

    // Test serialization
    let serialized = serde_json::to_string(&entry).unwrap();
    assert!(serialized.contains("\"/test/path\""));
    assert!(serialized.contains("1640995200"));
    assert!(serialized.contains("42"));

    // Test deserialization
    let deserialized: VisitHistoryEntry = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.path, entry.path);
    assert_eq!(deserialized.accessed_ts, entry.accessed_ts);
    assert_eq!(deserialized.count, entry.count);
}

#[test]
fn test_paths_with_special_characters() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();

    let mut history = HashMap::new();

    // Test paths with spaces, unicode, and commas (now supported)
    let paths = vec![
        PathBuf::from("/path with spaces"),
        PathBuf::from("/path/with/unicode/文档"),
        PathBuf::from("/path_with_underscores"),
        PathBuf::from("/path-with-dashes"),
        PathBuf::from("/path,with,commas"),
    ];

    for (i, path) in paths.iter().enumerate() {
        history.insert(
            path.clone(),
            VisitHistoryEntry {
                path: path.clone(),
                accessed_ts: 1640995200 + i as u64,
                count: i as u64 + 1,
            },
        );
    }

    // Save and load
    let save_result = save_visit_history(&history, Some(&config_dir));
    assert!(save_result.is_ok());

    let loaded_history = load_visit_history(Some(&config_dir)).unwrap();

    // Verify all entries are preserved
    assert_eq!(loaded_history.len(), paths.len());
    for path in &paths {
        assert!(loaded_history.contains_key(path));
    }
}

#[test]
fn test_paths_with_csv_problematic_characters() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();

    let mut history = HashMap::new();

    // Test that paths with commas can now be handled correctly
    let problematic_path = PathBuf::from("/path,with,commas");
    history.insert(
        problematic_path.clone(),
        VisitHistoryEntry {
            path: problematic_path.clone(),
            accessed_ts: 1640995200,
            count: 1,
        },
    );

    // Save should succeed
    let save_result = save_visit_history(&history, Some(&config_dir));
    assert!(save_result.is_ok());

    // Load should now succeed and correctly parse paths with commas
    let loaded_history = load_visit_history(Some(&config_dir)).unwrap();

    // The entry with commas should be loaded correctly
    assert_eq!(loaded_history.len(), 1);
    assert!(loaded_history.contains_key(&problematic_path));

    let loaded_entry = loaded_history.get(&problematic_path).unwrap();
    assert_eq!(loaded_entry.path, problematic_path);
    assert_eq!(loaded_entry.accessed_ts, 1640995200);
    assert_eq!(loaded_entry.count, 1);
}

#[test]
fn test_load_visit_history_handles_read_error() {
    // Test with a directory that doesn't exist and cannot be read
    let nonexistent_dir = PathBuf::from("/this/path/should/not/exist/12345");

    let history = load_visit_history(Some(&nonexistent_dir)).unwrap();

    // Should return empty history without panicking when file doesn't exist
    assert!(history.is_empty());
}

#[test]
fn test_load_visit_history_handles_file_read_error() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();
    fs::create_dir_all(&config_dir).unwrap();

    // Create a directory with the same name as the history file to cause a read error
    let history_file_path = config_dir.join("history.csv");
    fs::create_dir(&history_file_path).unwrap();

    let result = load_visit_history(Some(&config_dir));

    // Should return an error when trying to read a directory as a file
    assert!(result.is_err());
}

#[test]
fn test_paths_with_multiple_commas() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();

    let mut history = HashMap::new();

    // Test paths with different comma scenarios
    let paths = vec![
        ("/path,with,multiple,commas", 1640995200, 5),
        ("/single,comma", 1640995300, 2),
        ("/path,with,commas,in,dir,names/file", 1640995400, 1),
        ("/no/commas/here", 1640995500, 3),
        ("/path,with,trailing,comma,", 1640995600, 7),
    ];

    for (path_str, ts, count) in &paths {
        let path = PathBuf::from(*path_str);
        history.insert(
            path.clone(),
            VisitHistoryEntry {
                path: path.clone(),
                accessed_ts: *ts,
                count: *count,
            },
        );
    }

    // Save and load
    let save_result = save_visit_history(&history, Some(&config_dir));
    assert!(save_result.is_ok());

    let loaded_history = load_visit_history(Some(&config_dir)).unwrap();

    // Verify all entries are preserved correctly
    assert_eq!(loaded_history.len(), paths.len());

    for (path_str, expected_ts, expected_count) in &paths {
        let path = PathBuf::from(*path_str);
        let loaded_entry = loaded_history.get(&path).unwrap();
        assert_eq!(loaded_entry.path, path);
        assert_eq!(loaded_entry.accessed_ts, *expected_ts);
        assert_eq!(loaded_entry.count, *expected_count);
    }
}

#[test]
fn test_edge_cases_csv_parsing() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();
    fs::create_dir_all(&config_dir).unwrap();

    // Test edge case: line with only one field (should fail)
    let csv_content_single = r#"path,accessed_ts,count
single_field
"#;

    let history_file_path = config_dir.join("history.csv");
    fs::write(&history_file_path, csv_content_single).unwrap();

    let result = load_visit_history(Some(&config_dir));
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("expected at least 3 fields, found 1"));

    // Test edge case: line with exactly 3 fields (should work)
    let csv_content_exact = r#"path,accessed_ts,count
/simple/path,1640995200,5
"#;

    fs::write(&history_file_path, csv_content_exact).unwrap();
    let result = load_visit_history(Some(&config_dir));
    assert!(result.is_ok());
    let history = result.unwrap();
    assert_eq!(history.len(), 1);
    assert!(history.contains_key(&PathBuf::from("/simple/path")));

    // Test edge case: line with more than 3 fields due to commas in path (should work)
    let csv_content_extra = r#"path,accessed_ts,count
/path,with,many,commas,in,it,1640995200,5
"#;

    fs::write(&history_file_path, csv_content_extra).unwrap();
    let result = load_visit_history(Some(&config_dir));
    assert!(result.is_ok());
    let history = result.unwrap();
    assert_eq!(history.len(), 1);
    assert!(history.contains_key(&PathBuf::from("/path,with,many,commas,in,it")));

    let entry = history
        .get(&PathBuf::from("/path,with,many,commas,in,it"))
        .unwrap();
    assert_eq!(entry.accessed_ts, 1640995200);
    assert_eq!(entry.count, 5);
}

#[test]
fn test_navigate_to_nonexistent_directory_removes_from_history() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();

    // Create a temporary directory that we'll delete later
    let existing_dir = temp_dir.path().join("existing_dir");
    fs::create_dir_all(&existing_dir).unwrap();

    // Create initial visit history with the existing directory
    let mut initial_history = HashMap::new();
    initial_history.insert(
        existing_dir.clone(),
        VisitHistoryEntry {
            path: existing_dir.clone(),
            accessed_ts: 1640995200,
            count: 5,
        },
    );

    // Add another directory that will remain valid
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    initial_history.insert(
        home_dir.clone(),
        VisitHistoryEntry {
            path: home_dir.clone(),
            accessed_ts: 1640995100,
            count: 3,
        },
    );

    // Save the initial history
    save_visit_history(&initial_history, Some(&config_dir)).unwrap();

    // Create the app with the config directory
    let ctx = egui::Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx);
    let mut app =
        Kiorg::new_with_config_dir(&cc, Some(home_dir.clone()), Some(config_dir.clone())).unwrap();

    // Verify the history was loaded correctly
    assert_eq!(app.visit_history.len(), 2);
    assert!(app.visit_history.contains_key(&existing_dir));
    assert!(app.visit_history.contains_key(&home_dir));

    // Delete the directory to simulate it becoming non-existent
    fs::remove_dir_all(&existing_dir).unwrap();
    assert!(!existing_dir.exists());

    // Try to navigate to the non-existent directory
    app.navigate_to_dir(existing_dir.clone());

    // Verify the non-existent directory was removed from visit history
    assert_eq!(app.visit_history.len(), 1);
    assert!(!app.visit_history.contains_key(&existing_dir));
    assert!(app.visit_history.contains_key(&home_dir));

    // Verify the app is still in the original directory (navigation failed)
    assert_eq!(app.tab_manager.current_tab_ref().current_path, home_dir);

    // Verify that the error notification was triggered
    // Note: In a real UI test, we would check if toasts contain the error message
    // For now, we just verify the navigation didn't succeed
}
