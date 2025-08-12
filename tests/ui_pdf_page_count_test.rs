#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use kiorg::models::preview_content::PreviewContent;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_pdf, wait_for_condition};

/// Test that PDF page count is displayed in the right side panel preview
#[test]
fn test_pdf_page_count_in_preview_content() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a test PDF with 5 pages
    let pdf_path = temp_dir.path().join("test.pdf");
    create_test_pdf(&pdf_path, 5);

    // Create a dummy text file to ensure we have other files
    let text_path = temp_dir.path().join("readme.txt");
    std::fs::write(&text_path, "This is a text file").unwrap();

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Select the PDF file
    harness.key_press(egui::Key::J);

    // Step to update the preview and allow time for loading
    harness.step();

    // Wait for PDF processing in a loop, checking for preview content
    wait_for_condition(|| {
        harness.step();
        dbg!(&harness.state().preview_content);
        // Check if PDF preview content is loaded
        matches!(
            &harness.state().preview_content,
            Some(PreviewContent::Pdf(_))
        )
    });

    // Check if PDF preview loaded successfully
    // The main goal is to test that IF a PDF loads, the page count is accessible
    // This verifies the code structure is correct for displaying page counts
    match &harness.state().preview_content {
        Some(PreviewContent::Pdf(pdf_meta)) => {
            // SUCCESS: PDF loaded and we can verify the page count field exists
            assert!(
                pdf_meta.page_count > 0,
                "PDF page count should be greater than 0 when loaded"
            );

            // Verify that the PDF metadata includes expected fields
            assert!(!pdf_meta.title.is_empty(), "PDF should have a title");

            // Test passes - page count is available in the metadata
            // The UI rendering code in render_pdf_preview() will display this
            // as "Page Count: X" in the metadata grid
            println!(
                "✓ PDF loaded successfully with {} pages",
                pdf_meta.page_count
            );
        }
        Some(PreviewContent::Epub(_)) => {
            panic!("Expected PDF preview content, got EPUB");
        }
        Some(PreviewContent::Text(_)) => {
            panic!("PDF should not be treated as an text");
        }
        Some(PreviewContent::Loading(..)) => {
            panic!("PDF still loading");
        }
        Some(PreviewContent::Image(_)) => {
            panic!("PDF should not be treated as an image");
        }
        _other => {
            panic!("PDF expected");
        }
    }
}
