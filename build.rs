use std::env;
use std::path::Path;
use std::process;

fn validate_fonts_directory(fonts_path: &str) -> bool {
    let fonts_dir = Path::new(fonts_path);

    // Check if the directory exists and is actually a directory
    if !fonts_dir.is_dir() {
        return false;
    }

    // Check for fonts.json configuration file
    let fonts_json = fonts_dir.join("fonts.json");
    if !fonts_json.exists() {
        return false;
    }

    true
}

fn main() {
    // Get the STANDARD_FONTS path from the environment variable
    let standard_fonts_path = match env::var("STANDARD_FONTS") {
        Ok(path) => path,
        Err(_) => {
            eprintln!("Error: STANDARD_FONTS environment variable not set");
            process::exit(1);
        }
    };

    if !validate_fonts_directory(&standard_fonts_path) {
        eprintln!("Error: PDF fonts directory not properly initialized");
        eprintln!("Directory: {}", standard_fonts_path);
        eprintln!("Please ensure the git submodule for PDF fonts has been initialized:");
        eprintln!();
        eprintln!("  git submodule update --init --recursive");
        eprintln!();
        eprintln!("The fonts directory should contain:");
        eprintln!("  - fonts.json configuration file");
        eprintln!("  - One or more .pfb font files");
        process::exit(1);
    }

    println!("cargo:rerun-if-changed={}", standard_fonts_path);
    println!("cargo:rerun-if-env-changed=STANDARD_FONTS");
}
