#[path = "mod/ui_test_helpers.rs"]
mod ui_test_helpers;

use egui::Color32;
use egui::Key;
use egui_kittest::kittest::Queryable;
use kiorg::config::{Config, colors::AppColors};
use kiorg::theme::Theme;
use kiorg::ui::popup::PopupType;
use std::fs;
use tempfile::tempdir;
use ui_test_helpers::TestHarnessBuilder;
use ui_test_helpers::create_harness_with_config_dir;

fn theme_exists(theme_name: &str, config: &kiorg::config::Config) -> bool {
    // Check built-in themes
    if kiorg::theme::Theme::from_theme_key(theme_name).is_some() {
        return true;
    }

    // Check custom themes
    if let Some(custom_themes) = &config.custom_themes {
        return custom_themes.iter().any(|t| t.name == theme_name);
    }

    false
}

#[test]
fn test_custom_themes_config_loading() {
    let config_temp_dir = tempfile::tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a config with custom themes
    let custom_themes = vec![Theme {
        name: "my_custom_theme".to_string(),
        display_name: "My Custom Theme".to_string(),
        colors: AppColors {
            bg: Color32::from_rgb(0xff, 0x00, 0x00),
            bg_light: Color32::from_rgb(0xff, 0x22, 0x22),
            bg_extreme: Color32::from_rgb(0xdd, 0x00, 0x00),
            fg: Color32::from_rgb(0xff, 0xff, 0xff),
            highlight: Color32::from_rgb(0x00, 0xff, 0x00),
            link_text: Color32::from_rgb(0x00, 0x00, 0xff),
            link_underscore: Color32::from_rgb(0x00, 0xff, 0xff),
            bg_selected: Color32::from_rgb(0xff, 0xff, 0x00),
            bg_fill: Color32::from_rgb(0xff, 0x44, 0x44),
            bg_interactive_fill: Color32::from_rgb(0xff, 0x66, 0x66),
            bg_active: Color32::from_rgb(0xff, 0x88, 0x88),
            fg_selected: Color32::from_rgb(0xff, 0xff, 0xff),
            fg_light: Color32::from_rgb(0xcc, 0xcc, 0xcc),
            fg_folder: Color32::from_rgb(0xaa, 0xaa, 0xaa),
            success: Color32::from_rgb(0x00, 0xaa, 0x00),
            warn: Color32::from_rgb(0xff, 0xaa, 0x00),
            error: Color32::from_rgb(0xaa, 0x00, 0x00),
        },
    }];

    let config = Config {
        theme: Some("my_custom_theme".to_string()),
        custom_themes: Some(custom_themes),
        ..Default::default()
    };

    // Save the config
    fs::create_dir_all(&config_dir).unwrap();
    kiorg::config::save_config_with_override(&config, Some(&config_dir)).unwrap();

    // Load the config back
    let loaded_config = kiorg::config::load_config_with_override(Some(&config_dir))
        .expect("Should load config successfully");

    // Verify custom themes are loaded
    assert!(loaded_config.custom_themes.is_some());
    let loaded_custom_themes = loaded_config.custom_themes.as_ref().unwrap();
    assert_eq!(loaded_custom_themes.len(), 1);
    assert_eq!(loaded_custom_themes[0].name, "my_custom_theme");

    // Verify the custom theme has correct values
    let custom_theme = &loaded_custom_themes[0];
    assert_eq!(custom_theme.colors.bg, Color32::from_rgb(0xff, 0x00, 0x00));
    assert_eq!(
        custom_theme.colors.highlight,
        Color32::from_rgb(0x00, 0xff, 0x00)
    );
    assert_eq!(custom_theme.display_name, "My Custom Theme");

    // Verify theme exists function works
    assert!(theme_exists("my_custom_theme", &loaded_config));
    assert!(theme_exists("dark_kiorg", &loaded_config)); // Built-in theme should still exist
    assert!(!theme_exists("nonexistent_theme", &loaded_config));
}

