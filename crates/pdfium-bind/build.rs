use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use tar::Archive;

fn generate_bindgen(pdfium_include_dir: &Path) {
    // Generate bindings using bindgen
    let bindings = bindgen::Builder::default()
        .rust_target(env!("CARGO_PKG_RUST_VERSION").parse().expect("valid"))
        // Main PDFium header file
        .header(pdfium_include_dir.join("fpdfview.h").to_str().unwrap())
        // Include all other PDFium headers
        .header(pdfium_include_dir.join("fpdf_annot.h").to_str().unwrap())
        .header(
            pdfium_include_dir
                .join("fpdf_attachment.h")
                .to_str()
                .unwrap(),
        )
        .header(pdfium_include_dir.join("fpdf_catalog.h").to_str().unwrap())
        .header(
            pdfium_include_dir
                .join("fpdf_dataavail.h")
                .to_str()
                .unwrap(),
        )
        .header(pdfium_include_dir.join("fpdf_doc.h").to_str().unwrap())
        .header(pdfium_include_dir.join("fpdf_edit.h").to_str().unwrap())
        .header(pdfium_include_dir.join("fpdf_ext.h").to_str().unwrap())
        .header(pdfium_include_dir.join("fpdf_flatten.h").to_str().unwrap())
        .header(pdfium_include_dir.join("fpdf_formfill.h").to_str().unwrap())
        .header(pdfium_include_dir.join("fpdf_fwlevent.h").to_str().unwrap())
        .header(
            pdfium_include_dir
                .join("fpdf_javascript.h")
                .to_str()
                .unwrap(),
        )
        .header(pdfium_include_dir.join("fpdf_ppo.h").to_str().unwrap())
        .header(
            pdfium_include_dir
                .join("fpdf_progressive.h")
                .to_str()
                .unwrap(),
        )
        .header(pdfium_include_dir.join("fpdf_save.h").to_str().unwrap())
        .header(pdfium_include_dir.join("fpdf_searchex.h").to_str().unwrap())
        .header(
            pdfium_include_dir
                .join("fpdf_signature.h")
                .to_str()
                .unwrap(),
        )
        .header(
            pdfium_include_dir
                .join("fpdf_structtree.h")
                .to_str()
                .unwrap(),
        )
        .header(
            pdfium_include_dir
                .join("fpdf_sysfontinfo.h")
                .to_str()
                .unwrap(),
        )
        .header(pdfium_include_dir.join("fpdf_text.h").to_str().unwrap())
        .header(
            pdfium_include_dir
                .join("fpdf_thumbnail.h")
                .to_str()
                .unwrap(),
        )
        .header(
            pdfium_include_dir
                .join("fpdf_transformpage.h")
                .to_str()
                .unwrap(),
        )
        // Add include path for header dependencies
        .clang_arg(format!("-I{}", pdfium_include_dir.display()))
        // Generate bindings for all PDFium functions
        .allowlist_function("FPDF.*")
        .allowlist_type("FPDF.*")
        .allowlist_var("FPDF.*")
        // Generate bindings for common types
        .allowlist_type("FX_.*")
        .allowlist_var("FX_.*")
        // Parse callbacks and other complex types
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to $OUT_DIR/pdfium_bindings.rs
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("pdfium_bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn download_and_unpack(url: &str, target_dir: &Path, expected_checksum: &str) {
    if target_dir.exists() {
        return;
    }

    println!("Downloading PDFium from {}", url);

    let response = ureq::get(url)
        .call()
        .expect("Failed to download PDFium binary");

    if response.status() != 200 {
        panic!(
            "Failed to download PDFium binary from {}: {} {}",
            url,
            response.status(),
            response.status_text()
        );
    }

    let mut body = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut body)
        .expect("Failed to read PDFium binary body");

    let mut hasher = Sha256::new();
    hasher.update(&body);
    let hash = format!("{:x}", hasher.finalize());

    if hash != expected_checksum {
        panic!(
            "Checksum mismatch for PDFium binary from {}.\nExpected: {}\nActual:   {}",
            url, expected_checksum, hash
        );
    }

    let tar_gz = GzDecoder::new(&body[..]);
    let mut archive = Archive::new(tar_gz);

    fs::create_dir_all(target_dir).expect("Failed to create target directory");
    archive
        .unpack(target_dir)
        .expect("Failed to unpack PDFium binary");
}

fn download_staticlib() -> (PathBuf, PathBuf) {
    if !cfg!(target_os = "macos") {
        panic!("Automatic static library download is only supported on macOS");
    }

    let version = "7442c";
    let url = format!(
        "https://github.com/paulocoutinhox/pdfium-lib/releases/download/{version}/macos.tgz"
    );
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let pdfium_dir = out_dir.join(format!("pdfium-static-{}", version));
    let release_dir = pdfium_dir.join("release");
    let include_dir = release_dir.join("include");
    let lib_dir = release_dir.join("lib");
    if release_dir.exists() {
        return (include_dir, lib_dir);
    }

    download_and_unpack(
        &url,
        &pdfium_dir,
        "0d44e4d49bc01d1cd1b16a4a046de3ff2b703dea62a76f7fb9eda0a0e7dc5486",
    );

    (include_dir, lib_dir)
}

