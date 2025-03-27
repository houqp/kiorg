use egui::{self, Align2, Color32, Ui};
use chrono::{DateTime, Local};
use humansize::{format_size, BINARY};

use crate::models::dir_entry::DirEntry;
use crate::config::colors::AppColors;

pub const ROW_HEIGHT: f32 = 20.0;
const ICON_WIDTH: f32 = 24.0;
const DATE_WIDTH: f32 = 160.0;
const SIZE_WIDTH: f32 = 80.0;

pub fn draw_table_header(ui: &mut Ui, colors: &AppColors) {
    ui.style_mut().spacing.item_spacing.y = 2.0;
    
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), ROW_HEIGHT), egui::Sense::hover());
    let mut cursor = rect.left_top();
    
    let name_width = rect.width() - ICON_WIDTH - 10.0 - DATE_WIDTH - SIZE_WIDTH - 20.0;
    
    cursor.x += ICON_WIDTH + 10.0;
    
    for (text, width) in [
        ("Name", name_width),
        ("Modified", DATE_WIDTH),
        ("Size", SIZE_WIDTH),
    ] {
        ui.painter().text(
            cursor + egui::vec2(0.0, ROW_HEIGHT/2.0),
            Align2::LEFT_CENTER,
            text,
            egui::FontId::proportional(14.0),
            colors.gray
        );
        cursor.x += width;
    }
    
    ui.separator();
}

pub fn draw_entry_row(
    ui: &mut Ui,
    entry: &DirEntry,
    is_selected: bool,
    colors: &AppColors,
) -> bool {
    let row_height = 20.0;
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_height),
        egui::Sense::click(),
    );
    
    if is_selected {
        ui.painter().rect_filled(rect, 0.0, colors.selected_bg);
    }
    
    let mut cursor = rect.left_top();
    
    let icon_width = 24.0;
    let date_width = 180.0;
    let size_width = 80.0;
    let name_width = rect.width() - icon_width - 10.0 - date_width - size_width - 20.0;
    
    // Icon
    let icon = if entry.is_dir { "📁" } else { "📄" };
    let icon_color = if is_selected { Color32::WHITE } else { colors.gray };
    ui.painter().text(
        cursor + egui::vec2(10.0, row_height/2.0),
        Align2::LEFT_CENTER,
        icon,
        egui::FontId::proportional(14.0),
        icon_color
    );
    cursor.x += icon_width + 10.0;
    
    // Name with truncation
    let name_clip_rect = egui::Rect::from_min_size(
        cursor,
        egui::vec2(name_width, row_height)
    );
    
    let name_text = truncate_text(&entry.name, name_width);
    let name_color = if entry.is_dir { colors.blue } else { colors.fg };
    ui.painter().with_clip_rect(name_clip_rect).text(
        cursor + egui::vec2(0.0, row_height/2.0),
        Align2::LEFT_CENTER,
        &name_text,
        egui::FontId::proportional(14.0),
        name_color
    );
    cursor.x += name_width;
    
    // Modified date
    let modified_date = DateTime::<Local>::from(entry.modified)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let date_color = if is_selected { Color32::WHITE } else { colors.gray };
    ui.painter().text(
        cursor + egui::vec2(0.0, row_height/2.0),
        Align2::LEFT_CENTER,
        &modified_date,
        egui::FontId::proportional(14.0),
        date_color
    );
    cursor.x += date_width;
    
    // Size
    let size_str = if entry.is_dir {
        String::new()
    } else {
        format_size(entry.size, BINARY)
    };
    let size_color = if is_selected { Color32::WHITE } else { colors.gray };
    ui.painter().text(
        cursor + egui::vec2(0.0, row_height/2.0),
        Align2::LEFT_CENTER,
        &size_str,
        egui::FontId::proportional(14.0),
        size_color
    );
    
    response.clicked()
}

pub fn draw_parent_entry_row(
    ui: &mut Ui,
    entry: &DirEntry,
    is_selected: bool,
    colors: &AppColors,
) -> bool {
    let row_height = 20.0;
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_height),
        egui::Sense::click(),
    );
    
    if is_selected {
        ui.painter().rect_filled(rect, 0.0, colors.selected_bg);
    }
    
    let mut cursor = rect.left_top();
    
    let icon_width = 24.0;
    let name_width = rect.width() - icon_width - 10.0;
    
    // Icon
    let icon = if entry.is_dir { "📁" } else { "📄" };
    let icon_color = if is_selected { Color32::WHITE } else { colors.gray };
    ui.painter().text(
        cursor + egui::vec2(10.0, row_height/2.0),
        Align2::LEFT_CENTER,
        icon,
        egui::FontId::proportional(14.0),
        icon_color
    );
    cursor.x += icon_width + 10.0;
    
    // Name with truncation
    let name_clip_rect = egui::Rect::from_min_size(
        cursor,
        egui::vec2(name_width, row_height)
    );
    
    let name_text = truncate_text(&entry.name, name_width);
    let name_color = if entry.is_dir { colors.blue } else { colors.fg };
    ui.painter().with_clip_rect(name_clip_rect).text(
        cursor + egui::vec2(0.0, row_height/2.0),
        Align2::LEFT_CENTER,
        &name_text,
        egui::FontId::proportional(14.0),
        name_color
    );
    
    response.clicked()
}

fn truncate_text(text: &str, available_width: f32) -> String {
    let estimated_width = text.len() as f32 * 7.0;
    if estimated_width > available_width {
        let available_chars = ((available_width / 7.0) as usize).saturating_sub(3);
        if available_chars > 0 && available_chars < text.len() {
            let half = available_chars / 2;
            format!("{}...{}", 
                &text[..half],
                &text[text.len() - half..])
        } else {
            text.to_string()
        }
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("short", 100.0), "short");
        assert_eq!(
            truncate_text("very_long_filename.txt", 70.0),
            "very_...me.txt"
        );
    }
}