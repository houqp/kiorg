use crate::config::colors::AppColors;
use egui::Color32;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

// Theme key constants
pub const DARK_KIORG_KEY: &str = "dark_kiorg";
pub const LIGHT_ONEDARK_KEY: &str = "light_onedark";
pub const DARK_EVERFOREST_KEY: &str = "dark_everforest";
pub const LIGHT_EVERFOREST_KEY: &str = "light_everforest";
pub const MOLOKAI_KEY: &str = "molokai";
pub const DARK_TOKYONIGHT_KEY: &str = "dark_tokyonight";
pub const LIGHT_TOKYONIGHT_KEY: &str = "light_tokyonight";

// Static builtin themes - single source of truth
static DARK_KIORG_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    name: DARK_KIORG_KEY.to_string(),
    display_name: "Dark Kiorg".to_string(),
    colors: AppColors {
        bg: Color32::from_rgb(0x2c, 0x2e, 0x34),
        bg_light: Color32::from_rgb(0x3b, 0x3e, 0x48),
        bg_extreme: Color32::from_rgb(0x22, 0x22, 0x22),
        bg_fill: Color32::from_rgb(0x2d, 0x2d, 0x2d),
        bg_interactive_fill: Color32::from_rgb(0x3c, 0x3c, 0x3c),
        bg_active: Color32::from_rgb(0x37, 0x37, 0x37),
        fg: Color32::from_rgb(0xe2, 0xe2, 0xe3),
        fg_selected: Color32::from_rgb(0xe2, 0xe2, 0xe3),
        error: Color32::from_rgb(0xfc, 0x5d, 0x7c),
        warn: Color32::from_rgb(0xf3, 0x96, 0x60),
        highlight: Color32::from_rgb(0xe7, 0xc6, 0x64),
        success: Color32::from_rgb(0x9e, 0xd0, 0x72),
        link_underscore: Color32::from_rgb(0x76, 0xcc, 0xe0),
        fg_folder: Color32::from_rgb(0x7f, 0x84, 0xde),
        link_text: Color32::from_rgb(0xb3, 0x9d, 0xf3),
        bg_selected: Color32::from_rgb(0x45, 0x47, 0x5a),
        fg_light: Color32::from_rgb(0x7f, 0x84, 0x90),
    },
});

static LIGHT_ONEDARK_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    name: LIGHT_ONEDARK_KEY.to_string(),
    display_name: "Light One Dark".to_string(),
    colors: AppColors {
        bg: Color32::from_rgb(0xf0, 0xf0, 0xf0),
        bg_selected: Color32::from_rgb(0xea, 0xee, 0xf2),
        bg_light: Color32::from_rgb(0xd0, 0xd7, 0xde),
        bg_extreme: Color32::from_rgb(0xf8, 0xf8, 0xf8),
        bg_fill: Color32::from_rgb(0xdc, 0xdc, 0xdc),
        bg_interactive_fill: Color32::from_rgb(0xe6, 0xe6, 0xe6),
        bg_active: Color32::from_rgb(0xc9, 0xc9, 0xc9),
        fg: Color32::from_rgb(0x24, 0x29, 0x2f),
        fg_selected: Color32::from_rgb(0x24, 0x29, 0x2f),
        error: Color32::from_rgb(0xe4, 0x56, 0x49),
        warn: Color32::from_rgb(0xc1, 0x84, 0x01),
        highlight: Color32::from_rgb(0x40, 0x78, 0xf2),
        success: Color32::from_rgb(0x50, 0xa1, 0x4f),
        link_underscore: Color32::from_rgb(0x03, 0x66, 0xd6),
        fg_folder: Color32::from_rgb(0x6f, 0x42, 0xc1),
        link_text: Color32::from_rgb(0x03, 0x2f, 0x62),
        fg_light: Color32::from_rgb(0x58, 0x60, 0x69),
    },
});

static DARK_EVERFOREST_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    name: DARK_EVERFOREST_KEY.to_string(),
    display_name: "Dark Everforest".to_string(),
    colors: AppColors {
        bg: Color32::from_rgb(0x2d, 0x35, 0x3b),
        bg_selected: Color32::from_rgb(0x47, 0x52, 0x58),
        bg_light: Color32::from_rgb(0x3d, 0x48, 0x4d),
        bg_extreme: Color32::from_rgb(0x23, 0x2a, 0x2e),
        bg_fill: Color32::from_rgb(0x37, 0x41, 0x45),
        bg_interactive_fill: Color32::from_rgb(0x41, 0x4b, 0x50),
        bg_active: Color32::from_rgb(0x4a, 0x55, 0x5b),
        fg: Color32::from_rgb(0xd3, 0xc6, 0xaa),
        fg_selected: Color32::from_rgb(0xd3, 0xc6, 0xaa),
        error: Color32::from_rgb(0xe6, 0x7e, 0x80),
        warn: Color32::from_rgb(0xdb, 0xbc, 0x7f),
        highlight: Color32::from_rgb(0x7f, 0xbb, 0xb3),
        success: Color32::from_rgb(0xa7, 0xc0, 0x80),
        link_underscore: Color32::from_rgb(0x83, 0xc0, 0x92),
        fg_folder: Color32::from_rgb(0xa7, 0xc0, 0x80),
        link_text: Color32::from_rgb(0xd6, 0x99, 0xb6),
        fg_light: Color32::from_rgb(0x85, 0x92, 0x89),
    },
});

