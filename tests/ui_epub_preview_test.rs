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

    let _test_files = vec![epub_path, text_path]; // Keep references to prevent cleanup

    let mut harness = create_harness(&temp_dir);

    // Select the EPUB file
    {
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        // Find the index of the EPUB file
        let epub_index = tab
            .entries
            .iter()
            .position(|e| e.path.extension().unwrap_or_default() == "epub")
            .expect("EPUB file should be in the entries");
        tab.selected_index = epub_index;
    }

    // Step to update the preview
    harness.step();

    // Check if the preview content is an EPUB or loading
    let mut is_epub_content = false;

    // Try multiple steps to allow async loading to complete
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(10));
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

                // Cover image is optional, so we don't assert on it
                // Just check that the test runs without errors

                is_epub_content = true;
                break;
            }
            Some(PreviewContent::Pdf(_)) => {
                panic!("Expected EPUB document but got PDF");
            }
            Some(PreviewContent::Loading(..)) => {
                // Still loading, try another step
                harness.step();
            }
            Some(other) => {
                panic!(
                    "Preview content should be EPUB or Loading variant, got {:?}",
                    other
                );
            }
            None => panic!("Preview content should not be None"),
        }
    }

    assert!(
        is_epub_content,
        "Preview content should eventually be EPUB variant"
    );
}
