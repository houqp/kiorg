use egui::Color32;

pub fn hex_to_color32(hex: &str) -> Color32 {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color32::from_rgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_color32() {
        assert_eq!(hex_to_color32("#ff0000"), Color32::from_rgb(255, 0, 0));
        assert_eq!(hex_to_color32("00ff00"), Color32::from_rgb(0, 255, 0));
        assert_eq!(hex_to_color32("#0000ff"), Color32::from_rgb(0, 0, 255));
        // Test invalid input
        assert_eq!(hex_to_color32("invalid"), Color32::from_rgb(0, 0, 0));
    }
}
