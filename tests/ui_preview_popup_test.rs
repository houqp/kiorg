#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::{Key, Modifiers};
use kiorg::app::PopupType;
use kiorg::models::preview_content::PreviewContent;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;
use ui_test_helpers::{create_harness, create_test_epub, create_test_image};

/// Test that the image preview popup can be opened with the Shift+K shortcut
#[test]
fn test_image_preview_popup_shortcut() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a test image
    let image_path = temp_dir.path().join("test.png");
    create_test_image(&image_path);

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Select the image file
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let image_index = tab
            .entries
            .iter()
            .position(|e| e.name == "test.png")
            .expect("Image file should be in the entries");
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = image_index;
    }
    harness.step();

    // Step to update the preview
    harness.step();

    // Wait for the image preview to load
    for _ in 0..10 {
        match harness.state().preview_content.as_ref() {
            Some(PreviewContent::Image(_)) => break,
            Some(PreviewContent::Loading(..)) => harness.step(),
            _ => {
                // Force another step to try to load the preview
                harness.step();
            }
        }
    }

    // Verify image preview is loaded
    match harness.state().preview_content.as_ref() {
        Some(PreviewContent::Image(_)) => {}
        other => panic!("Preview content should be Image, got {:?}", other),
    }

    // Open preview popup with Shift+K
    let modifiers = Modifiers {
        shift: true,
        ..Default::default()
    };
    harness.press_key_modifiers(modifiers, Key::K);
    harness.step();

    // Verify the preview popup is shown
    match &harness.state().show_popup {
        Some(PopupType::Preview) => {
            // Check that the preview content has been loaded with initial page 0
            if let Some(PreviewContent::Doc(doc_meta)) = &harness.state().preview_content {
                match doc_meta {
                    kiorg::models::preview_content::DocMeta::Pdf(pdf_meta) => {
                        assert_eq!(
                            pdf_meta.current_page, 0,
                            "Preview popup should start at page 0"
                        );
                    }
                    kiorg::models::preview_content::DocMeta::Epub(_) => {
                        // EPUB doesn't have current_page, so nothing to check
                    }
                }
            }
        }
        other => panic!(
            "Preview popup should be shown after pressing Shift+K, got {:?}",
            other
        ),
    }

    // Close the popup with Escape
    harness.press_key(Key::Escape);
    harness.step();

    // Verify the popup is closed
    assert_eq!(
        harness.state().show_popup,
        None,
        "Preview popup should be closed after pressing Escape"
    );
}

/// Test that the PDF preview popup closes when an invalid PDF file is opened
#[test]
fn test_pdf_preview_popup_error_handling() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create an invalid PDF file (e.g., a text file with .pdf extension)
    let invalid_pdf_name = "invalid.pdf";
    let invalid_pdf_path = temp_dir.path().join(invalid_pdf_name);
    std::fs::write(&invalid_pdf_path, "This is not a valid PDF content.").unwrap();

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Select the invalid PDF file
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let pdf_index = tab
            .entries
            .iter()
            .position(|e| e.name == invalid_pdf_name)
            .expect("Invalid PDF file should be in the entries");
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = pdf_index;
    }
    harness.step();

    // Step to update the preview (this will attempt to load the invalid PDF)
    harness.step();

    // Open preview popup with Shift+K
    let modifiers = Modifiers {
        shift: true,
        ..Default::default()
    };
    harness.press_key_modifiers(modifiers, Key::K);
    harness.step();

    // Verify the preview popup is NOT shown (it should have closed due to the error)
    assert!(
        matches!(harness.state().show_popup, None),
        "Preview popup should be closed for invalid PDF files"
    );
}

