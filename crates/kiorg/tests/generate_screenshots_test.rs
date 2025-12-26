#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

#[cfg(feature = "snapshot")]
use egui::Key;
#[cfg(feature = "snapshot")]
use egui_kittest::kittest::Queryable;
#[cfg(feature = "snapshot")]
use kiorg::ui::popup::PopupType;
#[cfg(feature = "snapshot")]
use tempfile::tempdir;

#[cfg(feature = "snapshot")]
use ui_test_helpers::*;

#[cfg(feature = "snapshot")]
use kiorg::models::preview_content::PreviewContent;

/// Helper function to create a GIF from a list of PNG snapshots
#[cfg(feature = "snapshot")]
fn create_gif_from_snapshots(png_file_list: &[String], output_path: &str, delay_ms: u32) {
    use image::{Frame, codecs::gif::GifEncoder};
    use std::fs::File;

    let gif_file = File::create(output_path).expect("Failed to create GIF file");
    let mut encoder = GifEncoder::new(gif_file);
    encoder
        .set_repeat(image::codecs::gif::Repeat::Infinite)
        .expect("Failed to set GIF repeat");

    // Load each PNG and add it as a frame to the GIF
    for png_file in png_file_list {
        let png_path = format!("tests/snapshots/{png_file}");

        // Load the PNG image
        let img = image::open(&png_path)
            .unwrap_or_else(|_| panic!("Failed to open PNG file: {png_path}"));
        let rgba_img = img.to_rgba8();

        // Create a frame with the specified delay
        let frame = Frame::from_parts(
            rgba_img,
            0, // left offset
            0, // top offset
            image::Delay::from_numer_denom_ms(delay_ms, 1),
        );

        encoder
            .encode_frame(frame)
            .expect("Failed to encode GIF frame");
    }
}

// Main purpose of this test is to create snapshots for all the builtin themes
#[cfg(feature = "snapshot")]
#[test]
fn test_snapshot_all_themes() {
    // Create a temporary directory for the test files
    let temp_dir = tempdir().unwrap();
    let file1 = temp_dir.path().join("file1.txt");
    std::fs::write(&file1, "test content").unwrap();

    // Create a temporary directory for the config (no custom themes, just built-ins)
    let config_temp_dir = tempdir().unwrap();

    // Create the test harness with default config (only built-in themes)
    let mut harness = TestHarnessBuilder::new()
        .with_temp_dir(&temp_dir)
        .with_config_dir(config_temp_dir)
        .with_window_size(egui::Vec2::new(800.0, 400.0))
        .build();

    // Get all built-in themes dynamically
    let all_builtin_themes = kiorg::theme::Theme::all_themes();
    let builtin_theme_info: Vec<(&str, &str)> = all_builtin_themes
        .iter()
        .map(|theme| (theme.theme_key(), theme.display_name()))
        .collect();

    // Open themes popup once
    let default_theme_key = kiorg::theme::get_default_theme().theme_key();
    harness.state_mut().show_popup = Some(PopupType::Themes(default_theme_key.to_string()));
    harness.step();

    // Verify themes popup is open
    match &harness.state().show_popup {
        Some(PopupType::Themes(_)) => {
            // Popup is open, this is expected
        }
        _ => panic!("Themes popup should be open"),
    }

    // Verify all built-in themes are present in the popup using query_by_label
    for (_, display_name) in &builtin_theme_info {
        assert!(
            harness.query_by_label(display_name).is_some(),
            "Built-in theme '{display_name}' should be visible in the popup"
        );
    }
    harness.step();
    // NOTE: the 2nd step is needed so that the popup can be rendered at the right location
    harness.step();

    let mut png_file_list = vec![];

    for (expected_index, (theme_key, _display_name)) in builtin_theme_info.iter().enumerate() {
        // If this is not the first theme, navigate to it using arrow down
        if expected_index > 0 {
            harness.key_press(Key::ArrowDown);
            harness.step();
        }

        #[cfg(feature = "snapshot")]
        {
            let file_name = format!("theme_selection-{expected_index:02}-{theme_key}");
            harness.snapshot(file_name.as_str());
            png_file_list.push(format!("{file_name}.png"));
        }

        // Verify popup is still open during navigation and contains the expected theme key
        match &harness.state().show_popup {
            Some(PopupType::Themes(current_theme_key)) => {
                assert_eq!(
                    current_theme_key, theme_key,
                    "Theme key in popup should match expected theme key"
                );
            }
            _ => panic!("Themes popup should remain open during navigation"),
        }
    }

    {
        create_gif_from_snapshots(
            &png_file_list,
            "tests/snapshots/theme_selection_animation.gif",
            1000,
        );
    }
}

// Test to showcase different preview features in a GIF
#[cfg(feature = "snapshot")]
#[test]
fn test_preview_features_showcase() {
    image_extras::register();

    let temp_dir = tempdir().unwrap();

    // Create various file types to showcase preview features

    // 1. Rust source code file
    let rust_file = temp_dir.path().join("example.rs");
    std::fs::write(
        &rust_file,
        r#"// Example Rust source code
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("hello", "world");
    map.insert("foo", "bar");
    
    for (key, value) in &map {
        println!("{}: {}", key, value);
    }
}

#[derive(Debug, Clone)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    
    fn distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}
"#,
    )
    .unwrap();

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();

    // 2. PNG image - copy from assets
    let png_file = temp_dir.path().join("icon.png");
    let source_png = workspace_root.join("assets/icons/1024x1024@2x.png");
    std::fs::copy(&source_png, &png_file).unwrap();

    // 3. ZIP file (was EPUB)
    let zip_file = temp_dir.path().join("archive.zip");
    create_demo_zip(&zip_file);

    // 4. TOML config file - copy from Cargo.toml
    let toml_file = temp_dir.path().join("config.toml");
    let source_toml = manifest_dir.join("Cargo.toml");
    std::fs::copy(&source_toml, &toml_file).unwrap();

    // 5. PDF file
    let pdf_file = temp_dir.path().join("document.pdf");
    create_test_pdf(&pdf_file, 3);

    // Create the test harness with a larger window for better preview visibility
    let config_temp_dir = tempdir().unwrap();
    let mut harness = TestHarnessBuilder::new()
        .with_temp_dir(&temp_dir)
        .with_config_dir(config_temp_dir)
        .with_window_size(egui::Vec2::new(1200.0, 800.0))
        .build();

    // Initial step to render the file list
    harness.step();
    // Key press to trigger preview for the first entry
    harness.key_press(Key::ArrowDown);
    harness.step();
    harness.key_press(Key::ArrowUp);
    harness.step();

    {
        let mut png_file_list = vec![];

        for index in 0..5 {
            // Navigate to the file if not the first one
            if index > 0 {
                harness.key_press(Key::ArrowDown);
                harness.step();
            }

            // Wait for preview content to finish loading
            wait_for_condition(|| {
                harness.step();
                // Check if preview content is no longer loading
                !matches!(
                    &harness.state().preview_content,
                    Some(PreviewContent::Loading(..))
                )
            });
            harness.step();
            harness.step();
            harness.step();

            {
                let file_name = format!("preview_features-{:02}", index);
                harness.snapshot(file_name.as_str());
                png_file_list.push(format!("{file_name}.png"));
            }
        }

        create_gif_from_snapshots(
            &png_file_list,
            "tests/snapshots/preview_features_showcase.gif",
            1500,
        );
    }
}
