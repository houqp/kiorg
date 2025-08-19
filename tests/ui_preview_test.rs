#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use kiorg::models::preview_content::PreviewContent;
use tempfile::tempdir;
use ui_test_helpers::{
    
    create_harness, create_test_image, create_test_tar, create_test_video, create_test_zip, wait_for_condition,
,
};

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

    // Select the text file using J shortcut
    // Since entries are sorted by name, we can navigate to the text file
    harness.key_press(Key::J);
    harness.step();

    wait_for_condition(|| {
        match harness.state().preview_content.as_ref() {
            Some(PreviewContent::Text(_)) | Some(PreviewContent::HighlightedCode { .. }) => true, // Text preview loaded
            _ => {
                harness.step(); // Continue stepping until the text preview loads
                false
            }
        }
    });

    // Check if the preview content is text and contains the expected content
    match &harness.state().preview_content {
        Some(PreviewContent::Text(text)) => {
            assert!(
                text.contains("This is a test text file content"),
                "Preview content should contain the text file content"
            );
        }
        Some(PreviewContent::HighlightedCode {
            content,
            language: _,
        }) => {
            assert!(
                content.contains("This is a test text file content"),
                "Preview content should contain the text file content"
            );
        }
        Some(other) => {
            panic!("Preview content should be Text or HighlightedCode variant, got {other:?}");
        }
        None => panic!("Preview content should not be None"),
    }
}

/// Test for text preview of binary files
#[test]
fn test_binary_file_preview() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a binary file (using a simple byte array)
    let binary_path = temp_dir.path().join("binary.bin");
    let binary_data = [0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD, 0xFC];
    std::fs::write(&binary_path, binary_data).unwrap();

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Select the binary file using J shortcut
    // Since entries are sorted by name, we can navigate to the binary file
    harness.key_press(Key::J);
    harness.step();

    wait_for_condition(|| {
        harness.step();
        matches!(
            harness.state().preview_content.as_ref(),
            Some(PreviewContent::Text(_))
        )
    });

    // Check if the preview content is text and indicates it's a binary file
    match &harness.state().preview_content {
        Some(PreviewContent::Text(text)) => {
            assert!(
                text.contains("File type:") && text.contains("bytes"),
                "Preview content should indicate it's a binary file with size"
            );
        }
        Some(other) => {
            panic!("Preview content should be Text variant, got {other:?}");
        }
        None => panic!("Preview content should not be None"),
    }
}

/// Test for directory preview
#[test]
fn test_directory_preview() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a subdirectory
    let dir_path = temp_dir.path().join("subdir");
    std::fs::create_dir(&dir_path).unwrap();

    let binary_path = dir_path.join("binary.bin");
    let binary_data = [0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD, 0xFC];
    std::fs::write(&binary_path, binary_data).unwrap();

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Select the directory using J shortcut
    // Since entries are sorted by name, we can navigate to the directory
    harness.key_press(Key::J);
    harness.step();

    wait_for_condition(|| {
        harness.step();
        matches!(
            harness.state().preview_content.as_ref(),
            Some(PreviewContent::Directory(_))
        )
    });

    // Check if the preview content is text and indicates it's a directory
    match &harness.state().preview_content {
        Some(PreviewContent::Directory(dirs)) => {
            assert!(
                dirs.iter().any(|d| d.name == "binary.bin" && !d.is_dir),
                "Preview content should show directory entries in preview"
            );
        }
        Some(other) => {
            panic!("Preview content should be Directory variant, got {other:?}");
        }
        None => panic!("Preview content should not be None"),
    }
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

    // Select the image file using J shortcut
    // Since entries are sorted by name, we can navigate to the image file
    harness.key_press(Key::J);
    harness.step();

    wait_for_condition(|| {
        harness.step();
        matches!(
            harness.state().preview_content.as_ref(),
            Some(PreviewContent::Image(_))
        )
    });

    // Check if the preview content is an image
    match &harness.state().preview_content {
        Some(PreviewContent::Image(image_meta)) => {
            // Verify that metadata is present
            assert!(
                !image_meta.metadata.is_empty(),
                "Image metadata should not be empty"
            );
        }
        Some(other) => {
            panic!("Preview content should be Image variant, got {other:?}");
        }
        None => panic!("Preview content should not be None"),
    }
}