fn download_dynlib() -> (PathBuf, PathBuf) {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    let (os, arch, checksum) = match (target_os.as_str(), target_arch.as_str()) {
        ("macos", "aarch64") => (
            "mac",
            "arm64",
            "2ba3da9ce259832211229d521027cb5ccf2a82c0f14a790776cb798a572e122f",
        ),
        ("macos", "x86_64") => (
            "mac",
            "x64",
            "795e22857d9e8e9d7d2bd3e5523cec3204e59468c5dc860b0d4961218326a7c7",
        ),
        ("linux", "x86_64") => (
            "linux",
            "x64",
            "8154bde78e115fecee6f5b4984a9635502e4c2b4c615e5e8a4b35cf6726908e7",
        ),
        ("linux", "aarch64") => (
            "linux",
            "arm64",
            "944c6f64cd765722dc1f2c023806ad26383497ebc1152d5aa9e2d44caeba1145",
        ),
        ("windows", "x86_64") => (
            "win",
            "x64",
            "69b86460365c40a82ad5d7e3401c004b58090bf30ccb72f708cc502097a20f96",
        ),
        ("windows", "aarch64") => (
            "win",
            "arm64",
            "ca3b624f0c38fc30bd33b45394c93561513caf09f86d43fd67e0dd39421fe252",
        ),
        ("windows", "x86") => (
            "win",
            "x86",
            "34c0797b9e73013ca06355d8858bb8b9ea08baa489189224b63162790f6806ce",
        ),
        (os, arch) => panic!("Unsupported platform: {}-{}", os, arch),
    };

    let version = "7592";
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let pdfium_dir = out_dir.join(format!("pdfium-dyn-{}", version));
    let include_dir = pdfium_dir.join("include");
    let lib_dir = pdfium_dir.join("lib");

    if pdfium_dir.exists() {
        return (include_dir, lib_dir);
    }

    let filename = format!("pdfium-{}-{}.tgz", os, arch);
    let url = format!(
        "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F{}/{}",
        version, filename
    );

    download_and_unpack(&url, &pdfium_dir, checksum);

    (include_dir, lib_dir)
}

fn main() {
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=CoreGraphics");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=AppKit");
    }

    let feature_static = env::var("CARGO_FEATURE_STATIC").is_ok();
    let feature_dynamic = env::var("CARGO_FEATURE_DYNAMIC").is_ok();

    if feature_static && feature_dynamic {
        panic!("Both 'static' and 'dynamic' features are enabled. Please choose only one.");
    }

    println!("cargo:rerun-if-env-changed=PDFIUM_STATIC_LIB_PATH");
    println!("cargo:rerun-if-env-changed=PDFIUM_DYNAMIC_LIB_PATH");
    println!("cargo:rerun-if-env-changed=PDFIUM_INCLUDE_PATH");

    let env_static_lib_path = env::var("PDFIUM_STATIC_LIB_PATH").ok().map(PathBuf::from);
    let env_dynamic_lib_path = env::var("PDFIUM_DYNAMIC_LIB_PATH").ok().map(PathBuf::from);
    let env_include_path = env::var("PDFIUM_INCLUDE_PATH").ok().map(PathBuf::from);

    let pdfium_include_dir = if feature_static || !feature_dynamic {
        let (pdfium_include_dir, pdfium_lib_dir) = if let Some(static_lib_path) =
            env_static_lib_path
        {
            let include_dir = env_include_path
                .expect("PDFIUM_INCLUDE_PATH must be set when PDFIUM_STATIC_LIB_PATH is provided");
            let lib_dir = static_lib_path
                .parent()
                .expect("Invalid PDFIUM_STATIC_LIB_PATH")
                .to_path_buf();
            (include_dir, lib_dir)
        } else {
            download_staticlib()
        };

        println!("cargo:rerun-if-changed={}", pdfium_lib_dir.display());
        // Tell cargo to link against libpdfium.a
        println!(
            "cargo:rustc-link-search=native={}",
            pdfium_lib_dir.display()
        );
        println!("cargo:rustc-link-lib=static=pdfium");

        pdfium_include_dir
    } else if feature_dynamic {
        let (pdfium_include_dir, pdfium_lib_path) = if let Some(dynamic_lib_path) =
            env_dynamic_lib_path
        {
            let include_dir = env_include_path
                .expect("PDFIUM_INCLUDE_PATH must be set when PDFIUM_DYNAMIC_LIB_PATH is provided");
            (include_dir, dynamic_lib_path)
        } else {
            let (include_dir, lib_dir) = download_dynlib();
            let lib_path = if cfg!(target_os = "macos") {
                lib_dir.join("libpdfium.dylib")
            } else if cfg!(target_os = "windows") {
                // For Windows, the DLL is in the bin directory, not lib
                lib_dir.parent().unwrap().join("bin").join("pdfium.dll")
            } else {
                lib_dir.join("libpdfium.so")
            };
            (include_dir, lib_path)
        };

        println!(
            "cargo:rustc-env=PDFIUM_DYNLIB_PATH={}",
            pdfium_lib_path.display()
        );
        pdfium_include_dir
    } else {
        panic!("Neither 'static' nor 'dynamic' feature is enabled. Please enable at least one.");
    };

    println!("cargo:rerun-if-changed={}", pdfium_include_dir.display());

    generate_bindgen(&pdfium_include_dir);
}
