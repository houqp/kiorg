use crate::config::colors::AppColors;
use egui::hex_color;
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
        bg: hex_color!("#2c2e34"),
        bg_light: hex_color!("#3b3e48"),
        bg_extreme: hex_color!("#222222"),
        bg_fill: hex_color!("#2d2d2d"),
        bg_interactive_fill: hex_color!("#3c3c3c"),
        bg_active: hex_color!("#373737"),
        fg: hex_color!("#e2e2e3"),
        fg_selected: hex_color!("#e2e2e3"),
        error: hex_color!("#fc5d7c"),
        warn: hex_color!("#f39660"),
        highlight: hex_color!("#e7c664"),
        success: hex_color!("#9ed072"),
        link_underscore: hex_color!("#76cce0"),
        fg_folder: hex_color!("#7f84de"),
        link_text: hex_color!("#b39df3"),
        bg_selected: hex_color!("#45475a"),
        fg_light: hex_color!("#7f8490"),
    },
});

static LIGHT_ONEDARK_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    name: LIGHT_ONEDARK_KEY.to_string(),
    display_name: "Light One Dark".to_string(),
    colors: AppColors {
        bg: hex_color!("#f0f0f0"),
        bg_selected: hex_color!("#eaeef2"),
        bg_light: hex_color!("#d0d7de"),
        bg_extreme: hex_color!("#f8f8f8"),
        bg_fill: hex_color!("#dcdcdc"),
        bg_interactive_fill: hex_color!("#e6e6e6"),
        bg_active: hex_color!("#c9c9c9"),
        fg: hex_color!("#24292f"),
        fg_selected: hex_color!("#24292f"),
        error: hex_color!("#e45649"),
        warn: hex_color!("#c18401"),
        highlight: hex_color!("#4078f2"),
        success: hex_color!("#50a14f"),
        link_underscore: hex_color!("#0366d6"),
        fg_folder: hex_color!("#6f42c1"),
        link_text: hex_color!("#032f62"),
        fg_light: hex_color!("#586069"),
    },
});

static DARK_EVERFOREST_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    name: DARK_EVERFOREST_KEY.to_string(),
    display_name: "Dark Everforest".to_string(),
    colors: AppColors {
        bg: hex_color!("#2d353b"),
        bg_selected: hex_color!("#475258"),
        bg_light: hex_color!("#3d484d"),
        bg_extreme: hex_color!("#232a2e"),
        bg_fill: hex_color!("#374145"),
        bg_interactive_fill: hex_color!("#414b50"),
        bg_active: hex_color!("#4a555b"),
        fg: hex_color!("#d3c6aa"),
        fg_selected: hex_color!("#d3c6aa"),
        error: hex_color!("#e67e80"),
        warn: hex_color!("#dbbc7f"),
        highlight: hex_color!("#7fbbb3"),
        success: hex_color!("#a7c080"),
        link_underscore: hex_color!("#83c092"),
        fg_folder: hex_color!("#a7c080"),
        link_text: hex_color!("#d699b6"),
        fg_light: hex_color!("#859289"),
    },
});

static LIGHT_EVERFOREST_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    name: LIGHT_EVERFOREST_KEY.to_string(),
    display_name: "Light Everforest".to_string(),
    colors: AppColors {
        bg: hex_color!("#fdf6e3"),
        bg_selected: hex_color!("#f4f0d9"),
        bg_light: hex_color!("#f0ead2"),
        bg_extreme: hex_color!("#fffbef"),
        bg_fill: hex_color!("#efebd4"),
        bg_interactive_fill: hex_color!("#e6e2cc"),
        bg_active: hex_color!("#ddd8c0"),
        fg: hex_color!("#5c6a72"),
        fg_selected: hex_color!("#5c6a72"),
        error: hex_color!("#f85552"),
        warn: hex_color!("#dfa000"),
        highlight: hex_color!("#8da101"),
        success: hex_color!("#8da101"),
        link_underscore: hex_color!("#35a77c"),
        fg_folder: hex_color!("#3a94c5"),
        link_text: hex_color!("#df69ba"),
        fg_light: hex_color!("#a6b0a0"),
    },
});