/// Test for video preview with metadata extraction
#[test]
fn test_video_file_preview() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test video file
    let video_path = temp_dir.path().join("test_video.mp4");
    create_test_video(&video_path);

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Navigate to the video file
    harness.key_press(Key::J);
    harness.step();

    // Wait for video preview to load
    for _ in 0..100 {
        match harness.state().preview_content.as_ref() {
            Some(PreviewContent::Video(_)) => break, // Video preview loaded
            _ => {
                std::thread::sleep(std::time::Duration::from_millis(10));
                harness.step(); // Continue stepping until the video preview loads
            }
        }
    }

    // Check if the preview content is video
    match &harness.state().preview_content {
        Some(PreviewContent::Video(video_meta)) => {
            // Check that basic metadata is present
            assert!(
                !video_meta.title.is_empty(),
                "Video title should not be empty"
            );
            assert!(
                video_meta.title.contains("test_video.mp4"),
                "Video title should contain filename"
            );

            // Check for expected metadata fields
            assert!(
                video_meta.metadata.contains_key("File Size"),
                "Should have file size metadata"
            );
            assert!(
                video_meta.metadata.contains_key("File Type"),
                "Should have file type metadata"
            );

            // Verify file type is correct
            let file_type = video_meta.metadata.get("File Type").unwrap();
            assert_eq!(file_type, "MP4", "File type should be MP4");
        }
        Some(other) => {
            panic!("Preview content should be Video variant, got {other:?}");
        }
        None => panic!("Preview content should not be None"),
    }
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

    // Select the zip file using J shortcut
    // Since entries are sorted by name, we can navigate to the zip file
    harness.key_press(Key::J);
    harness.step();

    // Try multiple steps to allow async loading to complete
    wait_for_condition(|| {
        harness.step();
        if let Some(PreviewContent::Zip(entries)) = &harness.state().preview_content {
            // Verify zip entries
            if !entries.is_empty() {
                // Check for expected files
                let file1 = entries.iter().find(|e| e.name == "file1.txt");
                let file2 = entries.iter().find(|e| e.name == "file2.txt");
                let subdir = entries.iter().find(|e| e.name == "subdir/" && e.is_dir);

                return file1.is_some() && file2.is_some() && subdir.is_some();
            }
        }
        false
    });

    // Final verification
    if let Some(PreviewContent::Zip(entries)) = &harness.state().preview_content {
        assert!(!entries.is_empty(), "Zip entries should not be empty");

        let file1 = entries.iter().find(|e| e.name == "file1.txt");
        let file2 = entries.iter().find(|e| e.name == "file2.txt");
        let subdir = entries.iter().find(|e| e.name == "subdir/" && e.is_dir);

        assert!(file1.is_some(), "file1.txt should be in the zip entries");
        assert!(file2.is_some(), "file2.txt should be in the zip entries");
        assert!(subdir.is_some(), "subdir/ should be in the zip entries");
    } else {
        panic!(
            "Preview content should eventually be Zip variant, got {:?}",
            harness.state().preview_content
        );
    }
}

/// Test for tar preview
#[test]
fn test_tar_preview() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test tar file
    let tar_path = temp_dir.path().join("test.tar");
    create_test_tar(&tar_path);

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Select the tar file using J shortcut
    // Since entries are sorted by name, we can navigate to the tar file
    harness.key_press(Key::J);
    harness.step();

    // Try multiple steps to allow async loading to complete
    wait_for_condition(|| {
        harness.step();
        if let Some(PreviewContent::Tar(entries)) = &harness.state().preview_content {
            // Verify tar entries
            if !entries.is_empty() {
                // Check for expected files
                let file1 = entries.iter().find(|e| e.name == "file1.txt");
                let file2 = entries.iter().find(|e| e.name == "file2.txt");
                let subdir = entries.iter().find(|e| e.name == "subdir/" && e.is_dir);

                return file1.is_some() && file2.is_some() && subdir.is_some();
            }
        }
        false
    });

    // Final verification
    if let Some(PreviewContent::Tar(entries)) = &harness.state().preview_content {
        assert!(!entries.is_empty(), "Tar entries should not be empty");

        let file1 = entries.iter().find(|e| e.name == "file1.txt");
        let file2 = entries.iter().find(|e| e.name == "file2.txt");
        let subdir = entries.iter().find(|e| e.name == "subdir/" && e.is_dir);

        assert!(file1.is_some(), "file1.txt should be in the tar entries");
        assert!(file2.is_some(), "file2.txt should be in the tar entries");
        assert!(subdir.is_some(), "subdir/ should be in the tar entries");
    } else {
        panic!(
            "Preview content should eventually be Tar variant, got {:?}",
            harness.state().preview_content
        );
    }
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
            panic!("Preview content should be None or a 'No file selected' message, got {other:?}");
        }
    }
}
