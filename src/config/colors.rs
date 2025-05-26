use crate::utils::color::hex_to_color32;
use egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ColorScheme {
    pub bg: String,
    pub bg_dim: String,
    pub bg_light: String,
    pub fg: String,
    pub highlight: String,
    pub link_text: String,
    pub link_underscore: String,
    pub selected_bg: String,
    pub fg_light: String,
    pub fg_folder: String,
    pub success: String,
    pub warn: String,
    pub error: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            bg: "#2c2e34".to_string(),
            bg_dim: "#33353f".to_string(),
            bg_light: "#3b3e48".to_string(),
            fg: "#e2e2e3".to_string(),
            error: "#fc5d7c".to_string(),
            warn: "#f39660".to_string(),
            highlight: "#e7c664".to_string(),
            success: "#9ed072".to_string(),
            link_underscore: "#76cce0".to_string(),
            fg_folder: "#7f84de".to_string(),
            link_text: "#b39df3".to_string(),
            selected_bg: "#45475a".to_string(),
            fg_light: "#7f8490".to_string(),
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
    pub fg_light: Color32,
    pub fg_folder: Color32,
    pub highlight: Color32,
    pub link_text: Color32,
    pub link_underscore: Color32,
    pub warn: Color32,
    pub error: Color32,
    pub success: Color32,
}

impl AppColors {
    pub fn from_config(config: &ColorScheme) -> Self {
        Self {
            bg: hex_to_color32(&config.bg),
            bg_dim: hex_to_color32(&config.bg_dim),
            bg_light: hex_to_color32(&config.bg_light),
            fg: hex_to_color32(&config.fg),
            selected_bg: hex_to_color32(&config.selected_bg),
            fg_light: hex_to_color32(&config.fg_light),
            fg_folder: hex_to_color32(&config.fg_folder),
            highlight: hex_to_color32(&config.highlight),
            link_text: hex_to_color32(&config.link_text),
            error: hex_to_color32(&config.error),
            warn: hex_to_color32(&config.warn),
            success: hex_to_color32(&config.success),
            link_underscore: hex_to_color32(&config.link_underscore),
        }
    }

    pub fn to_visuals(&self) -> egui::Visuals {
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(self.fg);
        visuals.widgets.noninteractive.bg_fill = self.bg;
        visuals.widgets.inactive.bg_fill = self.bg_dim;
        visuals.widgets.hovered.bg_fill = self.bg_light;
        visuals.widgets.active.bg_fill = self.selected_bg;
        visuals.widgets.noninteractive.fg_stroke.color = self.fg;
        visuals.widgets.inactive.fg_stroke.color = self.fg;
        visuals.widgets.hovered.fg_stroke.color = self.fg;
        visuals.widgets.active.fg_stroke.color = self.fg;
        visuals.window_fill = self.bg;
        visuals.panel_fill = self.bg;
        visuals.warn_fg_color = self.warn;
        visuals.error_fg_color = self.error;
        visuals.hyperlink_color = self.link_underscore;
        visuals
    }
}

impl Default for AppColors {
    fn default() -> Self {
        Self::from_config(&ColorScheme::default())
    }
}

// Helper function to convert Color32 to hex string
#[inline]
fn color32_to_hex(color: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
}

// Custom serialization for AppColors
impl Serialize for AppColors {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Convert to ColorScheme for serialization
        let scheme = ColorScheme {
            bg: color32_to_hex(self.bg),
            bg_dim: color32_to_hex(self.bg_dim),
            bg_light: color32_to_hex(self.bg_light),
            fg: color32_to_hex(self.fg),
            selected_bg: color32_to_hex(self.selected_bg),
            fg_light: color32_to_hex(self.fg_light),
            fg_folder: color32_to_hex(self.fg_folder),
            highlight: color32_to_hex(self.highlight),
            link_text: color32_to_hex(self.link_text),
            link_underscore: color32_to_hex(self.link_underscore),
            error: color32_to_hex(self.error),
            warn: color32_to_hex(self.warn),
            success: color32_to_hex(self.success),
        };
        scheme.serialize(serializer)
    }
}

// Custom deserialization for AppColors
impl<'de> Deserialize<'de> for AppColors {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let scheme = ColorScheme::deserialize(deserializer)?;
        Ok(AppColors::from_config(&scheme))
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

    #[test]
    fn test_color32_to_hex() {
        let red = Color32::from_rgb(255, 0, 0);
        assert_eq!(color32_to_hex(red), "#ff0000");

        let green = Color32::from_rgb(0, 255, 0);
        assert_eq!(color32_to_hex(green), "#00ff00");

        let blue = Color32::from_rgb(0, 0, 255);
        assert_eq!(color32_to_hex(blue), "#0000ff");

        let custom = Color32::from_rgb(0x2c, 0x2e, 0x34);
        assert_eq!(color32_to_hex(custom), "#2c2e34");
    }
}
