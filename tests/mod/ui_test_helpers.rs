#![allow(dead_code)] // Allow unused code for helpers

use egui_kittest::Harness;
use kiorg::Kiorg;
use std::fmt::Write;
use std::fs::File;
use std::path::PathBuf;
use tar::{Builder, Header};
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

/// Create a test tar file with some entries
pub fn create_test_tar(path: &PathBuf) -> PathBuf {
    let file = File::create(path).unwrap();
    let mut tar = Builder::new(file);

    // Add file1.txt
    let mut header = Header::new_gnu();
    let content1 = b"Content of file1.txt";
    header.set_path("file1.txt").unwrap();
    header.set_size(content1.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append(&header, &content1[..]).unwrap();

    // Add file2.txt
    let mut header = Header::new_gnu();
    let content2 = b"Content of file2.txt";
    header.set_path("file2.txt").unwrap();
    header.set_size(content2.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append(&header, &content2[..]).unwrap();

    // Add subdirectory
    let mut header = Header::new_gnu();
    header.set_path("subdir/").unwrap();
    header.set_size(0);
    header.set_mode(0o755);
    header.set_entry_type(tar::EntryType::Directory);
    header.set_cksum();
    tar.append(&header, std::io::empty()).unwrap();

    // Add a file in the subdirectory
    let mut header = Header::new_gnu();
    let content3 = b"Content of file3.txt in subdir";
    header.set_path("subdir/file3.txt").unwrap();
    header.set_size(content3.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append(&header, &content3[..]).unwrap();

    tar.finish().unwrap();
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
        write!(kids_refs, "{} 0 R", 3 + i * 2).unwrap();
    }
    let pages = format!(
        "2 0 obj\n<<\n/Type /Pages\n/Count {page_count}\n/Kids [{kids_refs}]\n>>\nendobj\n"
    );
    content.extend_from_slice(pages.as_bytes());

    // Create page objects and their content streams
    for i in 0..page_count {
        let page_obj_num = 3 + i * 2;
        let content_obj_num = page_obj_num + 1;

        // Page object
        offsets.push(content.len());
        let page_content_text = format!("(Page {})", i + 1);
        let stream_content = format!("BT\n/F1 12 Tf\n72 720 Td\n{page_content_text} Tj\nET\n");
        let stream_length = stream_content.len();

        let page = format!(
            "{page_obj_num} 0 obj\n<<\n/Type /Page\n/Parent 2 0 R\n/MediaBox [0 0 612 792]\n/Contents {content_obj_num} 0 R\n/Resources <<\n/Font <<\n/F1 <<\n/Type /Font\n/Subtype /Type1\n/BaseFont /Helvetica\n>>\n>>\n>>\n>>\nendobj\n"
        );
        content.extend_from_slice(page.as_bytes());

        // Content stream object
        offsets.push(content.len());
        let content_stream = format!(
            "{content_obj_num} 0 obj\n<<\n/Length {stream_length}\n>>\nstream\n{stream_content}endstream\nendobj\n"
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
        writeln!(xref, "{offset:010} 00000 n ").unwrap();
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

/// Builder for creating TestHarness instances with a fluent API
pub struct TestHarnessBuilder {
    temp_dir: Option<PathBuf>,
    config_temp_dir: Option<tempfile::TempDir>,
    window_size: egui::Vec2,
}

impl TestHarnessBuilder {
    pub fn new() -> Self {
        Self {
            temp_dir: None,
            config_temp_dir: None,
            window_size: egui::Vec2::new(800.0, 800.0),
        }
    }

    pub fn with_temp_dir(mut self, temp_dir: &tempfile::TempDir) -> Self {
        self.temp_dir = Some(temp_dir.path().to_path_buf());
        self
    }

    pub fn with_config_dir(mut self, config_temp_dir: tempfile::TempDir) -> Self {
        self.config_temp_dir = Some(config_temp_dir);
        self
    }

    pub fn with_window_size(mut self, size: egui::Vec2) -> Self {
        self.window_size = size;
        self
    }

    pub fn build<'a>(self) -> TestHarness<'a> {
        let temp_dir = self.temp_dir.expect("temp_dir must be set");
        let config_temp_dir = self.config_temp_dir.unwrap_or_else(|| tempdir().unwrap());
        let test_config_dir = config_temp_dir.path().to_path_buf();

        std::fs::create_dir_all(&test_config_dir).unwrap();

        // Create a new egui context
        let ctx = egui::Context::default();
        let cc = eframe::CreationContext::_new_kittest(ctx);

        let app = Kiorg::new_with_config_dir(&cc, Some(temp_dir), Some(test_config_dir))
            .expect("Failed to create Kiorg app");

        // Create a test harness with more steps to ensure all events are processed
        let mut harness = Harness::builder()
            .with_size(self.window_size)
            .with_max_steps(20)
            .build_eframe(|_cc| app);
        // Run one step to initialize the app
        harness.step();

        let mut harness = TestHarness {
            harness,
            _config_temp_dir: config_temp_dir,
        };

        // Ensure consistent sort order for reliable selection and verification
        harness.ensure_sorted_by_name_ascending();

        harness
    }
}

/// Helper function to wait for a condition with sleep intervals
/// Runs the callback up to 300 times with 10ms sleep between attempts
pub fn wait_for_condition<F>(mut condition: F)
where
    F: FnMut() -> bool,
{
    for _ in 0..300 {
        if condition() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

pub fn create_harness<'a>(temp_dir: &tempfile::TempDir) -> TestHarness<'a> {
    TestHarnessBuilder::new().with_temp_dir(temp_dir).build()
}

pub fn create_harness_with_config_dir<'a>(
    temp_dir: &tempfile::TempDir,
    config_temp_dir: tempfile::TempDir,
) -> TestHarness<'a> {
    TestHarnessBuilder::new()
        .with_temp_dir(temp_dir)
        .with_config_dir(config_temp_dir)
        .build()
}

impl TestHarness<'_> {
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

#[inline]
pub fn cmd_modifiers() -> egui::Modifiers {
    egui::Modifiers {
        #[cfg(target_os = "macos")]
        mac_cmd: true,
        #[cfg(target_os = "macos")]
        command: true,
        #[cfg(target_os = "macos")]
        ctrl: false,
        #[cfg(not(target_os = "macos"))]
        ctrl: true,
        #[cfg(not(target_os = "macos"))]
        command: true,
        #[cfg(not(target_os = "macos"))]
        mac_cmd: false,
        ..Default::default()
    }
}

/// Create cross-platform Ctrl modifiers that work on all platforms
/// On macOS: Sets ctrl but NOT command (for pure ctrl shortcuts)
/// On Linux/Windows: Sets ctrl and command
#[inline]
pub fn ctrl_modifiers() -> egui::Modifiers {
    egui::Modifiers {
        #[cfg(target_os = "macos")]
        mac_cmd: false,
        #[cfg(target_os = "macos")]
        command: false,
        #[cfg(target_os = "macos")]
        ctrl: true,
        #[cfg(not(target_os = "macos"))]
        ctrl: true,
        #[cfg(not(target_os = "macos"))]
        command: true,
        #[cfg(not(target_os = "macos"))]
        mac_cmd: false,
        ..Default::default()
    }
}

/// Create a test video file with minimal valid MP4 content
pub fn create_test_video(path: &PathBuf) -> PathBuf {
    // Create a minimal valid MP4 file header
    // This is a very basic MP4 file structure that should be recognized as a video file
    let mp4_data: &[u8] = &[
        // ftyp box (file type box)
        0x00, 0x00, 0x00, 0x20, // box size (32 bytes)
        0x66, 0x74, 0x79, 0x70, // 'ftyp'
        0x69, 0x73, 0x6F, 0x6D, // major brand 'isom'
        0x00, 0x00, 0x02, 0x00, // minor version
        0x69, 0x73, 0x6F, 0x6D, // compatible brand 'isom'
        0x69, 0x73, 0x6F, 0x32, // compatible brand 'iso2'
        0x61, 0x76, 0x63, 0x31, // compatible brand 'avc1'
        0x6D, 0x70, 0x34, 0x31, // compatible brand 'mp41'
        // mdat box (media data box) - minimal empty media data
        0x00, 0x00, 0x00, 0x08, // box size (8 bytes)
        0x6D, 0x64, 0x61, 0x74, // 'mdat'
    ];

    std::fs::write(path, mp4_data).unwrap();
    assert!(path.exists());
    path.clone()
}
