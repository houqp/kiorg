#![allow(dead_code)] // Allow unused code for helpers

use egui_kittest::Harness;
use kiorg::Kiorg;
use std::fs::File;
use std::path::PathBuf;
use tempfile::tempdir;

/// Create files and directories from a list of paths.
/// Returns the created paths.
pub fn create_test_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    for path in paths {
        if path.extension().is_some() {
            File::create(path).unwrap();
        } else {
            std::fs::create_dir(path).unwrap();
        }
        assert!(path.exists());
    }
    paths.to_vec()
}

/// Create a test image file with minimal valid PNG content
pub fn create_test_image(path: &PathBuf) -> PathBuf {
    // Minimal valid PNG file (1x1 transparent pixel)
    let png_data: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    std::fs::write(path, png_data).unwrap();
    assert!(path.exists());
    path.clone()
}

/// Create a test zip file with some entries
pub fn create_test_zip(path: &PathBuf) -> PathBuf {
    use std::io::Write;
    use zip::write::FileOptions;

    let file = std::fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);

    // Use explicit type annotation to fix the compiler error
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755) as FileOptions<'_, ()>;

    // Add a few entries to the zip
    zip.start_file("file1.txt", options).unwrap();
    zip.write_all(b"Content of file1.txt").unwrap();

    zip.start_file("file2.txt", options).unwrap();
    zip.write_all(b"Content of file2.txt").unwrap();

    // Add a directory
    zip.add_directory("subdir", options).unwrap();

    // Add a file in the subdirectory
    zip.start_file("subdir/file3.txt", options).unwrap();
    zip.write_all(b"Content of file3.txt in subdir").unwrap();

    zip.finish().unwrap();
    assert!(path.exists());
    path.clone()
}

/// Create a minimal test EPUB file
pub fn create_test_epub(path: &PathBuf) -> PathBuf {
    use std::io::Write;
    use zip::write::FileOptions;

    let file = std::fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);

    // Use explicit type annotation to fix the compiler error
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755) as FileOptions<'_, ()>;

    // Add mimetype file (required for EPUB)
    zip.start_file("mimetype", options).unwrap();
    zip.write_all(b"application/epub+zip").unwrap();

    // Add META-INF directory and container.xml
    zip.add_directory("META-INF", options).unwrap();
    zip.start_file("META-INF/container.xml", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
    <rootfiles>
        <rootfile full-path="content.opf" media-type="application/oebps-package+xml"/>
    </rootfiles>
</container>"#,
    )
    .unwrap();

    // Add content.opf with metadata
    zip.start_file("content.opf", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="BookId">
    <metadata xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:opf="http://www.idpf.org/2007/opf">
        <dc:title>Test EPUB Book</dc:title>
        <dc:creator>Test Author</dc:creator>
        <dc:language>en</dc:language>
        <dc:identifier id="BookId">urn:uuid:12345678-1234-1234-1234-123456789012</dc:identifier>
        <dc:publisher>Test Publisher</dc:publisher>
        <dc:date>2023-01-01</dc:date>
    </metadata>
    <manifest>
        <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
        <item id="content" href="content.html" media-type="application/xhtml+xml"/>
    </manifest>
    <spine toc="ncx">
        <itemref idref="content"/>
    </spine>
</package>"#,
    )
    .unwrap();

    // Add a simple content file
    zip.start_file("content.html", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>Test EPUB</title>
</head>
<body>
    <h1>Test EPUB Content</h1>
    <p>This is a test EPUB file for testing purposes.</p>
</body>
</html>"#,
    )
    .unwrap();

    // Add a simple TOC file
    zip.start_file("toc.ncx", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE ncx PUBLIC "-//NISO//DTD ncx 2005-1//EN" "http://www.daisy.org/z3986/2005/ncx-2005-1.dtd">
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
    <head>
        <meta name="dtb:uid" content="urn:uuid:12345678-1234-1234-1234-123456789012"/>
    </head>
    <docTitle><text>Test EPUB Book</text></docTitle>
    <navMap>
        <navPoint id="navpoint-1" playOrder="1">
            <navLabel><text>Start</text></navLabel>
            <content src="content.html"/>
        </navPoint>
    </navMap>
</ncx>"#,
    )
    .unwrap();

    zip.finish().unwrap();
    assert!(path.exists());
    path.clone()
}

/// Create a minimal test PDF file with multiple pages
pub fn create_test_pdf(path: &PathBuf, page_count: usize) -> PathBuf {
    // Create a minimal multi-page PDF using a simple approach
    // This creates a basic PDF structure with the specified number of pages
    let pdf_content = create_minimal_pdf_content(page_count);
    std::fs::write(path, pdf_content).unwrap();
    assert!(path.exists());
    path.clone()
}

