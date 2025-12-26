// add an explicit extern crate dependency to your crate root, since the link-cplusplus crate will
// be otherwise unused and its link flags dropped.
extern crate link_cplusplus;

use std::ffi::CString;
use std::os::raw::c_void;
use std::path::Path;
use std::sync::Once;

mod ffi;

static PDFIUM_INIT: Once = Once::new();

/// Cleanup old cached PDFium library files from the cache directory.
pub fn cleanup_cache() {
    #[cfg(feature = "dynamic")]
    ffi::cleanup_cache();
}

// Helper function to convert FPDF_WSTR (u8 byte slice representing UTF-16LE) to Rust String
fn fpdf_wstr_to_string(fpdf_wstr_bytes: *mut u8, len_bytes: usize) -> Option<String> {
    if fpdf_wstr_bytes.is_null() || len_bytes == 0 {
        return None;
    }
    // Ensure the byte length is even for UTF-16 conversion, or handle it as an error
    if len_bytes % 2 != 0 {
        return None; // Malformed UTF-16 string
    }

    let char_count = len_bytes / 2;
    // Cast *mut u8 to *mut u16 to create the slice of u16s
    let u16_ptr = fpdf_wstr_bytes as *mut u16;
    let slice = unsafe { std::slice::from_raw_parts(u16_ptr, char_count) };

    // Trim trailing null terminator if present
    let actual_slice = if !slice.is_empty() && slice[char_count - 1] == 0 {
        &slice[..char_count - 1]
    } else {
        slice
    };

    String::from_utf16(actual_slice).ok()
}

fn get_last_error_message() -> String {
    let error_code = unsafe { ffi::FPDF_GetLastError() as u32 };
    match error_code {
        ffi::FPDF_ERR_SUCCESS => "Success".to_string(),
        ffi::FPDF_ERR_UNKNOWN => "Unknown error".to_string(),
        ffi::FPDF_ERR_FILE => "File not found or could not be opened".to_string(),
        ffi::FPDF_ERR_FORMAT => "File not in PDF format or corrupted".to_string(),
        ffi::FPDF_ERR_PASSWORD => "Password required or incorrect password".to_string(),
        ffi::FPDF_ERR_SECURITY => "Unsupported security scheme".to_string(),
        ffi::FPDF_ERR_PAGE => "Page not found or content error".to_string(),
        _ => format!("Unknown PDFium error code: {error_code}"),
    }
}

unsafe impl Send for PdfDocument {}

pub struct PdfDocument {
    doc: ffi::FPDF_DOCUMENT,
}