static MOLOKAI_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    name: MOLOKAI_KEY.to_string(),
    display_name: "Molokai".to_string(),
    colors: AppColors {
        bg: hex_color!("#1b1d1e"),
        bg_selected: hex_color!("#403d3d"),
        bg_light: hex_color!("#2d2e2e"),
        bg_extreme: hex_color!("#0f1011"),
        bg_fill: hex_color!("#232526"),
        bg_interactive_fill: hex_color!("#2d2f30"),
        bg_active: hex_color!("#3a3c3d"),
        fg: hex_color!("#f8f8f2"),
        fg_selected: hex_color!("#f8f8f2"),
        error: hex_color!("#f92672"),
        warn: hex_color!("#fd971f"),
        highlight: hex_color!("#e6db74"),
        success: hex_color!("#a6e22e"),
        link_underscore: hex_color!("#66d9ef"),
        fg_folder: hex_color!("#ae81ff"),
        link_text: hex_color!("#fd5ff0"),
        fg_light: hex_color!("#75715e"),
    },
});

static DARK_TOKYONIGHT_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    name: DARK_TOKYONIGHT_KEY.to_string(),
    display_name: "Dark Tokyo Night".to_string(),
    colors: AppColors {
        bg: hex_color!("#1a1b26"),
        bg_selected: hex_color!("#2d3f76"),
        bg_light: hex_color!("#24283b"),
        bg_extreme: hex_color!("#16161e"),
        bg_fill: hex_color!("#1f2335"),
        bg_interactive_fill: hex_color!("#292e42"),
        bg_active: hex_color!("#3b4261"),
        fg: hex_color!("#c0caf5"),
        fg_selected: hex_color!("#c0caf5"),
        error: hex_color!("#f7768e"),
        warn: hex_color!("#e0af68"),
        highlight: hex_color!("#e0af68"),
        success: hex_color!("#9ece6a"),
        link_underscore: hex_color!("#7dcfff"),
        fg_folder: hex_color!("#7aa2f7"),
        link_text: hex_color!("#ad8ee6"),
        fg_light: hex_color!("#565f89"),
    },
});

static LIGHT_TOKYONIGHT_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    name: LIGHT_TOKYONIGHT_KEY.to_string(),
    display_name: "Light Tokyo Night".to_string(),
    colors: AppColors {
        bg: hex_color!("#d5d6db"),
        bg_selected: hex_color!("#e1e2f7"),
        bg_light: hex_color!("#cbccd1"),
        bg_extreme: hex_color!("#e9e9ed"),
        bg_fill: hex_color!("#dfe0e5"),
        bg_interactive_fill: hex_color!("#c9cad0"),
        bg_active: hex_color!("#b5b6bb"),
        fg: hex_color!("#343b58"),
        fg_selected: hex_color!("#343b58"),
        error: hex_color!("#8c4351"),
        warn: hex_color!("#8f5e15"),
        highlight: hex_color!("#8f5e15"),
        success: hex_color!("#485e30"),
        link_underscore: hex_color!("#166775"),
        fg_folder: hex_color!("#34548a"),
        link_text: hex_color!("#5a4a78"),
        fg_light: hex_color!("#8990b3"),
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
        match &config.theme {
            // Load colors based on theme name
            Some(theme_name) => {
                // First check if it's a custom theme
                if let Some(custom_themes) = &config.custom_themes
                    && let Some(custom_theme) = custom_themes.iter().find(|t| t.name == *theme_name)
                {
                    return custom_theme.colors.clone();
                }

                // Then check built-in themes
                let theme_selection =
                    Self::from_theme_key(theme_name).unwrap_or_else(get_default_theme);
                theme_selection.get_colors().clone()
            }
            // Fallback to default (should not happen due to theme initialization)
            None => get_default_theme().get_colors().clone(),
        }
    }
}