/// Generate minimal PDF content with the specified number of pages
fn create_minimal_pdf_content(page_count: usize) -> Vec<u8> {
    // Create a properly structured PDF that PDFium can parse
    let mut content = Vec::new();
    let mut offsets = Vec::new();

    // PDF Header
    let header = b"%PDF-1.4\n";
    content.extend_from_slice(header);

    // Track object offsets for xref table
    offsets.push(0); // Object 0 (free)

    // Object 1: Catalog
    offsets.push(content.len());
    let catalog = "1 0 obj\n<<\n/Type /Catalog\n/Pages 2 0 R\n>>\nendobj\n".to_string();
    content.extend_from_slice(catalog.as_bytes());

    // Object 2: Pages
    offsets.push(content.len());
    let mut kids_refs = String::new();
    for i in 0..page_count {
        if i > 0 {
            kids_refs.push(' ');
        }
        kids_refs.push_str(&format!("{} 0 R", 3 + i * 2));
    }
    let pages = format!(
        "2 0 obj\n<<\n/Type /Pages\n/Count {}\n/Kids [{}]\n>>\nendobj\n",
        page_count, kids_refs
    );
    content.extend_from_slice(pages.as_bytes());

    // Create page objects and their content streams
    for i in 0..page_count {
        let page_obj_num = 3 + i * 2;
        let content_obj_num = page_obj_num + 1;

        // Page object
        offsets.push(content.len());
        let page_content_text = format!("(Page {})", i + 1);
        let stream_content = format!("BT\n/F1 12 Tf\n72 720 Td\n{} Tj\nET\n", page_content_text);
        let stream_length = stream_content.len();

        let page = format!(
            "{} 0 obj\n<<\n/Type /Page\n/Parent 2 0 R\n/MediaBox [0 0 612 792]\n/Contents {} 0 R\n/Resources <<\n/Font <<\n/F1 <<\n/Type /Font\n/Subtype /Type1\n/BaseFont /Helvetica\n>>\n>>\n>>\n>>\nendobj\n",
            page_obj_num, content_obj_num
        );
        content.extend_from_slice(page.as_bytes());

        // Content stream object
        offsets.push(content.len());
        let content_stream = format!(
            "{} 0 obj\n<<\n/Length {}\n>>\nstream\n{}endstream\nendobj\n",
            content_obj_num, stream_length, stream_content
        );
        content.extend_from_slice(content_stream.as_bytes());
    }

    // Cross-reference table
    let xref_offset = content.len();
    let mut xref = format!("xref\n0 {}\n", offsets.len());

    // First entry (object 0) is always free
    xref.push_str("0000000000 65535 f \n");

    // Add entries for all other objects
    for offset in offsets.iter().skip(1) {
        xref.push_str(&format!("{:010} 00000 n \n", offset));
    }

    content.extend_from_slice(xref.as_bytes());

    // Trailer
    let trailer = format!(
        "trailer\n<<\n/Size {}\n/Root 1 0 R\n>>\nstartxref\n{}\n%%EOF\n",
        offsets.len(),
        xref_offset
    );
    content.extend_from_slice(trailer.as_bytes());

    content
}

// Wrapper to hold both the harness and the config temp directory to prevent premature cleanup
pub struct TestHarness<'a> {
    pub harness: Harness<'a, Kiorg>,
    _config_temp_dir: tempfile::TempDir, // Prefixed with _ to indicate it's only kept for its Drop behavior
}

pub fn create_harness<'a>(temp_dir: &tempfile::TempDir) -> TestHarness<'a> {
    // Create a separate temporary directory for config files
    let config_temp_dir = tempdir().unwrap();
    create_harness_with_config_dir(temp_dir, config_temp_dir)
}

pub fn create_harness_with_config_dir<'a>(
    temp_dir: &tempfile::TempDir,
    config_temp_dir: tempfile::TempDir,
) -> TestHarness<'a> {
    let test_config_dir = config_temp_dir.path().to_path_buf();
    std::fs::create_dir_all(&test_config_dir).unwrap();

    // Create a new egui context
    let ctx = egui::Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx.clone());

    let app = Kiorg::new_with_config_dir(
        &cc,
        Some(temp_dir.path().to_path_buf()),
        Some(test_config_dir),
    )
    .expect("Failed to create Kiorg app");

    // Create a test harness with more steps to ensure all events are processed
    let harness = Harness::builder()
        .with_size(egui::Vec2::new(800.0, 600.0))
        .with_max_steps(20)
        .build_eframe(|_cc| app);

    // Run one step to initialize the app
    let mut harness = harness;
    harness.step();

    let mut harness = TestHarness {
        harness,
        _config_temp_dir: config_temp_dir,
    };
    // Ensure consistent sort order for reliable selection and verification
    harness.ensure_sorted_by_name_ascending();

    harness
}

impl<'a> TestHarness<'a> {
    /// Ensures the current tab's entries are sorted by Name/Ascending.
    pub fn ensure_sorted_by_name_ascending(&mut self) {
        // Toggle twice on the TabManager to ensure Ascending order regardless of the initial state
        self.harness
            .state_mut()
            .tab_manager
            .toggle_sort(kiorg::models::tab::SortColumn::Name); // Sets Name/Descending or None
        self.harness
            .state_mut()
            .tab_manager
            .toggle_sort(kiorg::models::tab::SortColumn::Name); // Sets Name/Ascending
                                                                // sort_all_tabs is called implicitly by toggle_sort now, no need for explicit call
        self.harness.step(); // Allow sort to apply and UI to update
    }
}

// Add methods to TestHarness to delegate to the inner harness
impl<'a> std::ops::Deref for TestHarness<'a> {
    type Target = Harness<'a, Kiorg>;

    fn deref(&self) -> &Self::Target {
        &self.harness
    }
}

impl std::ops::DerefMut for TestHarness<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.harness
    }
}
