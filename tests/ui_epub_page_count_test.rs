#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use kiorg::models::preview_content::PreviewContent;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_epub};

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