#[test]
fn test_custom_themes_color_loading() {
    let custom_themes = vec![Theme {
        name: "test_custom".to_string(),
        display_name: "Test Custom Theme".to_string(),
        colors: AppColors {
            bg: Color32::from_rgb(0x12, 0x34, 0x56),
            bg_light: Color32::from_rgb(0x23, 0x45, 0x67),
            bg_extreme: Color32::from_rgb(0x01, 0x23, 0x45),
            fg: Color32::from_rgb(0xff, 0xff, 0xff),
            highlight: Color32::from_rgb(0xff, 0x66, 0x00),
            link_text: Color32::from_rgb(0x00, 0x66, 0xff),
            link_underscore: Color32::from_rgb(0x00, 0xff, 0xcc),
            bg_selected: Color32::from_rgb(0xff, 0xcc, 0x00),
            bg_fill: Color32::from_rgb(0x34, 0x56, 0x78),
            bg_interactive_fill: Color32::from_rgb(0x45, 0x67, 0x89),
            bg_active: Color32::from_rgb(0x56, 0x78, 0x9a),
            fg_selected: Color32::from_rgb(0xff, 0xff, 0xff),
            fg_light: Color32::from_rgb(0xaa, 0xaa, 0xaa),
            fg_folder: Color32::from_rgb(0x99, 0x99, 0x99),
            success: Color32::from_rgb(0x00, 0xcc, 0x00),
            warn: Color32::from_rgb(0xcc, 0x66, 0x00),
            error: Color32::from_rgb(0xcc, 0x00, 0x00),
        },
    }];

    let config = Config {
        theme: Some("test_custom".to_string()),
        custom_themes: Some(custom_themes),
        ..Default::default()
    };

    // Test color loading from custom theme
    let colors = kiorg::theme::Theme::load_colors_from_config(&config);

    // Verify the colors match our custom theme
    assert_eq!(
        colors.bg,
        kiorg::config::colors::hex_to_color32("#123456").unwrap()
    );
    assert_eq!(
        colors.highlight,
        kiorg::config::colors::hex_to_color32("#ff6600").unwrap()
    );
    assert_eq!(
        colors.link_text,
        kiorg::config::colors::hex_to_color32("#0066ff").unwrap()
    );
}

#[test]
fn test_themes_popup_with_custom_themes() {
    // Create a temporary directory for the test files
    let temp_dir = tempdir().unwrap();
    let file1 = temp_dir.path().join("file1.txt");
    std::fs::write(&file1, "test content").unwrap();

    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a config file with custom themes
    let config_path = config_dir.join("config.toml");
    std::fs::create_dir_all(&config_dir).unwrap();

    // Create TOML config content with custom theme
    let config_content = r###"theme = "dark_kiorg"

[[custom_themes]]
name = "custom_test_theme"
display_name = "Custom Test Theme"

[custom_themes.colors]
bg = "#112233"
bg_light = "#223344"
bg_extreme = "#001122"
fg = "#ffffff"
highlight = "#ffaa33"
link_text = "#3366ff"
link_underscore = "#33ffaa"
bg_selected = "#aaff33"
bg_fill = "#334455"
bg_interactive_fill = "#445566"
bg_active = "#556677"
fg_selected = "#ffffff"
fg_light = "#bbbbbb"
fg_folder = "#888888"
success = "#33aa33"
warn = "#aa6633"
error = "#aa3333"
"###;

    std::fs::write(&config_path, config_content).unwrap();

    // Create the test harness with the custom config directory
    let mut harness = create_harness_with_config_dir(&temp_dir, config_temp_dir);

    // Verify the custom theme is loaded
    assert!(harness.state().config.custom_themes.is_some());
    assert_eq!(
        harness.state().config.custom_themes.as_ref().unwrap().len(),
        1
    );

    // Manually trigger themes popup (simulating menu click)
    harness.state_mut().show_popup = Some(PopupType::Themes("dark_kiorg".to_string()));
    harness.step();

    // Verify themes popup is open
    match &harness.state().show_popup {
        Some(PopupType::Themes(_)) => {
            // Popup is open, this is expected
        }
        _ => panic!("Themes popup should be open"),
    }

    // Test closing with Q (Exit shortcut)
    harness.key_press(Key::Q);
    harness.step();
    assert_eq!(
        harness.state().show_popup,
        None,
        "Themes popup should close with Q"
    );
}

