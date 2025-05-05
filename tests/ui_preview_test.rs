mod ui_test_helpers;

use kiorg::models::preview_content::PreviewContent;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_image, create_test_zip};

/// Test for text preview of regular text files
#[test]
fn test_text_file_preview() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a text file with content
    let text_path = temp_dir.path().join("test.txt");
    std::fs::write(&text_path, "This is a test text file content").unwrap();

    // Start the harness
    let mut harness = create_harness(&temp_dir);
    harness.ensure_sorted_by_name_ascending();

    // Select the text file
    {
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        let text_index = tab
            .entries
            .iter()
            .position(|e| e.path.extension().unwrap_or_default() == "txt")
            .expect("Text file should be in the entries");
        tab.selected_index = text_index;
    }

    // Step to update the preview
    harness.step();

    // Check if the preview content is text and contains the expected content
    match &harness.state().preview_content {
        Some(PreviewContent::Text(text)) => {
            assert!(
                text.contains("This is a test text file content"),
                "Preview content should contain the text file content"
            );
        }
        Some(other) => {
            panic!("Preview content should be Text variant, got {:?}", other);
        }
        None => panic!("Preview content should not be None"),
    };
}

/// Test for text preview of binary files
#[test]
fn test_binary_file_preview() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a binary file (using a simple byte array)
    let binary_path = temp_dir.path().join("binary.bin");
    let binary_data = [0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD, 0xFC];
    std::fs::write(&binary_path, &binary_data).unwrap();

    // Start the harness
    let mut harness = create_harness(&temp_dir);
    harness.ensure_sorted_by_name_ascending();

    // Select the binary file
    {
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        let binary_index = tab
            .entries
            .iter()
            .position(|e| e.path.extension().unwrap_or_default() == "bin")
            .expect("Binary file should be in the entries");
        tab.selected_index = binary_index;
    }

    // Step to update the preview
    harness.step();

    // Check if the preview content is text and indicates it's a binary file
    match &harness.state().preview_content {
        Some(PreviewContent::Text(text)) => {
            assert!(
                text.contains("Binary file:") && text.contains("bytes"),
                "Preview content should indicate it's a binary file with size"
            );
        }
        Some(other) => {
            panic!("Preview content should be Text variant, got {:?}", other);
        }
        None => panic!("Preview content should not be None"),
    };
}

/// Test for directory preview
#[test]
fn test_directory_preview() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a subdirectory
    let dir_path = temp_dir.path().join("subdir");
    std::fs::create_dir(&dir_path).unwrap();

    // Start the harness
    let mut harness = create_harness(&temp_dir);
    harness.ensure_sorted_by_name_ascending();

    // Select the directory
    {
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        let dir_index = tab
            .entries
            .iter()
            .position(|e| e.is_dir && e.name == "subdir")
            .expect("Directory should be in the entries");
        tab.selected_index = dir_index;
    }

    // Step to update the preview
    harness.step();

    // Check if the preview content is text and indicates it's a directory
    match &harness.state().preview_content {
        Some(PreviewContent::Text(text)) => {
            assert!(
                text.contains("Directory:") && text.contains("subdir"),
                "Preview content should indicate it's a directory"
            );
        }
        Some(other) => {
            panic!("Preview content should be Text variant, got {:?}", other);
        }
        None => panic!("Preview content should not be None"),
    };
}

/// Test for image preview
#[test]
fn test_image_preview() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test image
    let image_path = temp_dir.path().join("test.png");
    create_test_image(&image_path);

    // Start the harness
    let mut harness = create_harness(&temp_dir);
    harness.ensure_sorted_by_name_ascending();

    // Select the image file
    {
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        let image_index = tab
            .entries
            .iter()
            .position(|e| e.path.extension().unwrap_or_default() == "png")
            .expect("Image file should be in the entries");
        tab.selected_index = image_index;
    }

    // Step to update the preview
    harness.step();

    // Check if the preview content is an image
    match &harness.state().preview_content {
        Some(PreviewContent::Image(uri)) => {
            assert!(
                uri.contains("file://") && uri.contains(".png"),
                "Image URI should contain file:// protocol and .png extension"
            );
        }
        Some(other) => {
            panic!("Preview content should be Image variant, got {:?}", other);
        }
        None => panic!("Preview content should not be None"),
    };
}

