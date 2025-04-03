use chrono::{DateTime, Local};
use egui::{Align2, Color32, Ui};
use humansize::{format_size, BINARY};

use crate::config::colors::AppColors;
use crate::models::dir_entry::DirEntry;

pub const ROW_HEIGHT: f32 = 16.0;
const ICON_WIDTH: f32 = 24.0;
const DATE_WIDTH: f32 = 160.0;
const SIZE_WIDTH: f32 = 80.0;

#[derive(Debug)]
pub struct EntryRowParams<'a> {
    pub entry: &'a DirEntry,
    pub is_selected: bool,
    pub colors: &'a AppColors,
    pub rename_mode: bool,
    pub new_name: &'a mut String,
    pub rename_focus: bool,
    pub is_marked: bool,
    pub is_bookmarked: bool,
}

pub fn draw_table_header(ui: &mut Ui, colors: &AppColors) {
    ui.style_mut().spacing.item_spacing.y = 2.0;

    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), ROW_HEIGHT),
        egui::Sense::hover(),
    );
    let mut cursor = rect.left_top();

    let name_width = rect.width() - ICON_WIDTH - 10.0 - DATE_WIDTH - SIZE_WIDTH - 20.0;

    cursor.x += ICON_WIDTH + 10.0;

    for (text, width) in [
        ("Name", name_width),
        ("Modified", DATE_WIDTH),
        ("Size", SIZE_WIDTH),
    ] {
        ui.painter().text(
            cursor + egui::vec2(0.0, ROW_HEIGHT / 2.0),
            Align2::LEFT_CENTER,
            text,
            egui::FontId::proportional(12.0),
            colors.gray,
        );
        cursor.x += width;
    }

    ui.separator();
}

pub fn draw_entry_row(ui: &mut Ui, params: EntryRowParams<'_>) -> bool {
    let EntryRowParams {
        entry,
        is_selected,
        colors,
        rename_mode,
        new_name,
        rename_focus,
        is_marked,
        is_bookmarked,
    } = params;

    let row_height = 20.0;
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_height),
        egui::Sense::click(),
    );

    if is_marked {
        ui.painter().rect_filled(rect, 0.0, colors.bg_light);
    } else if is_selected {
        ui.painter().rect_filled(rect, 0.0, colors.selected_bg);
    }

    let mut cursor = rect.left_top();

    let icon_width = 24.0;
    let date_width = 180.0;
    let size_width = 80.0;
    let name_width = rect.width() - icon_width - 10.0 - date_width - size_width - 20.0;

    // Draw the base icon (folder or file)
    let base_icon = if entry.is_dir { "📁" } else { "📄" };
    let icon_color = if is_selected {
        Color32::WHITE
    } else {
        colors.gray
    };
    ui.painter().text(
        cursor + egui::vec2(10.0, row_height / 2.0),
        Align2::LEFT_CENTER,
        base_icon,
        egui::FontId::proportional(14.0),
        icon_color,
    );

    // Draw bookmark indicator as a separate element if needed
    if entry.is_dir && is_bookmarked {
        ui.painter().text(
            cursor + egui::vec2(22.0, row_height / 2.0),
            Align2::LEFT_CENTER,
            "🔖",
            egui::FontId::proportional(10.0), // Even smaller font for the bookmark icon
            if is_selected {
                Color32::WHITE
            } else {
                colors.gray.gamma_multiply(1.2)
            }, // More subtle color
        );
    }
    cursor.x += icon_width + 10.0;

    // Name with truncation or rename input
    let name_clip_rect = egui::Rect::from_min_size(cursor, egui::vec2(name_width, row_height));

    if rename_mode && is_selected {
        ui.painter().with_clip_rect(name_clip_rect).text(
            cursor + egui::vec2(0.0, row_height / 2.0),
            Align2::LEFT_CENTER,
            "> ",
            egui::FontId::proportional(14.0),
            colors.yellow,
        );
        cursor.x += 14.0; // Width of "> " prefix

        let text_edit = egui::TextEdit::singleline(new_name)
            .desired_width(name_width - 14.0)
            .font(egui::FontId::proportional(14.0))
            .text_color(colors.yellow)
            .frame(false);

        let text_edit_response = ui.allocate_ui_with_layout(
            egui::vec2(name_width - 14.0, row_height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| ui.add(text_edit),
        );

        if rename_focus {
            ui.ctx().memory_mut(|mem| {
                mem.request_focus(text_edit_response.inner.id);
            });
        }
    } else {
        let name_text = truncate_text(&entry.name, name_width);
        let name_color = if entry.is_dir { colors.blue } else { colors.fg };
        ui.painter().with_clip_rect(name_clip_rect).text(
            cursor + egui::vec2(0.0, row_height / 2.0),
            Align2::LEFT_CENTER,
            &name_text,
            egui::FontId::proportional(14.0),
            name_color,
        );
    }
    cursor.x += name_width;

    // Modified date
    let modified_date = DateTime::<Local>::from(entry.modified)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let date_color = if is_selected {
        Color32::WHITE
    } else {
        colors.gray
    };
    ui.painter().text(
        cursor + egui::vec2(0.0, row_height / 2.0),
        Align2::LEFT_CENTER,
        &modified_date,
        egui::FontId::proportional(14.0),
        date_color,
    );
    cursor.x += date_width;

    // Size
    let size_str = if entry.is_dir {
        String::new()
    } else {
        format_size(entry.size, BINARY)
    };
    let size_color = if is_selected {
        Color32::WHITE
    } else {
        colors.gray
    };
    ui.painter().text(
        cursor + egui::vec2(0.0, row_height / 2.0),
        Align2::LEFT_CENTER,
        &size_str,
        egui::FontId::proportional(14.0),
        size_color,
    );

    response.clicked()
}

