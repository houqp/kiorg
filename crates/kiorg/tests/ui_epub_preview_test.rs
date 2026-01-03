#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use kiorg::models::preview_content::PreviewContent;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_epub, wait_for_condition};

/// Test for EPUB preview
#[test]
fn test_epub_preview() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create test files including an EPUB file
    let epub_path = temp_dir.path().join("test.epub");
    create_test_epub(&epub_path);

    // Create a text file for comparison
    let text_path = temp_dir.path().join("test.txt");
    std::fs::write(&text_path, "This is a text file").unwrap();

    let mut harness = create_harness(&temp_dir);

    harness.key_press(egui::Key::J);
    harness.step();
    harness.key_press(egui::Key::K);
    harness.step();

    // Try multiple steps to allow async loading to complete
    wait_for_condition(|| match &harness.state().preview_content {
        Some(PreviewContent::Ebook(_)) => true,
        _ => {
            harness.step();
            false
        }
    });

    match &harness.state().preview_content {
        Some(PreviewContent::Ebook(ebook_meta)) => {
            // Verify ebook metadata
            assert!(
                !ebook_meta.metadata.is_empty(),
                "Ebook metadata should not be empty"
            );

            // Check for expected metadata fields
            let creator = ebook_meta
                .metadata
                .get("creator")
                .or_else(|| ebook_meta.metadata.get("dc:creator"));
            let language = ebook_meta
                .metadata
                .get("language")
                .or_else(|| ebook_meta.metadata.get("dc:language"));

            // Title is now stored in the title field, not in metadata
            assert!(!ebook_meta.title.is_empty(), "Title should not be empty");
            assert!(creator.is_some(), "Creator should be in the ebook metadata");
            assert!(
                language.is_some(),
                "Language should be in the ebook metadata"
            );

            // Check specific values
            assert_eq!(ebook_meta.title, "Demo EPUB Book");

            assert!(
                ebook_meta.page_count > 0,
                "Ebook page count should be available and greater than 0"
            );

            if let Some(creator_value) = creator {
                assert!(
                    creator_value.contains("Test Author"),
                    "Creator should contain 'Test Author'"
                );
            }
        }
        Some(_) => {
            panic!("Preview content should be EPUB, but got something else");
        }
        None => panic!("Preview content should not be None"),
    }
}

/// Test that EPUB metadata contains page count for right panel display
#[test]
fn test_epub_page_count_metadata_available() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a test EPUB
    let epub_path = temp_dir.path().join("test_metadata.epub");
    create_test_epub(&epub_path);

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Select the EPUB file
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let epub_index = tab
            .entries
            .iter()
            .position(|e| e.name == "test_metadata.epub")
            .expect("EPUB file should be in the entries");
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = epub_index;
    }

    // Step to update the preview
    harness.step();

    // Wait for EPUB processing in a loop, checking for preview content
    let mut epub_loaded = false;
    wait_for_condition(|| {
        harness.step();

        // Check if ebook preview content is loaded
        if let Some(PreviewContent::Ebook(ebook_meta)) = &harness.state().preview_content {
            // Verify page count is accessible for right panel display
            assert!(
                ebook_meta.page_count > 0,
                "Ebook page count should be available and greater than 0"
            );

            // Verify standard ebook metadata is present
            assert!(
                !ebook_meta.title.is_empty(),
                "Ebook should have a non-empty title"
            );
            assert!(
                !ebook_meta.metadata.is_empty(),
                "Ebook should have metadata"
            );

            // Check for expected metadata fields from the test ebook
            assert!(
                ebook_meta.metadata.contains_key("creator")
                    || ebook_meta.metadata.contains_key("Creator")
                    || ebook_meta.metadata.contains_key("author")
                    || ebook_meta.metadata.contains_key("Author"),
                "Ebook should contain author/creator metadata"
            );

            epub_loaded = true;
            true
        } else {
            false
        }
    });

    assert!(
        epub_loaded,
        "EPUB should have loaded within the timeout period"
    );
}
