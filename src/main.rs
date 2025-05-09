use clap::Parser;
use eframe::egui;
use std::fs;
use std::path::PathBuf;
use tracing_subscriber::{fmt, EnvFilter};

use kiorg::app::Kiorg;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to open (default: use saved state or current directory)
    directory: Option<PathBuf>,
}

fn init_tracing() {
    // Get log level from environment variable or use "info" as default
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,font=error,pdf_render=error"));

    // Initialize the tracing subscriber
    fmt::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .init();
}

fn main() -> Result<(), eframe::Error> {
    init_tracing();

    let args = Args::parse();

    // If a directory is provided, validate and canonicalize it
    let initial_dir = if let Some(dir) = args.directory {
        // Validate the provided directory
        if !dir.exists() {
            eprintln!("Error: Directory '{}' does not exist", dir.display());
            std::process::exit(1);
        }

        if !dir.is_dir() {
            eprintln!("Error: '{}' is not a directory", dir.display());
            std::process::exit(1);
        }

        // Canonicalize the path to get absolute path
        let canonical_dir = match fs::canonicalize(&dir) {
            Ok(path) => path,
            Err(e) => {
                eprintln!(
                    "Error: Failed to canonicalize path '{}': {}",
                    dir.display(),
                    e
                );
                std::process::exit(1);
            }
        };

        Some(canonical_dir)
    } else {
        // No directory provided, use None to load from saved state
        None
    };

    // Load the app icon from embedded data
    let icon_data = kiorg::utils::icon::load_app_icon();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(icon_data),
        ..Default::default()
    };

    eframe::run_native(
        "Kiorg",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            egui_zhcn_fonts::add_sys_ui_fonts(&cc.egui_ctx);
            Ok(Box::new(Kiorg::new(cc, initial_dir)))
        }),
    )
}
