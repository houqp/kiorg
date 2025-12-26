#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use std::{fs, io::Write as _, sync::OnceLock};

use libloading::{Library, Symbol};

pub use super::bindgen::*;

const LIBPDFIUM_DYLIB: &[u8] = include_bytes!(env!("PDFIUM_DYNLIB_PATH"));
static LIBPDFIUM_LIB: OnceLock<Library> = OnceLock::new();

#[cfg(target_os = "windows")]
static LIBPDFIUM_DYLIB_PATH: OnceLock<std::path::PathBuf> = OnceLock::new();

pub fn cleanup_cache() {
    #[cfg(target_os = "windows")]
    {
        if let Some(path) = LIBPDFIUM_DYLIB_PATH.get() {
            let _ = fs::remove_file(path);
        }
    }
}

fn libpdfium() -> &'static Library {
    LIBPDFIUM_LIB.get_or_init(|| unsafe {
        #[cfg(target_os = "windows")]
        {
            let path = LIBPDFIUM_DYLIB_PATH
                .get_or_init(|| std::env::temp_dir().join("kiorg-libpdfium.dll"));
            // Always try to overwrite the file to ensure we have the latest embedded version.
            // If it's locked by another process, we ignore the error and try to load whatever is there.
            let _ = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .and_then(|mut file| file.write_all(LIBPDFIUM_DYLIB));

            Library::new(path).expect("failed to load dynamic libpdfium")
        }

        #[cfg(not(target_os = "windows"))]
        {
            use tempfile::Builder;
            let mut file = Builder::new()
                .prefix("libpdfium")
                .tempfile()
                .expect("failed to create temp file");
            file.write_all(LIBPDFIUM_DYLIB)
                .expect("failed to write to temp file");
            let path = file.into_temp_path(); // close file
            let lib = Library::new(&path).expect("failed to load dynamic libpdfium");
            // On Unix, we can safely delete the file immediately after it's loaded into memory.
            let _ = fs::remove_file(path);
            lib
        }
    })
}

#[inline]
fn bind<'a, T>(function: &[u8]) -> Symbol<'a, T> {
    unsafe { libpdfium().get(function).unwrap() }
}

macro_rules! dylib_cfn {
    ($name:ident ( $($arg_name:ident : $arg_type:ty),* $(,)? ) $(-> $ret_type:ty)?) => {
        #[allow(clippy::too_many_arguments)]
        pub unsafe fn $name($($arg_name : $arg_type),*) $(-> $ret_type)? {
            static LOCK: OnceLock<unsafe extern "C" fn($($arg_type),*) $(-> $ret_type)?> = OnceLock::new();
            let f = LOCK.get_or_init(|| {
                let symbol: Symbol<'static, unsafe extern "C" fn($($arg_type),*) $(-> $ret_type)?> =
                    bind(concat!(stringify!($name), "\0").as_bytes());
                *symbol
            });
            f($($arg_name),*)
        }
    };
}

dylib_cfn!(FPDF_InitLibrary());

dylib_cfn!(FPDF_LoadDocument(
    file_path: *const std::os::raw::c_char,
    password: *const std::os::raw::c_char,
) -> FPDF_DOCUMENT);

dylib_cfn!(FPDF_GetPageCount(document: FPDF_DOCUMENT) -> std::os::raw::c_int);

dylib_cfn!(FPDF_GetFileVersion(
    document: FPDF_DOCUMENT,
    fileVersion: *mut std::os::raw::c_int,
) -> std::os::raw::c_int);

dylib_cfn!(FPDF_GetMetaText(
    document: FPDF_DOCUMENT,
    tag: FPDF_BYTESTRING,
    buffer: *mut std::os::raw::c_void,
    buflen: std::os::raw::c_ulong,
) -> std::os::raw::c_ulong);

dylib_cfn!(FPDF_LoadPage(document: FPDF_DOCUMENT, page_index: std::os::raw::c_int) -> FPDF_PAGE);

dylib_cfn!(FPDF_GetPageWidthF(page: FPDF_PAGE) -> f32);

dylib_cfn!(FPDF_GetPageHeightF(page: FPDF_PAGE) -> f32);

dylib_cfn!(FPDF_ClosePage(page: FPDF_PAGE));

dylib_cfn!(FPDFBitmap_CreateEx(
    width: std::os::raw::c_int,
    height: std::os::raw::c_int,
    format: std::os::raw::c_int,
    first_scan: *mut std::os::raw::c_void,
    stride: std::os::raw::c_int,
) -> FPDF_BITMAP);

dylib_cfn!(FPDFBitmap_FillRect(
    bitmap: FPDF_BITMAP,
    left: std::os::raw::c_int,
    top: std::os::raw::c_int,
    width: std::os::raw::c_int,
    height: std::os::raw::c_int,
    color: std::os::raw::c_uint,
));

dylib_cfn!(FPDF_RenderPageBitmap(
    bitmap: FPDF_BITMAP,
    page: FPDF_PAGE,
    start_x: std::os::raw::c_int,
    start_y: std::os::raw::c_int,
    size_x: std::os::raw::c_int,
    size_y: std::os::raw::c_int,
    rotate: std::os::raw::c_int,
    flags: std::os::raw::c_int,
));

dylib_cfn!(FPDFBitmap_Destroy(bitmap: FPDF_BITMAP));

dylib_cfn!(FPDF_CloseDocument(document: FPDF_DOCUMENT));
dylib_cfn!(FPDF_GetLastError() -> std::os::raw::c_ulong);