#[test]
fn test_all_themes_with_custom() {
    let mut config = Config::default();

    // Test with no custom themes
    let themes = kiorg::theme::Theme::all_themes_with_custom(&config);
    let builtin_count = kiorg::theme::Theme::all_themes().len();
    assert_eq!(themes.len(), builtin_count);

    // Add custom themes
    let custom_themes = vec![
        Theme {
            name: "custom1".to_string(),
            display_name: "Custom Theme 1".to_string(),
            colors: AppColors {
                bg: Color32::from_rgb(0x00, 0x00, 0x00),
                bg_light: Color32::from_rgb(0x11, 0x11, 0x11),
                bg_extreme: Color32::from_rgb(0x00, 0x00, 0x00),
                fg: Color32::from_rgb(0xff, 0xff, 0xff),
                highlight: Color32::from_rgb(0xff, 0x00, 0x00),
                link_text: Color32::from_rgb(0x00, 0x00, 0xff),
                link_underscore: Color32::from_rgb(0x00, 0xff, 0xff),
                bg_selected: Color32::from_rgb(0xff, 0xff, 0x00),
                bg_fill: Color32::from_rgb(0x22, 0x22, 0x22),
                bg_interactive_fill: Color32::from_rgb(0x33, 0x33, 0x33),
                bg_active: Color32::from_rgb(0x44, 0x44, 0x44),
                fg_selected: Color32::from_rgb(0xff, 0xff, 0xff),
                fg_light: Color32::from_rgb(0xaa, 0xaa, 0xaa),
                fg_folder: Color32::from_rgb(0x99, 0x99, 0x99),
                success: Color32::from_rgb(0x00, 0xff, 0x00),
                warn: Color32::from_rgb(0xff, 0xff, 0x00),
                error: Color32::from_rgb(0xff, 0x00, 0x00),
            },
        },
        Theme {
            name: "custom2".to_string(),
            display_name: "Custom Theme 2".to_string(),
            colors: AppColors {
                bg: Color32::from_rgb(0xff, 0xff, 0xff),
                bg_light: Color32::from_rgb(0xee, 0xee, 0xee),
                bg_extreme: Color32::from_rgb(0xff, 0xff, 0xff),
                fg: Color32::from_rgb(0x00, 0x00, 0x00),
                highlight: Color32::from_rgb(0x00, 0x00, 0xff),
                link_text: Color32::from_rgb(0xff, 0x00, 0x00),
                link_underscore: Color32::from_rgb(0xff, 0x00, 0xff),
                bg_selected: Color32::from_rgb(0x00, 0x00, 0xff),
                bg_fill: Color32::from_rgb(0xdd, 0xdd, 0xdd),
                bg_interactive_fill: Color32::from_rgb(0xcc, 0xcc, 0xcc),
                bg_active: Color32::from_rgb(0xbb, 0xbb, 0xbb),
                fg_selected: Color32::from_rgb(0x00, 0x00, 0x00),
                fg_light: Color32::from_rgb(0x55, 0x55, 0x55),
                fg_folder: Color32::from_rgb(0x66, 0x66, 0x66),
                success: Color32::from_rgb(0x00, 0xaa, 0x00),
                warn: Color32::from_rgb(0xaa, 0x66, 0x00),
                error: Color32::from_rgb(0xaa, 0x00, 0x00),
            },
        },
    ];
    config.custom_themes = Some(custom_themes);

    // Test with custom themes
    let themes_with_custom = kiorg::theme::Theme::all_themes_with_custom(&config);
    assert_eq!(themes_with_custom.len(), builtin_count + 2);

    // Check that custom themes are included
    let custom_theme_names: Vec<&str> = themes_with_custom
        .iter()
        .filter(|theme| {
            // Check if theme key matches our custom theme names
            theme.theme_key() == "custom1" || theme.theme_key() == "custom2"
        })
        .map(|theme| theme.theme_key())
        .collect();

    assert!(custom_theme_names.contains(&"custom1"));
    assert!(custom_theme_names.contains(&"custom2"));
}

#[test]
fn test_custom_theme_fallback_to_builtin() {
    // Test config with non-existent theme falls back to built-in
    let config = Config {
        theme: Some("nonexistent_theme".to_string()),
        custom_themes: None,
        ..Default::default()
    };

    // Should fall back to default theme colors
    let colors = kiorg::theme::Theme::load_colors_from_config(&config);
    let default_colors = kiorg::theme::get_default_theme().get_colors();

    // Compare a few key colors to ensure fallback worked
    assert_eq!(colors.bg, default_colors.bg);
    assert_eq!(colors.fg, default_colors.fg);
    assert_eq!(colors.highlight, default_colors.highlight);
}

