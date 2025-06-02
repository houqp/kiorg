use crate::config::colors::AppColors;
use crate::config::colors::ColorScheme;
use std::sync::LazyLock;

// Theme key constants
pub const DARK_KIORG_KEY: &str = "dark_kiorg";
pub const LIGHT_ONEDARK_KEY: &str = "light_onedark";

// Static builtin themes - single source of truth
static DARK_KIORG_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    key: DARK_KIORG_KEY.to_string(),
    display_name: "Dark Kiorg".to_string(),
    colors: AppColors::from_scheme(&ColorScheme {
        bg: "#2c2e34".to_string(),
        bg_light: "#3b3e48".to_string(),
        bg_extreme: "#222222".to_string(),
        bg_fill: "#2d2d2d".to_string(),
        bg_interactive_fill: "#3c3c3c".to_string(),
        bg_active: "#373737".to_string(),
        fg: "#e2e2e3".to_string(),
        fg_selected: "#e2e2e3".to_string(),
        error: "#fc5d7c".to_string(),
        warn: "#f39660".to_string(),
        highlight: "#e7c664".to_string(),
        success: "#9ed072".to_string(),
        link_underscore: "#76cce0".to_string(),
        fg_folder: "#7f84de".to_string(),
        link_text: "#b39df3".to_string(),
        bg_selected: "#45475a".to_string(),
        fg_light: "#7f8490".to_string(),
    }),
});

static LIGHT_ONEDARK_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    key: LIGHT_ONEDARK_KEY.to_string(),
    display_name: "Light One Dark".to_string(),
    colors: AppColors::from_scheme(&ColorScheme {
        bg: "#f0f0f0".to_string(),
        bg_selected: "#eaeef2".to_string(),
        bg_light: "#d0d7de".to_string(),
        bg_extreme: "#f8f8f8".to_string(),
        bg_fill: "#dcdcdc".to_string(),
        bg_interactive_fill: "#e6e6e6".to_string(),
        bg_active: "#c9c9c9".to_string(),
        fg: "#24292f".to_string(),
        fg_selected: "#24292f".to_string(),
        error: "#e45649".to_string(),
        warn: "#c18401".to_string(),
        highlight: "#4078f2".to_string(),
        success: "#50a14f".to_string(),
        link_underscore: "#0366d6".to_string(),
        fg_folder: "#6f42c1".to_string(),
        link_text: "#032f62".to_string(),
        fg_light: "#586069".to_string(),
    }),
});

// All builtin themes
static ALL_THEMES: LazyLock<Vec<&Theme>> =
    LazyLock::new(|| vec![&DARK_KIORG_THEME, &LIGHT_ONEDARK_THEME]);

#[must_use]
pub fn get_default_theme() -> &'static Theme {
    &DARK_KIORG_THEME
}

#[derive(Debug, Clone)]
pub struct Theme {
    key: String,
    display_name: String,
    colors: AppColors,
}

impl Theme {
    #[must_use]
    pub fn new(key: &str, display_name: &str, colors: AppColors) -> Self {
        Self {
            key: key.to_string(),
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
        &self.key
    }

    #[must_use]
    pub fn from_theme_key(key: &str) -> Option<&'static Self> {
        match key {
            DARK_KIORG_KEY => Some(&DARK_KIORG_THEME),
            LIGHT_ONEDARK_KEY => Some(&LIGHT_ONEDARK_THEME),
            _ => None,
        }
    }

    #[must_use]
    pub fn all_themes() -> &'static [&'static Self] {
        &ALL_THEMES
    }

    /// Load colors based on theme name from config, with fallback logic
    pub fn load_colors_from_config(config: &crate::config::Config) -> AppColors {
        match (&config.colors, &config.theme) {
            // If explicit colors are provided in config, use them
            (Some(color_scheme), _) => AppColors::from_scheme(color_scheme),
            // Otherwise, load colors based on theme name
            (None, Some(theme_name)) => {
                let theme_selection =
                    Self::from_theme_key(theme_name).unwrap_or_else(get_default_theme);
                theme_selection.get_colors().clone()
            }
            // Fallback to default (should not happen due to theme initialization)
            (None, None) => get_default_theme().get_colors().clone(),
        }
    }
}
