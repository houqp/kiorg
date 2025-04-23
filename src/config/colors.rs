use crate::utils::color::hex_to_color32;
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
    pub blue: Color32,
    pub orange: Color32,
}

impl AppColors {
    pub fn from_config(config: &ColorScheme) -> Self {
        Self {
            bg: hex_to_color32(&config.bg),
            bg_dim: hex_to_color32(&config.bg_dim),
            bg_light: hex_to_color32(&config.bg_light),
            fg: hex_to_color32(&config.fg),
            selected_bg: hex_to_color32(&config.selected_bg),
            gray: hex_to_color32(&config.gray),
            yellow: hex_to_color32(&config.yellow),
            blue: hex_to_color32(&config.blue),
            orange: hex_to_color32(&config.orange),
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
        visuals
    }
}

impl Default for AppColors {
    fn default() -> Self {
        Self::from_config(&ColorScheme::default())
    }
}

// Custom serialization for AppColors
impl Serialize for AppColors {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Convert to ColorScheme for serialization
        let scheme = ColorScheme {
            bg: format!("#{:02x}{:02x}{:02x}", self.bg.r(), self.bg.g(), self.bg.b()),
            bg_dim: format!(
                "#{:02x}{:02x}{:02x}",
                self.bg_dim.r(),
                self.bg_dim.g(),
                self.bg_dim.b()
            ),
            bg_light: format!(
                "#{:02x}{:02x}{:02x}",
                self.bg_light.r(),
                self.bg_light.g(),
                self.bg_light.b()
            ),
            fg: format!("#{:02x}{:02x}{:02x}", self.fg.r(), self.fg.g(), self.fg.b()),
            selected_bg: format!(
                "#{:02x}{:02x}{:02x}",
                self.selected_bg.r(),
                self.selected_bg.g(),
                self.selected_bg.b()
            ),
            gray: format!(
                "#{:02x}{:02x}{:02x}",
                self.gray.r(),
                self.gray.g(),
                self.gray.b()
            ),
            yellow: format!(
                "#{:02x}{:02x}{:02x}",
                self.yellow.r(),
                self.yellow.g(),
                self.yellow.b()
            ),
            blue: format!(
                "#{:02x}{:02x}{:02x}",
                self.blue.r(),
                self.blue.g(),
                self.blue.b()
            ),
            orange: format!(
                "#{:02x}{:02x}{:02x}",
                self.orange.r(),
                self.orange.g(),
                self.orange.b()
            ),
            red: "#fc5d7c".to_string(),
            purple: "#b39df3".to_string(),
            green: "#9ed072".to_string(),
            aqua: "#76cce0".to_string(),
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
}