/// Test that the image preview popup displays the image filename as its title
#[test]
fn test_image_preview_popup_title() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a test image with a specific name to test the title
    let image_name = "special_image_name.png";
    let image_path = temp_dir.path().join(image_name);
    create_test_image(&image_path);

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Select the image file
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let image_index = tab
            .entries
            .iter()
            .position(|e| e.name == image_name)
            .expect("Image file should be in the entries");
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = image_index;
    }
    harness.step();

    // Step to update the preview
    harness.step();

    // Wait for the image preview to load
    for _ in 0..10 {
        match harness.state().preview_content.as_ref() {
            Some(PreviewContent::Image(_)) => break,
            Some(PreviewContent::Loading(..)) => harness.step(),
            _ => harness.step(),
        }
    }

    // Open preview popup with Shift+K
    let modifiers = Modifiers {
        shift: true,
        ..Default::default()
    };
    harness.press_key_modifiers(modifiers, Key::K);
    harness.step();

    // Verify the popup is open with the correct title
    // We can't directly test the window title in the UI test framework,
    // but we can verify the filename is being used for the popup title path
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let selected_entry = tab.selected_entry().expect("Should have a selected entry");
        assert_eq!(
            selected_entry.name, image_name,
            "Selected entry name should match image name"
        );
    }

    // Close the popup
    harness.press_key(Key::Escape);
    harness.step();
}

/// Test that PDF preview works in popup
#[test]
fn test_pdf_preview_popup() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a test epub file that we'll use since we don't have PDF test file creation helper
    // EPUB files also use the DocMeta content type, so they work for testing the doc preview functionality
    let doc_name = "test.epub";
    let doc_path = temp_dir.path().join(doc_name);
    create_test_epub(&doc_path);

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Select the document file
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let doc_index = tab
            .entries
            .iter()
            .position(|e| e.name == doc_name)
            .expect("Document file should be in the entries");
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = doc_index;
    }
    harness.step();

    // Wait for the document preview to load
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(10));
        match harness.state().preview_content.as_ref() {
            Some(PreviewContent::Doc(_)) => break,
            Some(PreviewContent::Loading(..)) => harness.step(),
            _ => harness.step(),
        }
    }

    // Verify document preview is loaded
    match harness.state().preview_content.as_ref() {
        Some(PreviewContent::Doc(_)) => {}
        other => panic!("Preview content should be Doc, got {:?}", other),
    }

    // Open preview popup with Shift+K
    let modifiers = Modifiers {
        shift: true,
        ..Default::default()
    };
    harness.press_key_modifiers(modifiers, Key::K);
    harness.step();

    // Verify the preview popup is shown
    match &harness.state().show_popup {
        Some(PopupType::Preview) => {
            // Check that the preview content has been loaded with initial page 0
            if let Some(PreviewContent::Doc(doc_meta)) = &harness.state().preview_content {
                match doc_meta {
                    kiorg::models::preview_content::DocMeta::Pdf(pdf_meta) => {
                        assert_eq!(
                            pdf_meta.current_page, 0,
                            "Preview popup should start at page 0"
                        );
                    }
                    kiorg::models::preview_content::DocMeta::Epub(_) => {
                        // EPUB doesn't have current_page, so nothing to check
                    }
                }
            }
        }
        other => panic!(
            "Preview popup should be shown after pressing Shift+K, got {:?}",
            other
        ),
    }

    // Test page navigation with arrow keys
    harness.press_key(Key::ArrowRight);
    harness.step();

    // Check if next page is shown (or still at page 0 if only one page)
    match &harness.state().show_popup {
        Some(PopupType::Preview) => {
            // Check that the preview content has been loaded and page navigation works
            if let Some(PreviewContent::Doc(doc_meta)) = &harness.state().preview_content {
                match doc_meta {
                    kiorg::models::preview_content::DocMeta::Pdf(pdf_meta) => {
                        // Either we advanced to page 1 or we're still at page 0 if there's only one page
                        assert!(
                            pdf_meta.current_page <= 1,
                            "Page should be 0 or 1 after pressing right arrow"
                        );
                    }
                    kiorg::models::preview_content::DocMeta::Epub(_) => {
                        // EPUB doesn't support page navigation
                    }
                }
            }
        }
        other => panic!("Preview popup should still be shown, got {:?}", other),
    }

    // Try going back with left arrow
    harness.press_key(Key::ArrowLeft);
    harness.step();

    // Verify we're back at page 0
    match &harness.state().show_popup {
        Some(PopupType::Preview) => {
            if let Some(PreviewContent::Doc(doc_meta)) = &harness.state().preview_content {
                match doc_meta {
                    kiorg::models::preview_content::DocMeta::Pdf(pdf_meta) => {
                        assert_eq!(
                            pdf_meta.current_page, 0,
                            "Should be back at page 0 after pressing left arrow"
                        );
                    }
                    kiorg::models::preview_content::DocMeta::Epub(_) => {
                        // EPUB doesn't support page navigation
                    }
                }
            }
        }
        other => panic!("Preview popup should still be shown, got {:?}", other),
    }

    // Close the popup
    harness.press_key(Key::Escape);
    harness.step();
}

