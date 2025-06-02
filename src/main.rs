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
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info,font=error,pdf_render=error,eframe=error,winit=error")
    });

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
            return kiorg::startup_error::StartupErrorApp::show_error_dialog(
                format!("Directory '{}' does not exist", dir.display()),
                "Filesystem Error".to_string(),
                Some(format!("Requested directory: {}", dir.display())),
            );
        }

        if !dir.is_dir() {
            return kiorg::startup_error::StartupErrorApp::show_error_dialog(
                format!("'{}' is not a directory", dir.display()),
                "Filesystem Error".to_string(),
                Some(format!("Path provided: {}", dir.display())),
            );
        }

        // Canonicalize the path to get absolute path
        let canonical_dir = match fs::canonicalize(&dir) {
            Ok(path) => path,
            Err(e) => {
                return kiorg::startup_error::StartupErrorApp::show_error_dialog(
                    format!("Failed to canonicalize path '{}': {}", dir.display(), e),
                    "Permission Error".to_string(),
                    Some(format!("Path provided: {}", dir.display())),
                );
            }
        };

        Some(canonical_dir)
    } else {
        // No directory provided, use None to load from saved state
        None
    };

    // Check configuration first before creating the GUI
    if let Err(config_error) = kiorg::config::load_config_with_override(None) {
        // Configuration error detected - show error dialog with config path from error
        return kiorg::startup_error::StartupErrorApp::show_config_error_with_path(
            config_error.to_string(),
            &config_error.config_path().display().to_string(),
        );
    }

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

            let fonts = kiorg::font::load_system_fonts(egui::FontDefinitions::default());
            cc.egui_ctx.set_fonts(fonts);

            match Kiorg::new(cc, initial_dir) {
                Ok(app) => Ok(Box::new(app)),
                Err(e) => {
                    // For other errors during app initialization, we can't show a GUI dialog
                    // since we're already in the eframe context, so we still exit
                    eprintln!("Error initializing Kiorg: {e}");
                    std::process::exit(1);
                }
            }
        }),
    )
}