static LIGHT_EVERFOREST_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    name: LIGHT_EVERFOREST_KEY.to_string(),
    display_name: "Light Everforest".to_string(),
    colors: AppColors {
        bg: Color32::from_rgb(0xfd, 0xf6, 0xe3),
        bg_selected: Color32::from_rgb(0xf4, 0xf0, 0xd9),
        bg_light: Color32::from_rgb(0xf0, 0xea, 0xd2),
        bg_extreme: Color32::from_rgb(0xff, 0xfb, 0xef),
        bg_fill: Color32::from_rgb(0xef, 0xeb, 0xd4),
        bg_interactive_fill: Color32::from_rgb(0xe6, 0xe2, 0xcc),
        bg_active: Color32::from_rgb(0xdd, 0xd8, 0xc0),
        fg: Color32::from_rgb(0x5c, 0x6a, 0x72),
        fg_selected: Color32::from_rgb(0x5c, 0x6a, 0x72),
        error: Color32::from_rgb(0xf8, 0x55, 0x52),
        warn: Color32::from_rgb(0xdf, 0xa0, 0x00),
        highlight: Color32::from_rgb(0x8d, 0xa1, 0x01),
        success: Color32::from_rgb(0x8d, 0xa1, 0x01),
        link_underscore: Color32::from_rgb(0x35, 0xa7, 0x7c),
        fg_folder: Color32::from_rgb(0x3a, 0x94, 0xc5),
        link_text: Color32::from_rgb(0xdf, 0x69, 0xba),
        fg_light: Color32::from_rgb(0xa6, 0xb0, 0xa0),
    },
});

static MOLOKAI_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    name: MOLOKAI_KEY.to_string(),
    display_name: "Molokai".to_string(),
    colors: AppColors {
        bg: Color32::from_rgb(0x1b, 0x1d, 0x1e),
        bg_selected: Color32::from_rgb(0x40, 0x3d, 0x3d),
        bg_light: Color32::from_rgb(0x2d, 0x2e, 0x2e),
        bg_extreme: Color32::from_rgb(0x0f, 0x10, 0x11),
        bg_fill: Color32::from_rgb(0x23, 0x25, 0x26),
        bg_interactive_fill: Color32::from_rgb(0x2d, 0x2f, 0x30),
        bg_active: Color32::from_rgb(0x3a, 0x3c, 0x3d),
        fg: Color32::from_rgb(0xf8, 0xf8, 0xf2),
        fg_selected: Color32::from_rgb(0xf8, 0xf8, 0xf2),
        error: Color32::from_rgb(0xf9, 0x26, 0x72),
        warn: Color32::from_rgb(0xfd, 0x97, 0x1f),
        highlight: Color32::from_rgb(0xe6, 0xdb, 0x74),
        success: Color32::from_rgb(0xa6, 0xe2, 0x2e),
        link_underscore: Color32::from_rgb(0x66, 0xd9, 0xef),
        fg_folder: Color32::from_rgb(0xae, 0x81, 0xff),
        link_text: Color32::from_rgb(0xfd, 0x5f, 0xf0),
        fg_light: Color32::from_rgb(0x75, 0x71, 0x5e),
    },
});

static DARK_TOKYONIGHT_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    name: DARK_TOKYONIGHT_KEY.to_string(),
    display_name: "Dark Tokyo Night".to_string(),
    colors: AppColors {
        bg: Color32::from_rgb(0x1a, 0x1b, 0x26),
        bg_selected: Color32::from_rgb(0x2d, 0x3f, 0x76),
        bg_light: Color32::from_rgb(0x24, 0x28, 0x3b),
        bg_extreme: Color32::from_rgb(0x16, 0x16, 0x1e),
        bg_fill: Color32::from_rgb(0x1f, 0x23, 0x35),
        bg_interactive_fill: Color32::from_rgb(0x29, 0x2e, 0x42),
        bg_active: Color32::from_rgb(0x3b, 0x42, 0x61),
        fg: Color32::from_rgb(0xc0, 0xca, 0xf5),
        fg_selected: Color32::from_rgb(0xc0, 0xca, 0xf5),
        error: Color32::from_rgb(0xf7, 0x76, 0x8e),
        warn: Color32::from_rgb(0xe0, 0xaf, 0x68),
        highlight: Color32::from_rgb(0xbb, 0x9a, 0xf7),
        success: Color32::from_rgb(0x9e, 0xce, 0x6a),
        link_underscore: Color32::from_rgb(0x7d, 0xcf, 0xff),
        fg_folder: Color32::from_rgb(0x7a, 0xa2, 0xf7),
        link_text: Color32::from_rgb(0xad, 0x8e, 0xe6),
        fg_light: Color32::from_rgb(0x56, 0x5f, 0x89),
    },
});

