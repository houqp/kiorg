use eframe::egui;
use std::path::PathBuf;
use clap::Parser;
use std::fs;

use kiorg::app::Kiorg;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to open (default: current directory)
    #[arg(default_value = ".")]
    directory: PathBuf,
}

fn main() -> Result<(), eframe::Error> {
    let args = Args::parse();
    
    // Validate and canonicalize the provided directory
    if !args.directory.exists() {
        eprintln!("Error: Directory '{}' does not exist", args.directory.display());
        std::process::exit(1);
    }
    
    if !args.directory.is_dir() {
        eprintln!("Error: '{}' is not a directory", args.directory.display());
        std::process::exit(1);
    }

    // Canonicalize the path to get absolute path
    let canonical_dir = match fs::canonicalize(&args.directory) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: Failed to canonicalize path '{}': {}", args.directory.display(), e);
            std::process::exit(1);
        }
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Kiorg",
        options,
        Box::new(|cc| Ok(Box::new(Kiorg::new(cc, canonical_dir))))
    )
}