pub fn draw_parent_entry_row(
    ui: &mut Ui,
    entry: &DirEntry,
    is_selected: bool,
    colors: &AppColors,
    is_bookmarked: bool,
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

    // Draw the base icon (folder or file)
    let base_icon = if entry.is_dir { "📁" } else { "📄" };
    let icon_color = if is_selected {
        Color32::WHITE
    } else {
        colors.gray
    };
    ui.painter().text(
        cursor + egui::vec2(10.0, row_height / 2.0),
        Align2::LEFT_CENTER,
        base_icon,
        egui::FontId::proportional(14.0),
        icon_color,
    );

    // Draw bookmark indicator as a separate element if needed
    if entry.is_dir && is_bookmarked {
        ui.painter().text(
            cursor + egui::vec2(22.0, row_height / 2.0),
            Align2::LEFT_CENTER,
            "🔖",
            egui::FontId::proportional(10.0), // Smaller font for the bookmark icon
            if is_selected {
                Color32::WHITE
            } else {
                colors.gray.gamma_multiply(1.2)
            }, // Subtle color
        );
    }
    cursor.x += icon_width + 10.0;

    // Name with truncation
    let name_text = truncate_text(&entry.name, name_width);
    let name_color = if entry.is_dir { colors.blue } else { colors.fg };
    ui.painter().text(
        cursor + egui::vec2(0.0, row_height / 2.0),
        Align2::LEFT_CENTER,
        &name_text,
        egui::FontId::proportional(14.0),
        name_color,
    );

    response.clicked()
}

fn truncate_text(text: &str, available_width: f32) -> String {
    // Approximate width of a character in pixels
    let char_width = 8.0;
    let max_chars = (available_width / char_width) as usize;

    if text.len() <= max_chars {
        return text.to_string();
    }

    let half_chars = (max_chars - 3) / 2;
    let start = &text[..half_chars];
    let end = &text[text.len() - half_chars..];

    format!("{}...{}", start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("short", 100.0), "short");
        assert_eq!(
            truncate_text("this_is_a_very_long_filename.txt", 80.0),
            "thi...txt"
        );
    }
}