/// Test for zip preview
#[test]
fn test_zip_preview() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test zip file
    let zip_path = temp_dir.path().join("test.zip");
    create_test_zip(&zip_path);

    // Start the harness
    let mut harness = create_harness(&temp_dir);
    harness.ensure_sorted_by_name_ascending();

    // Select the zip file
    {
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        let zip_index = tab
            .entries
            .iter()
            .position(|e| e.path.extension().unwrap_or_default() == "zip")
            .expect("Zip file should be in the entries");
        tab.selected_index = zip_index;
    }

    // Step to update the preview
    harness.step();

    // Check if the preview content is a zip or loading
    let mut is_zip_content = false;

    // Try multiple steps to allow async loading to complete
    for _ in 0..20 {
        match &harness.state().preview_content {
            Some(PreviewContent::Zip(entries)) => {
                // Verify zip entries
                assert!(!entries.is_empty(), "Zip entries should not be empty");

                // Check for expected files
                let file1 = entries.iter().find(|e| e.name == "file1.txt");
                let file2 = entries.iter().find(|e| e.name == "file2.txt");
                let subdir = entries.iter().find(|e| e.name == "subdir/" && e.is_dir);

                assert!(file1.is_some(), "file1.txt should be in the zip entries");
                assert!(file2.is_some(), "file2.txt should be in the zip entries");
                assert!(subdir.is_some(), "subdir/ should be in the zip entries");

                is_zip_content = true;
                break;
            }
            Some(PreviewContent::Loading(..)) => {
                // Still loading, try another step
                harness.step();
            }
            Some(other) => {
                panic!(
                    "Preview content should be Zip or Loading variant, got {:?}",
                    other
                );
            }
            None => panic!("Preview content should not be None"),
        }
    }

    assert!(
        is_zip_content,
        "Preview content should eventually be Zip variant"
    );
}

/// Test for loading state preview
///
/// Note: This test is more of a verification that the Loading state exists
/// and works correctly in the code, rather than actually testing the UI
/// transition, which happens too quickly in tests to reliably capture.
#[test]
fn test_loading_preview() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test zip file (we'll use zip since it triggers async loading)
    let zip_path = temp_dir.path().join("test.zip");
    create_test_zip(&zip_path);

    // Start the harness
    let mut harness = create_harness(&temp_dir);
    harness.ensure_sorted_by_name_ascending();

    // Select the zip file
    {
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        let zip_index = tab
            .entries
            .iter()
            .position(|e| e.path.extension().unwrap_or_default() == "zip")
            .expect("Zip file should be in the entries");
        tab.selected_index = zip_index;
    }

    // Instead of testing the actual loading state (which happens too quickly in tests),
    // we'll verify that the zip content eventually loads correctly

    // Step to update the preview
    harness.step();

    // Check if the preview content is a zip
    let mut is_zip_content = false;
    for _ in 0..10 {
        match &harness.state().preview_content {
            Some(PreviewContent::Zip(entries)) => {
                // Verify zip entries
                assert!(!entries.is_empty(), "Zip entries should not be empty");
                is_zip_content = true;
                break;
            }
            _ => harness.step(),
        }
    }

    assert!(
        is_zip_content,
        "Preview should eventually load as Zip content"
    );

    // Verify that the Loading state exists in the PreviewContent enum
    // This is a compile-time check that the variant exists
    let _: Option<PreviewContent> = Some(PreviewContent::Loading(std::path::PathBuf::new(), None));
}

/// Test for no preview when no file is selected
#[test]
fn test_none_preview() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a test file
    let text_path = temp_dir.path().join("test.txt");
    std::fs::write(&text_path, "This is a test text file content").unwrap();

    // Start the harness with an empty directory (not the one with our file)
    // This ensures there are no files to select
    let empty_dir = tempdir().unwrap();
    let mut harness = create_harness(&empty_dir);

    // Step to update the preview
    harness.step();

    // Check if the preview content is None or a default "No file selected" message
    match &harness.state().preview_content {
        None => {
            // This is the expected case - no preview content
        }
        Some(PreviewContent::Text(text)) if text.contains("No file selected") => {
            // This is also acceptable - a text message indicating no selection
        }
        Some(other) => {
            panic!(
                "Preview content should be None or a 'No file selected' message, got {:?}",
                other
            );
        }
    }
}