#[test]
fn test_themes_popup_shows_custom_themes() {
    // Create a temporary directory for the test files
    let temp_dir = tempdir().unwrap();
    let file1 = temp_dir.path().join("file1.txt");
    std::fs::write(&file1, "test content").unwrap();

    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a config file with custom themes
    let config_path = config_dir.join("config.toml");
    std::fs::create_dir_all(&config_dir).unwrap();

    // Create TOML config content with custom theme
    let config_content = r###"theme = "my_custom"

[[custom_themes]]
name = "my_custom"
display_name = "My Custom Theme"

[custom_themes.colors]
bg = "#000000"
bg_light = "#111111"
bg_extreme = "#000000"
fg = "#ffffff"
highlight = "#ff0000"
link_text = "#0000ff"
link_underscore = "#0000ff"
bg_selected = "#222222"
bg_fill = "#000000"
bg_interactive_fill = "#111111"
bg_active = "#333333"
fg_selected = "#ffffff"
fg_light = "#aaaaaa"
fg_folder = "#00ff00"
success = "#00ff00"
warn = "#ffff00"
error = "#ff0000"
"###;

    std::fs::write(&config_path, config_content).unwrap();

    // Create the test harness with the custom config directory
    let harness = create_harness_with_config_dir(&temp_dir, config_temp_dir);

    // Verify the custom theme is loaded properly
    assert_eq!(harness.state().config.theme, Some("my_custom".to_string()));
    assert!(harness.state().config.custom_themes.is_some());
    assert_eq!(
        harness.state().config.custom_themes.as_ref().unwrap().len(),
        1
    );

    // Test that all themes (including custom ones) are available
    let all_themes = kiorg::theme::Theme::all_themes_with_custom(&harness.state().config);
    let builtin_count = kiorg::theme::Theme::all_themes().len();
    assert_eq!(all_themes.len(), builtin_count + 1);

    // Verify our custom theme is present
    let custom_theme = all_themes.iter().find(|t| t.theme_key() == "my_custom");
    assert!(custom_theme.is_some());
    assert_eq!(custom_theme.unwrap().display_name(), "My Custom Theme");
}