static LIGHT_TOKYONIGHT_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    name: LIGHT_TOKYONIGHT_KEY.to_string(),
    display_name: "Light Tokyo Night".to_string(),
    colors: AppColors {
        bg: Color32::from_rgb(0xd5, 0xd6, 0xdb),
        bg_selected: Color32::from_rgb(0xe1, 0xe2, 0xf7),
        bg_light: Color32::from_rgb(0xcb, 0xcc, 0xd1),
        bg_extreme: Color32::from_rgb(0xe9, 0xe9, 0xed),
        bg_fill: Color32::from_rgb(0xdf, 0xe0, 0xe5),
        bg_interactive_fill: Color32::from_rgb(0xc9, 0xca, 0xd0),
        bg_active: Color32::from_rgb(0xb5, 0xb6, 0xbb),
        fg: Color32::from_rgb(0x34, 0x3b, 0x58),
        fg_selected: Color32::from_rgb(0x34, 0x3b, 0x58),
        error: Color32::from_rgb(0x8c, 0x43, 0x51),
        warn: Color32::from_rgb(0x8f, 0x5e, 0x15),
        highlight: Color32::from_rgb(0x5a, 0x4a, 0x78),
        success: Color32::from_rgb(0x48, 0x5e, 0x30),
        link_underscore: Color32::from_rgb(0x16, 0x67, 0x75),
        fg_folder: Color32::from_rgb(0x34, 0x54, 0x8a),
        link_text: Color32::from_rgb(0x5a, 0x4a, 0x78),
        fg_light: Color32::from_rgb(0x89, 0x90, 0xb3),
    },
});

// All builtin themes
static ALL_THEMES: LazyLock<Vec<&Theme>> = LazyLock::new(|| {
    vec![
        &DARK_KIORG_THEME,
        &LIGHT_ONEDARK_THEME,
        &DARK_EVERFOREST_THEME,
        &LIGHT_EVERFOREST_THEME,
        &MOLOKAI_THEME,
        &DARK_TOKYONIGHT_THEME,
        &LIGHT_TOKYONIGHT_THEME,
    ]
});

#[must_use]
pub fn get_default_theme() -> &'static Theme {
    &DARK_KIORG_THEME
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub display_name: String,
    pub colors: AppColors,
}

impl Theme {
    #[must_use]
    pub fn new(key: &str, display_name: &str, colors: AppColors) -> Self {
        Self {
            name: key.to_string(),
            display_name: display_name.to_string(),
            colors,
        }
    }

    #[must_use]
    pub const fn get_colors(&self) -> &AppColors {
        &self.colors
    }

    #[must_use]
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    #[must_use]
    pub fn theme_key(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn from_theme_key(key: &str) -> Option<&'static Self> {
        match key {
            DARK_KIORG_KEY => Some(&DARK_KIORG_THEME),
            LIGHT_ONEDARK_KEY => Some(&LIGHT_ONEDARK_THEME),
            DARK_EVERFOREST_KEY => Some(&DARK_EVERFOREST_THEME),
            LIGHT_EVERFOREST_KEY => Some(&LIGHT_EVERFOREST_THEME),
            MOLOKAI_KEY => Some(&MOLOKAI_THEME),
            DARK_TOKYONIGHT_KEY => Some(&DARK_TOKYONIGHT_THEME),
            LIGHT_TOKYONIGHT_KEY => Some(&LIGHT_TOKYONIGHT_THEME),
            _ => None,
        }
    }

    #[must_use]
    pub fn all_themes() -> &'static [&'static Self] {
        &ALL_THEMES
    }

    /// Get all available themes including custom themes from config
    pub fn all_themes_with_custom(config: &crate::config::Config) -> Vec<Theme> {
        let mut themes = Vec::new();

        // Add built-in themes
        for builtin_theme in Self::all_themes() {
            themes.push((*builtin_theme).clone());
        }

        // Add custom themes
        if let Some(custom_themes) = &config.custom_themes {
            for custom_theme in custom_themes {
                themes.push(custom_theme.clone());
            }
        }

        themes
    }

    /// Load colors based on theme name from config, with fallback logic
    pub fn load_colors_from_config(config: &crate::config::Config) -> AppColors {
        match (&config.colors, &config.theme) {
            // If explicit colors are provided in config, use them
            (Some(colors), _) => colors.clone(),
            // Otherwise, load colors based on theme name
            (None, Some(theme_name)) => {
                // First check if it's a custom theme
                if let Some(custom_themes) = &config.custom_themes {
                    if let Some(custom_theme) = custom_themes.iter().find(|t| t.name == *theme_name)
                    {
                        return custom_theme.colors.clone();
                    }
                }

                // Then check built-in themes
                let theme_selection =
                    Self::from_theme_key(theme_name).unwrap_or_else(get_default_theme);
                theme_selection.get_colors().clone()
            }
            // Fallback to default (should not happen due to theme initialization)
            (None, None) => get_default_theme().get_colors().clone(),
        }
    }
}
