#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
mod bindgen_incl {
    include!(concat!(env!("OUT_DIR"), "/pdfium_bindings.rs"));
}

// symbols shared by both dynamic and static builds
pub use bindgen_incl::{
    FPDFBitmap_BGRA, FPDF_ANNOT, FPDF_DOCUMENT, FPDF_ERR_FILE, FPDF_ERR_FORMAT, FPDF_ERR_PAGE,
    FPDF_ERR_PASSWORD, FPDF_ERR_SECURITY, FPDF_ERR_SUCCESS, FPDF_ERR_UNKNOWN, FPDF_LCD_TEXT,
    FPDF_PRINTING,
};

#[cfg(not(feature = "static"))]
pub use bindgen_incl::{FPDF_BITMAP, FPDF_BYTESTRING, FPDF_PAGE};

// in static build, reuse bindgen symbols directly
#[cfg(feature = "static")]
pub use bindgen_incl::{
    FPDFBitmap_CreateEx, FPDFBitmap_Destroy, FPDFBitmap_FillRect, FPDF_CloseDocument,
    FPDF_ClosePage, FPDF_GetFileVersion, FPDF_GetLastError, FPDF_GetMetaText, FPDF_GetPageCount,
    FPDF_GetPageHeightF, FPDF_GetPageWidthF, FPDF_InitLibrary, FPDF_LoadDocument, FPDF_LoadPage,
    FPDF_RenderPageBitmap,
};
