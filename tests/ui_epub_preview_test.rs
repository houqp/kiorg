#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use kiorg::models::preview_content::PreviewContent;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_epub};

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

    harness.press_key(egui::Key::J);
    harness.step();
    harness.press_key(egui::Key::K);
    harness.step();

    // Try multiple steps to allow async loading to complete
    for _ in 0..300 {
        match &harness.state().preview_content {
            Some(PreviewContent::Epub(epub_meta)) => {
                // Verify EPUB metadata
                assert!(
                    !epub_meta.metadata.is_empty(),
                    "EPUB metadata should not be empty"
                );

                // Check for expected metadata fields
                let creator = epub_meta
                    .metadata
                    .get("creator")
                    .or_else(|| epub_meta.metadata.get("dc:creator"));
                let language = epub_meta
                    .metadata
                    .get("language")
                    .or_else(|| epub_meta.metadata.get("dc:language"));

                // Title is now stored in the title field, not in metadata
                assert!(!epub_meta.title.is_empty(), "Title should not be empty");
                assert!(creator.is_some(), "Creator should be in the EPUB metadata");
                assert!(
                    language.is_some(),
                    "Language should be in the EPUB metadata"
                );

                // Check specific values
                assert!(
                    epub_meta.title.contains("Test EPUB Book"),
                    "Title should contain 'Test EPUB Book'"
                );

                if let Some(creator_value) = creator {
                    assert!(
                        creator_value.contains("Test Author"),
                        "Creator should contain 'Test Author'"
                    );
                }

                return;
            }
            Some(_) => {
                std::thread::sleep(std::time::Duration::from_millis(10));
                harness.step();
            }
            None => panic!("Preview content should not be None"),
        }
    }

    panic!(
        "Preview content should eventually be EPUB variant, got: {:?}",
        harness.state().preview_content
    );
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
    for _ in 0..50 {
        // Max iterations but exit early when loaded
        std::thread::sleep(std::time::Duration::from_millis(10));
        harness.step();

        // Check if EPUB preview content is loaded
        if let Some(PreviewContent::Epub(epub_meta)) = &harness.state().preview_content {
            // Verify page count is accessible for right panel display
            assert!(
                epub_meta.page_count > 0,
                "EPUB page count should be available and greater than 0"
            );

            // Verify standard EPUB metadata is present
            assert!(
                !epub_meta.title.is_empty(),
                "EPUB should have a non-empty title"
            );
            assert!(!epub_meta.metadata.is_empty(), "EPUB should have metadata");

            // Check for expected metadata fields from the test EPUB
            assert!(
                epub_meta.metadata.contains_key("creator")
                    || epub_meta.metadata.contains_key("Creator")
                    || epub_meta.metadata.contains_key("author")
                    || epub_meta.metadata.contains_key("Author"),
                "EPUB should contain author/creator metadata"
            );

            epub_loaded = true;
            break;
        }
    }

    assert!(
        epub_loaded,
        "EPUB should have loaded within the timeout period"
    );
}