#[test]
fn test_multiple_custom_themes_in_popup() {
    // Create a temporary directory for the test files
    let temp_dir = tempdir().unwrap();
    let file1 = temp_dir.path().join("file1.txt");
    std::fs::write(&file1, "test content").unwrap();

    // Create a temporary directory for the config
    let config_temp_dir = tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Create a config file with multiple custom themes
    let config_path = config_dir.join("config.toml");
    std::fs::create_dir_all(&config_dir).unwrap();

    // Create TOML config content with multiple custom themes
    let config_content = r###"theme = "dark_theme"

[[custom_themes]]
name = "dark_theme"
display_name = "Dark Custom Theme"

[custom_themes.colors]
bg = "#1a1a1a"
bg_light = "#2a2a2a"
bg_extreme = "#0a0a0a"
fg = "#ffffff"
highlight = "#ff6b6b"
link_text = "#4ecdc4"
link_underscore = "#4ecdc4"
bg_selected = "#333333"
bg_fill = "#1a1a1a"
bg_interactive_fill = "#2a2a2a"
bg_active = "#444444"
fg_selected = "#ffffff"
fg_light = "#cccccc"
fg_folder = "#95e1d3"
success = "#6bcf7f"
warn = "#fce38a"
error = "#ff6b6b"

[[custom_themes]]
name = "light_theme"
display_name = "Light Custom Theme"

[custom_themes.colors]
bg = "#f8f9fa"
bg_light = "#e9ecef"
bg_extreme = "#ffffff"
fg = "#212529"
highlight = "#007bff"
link_text = "#6610f2"
link_underscore = "#6610f2"
bg_selected = "#007bff"
bg_fill = "#f8f9fa"
bg_interactive_fill = "#e9ecef"
bg_active = "#dee2e6"
fg_selected = "#ffffff"
fg_light = "#6c757d"
fg_folder = "#28a745"
success = "#28a745"
warn = "#ffc107"
error = "#dc3545"

[[custom_themes]]
name = "purple_theme"
display_name = "Purple Custom Theme"

[custom_themes.colors]
bg = "#2d1b69"
bg_light = "#3d2b79"
bg_extreme = "#1d0b59"
fg = "#ffffff"
highlight = "#9c88ff"
link_text = "#ff88dc"
link_underscore = "#ff88dc"
bg_selected = "#4d3b89"
bg_fill = "#2d1b69"
bg_interactive_fill = "#3d2b79"
bg_active = "#5d4b99"
fg_selected = "#ffffff"
fg_light = "#b19cd9"
fg_folder = "#88ffb4"
success = "#88ffb4"
warn = "#ffdc88"
error = "#ff8888"
"###;

    std::fs::write(&config_path, config_content).unwrap();

    // Create the test harness with the custom config directory
    let mut harness = create_harness_with_config_dir(&temp_dir, config_temp_dir);

    // Verify the custom themes are loaded
    assert!(harness.state().config.custom_themes.is_some());
    let custom_themes = harness.state().config.custom_themes.as_ref().unwrap();
    assert_eq!(custom_themes.len(), 3);

    // Verify theme names and display names
    let theme_names: Vec<&str> = custom_themes.iter().map(|t| t.name.as_str()).collect();
    assert!(theme_names.contains(&"dark_theme"));
    assert!(theme_names.contains(&"light_theme"));
    assert!(theme_names.contains(&"purple_theme"));

    // Verify display names
    let display_names: Vec<&str> = custom_themes
        .iter()
        .map(|t| t.display_name.as_str())
        .collect();
    assert!(display_names.contains(&"Dark Custom Theme"));
    assert!(display_names.contains(&"Light Custom Theme"));
    assert!(display_names.contains(&"Purple Custom Theme"));

    // Test that all themes (including custom ones) are available
    let all_themes = kiorg::theme::Theme::all_themes_with_custom(&harness.state().config);
    let builtin_count = kiorg::theme::Theme::all_themes().len();
    assert_eq!(all_themes.len(), builtin_count + 3);

    // Manually trigger themes popup
    harness.state_mut().show_popup = Some(PopupType::Themes("dark_theme".to_string()));
    harness.step();

    // Verify themes popup is open
    match &harness.state().show_popup {
        Some(PopupType::Themes(_)) => {
            // Popup is open, this is expected
        }
        _ => panic!("Themes popup should be open"),
    }

    // Use query_by_label to verify each custom theme is present in the popup
    assert!(
        harness.query_by_label("Dark Custom Theme").is_some(),
        "Dark Custom Theme should be visible in the popup"
    );
    assert!(
        harness.query_by_label("Light Custom Theme").is_some(),
        "Light Custom Theme should be visible in the popup"
    );
    assert!(
        harness.query_by_label("Purple Custom Theme").is_some(),
        "Purple Custom Theme should be visible in the popup"
    );

    // Also verify some built-in themes are still present
    assert!(
        harness.query_by_label("Dark Kiorg").is_some(),
        "Built-in Dark Kiorg theme should still be visible"
    );
    assert!(
        harness.query_by_label("Light One Dark").is_some(),
        "Built-in Light One Dark theme should still be visible"
    );

    // Verify that a non-existent theme is not present
    assert!(
        harness.query_by_label("Non-existent Theme").is_none(),
        "Non-existent theme should not be found"
    );

    // Test color changes by selecting a custom theme
    // First, record the initial colors (should be from dark_theme since that's the config theme)
    let initial_colors = harness.state().colors.clone();

    // The config is set to use "dark_theme" which has highlight = "#ff6b6b"
    // Verify we start with the dark_theme colors
    assert_eq!(
        initial_colors.highlight,
        kiorg::config::colors::hex_to_color32("#ff6b6b").unwrap(),
        "Initial theme should have dark_theme highlight color"
    );

    // Navigate to the "Light Custom Theme" by pressing Down arrow
    // We need to find the index of "Light Custom Theme" in the themes list
    let all_themes = kiorg::theme::Theme::all_themes_with_custom(&harness.state().config);
    let light_theme_index = all_themes
        .iter()
        .position(|t| t.theme_key() == "light_theme")
        .expect("Light theme should exist");

    let dark_theme_index = all_themes
        .iter()
        .position(|t| t.theme_key() == "dark_theme")
        .expect("Dark theme should exist");

    // Navigate from dark_theme to light_theme
    let steps_to_light = if light_theme_index > dark_theme_index {
        light_theme_index - dark_theme_index
    } else {
        // Wrap around case
        all_themes.len() - dark_theme_index + light_theme_index
    };

    for _ in 0..steps_to_light {
        harness.key_press(Key::ArrowDown);
        harness.step();
    }

    // Apply the light theme by pressing Enter
    harness.key_press(Key::Enter);
    harness.step();

    // Verify the popup closed and theme was applied
    assert_eq!(
        harness.state().show_popup,
        None,
        "Themes popup should close after selecting a theme"
    );

    // Verify the colors changed to the light theme colors
    let new_colors = harness.state().colors.clone();
    assert_eq!(
        new_colors.highlight,
        kiorg::config::colors::hex_to_color32("#007bff").unwrap(),
        "Highlight color should change to light theme color"
    );
    assert_eq!(
        new_colors.bg,
        kiorg::config::colors::hex_to_color32("#f8f9fa").unwrap(),
        "Background color should change to light theme color"
    );
    assert_eq!(
        new_colors.fg,
        kiorg::config::colors::hex_to_color32("#212529").unwrap(),
        "Foreground color should change to light theme color"
    );

    // Verify the theme config was updated
    assert_eq!(
        harness.state().config.theme,
        Some("light_theme".to_string()),
        "Config theme should be updated to light_theme"
    );

    // Verify colors are different from initial
    assert_ne!(
        initial_colors.highlight, new_colors.highlight,
        "Colors should have changed from initial"
    );
    assert_ne!(
        initial_colors.bg, new_colors.bg,
        "Background colors should have changed from initial"
    );

    // Test switching to another custom theme (Purple Custom Theme)
    // Open the themes popup again
    harness.state_mut().show_popup = Some(PopupType::Themes("light_theme".to_string()));
    harness.step();

    // Navigate to the Purple Custom Theme
    let purple_theme_index = all_themes
        .iter()
        .position(|t| t.theme_key() == "purple_theme")
        .expect("Purple theme should exist");

    let light_theme_index = all_themes
        .iter()
        .position(|t| t.theme_key() == "light_theme")
        .expect("Light theme should exist");

    // Navigate from light_theme to purple_theme
    let steps_to_purple = if purple_theme_index > light_theme_index {
        purple_theme_index - light_theme_index
    } else {
        // Wrap around case
        all_themes.len() - light_theme_index + purple_theme_index
    };

    for _ in 0..steps_to_purple {
        harness.key_press(Key::ArrowDown);
        harness.step();
    }

    // Apply the purple theme by pressing Enter
    harness.key_press(Key::Enter);
    harness.step();

    // Verify the popup closed and purple theme was applied
    assert_eq!(
        harness.state().show_popup,
        None,
        "Themes popup should close after selecting purple theme"
    );

    // Verify the colors changed to the purple theme colors
    let purple_colors = harness.state().colors.clone();
    assert_eq!(
        purple_colors.highlight,
        kiorg::config::colors::hex_to_color32("#9c88ff").unwrap(),
        "Highlight color should change to purple theme color"
    );
    assert_eq!(
        purple_colors.bg,
        kiorg::config::colors::hex_to_color32("#2d1b69").unwrap(),
        "Background color should change to purple theme color"
    );
    assert_eq!(
        purple_colors.fg,
        kiorg::config::colors::hex_to_color32("#ffffff").unwrap(),
        "Foreground color should change to purple theme color"
    );
    assert_eq!(
        purple_colors.link_text,
        kiorg::config::colors::hex_to_color32("#ff88dc").unwrap(),
        "Link text color should change to purple theme color"
    );

    // Verify the theme config was updated to purple theme
    assert_eq!(
        harness.state().config.theme,
        Some("purple_theme".to_string()),
        "Config theme should be updated to purple_theme"
    );

    // Verify purple colors are different from both initial and light theme colors
    assert_ne!(
        initial_colors.highlight, purple_colors.highlight,
        "Purple colors should be different from initial colors"
    );
    assert_ne!(
        new_colors.highlight, purple_colors.highlight,
        "Purple colors should be different from light theme colors"
    );
    assert_ne!(
        new_colors.bg, purple_colors.bg,
        "Purple background should be different from light theme background"
    );
}