impl PdfDocument {
    pub fn open(path: &Path) -> Result<Self, String> {
        PDFIUM_INIT.call_once(|| unsafe {
            ffi::FPDF_InitLibrary();
        });
        // PDFium requires an absolute path. canonicalize() provides this but also adds \\?\ prefix on Windows.
        let abs_path = path
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize path: {e}"))?;
        let path_str = abs_path.to_str().ok_or("Invalid UTF-8 in path")?;

        #[cfg(target_os = "windows")]
        let c_path = {
            let s = if path_str.starts_with(r"\\?\UNC\") {
                format!(r"\\{}", &path_str[8..])
            } else {
                path_str
                    .strip_prefix(r"\\?\")
                    .unwrap_or(path_str)
                    .to_string()
            };
            CString::new(s).unwrap()
        };
        #[cfg(not(target_os = "windows"))]
        let c_path = CString::new(path_str).unwrap();

        let doc = unsafe { ffi::FPDF_LoadDocument(c_path.as_ptr(), std::ptr::null()) };
        if doc.is_null() {
            Err(format!(
                "Failed to load PDF document {}: {}",
                path.display(),
                get_last_error_message()
            ))
        } else {
            Ok(Self { doc })
        }
    }

    pub fn page_count(&self) -> isize {
        unsafe { ffi::FPDF_GetPageCount(self.doc) as isize }
    }

    pub fn get_pdf_version(&self) -> i32 {
        let mut file_version = 0;
        unsafe {
            ffi::FPDF_GetFileVersion(self.doc, &mut file_version);
        }
        file_version
    }

    pub fn get_metadata_value(&self, key: &str) -> Option<String> {
        let c_field = CString::new(key).ok()?;

        let mut buffer = vec![0u8; 256];
        let mut buffer_capacity = buffer.len();

        // Get meta-data |tag| content from |document|.
        //
        //   document - handle to the document.
        //   tag      - the tag to retrieve. The tag can be one of:
        //                Title, Author, Subject, Keywords, Creator, Producer,
        //                CreationDate, or ModDate.
        //              For detailed explanations of these tags and their respective
        //              values, please refer to PDF Reference 1.6, section 10.2.1,
        //              'Document Information Dictionary'.
        //   buffer   - a buffer for the tag. May be NULL.
        //   buflen   - the length of the buffer, in bytes. May be 0.
        //
        // Returns the number of bytes in the tag, including trailing zeros.
        //
        // The |buffer| is always encoded in UTF-16LE. The |buffer| is followed by two
        // bytes of zeros indicating the end of the string.  If |buflen| is less than
        // the returned length, or |buffer| is NULL, |buffer| will not be modified.
        //
        // For linearized files, FPDFAvail_IsFormAvail must be called before this, and
        // it must have returned PDF_FORM_AVAIL or PDF_FORM_NOTEXIST. Before that, there
        // is no guarantee the metadata has been loaded.
        let mut req_len_bytes = unsafe {
            ffi::FPDF_GetMetaText(
                self.doc,
                c_field.as_ptr(),
                buffer.as_mut_ptr() as *mut c_void,
                buffer_capacity as std::os::raw::c_ulong,
            )
        } as usize;
        if req_len_bytes <= 2 {
            return None;
        }

        // initial buffer not large enough, resize with the actual size
        if req_len_bytes > buffer_capacity {
            buffer.resize(req_len_bytes, 0);
            buffer_capacity = req_len_bytes;
            req_len_bytes = unsafe {
                ffi::FPDF_GetMetaText(
                    self.doc,
                    c_field.as_ptr(),
                    buffer.as_mut_ptr() as *mut c_void,
                    buffer_capacity as std::os::raw::c_ulong,
                )
            } as usize;
        }
        if req_len_bytes > 2 {
            fpdf_wstr_to_string(buffer.as_mut_ptr(), req_len_bytes)
        } else {
            None
        }
    }
    pub fn render_page(&self, page_number: isize, dpi: f32) -> Result<(Vec<u8>, i32, i32), String> {
        let page = unsafe { ffi::FPDF_LoadPage(self.doc, page_number as i32) };
        if page.is_null() {
            return Err(format!("Failed to load page {}", page_number));
        }

        let page_width = unsafe { ffi::FPDF_GetPageWidthF(page) };
        let page_height = unsafe { ffi::FPDF_GetPageHeightF(page) };

        let width = (page_width * dpi / 72.0).round() as i32;
        let height = (page_height * dpi / 72.0).round() as i32;

        if width <= 0 || height <= 0 {
            unsafe {
                ffi::FPDF_ClosePage(page);
            }
            return Err(format!(
                "Invalid page dimensions: width={} height={}",
                width, height
            ));
        }

        let stride = width * 4;
        let buffer_size = (stride * height) as usize;

        let bitmap_buffer = unsafe {
            std::alloc::alloc(std::alloc::Layout::from_size_align(buffer_size, 4).unwrap())
        };
        if bitmap_buffer.is_null() {
            unsafe {
                ffi::FPDF_ClosePage(page);
            }
            return Err("Failed to allocate bitmap buffer".to_string());
        }

        let bitmap = unsafe {
            ffi::FPDFBitmap_CreateEx(
                width,
                height,
                ffi::FPDFBitmap_BGRA as i32,
                bitmap_buffer as *mut c_void,
                stride,
            )
        };
        if bitmap.is_null() {
            unsafe {
                std::alloc::dealloc(
                    bitmap_buffer,
                    std::alloc::Layout::from_size_align(buffer_size, 4).unwrap(),
                );
                ffi::FPDF_ClosePage(page);
            }
            return Err("Failed to create bitmap".to_string());
        }

        // Fill bitmap with white
        unsafe {
            ffi::FPDFBitmap_FillRect(bitmap, 0, 0, width, height, 0xFFFFFFFF);
        }

        unsafe {
            ffi::FPDF_RenderPageBitmap(
                bitmap,
                page,
                0, // start_x
                0, // start_y
                width,
                height,
                0,                                                                  // rotate_flag
                (ffi::FPDF_LCD_TEXT | ffi::FPDF_PRINTING | ffi::FPDF_ANNOT) as i32, // flags
            );
        }

        let mut pixel_data = vec![0u8; buffer_size];
        unsafe {
            std::ptr::copy_nonoverlapping(bitmap_buffer, pixel_data.as_mut_ptr(), buffer_size);
        }

        // Pdfium outputs BGRA, but we need RGBA for egui.
        // Swap B (index 0) and R (index 2) for each pixel.
        for i in (0..buffer_size).step_by(4) {
            pixel_data.swap(i, i + 2);
        }

        unsafe {
            ffi::FPDFBitmap_Destroy(bitmap);
            std::alloc::dealloc(
                bitmap_buffer,
                std::alloc::Layout::from_size_align(buffer_size, 4).unwrap(),
            );
            ffi::FPDF_ClosePage(page);
        }

        Ok((pixel_data, width, height))
    }
}

impl Drop for PdfDocument {
    fn drop(&mut self) {
        unsafe {
            ffi::FPDF_CloseDocument(self.doc);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_metadata_value_end_to_end() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();

        // Minimal PDF with a long Title in the Info dict.
        // We use a long string to test the buffer reallocation logic.
        let long_title = "A".repeat(5000);
        let pdf_content = format!(
            "%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
             2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\
             3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n\
             4 0 obj\n<< /Title ({}) >>\nendobj\n\
             trailer\n<< /Root 1 0 R /Info 4 0 R /Size 5 >>\n\
             %%EOF",
            long_title
        );

        temp_file.write_all(pdf_content.as_bytes()).unwrap();

        let doc = PdfDocument::open(temp_file.path()).expect("Failed to open PDF");
        let title = doc
            .get_metadata_value("Title")
            .expect("Failed to get Title");

        assert_eq!(title, long_title);
    }

    #[test]
    fn test_open_non_existent_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let non_existent_file = temp_dir.path().join("non_existent.pdf");
        let result = PdfDocument::open(&non_existent_file);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(
            err.contains("No such file or directory")
                || err.contains("File not found or could not be opened")
        );
    }

    #[test]
    fn test_open_corrupted_file() {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"not a pdf").unwrap();
        let result = PdfDocument::open(temp_file.path());
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.contains("File not in PDF format or corrupted"));
    }
}