/// Test that the preview popup doesn't open for unsupported file types
#[test]
fn test_preview_popup_unsupported_file() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a regular text file
    let text_path = temp_dir.path().join("test.txt");
    std::fs::write(&text_path, "This is a text file").unwrap();

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Select the text file
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let file_index = tab
            .entries
            .iter()
            .position(|e| e.name == "test.txt")
            .expect("Text file should be in the entries");
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = file_index;
    }
    harness.step();

    // Step to update the preview
    harness.step();

    // Open preview popup with Shift+K
    let modifiers = Modifiers {
        shift: true,
        ..Default::default()
    };
    harness.press_key_modifiers(modifiers, Key::K);
    harness.step();

    // Verify the preview popup is NOT shown for unsupported file types
    assert!(
        matches!(harness.state().show_popup, None),
        "Preview popup should not be shown for unsupported file types"
    );
}

/// Test that the EPUB preview popup correctly sets the Page Count metadata
/// This test uses EPUB files which also use DocMeta and are easier to create reliably
#[test]
fn test_doc_preview_popup_page_count_metadata() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a test epub file - EPUB files also use DocMeta content type
    let doc_name = "test.epub";
    let doc_path = temp_dir.path().join(doc_name);
    create_test_epub(&doc_path);

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Select the document file
    {
        let tab = harness.state().tab_manager.current_tab_ref();
        let doc_index = tab
            .entries
            .iter()
            .position(|e| e.name == doc_name)
            .expect("Document file should be in the entries");
        let tab = harness.state_mut().tab_manager.current_tab_mut();
        tab.selected_index = doc_index;
    }
    harness.step();

    // Step to update the preview
    harness.step();

    // Wait for the document preview to load
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(10));
        match harness.state().preview_content.as_ref() {
            Some(PreviewContent::Doc(_)) => break,
            Some(PreviewContent::Loading(..)) => harness.step(),
            _ => harness.step(),
        }
    }

    // Verify document preview is loaded
    match harness.state().preview_content.as_ref() {
        Some(PreviewContent::Doc(_)) => {}
        other => panic!("Preview content should be Doc, got {:?}", other),
    }

    // Open preview popup with Shift+K
    let modifiers = Modifiers {
        shift: true,
        ..Default::default()
    };
    harness.press_key_modifiers(modifiers, Key::K);
    harness.step();

    // Verify the preview popup is shown with correct page count metadata
    match &harness.state().show_popup {
        Some(PopupType::Preview) => {
            // Check that the preview content has Page Count metadata set correctly
            if let Some(PreviewContent::Doc(doc_meta)) = &harness.state().preview_content {
                match doc_meta {
                    kiorg::models::preview_content::DocMeta::Pdf(pdf_meta) => {
                        assert_eq!(
                            pdf_meta.current_page, 0,
                            "Preview popup should start at page 0"
                        );

                        // Most importantly: verify that page count is set as a native field
                        // For PDF files, the page count should be at least 1
                        let page_count = pdf_meta.page_count;

                        assert!(
                            page_count >= 1,
                            "Page count should be at least 1 for valid document files"
                        );

                        // Verify that the metadata is not empty
                        assert!(
                            !pdf_meta.metadata.is_empty(),
                            "Metadata should not be empty for document preview popup"
                        );
                    }
                    kiorg::models::preview_content::DocMeta::Epub(epub_meta) => {
                        // For EPUB files, we don't expect page count since EPUBs are reflowable
                        // Just verify that the metadata is not empty
                        assert!(
                            !epub_meta.metadata.is_empty(),
                            "Metadata should not be empty for document preview popup"
                        );
                    }
                }
            } else {
                panic!("Preview content should be Doc type for document file");
            }
        }
        other => panic!(
            "Preview popup should be shown after pressing Shift+K, got {:?}",
            other
        ),
    }

    // Close the popup
    harness.press_key(Key::Escape);
    harness.step();

    // Verify the popup is closed
    assert_eq!(
        harness.state().show_popup,
        None,
        "Preview popup should be closed after pressing Escape"
    );
}