#[test]
fn test_invalid_custom_theme_config_values() {
    // Test that invalid color values cause config loading to fail
    let config_temp_dir = tempfile::tempdir().unwrap();
    let config_dir = config_temp_dir.path().to_path_buf();

    // Test 1: Invalid hex color values
    let config_content_invalid_colors = r###"theme = "invalid_colors_theme"

[[custom_themes]]
name = "invalid_colors_theme"
display_name = "Invalid Colors Theme"

[custom_themes.colors]
bg = "not_a_color"
bg_light = "#gggggg"
bg_extreme = "#12345"
fg = "#ffffff"
highlight = "invalid"
link_text = "#0000ff"
link_underscore = "#0000ff"
bg_selected = "#222222"
bg_fill = "#000000"
bg_interactive_fill = "#111111"
bg_active = "#333333"
fg_selected = "#ffffff"
fg_light = "#aaaaaa"
fg_folder = "#00ff00"
success = "#00ff00"
warn = "#ffff00"
error = "#ff0000"
"###;

    // Create config file with invalid colors
    let config_path = config_dir.join("config.toml");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(&config_path, config_content_invalid_colors).unwrap();

    // Load the config - this should fail with invalid colors
    let result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(
        result.is_err(),
        "Config loading should fail with invalid color values"
    );

    // Test 2: Valid hex colors should work
    let config_content_valid_colors = r###"theme = "valid_colors_theme"

[[custom_themes]]
name = "valid_colors_theme"
display_name = "Valid Colors Theme"

[custom_themes.colors]
bg = "#123456"
bg_light = "#234567"
bg_extreme = "#012345"
fg = "#ffffff"
highlight = "#ff6600"
link_text = "#0000ff"
link_underscore = "#0000ff"
bg_selected = "#222222"
bg_fill = "#000000"
bg_interactive_fill = "#111111"
bg_active = "#333333"
fg_selected = "#ffffff"
fg_light = "#aaaaaa"
fg_folder = "#00ff00"
success = "#00ff00"
warn = "#ffff00"
error = "#ff0000"
"###;

    std::fs::write(&config_path, config_content_valid_colors).unwrap();

    // Load the config - this should succeed with valid colors
    let loaded_config = kiorg::config::load_config_with_override(Some(&config_dir))
        .expect("Config should load with valid color values");

    // Verify the custom theme is loaded
    assert!(loaded_config.custom_themes.is_some());
    let custom_themes = loaded_config.custom_themes.as_ref().unwrap();
    assert_eq!(custom_themes.len(), 1);
    assert_eq!(custom_themes[0].name, "valid_colors_theme");

    // Verify colors are parsed correctly
    let colors = &custom_themes[0].colors;
    assert_eq!(colors.bg, egui::Color32::from_rgb(0x12, 0x34, 0x56));
    assert_eq!(colors.fg, egui::Color32::from_rgb(0xff, 0xff, 0xff));
    assert_eq!(colors.highlight, egui::Color32::from_rgb(0xff, 0x66, 0x00));

    // Test 3: Malformed TOML structure should also fail
    let config_content_malformed = r###"theme = "malformed_theme"

[[custom_themes
name = "missing_bracket"
display_name = "Malformed Theme"
"###;

    std::fs::write(&config_path, config_content_malformed).unwrap();

    // This should return an error due to malformed TOML
    let result = kiorg::config::load_config_with_override(Some(&config_dir));
    assert!(result.is_err(), "Malformed TOML should return an error");
}

// Main purpose of this test is to create snapshots for all the builtin themes
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

    #[cfg(feature = "snapshot")]
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

    #[cfg(feature = "snapshot")]
    {
        // Create GIF from PNG snapshots
        use image::{Frame, codecs::gif::GifEncoder};
        use std::fs::File;

        let gif_output_path = "tests/snapshots/theme_selection_animation.gif";
        let gif_file = File::create(gif_output_path).expect("Failed to create GIF file");
        let mut encoder = GifEncoder::new(gif_file);
        encoder
            .set_repeat(image::codecs::gif::Repeat::Infinite)
            .expect("Failed to set GIF repeat");

        // Load each PNG and add it as a frame to the GIF
        for png_file in &png_file_list {
            let png_path = format!("tests/snapshots/{png_file}");

            // Load the PNG image
            let img = image::open(&png_path)
                .unwrap_or_else(|_| panic!("Failed to open PNG file: {png_path}"));
            let rgba_img = img.to_rgba8();

            // Create a frame with delay (500ms per frame for good visibility)
            let frame = Frame::from_parts(
                rgba_img,
                0,                                          // left offset
                0,                                          // top offset
                image::Delay::from_numer_denom_ms(1000, 1), // 500ms delay
            );

            encoder
                .encode_frame(frame)
                .expect("Failed to encode GIF frame");
        }
    }
}
