use egui::Color32;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// Helper function to convert hex string to Color32
// Returns an error for invalid hex strings
#[must_use]
pub fn hex_to_color32(hex: &str) -> Result<Color32, String> {
    let hex = hex.trim_start_matches('#');

    // Ensure hex string is at least 6 characters long
    if hex.len() < 6 {
        return Err(format!(
            "Hex color string '{}' is too short, expected at least 6 characters",
            hex
        ));
    }
    let r = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| format!("Failed to parse red component from '{}'", &hex[0..2]))?;
    let g = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| format!("Failed to parse green component from '{}'", &hex[2..4]))?;
    let b = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| format!("Failed to parse blue component from '{}'", &hex[4..6]))?;

    Ok(Color32::from_rgb(r, g, b))
}

// Helper function to convert Color32 to hex string
#[inline]
fn color32_to_hex(color: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
}

// Custom serialization for Color32 as hex string
fn serialize_color<S>(color: &Color32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&color32_to_hex(*color))
}

// Custom deserialization for Color32 from hex string
fn deserialize_color<'de, D>(deserializer: D) -> Result<Color32, D::Error>
where
    D: Deserializer<'de>,
{
    let hex = String::deserialize(deserializer)?;
    hex_to_color32(&hex).map_err(serde::de::Error::custom)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppColors {
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub fg: Color32,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub bg: Color32,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub bg_light: Color32,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub bg_extreme: Color32,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub bg_selected: Color32,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub bg_fill: Color32,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub bg_interactive_fill: Color32,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub bg_active: Color32,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub fg_selected: Color32,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub fg_light: Color32,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub fg_folder: Color32,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub highlight: Color32,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub link_text: Color32,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub link_underscore: Color32,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub warn: Color32,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub error: Color32,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub success: Color32,
}

impl AppColors {
    #[must_use]
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
        visuals.warn_fg_color = self.warn;
        visuals.error_fg_color = self.error;

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

#[cfg(test)]
mod tests {
    use super::*;

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
        // Valid color strings
        assert_eq!(hex_to_color32("#ff0000"), Ok(Color32::from_rgb(255, 0, 0)));
        assert_eq!(hex_to_color32("00ff00"), Ok(Color32::from_rgb(0, 255, 0)));
        assert_eq!(hex_to_color32("#0000ff"), Ok(Color32::from_rgb(0, 0, 255)));
        assert_eq!(
            hex_to_color32("#ffffff"),
            Ok(Color32::from_rgb(255, 255, 255))
        );
        assert_eq!(hex_to_color32("#000000"), Ok(Color32::from_rgb(0, 0, 0)));

        // Test various invalid inputs - all should return errors

        // Invalid characters
        assert!(hex_to_color32("invalid").is_err());
        assert!(hex_to_color32("#gggggg").is_err());
        assert!(hex_to_color32("not_a_color").is_err());
        assert!(hex_to_color32("#zzzzzz").is_err());

        // Short strings
        assert!(hex_to_color32("#12345").is_err());
        assert!(hex_to_color32("#123").is_err());
        assert!(hex_to_color32("#ff").is_err());
        assert!(hex_to_color32("").is_err());
        assert!(hex_to_color32("#").is_err());

        // Mixed valid/invalid characters
        assert!(hex_to_color32("#ff00gg").is_err());
        assert!(hex_to_color32("#12g456").is_err());

        // Valid uppercase
        assert_eq!(hex_to_color32("#FF0000"), Ok(Color32::from_rgb(255, 0, 0)));
        assert_eq!(
            hex_to_color32("#ABCDEF"),
            Ok(Color32::from_rgb(171, 205, 239))
        );

        // Test specific error messages
        assert_eq!(
            hex_to_color32("#12345").unwrap_err(),
            "Hex color string '12345' is too short, expected at least 6 characters"
        );
        assert_eq!(
            hex_to_color32("#gggggg").unwrap_err(),
            "Failed to parse red component from 'gg'"
        );
    }

    #[test]
    fn test_app_colors_serialization() {
        let app_colors = AppColors {
            bg: Color32::from_rgb(44, 46, 52),
            bg_light: Color32::from_rgb(59, 62, 72),
            bg_extreme: Color32::from_rgb(34, 34, 34),
            fg: Color32::from_rgb(226, 226, 227),
            highlight: Color32::from_rgb(231, 198, 100),
            link_text: Color32::from_rgb(179, 157, 243),
            link_underscore: Color32::from_rgb(118, 204, 224),
            bg_selected: Color32::from_rgb(69, 71, 90),
            bg_fill: Color32::from_rgb(45, 45, 45),
            bg_interactive_fill: Color32::from_rgb(60, 60, 60),
            bg_active: Color32::from_rgb(55, 55, 55),
            fg_selected: Color32::from_rgb(226, 226, 227),
            fg_light: Color32::from_rgb(127, 132, 144),
            fg_folder: Color32::from_rgb(127, 132, 222),
            success: Color32::from_rgb(158, 208, 114),
            warn: Color32::from_rgb(243, 150, 96),
            error: Color32::from_rgb(252, 93, 124),
        };

        // Test serialization
        let serialized = serde_json::to_string(&app_colors).expect("Failed to serialize");
        assert!(serialized.contains("\"bg\":\"#2c2e34\""));
        assert!(serialized.contains("\"fg\":\"#e2e2e3\""));

        // Test deserialization
        let deserialized: AppColors =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(deserialized.bg, app_colors.bg);
        assert_eq!(deserialized.fg, app_colors.fg);
        assert_eq!(deserialized.highlight, app_colors.highlight);
    }

    #[test]
    fn test_app_colors_deserialization_with_invalid_color() {
        // Simple test with one invalid color to verify error handling
        let json = "{\"bg\":\"invalid_color\",\"fg\":\"#ffffff\",\"bg_light\":\"#000000\",\"bg_extreme\":\"#000000\",\"bg_selected\":\"#000000\",\"bg_fill\":\"#000000\",\"bg_interactive_fill\":\"#000000\",\"bg_active\":\"#000000\",\"fg_selected\":\"#ffffff\",\"fg_light\":\"#808080\",\"fg_folder\":\"#808080\",\"highlight\":\"#ffff00\",\"link_text\":\"#0064ff\",\"link_underscore\":\"#0064ff\",\"warn\":\"#ffa500\",\"error\":\"#ff0000\",\"success\":\"#00ff00\"}";

        let result: Result<AppColors, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
