use egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ColorScheme {
    pub bg: String,
    pub bg_dim: String,
    pub bg_light: String,
    pub fg: String,
    pub red: String,
    pub orange: String,
    pub yellow: String,
    pub green: String,
    pub aqua: String,
    pub blue: String,
    pub purple: String,
    pub selected_bg: String,
    pub gray: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            bg: "#2c2e34".to_string(),
            bg_dim: "#33353f".to_string(),
            bg_light: "#3b3e48".to_string(),
            fg: "#e2e2e3".to_string(),
            red: "#fc5d7c".to_string(),
            orange: "#f39660".to_string(),
            yellow: "#e7c664".to_string(),
            green: "#9ed072".to_string(),
            aqua: "#76cce0".to_string(),
            blue: "#7f84de".to_string(),
            purple: "#b39df3".to_string(),
            selected_bg: "#45475a".to_string(),
            gray: "#7f8490".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppColors {
    pub fg: Color32,
    pub bg: Color32,
    pub bg_dim: Color32,
    pub bg_light: Color32,
    pub selected_bg: Color32,
    pub gray: Color32,
    pub yellow: Color32,
}

impl AppColors {
    pub fn from_config(config: &ColorScheme) -> Self {
        use crate::utils::color::hex_to_color32;
        
        Self {
            bg: hex_to_color32(&config.bg),
            bg_dim: hex_to_color32(&config.bg_dim),
            bg_light: hex_to_color32(&config.bg_light),
            fg: hex_to_color32(&config.fg),
            selected_bg: hex_to_color32(&config.selected_bg),
            gray: hex_to_color32(&config.gray),
            yellow: hex_to_color32(&config.yellow),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_scheme_default() {
        let scheme = ColorScheme::default();
        assert_eq!(scheme.bg, "#2c2e34");
        assert_eq!(scheme.fg, "#e2e2e3");
    }

    #[test]
    fn test_app_colors_from_config() {
        let scheme = ColorScheme::default();
        let colors = AppColors::from_config(&scheme);
        assert_eq!(colors.bg, Color32::from_rgb(0x2c, 0x2e, 0x34));
    }
} 