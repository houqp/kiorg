# pdfium-bind

[![Crates.io](https://img.shields.io/crates/v/pdfium-bind.svg)](https://crates.io/crates/pdfium-bind) [![docs.rs](https://img.shields.io/docsrs/pdfium-bind)](https://docs.rs/pdfium-bind)

Rust FFI bindings and a high-level wrapper for [PDFium](https://pdfium.googlesource.com/pdfium/).

**Designed for ease of distribution:** This crate eliminates the need for users to manually install PDFium on their systems. It **embeds prebuilt PDFium binaries** directly into your executable.

## Features

- **`dynamic`** (Default): **Embeds the PDFium dynamic library** within your binary. At runtime, it extracts the library to a temporary file and loads it.
- **`static`**: Links PDFium statically at build time.

## Configuration

You can override the PDFium library and headers used during build by setting the following environment variables:

- `PDFIUM_STATIC_LIB_PATH`: Path to `libpdfium.a` (for `static` feature).
- `PDFIUM_DYNAMIC_LIB_PATH`: Path to the dynamic library (e.g., `.so`, `.dylib`, or `.dll`) (for `dynamic` feature).
- `PDFIUM_INCLUDE_PATH`: Path to the directory containing PDFium headers (required if any of the above are set).

If these variables are not set, the build script will automatically download the appropriate PDFium binary for your platform.

## Usage

```rust
use pdfium_bind::PdfDocument;
use std::path::Path;

fn main() -> Result<(), String> {
    // Open a PDF document
    let path = Path::new("example.pdf");
    let doc = PdfDocument::open(path)?;

    // Get page count
    let count = doc.page_count();
    println!("Page count: {}", count);

    // Get PDF version
    println!("PDF version: {}", doc.get_pdf_version());

    // Get metadata
    if let Some(title) = doc.get_metadata_value("Title") {
        println!("Title: {}", title);
    }

    // Render a page (e.g., page 0 at 300 DPI)
    // Returns (pixel_data, width, height) where pixel_data is RGBA
    let (pixels, width, height) = doc.render_page(0, 300.0)?;
    println!("Rendered page size: {}x{}", width, height);

    Ok(())
}
```

### Windows-specific Cache Cleanup

When using the `dynamic` feature on Windows, the PDFium DLL is extracted to a temporary location. If you want to ensure this file is cleaned up when your application exits, you can call:

```rust
pdfium_bind::cleanup_cache();
```

On Unix platforms, the temporary file is deleted immediately after being loaded into memory.

## Automatic Downloads

The crate automatically downloads pre-built PDFium binaries for supported platforms:
- **Dynamic linking**: Downloads from [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries).
- **Static linking**: Downloads from [paulocoutinhox/pdfium-lib](https://github.com/paulocoutinhox/pdfium-lib) (currently macOS only).

## Supported Platforms

- **macOS** (aarch64, x86_64)
- **Linux** (x86_64, aarch64)
- **Windows** (x86_64, aarch64, x86)
