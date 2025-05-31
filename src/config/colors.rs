use egui::Color32;
use serde::{Deserialize, Serialize};

// Helper function to convert hex string to Color32
pub fn hex_to_color32(hex: &str) -> Color32 {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color32::from_rgb(r, g, b)
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ColorScheme {
    pub bg: String,
    // Background for marked elements
    pub bg_light: String,
    // Background for scrollbars and text edit areas including the search bar
    pub bg_extreme: String,
    pub fg: String,
    pub highlight: String,
    pub link_text: String,
    // Links and cursor
    pub link_underscore: String,
    pub bg_selected: String,
    // Background color for window title bar.
    pub bg_fill: String,
    // Background color for buttons.
    pub bg_interactive_fill: String,
    // Background color for elements that are being actively interacted with (e.g. clicked)
    pub bg_active: String,
    pub fg_selected: String,
    pub fg_light: String,
    // Folder names and Grid title
    pub fg_folder: String,
    pub success: String,
    pub warn: String,
    pub error: String,
}

#[derive(Clone, Debug)]
pub struct AppColors {
    pub fg: Color32,
    pub bg: Color32,
    pub bg_light: Color32,
    pub bg_extreme: Color32,
    pub bg_selected: Color32,
    pub bg_fill: Color32,
    pub bg_interactive_fill: Color32,
    pub bg_active: Color32,
    pub fg_selected: Color32,
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
    pub fn from_scheme(config: &ColorScheme) -> Self {
        Self {
            bg: hex_to_color32(&config.bg),
            bg_light: hex_to_color32(&config.bg_light),
            bg_extreme: hex_to_color32(&config.bg_extreme),
            bg_fill: hex_to_color32(&config.bg_fill),
            bg_active: hex_to_color32(&config.bg_active),
            bg_interactive_fill: hex_to_color32(&config.bg_interactive_fill),
            fg: hex_to_color32(&config.fg),
            bg_selected: hex_to_color32(&config.bg_selected),
            fg_selected: hex_to_color32(&config.fg_selected),
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

    pub fn to_color_scheme(&self) -> ColorScheme {
        ColorScheme {
            bg: color32_to_hex(self.bg),
            bg_light: color32_to_hex(self.bg_light),
            bg_extreme: color32_to_hex(self.bg_extreme),
            bg_active: color32_to_hex(self.bg_active),
            bg_fill: color32_to_hex(self.bg_fill),
            bg_interactive_fill: color32_to_hex(self.bg_interactive_fill),
            fg: color32_to_hex(self.fg),
            bg_selected: color32_to_hex(self.bg_selected),
            fg_selected: color32_to_hex(self.fg_selected),
            fg_light: color32_to_hex(self.fg_light),
            fg_folder: color32_to_hex(self.fg_folder),
            highlight: color32_to_hex(self.highlight),
            link_text: color32_to_hex(self.link_text),
            link_underscore: color32_to_hex(self.link_underscore),
            error: color32_to_hex(self.error),
            warn: color32_to_hex(self.warn),
            success: color32_to_hex(self.success),
        }
    }

    pub fn to_visuals(&self) -> egui::Visuals {
        let mut visuals = egui::Visuals::dark();

        visuals.window_shadow = egui::Shadow {
            offset: [4, 4],
            blur: 12,
            spread: 0,
            color: Color32::from_black_alpha(96),
        };
        visuals.popup_shadow = egui::Shadow {
            offset: [2, 2],
            blur: 6,
            spread: 0,
            color: Color32::from_black_alpha(96),
        };

        visuals.override_text_color = Some(self.fg);

        visuals.widgets.noninteractive.bg_fill = self.bg_light;
        visuals.widgets.noninteractive.weak_bg_fill = self.bg_light;
        visuals.widgets.noninteractive.fg_stroke.color = self.fg;
        // separator line color
        visuals.widgets.noninteractive.bg_stroke.color = self.bg_light;

        visuals.widgets.active.bg_fill = self.bg_active;
        visuals.widgets.active.weak_bg_fill = self.bg_active;
        visuals.widgets.active.fg_stroke.color = self.fg;

        // window title bar background
        visuals.widgets.open.bg_fill = self.bg_fill;
        visuals.widgets.open.weak_bg_fill = self.bg_fill;
        visuals.widgets.open.fg_stroke.color = self.fg;

        // buttons
        visuals.widgets.inactive.fg_stroke.color = self.fg;
        visuals.widgets.inactive.bg_fill = self.bg_interactive_fill;
        visuals.widgets.inactive.weak_bg_fill = self.bg_interactive_fill;

        visuals.widgets.hovered.fg_stroke.color = self.fg;
        visuals.widgets.hovered.bg_fill = self.bg_fill;
        visuals.widgets.hovered.weak_bg_fill = self.bg_fill;

        visuals.selection.bg_fill = self.bg_selected;

        // background for window popup
        visuals.window_fill = self.bg;
        visuals.panel_fill = self.bg;
        visuals.warn_fg_color = self.warn;
        visuals.error_fg_color = self.error;
        visuals.hyperlink_color = self.link_underscore;

        // scrollbar and text edit background
        visuals.extreme_bg_color = self.bg_extreme;
        visuals.text_cursor.stroke.color = self.link_underscore;

        visuals
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
        let scheme = self.to_color_scheme();
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
        Ok(AppColors::from_scheme(&scheme))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_colors_from_config() {
        // Create a test scheme to verify conversion works
        let test_scheme = ColorScheme {
            bg: "#2c2e34".to_string(),
            bg_light: "#3b3e48".to_string(),
            bg_extreme: "#1a1a1a".to_string(),
            fg: "#e2e2e3".to_string(),
            highlight: "#e7c664".to_string(),
            link_text: "#b39df3".to_string(),
            link_underscore: "#76cce0".to_string(),
            bg_selected: "#45475a".to_string(),
            bg_fill: "#2d2d2d".to_string(),
            bg_interactive_fill: "#3c3c3c".to_string(),
            bg_active: "#373737".to_string(),
            fg_selected: "#e2e2e3".to_string(),
            fg_light: "#7f8490".to_string(),
            fg_folder: "#7f84de".to_string(),
            success: "#9ed072".to_string(),
            warn: "#f39660".to_string(),
            error: "#fc5d7c".to_string(),
        };
        let converted_colors = AppColors::from_scheme(&test_scheme);
        assert_eq!(color32_to_hex(converted_colors.bg), test_scheme.bg);
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

    #[test]
    fn test_hex_to_color32() {
        assert_eq!(hex_to_color32("#ff0000"), Color32::from_rgb(255, 0, 0));
        assert_eq!(hex_to_color32("00ff00"), Color32::from_rgb(0, 255, 0));
        assert_eq!(hex_to_color32("#0000ff"), Color32::from_rgb(0, 0, 255));
        // Test invalid input
        assert_eq!(hex_to_color32("invalid"), Color32::from_rgb(0, 0, 0));
    }
}
