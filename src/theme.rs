use crate::config::colors::AppColors;
use crate::config::colors::ColorScheme;
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

static DARK_EVERFOREST_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    key: DARK_EVERFOREST_KEY.to_string(),
    display_name: "Dark Everforest".to_string(),
    colors: AppColors::from_scheme(&ColorScheme {
        bg: "#2d353b".to_string(),
        bg_selected: "#475258".to_string(),
        bg_light: "#3d484d".to_string(),
        bg_extreme: "#232a2e".to_string(),
        bg_fill: "#374145".to_string(),
        bg_interactive_fill: "#414b50".to_string(),
        bg_active: "#4a555b".to_string(),
        fg: "#d3c6aa".to_string(),
        fg_selected: "#d3c6aa".to_string(),
        error: "#e67e80".to_string(),
        warn: "#dbbc7f".to_string(),
        highlight: "#7fbbb3".to_string(),
        success: "#a7c080".to_string(),
        link_underscore: "#83c092".to_string(),
        fg_folder: "#a7c080".to_string(),
        link_text: "#d699b6".to_string(),
        fg_light: "#859289".to_string(),
    }),
});

static LIGHT_EVERFOREST_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    key: LIGHT_EVERFOREST_KEY.to_string(),
    display_name: "Light Everforest".to_string(),
    colors: AppColors::from_scheme(&ColorScheme {
        bg: "#fdf6e3".to_string(),
        bg_selected: "#f4f0d9".to_string(),
        bg_light: "#f0ead2".to_string(),
        bg_extreme: "#fffbef".to_string(),
        bg_fill: "#efebd4".to_string(),
        bg_interactive_fill: "#e6e2cc".to_string(),
        bg_active: "#ddd8c0".to_string(),
        fg: "#5c6a72".to_string(),
        fg_selected: "#5c6a72".to_string(),
        error: "#f85552".to_string(),
        warn: "#dfa000".to_string(),
        highlight: "#8da101".to_string(),
        success: "#8da101".to_string(),
        link_underscore: "#35a77c".to_string(),
        fg_folder: "#3a94c5".to_string(),
        link_text: "#df69ba".to_string(),
        fg_light: "#a6b0a0".to_string(),
    }),
});

static MOLOKAI_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    key: MOLOKAI_KEY.to_string(),
    display_name: "Molokai".to_string(),
    colors: AppColors::from_scheme(&ColorScheme {
        bg: "#1b1d1e".to_string(),
        bg_selected: "#403d3d".to_string(),
        bg_light: "#2d2e2e".to_string(),
        bg_extreme: "#0f1011".to_string(),
        bg_fill: "#232526".to_string(),
        bg_interactive_fill: "#2d2f30".to_string(),
        bg_active: "#3a3c3d".to_string(),
        fg: "#f8f8f2".to_string(),
        fg_selected: "#f8f8f2".to_string(),
        error: "#f92672".to_string(),
        warn: "#fd971f".to_string(),
        highlight: "#e6db74".to_string(),
        success: "#a6e22e".to_string(),
        link_underscore: "#66d9ef".to_string(),
        fg_folder: "#ae81ff".to_string(),
        link_text: "#fd5ff0".to_string(),
        fg_light: "#75715e".to_string(),
    }),
});

static DARK_TOKYONIGHT_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    key: DARK_TOKYONIGHT_KEY.to_string(),
    display_name: "Dark Tokyo Night".to_string(),
    colors: AppColors::from_scheme(&ColorScheme {
        bg: "#1a1b26".to_string(),
        bg_selected: "#2d3f76".to_string(),
        bg_light: "#24283b".to_string(),
        bg_extreme: "#16161e".to_string(),
        bg_fill: "#1f2335".to_string(),
        bg_interactive_fill: "#292e42".to_string(),
        bg_active: "#3b4261".to_string(),
        fg: "#c0caf5".to_string(),
        fg_selected: "#c0caf5".to_string(),
        error: "#f7768e".to_string(),
        warn: "#e0af68".to_string(),
        highlight: "#bb9af7".to_string(),
        success: "#9ece6a".to_string(),
        link_underscore: "#7dcfff".to_string(),
        fg_folder: "#7aa2f7".to_string(),
        link_text: "#ad8ee6".to_string(),
        fg_light: "#565f89".to_string(),
    }),
});

static LIGHT_TOKYONIGHT_THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    key: LIGHT_TOKYONIGHT_KEY.to_string(),
    display_name: "Light Tokyo Night".to_string(),
    colors: AppColors::from_scheme(&ColorScheme {
        bg: "#d5d6db".to_string(),
        bg_selected: "#e1e2f7".to_string(),
        bg_light: "#cbccd1".to_string(),
        bg_extreme: "#e9e9ed".to_string(),
        bg_fill: "#dfe0e5".to_string(),
        bg_interactive_fill: "#c9cad0".to_string(),
        bg_active: "#b5b6bb".to_string(),
        fg: "#343b58".to_string(),
        fg_selected: "#343b58".to_string(),
        error: "#8c4351".to_string(),
        warn: "#8f5e15".to_string(),
        highlight: "#5a4a78".to_string(),
        success: "#485e30".to_string(),
        link_underscore: "#166775".to_string(),
        fg_folder: "#34548a".to_string(),
        link_text: "#5a4a78".to_string(),
        fg_light: "#8990b3".to_string(),
    }),
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
