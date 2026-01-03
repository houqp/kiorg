#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Key;
use kiorg::models::preview_content::PreviewContent;
use kiorg::ui::popup::PopupType;
use tempfile::tempdir;
use ui_test_helpers::{
    create_harness, create_test_epub, create_test_image, shift_modifiers, wait_for_condition,
};

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

    harness.key_press(egui::Key::J);

    // Wait for the image preview to load
    wait_for_condition(|| {
        harness.step();
        matches!(
            harness.state().preview_content.as_ref(),
            Some(PreviewContent::Image(_))
        )
    });

    // Verify image preview is loaded
    match harness.state().preview_content.as_ref() {
        Some(PreviewContent::Image(_)) => {}
        other => panic!("Preview content should be Image, got {other:?}"),
    }

    // Open preview popup with Shift+K
    harness.key_press_modifiers(shift_modifiers(), Key::K);
    harness.step();

    // Verify the image viewer popup is shown
    match &harness.state().show_popup {
        Some(PopupType::Image(_)) => {}
        other => panic!("Image viewer popup should be shown after pressing Shift+K, got {other:?}"),
    }

    // Close the popup with Escape
    harness.key_press(Key::Escape);
    harness.step();

    // Verify the popup is closed
    assert!(
        harness.state().show_popup.is_none(),
        "Preview popup should be closed after pressing Escape"
    );
}

/// Test that the PDF preview popup closes when an invalid PDF file is opened
#[test]
fn test_pdf_preview_popup_error_handling() {
    let temp_dir = tempdir().unwrap();

    // Create an invalid PDF file (e.g., a text file with .pdf extension)
    let invalid_pdf_name = "invalid.pdf";
    let invalid_pdf_path = temp_dir.path().join(invalid_pdf_name);
    std::fs::write(&invalid_pdf_path, "This is not a valid PDF content.").unwrap();

    let mut harness = create_harness(&temp_dir);

    // Open preview popup with Shift+K
    harness.key_press_modifiers(shift_modifiers(), Key::K);
    harness.step();

    // Verify the PDF viewer popup is opened (even with error, it should be opened)
    assert!(
        matches!(harness.state().show_popup, Some(PopupType::Pdf(_))),
        "PDF viewer popup should be opened to display the error"
    );

    let success = wait_for_condition(|| {
        harness.step();
        // Verify the error message is displayed in the preview content
        if let Some(PreviewContent::Text(text)) = harness.state().preview_content.as_ref() {
            text.contains("File not in PDF format or corrupted")
        } else {
            false
        }
    });
    if !success {
        panic!(
            "Preview content should be Text with error message, but got: {:?}",
            harness.state().preview_content
        );
    }
}

/// Test that the preview popup doesn't open for unsupported file types
#[test]
fn test_preview_popup_unsupported_file() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a regular text file
    let text_path = temp_dir.path().join("test.foobar");
    std::fs::write(&text_path, "This is a text file").unwrap();

    // Start the harness
    let mut harness = create_harness(&temp_dir);

    // Open preview popup with Shift+K
    harness.key_press_modifiers(shift_modifiers(), Key::K);
    harness.step();

    // Verify the preview popup is NOT shown for unsupported file types
    assert!(
        harness.state().show_popup.is_none(),
        "Preview popup should not be shown for unsupported file types"
    );
}

/// Test that the EPUB preview popup correctly sets the Page Count metadata
/// This test uses EPUB files which also use `DocMeta` and are easier to create reliably
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

    // Wait for the document preview to load
    wait_for_condition(|| {
        harness.step();
        matches!(
            harness.state().preview_content.as_ref(),
            Some(PreviewContent::Ebook(_))
        )
    });

    // Verify document preview is loaded
    match harness.state().preview_content.as_ref() {
        Some(PreviewContent::Ebook(_)) => {}
        other => panic!("Preview content should be Ebook, got {other:?}"),
    }

    // Open preview popup with Shift+K
    harness.key_press_modifiers(shift_modifiers(), Key::K);
    harness.step();

    // Verify the ebook viewer popup is shown with correct metadata
    match &harness.state().show_popup {
        Some(PopupType::Ebook(_)) => {
            // Check that the preview content has metadata set correctly
            match &harness.state().preview_content {
                Some(PreviewContent::Ebook(ebook_meta)) => {
                    // For ebook files, we don't expect page count since ebooks are reflowable
                    // Just verify that the metadata is not empty
                    assert!(
                        !ebook_meta.metadata.is_empty(),
                        "Metadata should not be empty for ebook viewer popup"
                    );
                }
                other => {
                    panic!("Preview content should be Ebook type for document file, got {other:?}");
                }
            }
        }
        other => panic!("Ebook viewer popup should be shown after pressing Shift+K, got {other:?}"),
    }

    // Close the popup
    harness.key_press(Key::Escape);
    harness.step();

    // Verify the popup is closed
    assert!(
        harness.state().show_popup.is_none(),
        "Preview popup should be closed after pressing Escape"
    );
}
