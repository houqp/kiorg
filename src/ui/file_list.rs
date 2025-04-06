use chrono::{DateTime, Local};
use egui::{Align2, Color32, Ui};
use humansize::{format_size, BINARY};

use crate::config::colors::AppColors;
use crate::models::dir_entry::DirEntry;
use crate::models::tab::{SortColumn, SortOrder};
use crate::ui::style::{HEADER_FONT_SIZE, HEADER_ROW_HEIGHT};

const ICON_SIZE: f32 = 14.0;
const ICON_WIDTH: f32 = 22.0;
const HORIZONTAL_PADDING: f32 = 10.0;
const INTER_COLUMN_PADDING: f32 = 10.0; // Explicit padding between columns
const MODIFIED_DATE_WIDTH: f32 = 160.0;
const FILE_SIZE_WIDTH: f32 = 80.0;
pub const ROW_HEIGHT: f32 = 20.0;

pub struct TableHeaderParams<'a> {
    pub colors: &'a AppColors,
    pub sort_column: &'a SortColumn,
    pub sort_order: &'a SortOrder,
    pub on_sort: &'a mut dyn FnMut(SortColumn),
}

pub fn draw_table_header(ui: &mut Ui, params: &mut TableHeaderParams) {
    ui.style_mut().spacing.item_spacing.y = 2.0;

    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), HEADER_ROW_HEIGHT),
        egui::Sense::hover(), // Sense hover on the whole row for potential background effects
    );
    let mut cursor = rect.left_top();

    // Calculate total fixed width (icon + date + size + paddings)
    let fixed_width_total =
        ICON_WIDTH
            + HORIZONTAL_PADDING // Padding after icon
            + MODIFIED_DATE_WIDTH
            + INTER_COLUMN_PADDING // Padding between Modified and Size
            + FILE_SIZE_WIDTH
            + HORIZONTAL_PADDING; // Padding at the end

    // Name width takes remaining space
    let name_width = (rect.width() - fixed_width_total).max(0.0);

    // Advance cursor past the icon area
    cursor.x += ICON_WIDTH + HORIZONTAL_PADDING;

    // --- Draw Name Column ---
    let name_col_rect = egui::Rect::from_min_size(cursor, egui::vec2(name_width,HEADER_ROW_HEIGHT));
    draw_header_column(ui, params, name_col_rect, "Name", SortColumn::Name);
    cursor.x += name_width + INTER_COLUMN_PADDING; // Advance cursor including padding

    // --- Draw Modified Column ---
    let mod_col_rect = egui::Rect::from_min_size(cursor, egui::vec2(MODIFIED_DATE_WIDTH,HEADER_ROW_HEIGHT));
    draw_header_column(ui, params, mod_col_rect, "Modified", SortColumn::Modified);
    cursor.x += MODIFIED_DATE_WIDTH + INTER_COLUMN_PADDING; // Advance cursor including padding

    // --- Draw Size Column ---
    let size_col_rect = egui::Rect::from_min_size(cursor, egui::vec2(FILE_SIZE_WIDTH,HEADER_ROW_HEIGHT));
    draw_header_column(ui, params, size_col_rect, "Size", SortColumn::Size);
    // No cursor advance needed after the last column

    ui.separator();
}

// Helper function to draw a single header column
fn draw_header_column(
    ui: &mut Ui,
    params: &mut TableHeaderParams,
    col_rect: egui::Rect,
    text: &str,
    column: SortColumn,
) {
    let is_sorted = params.sort_column == &column;
    let sort_indicator = if is_sorted {
        match params.sort_order {
            SortOrder::Ascending => " \u{2B89}",
            SortOrder::Descending => " \u{2B8B}",
        }
    } else {
        ""
    };

    let text_color = if is_sorted {
        params.colors.yellow
    } else {
        // Use gray as base, hover will change cursor, not color here
        params.colors.gray
    };

    let header_text = format!("{}{}", text, sort_indicator);

    // Interact with the calculated rectangle
    let response = ui.interact(col_rect, ui.id().with(&column), egui::Sense::click());

    if response.clicked() {
        (params.on_sort)(column);
    }

    // Change cursor on hover
    if response.hovered() {
        response.on_hover_cursor(egui::CursorIcon::PointingHand);
    }

    // Draw the text using the painter, ensuring left alignment
    ui.painter().text(
        col_rect.left_center(), // Position text at the vertical center, horizontal left
        Align2::LEFT_CENTER,
        header_text,
        egui::FontId::proportional(HEADER_FONT_SIZE), // Match entry row font size
        text_color,
    );
}

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

fn draw_icon(ui: &mut Ui, cursor: egui::Pos2, is_dir: bool, is_selected: bool, colors: &AppColors, is_bookmarked: bool) -> f32{
    // Draw the base icon (folder or file)
    let base_icon = if is_dir { "📁" } else { "📄" };
    let icon_color = if is_selected {
        Color32::WHITE
    } else {
        colors.gray
    };
    ui.painter().text(
        cursor + egui::vec2(HORIZONTAL_PADDING, ROW_HEIGHT / 2.0),
        Align2::LEFT_CENTER,
        base_icon,
        egui::FontId::proportional(ICON_SIZE),
        icon_color,
    );

    // Draw bookmark indicator as a separate element if needed
    // Position it slightly offset within the icon area for better look
    if is_dir && is_bookmarked {
        ui.painter().text(
            cursor + egui::vec2(2.0, ROW_HEIGHT * 0.5), // Adjusted position
            Align2::LEFT_CENTER,
            "🔖",
            egui::FontId::proportional(ICON_SIZE * 0.7), // Even smaller font for the bookmark icon
            if is_selected {
                Color32::WHITE
            } else {
                colors.gray.gamma_multiply(1.2)
            }, // More subtle color
        );
    }

    ICON_WIDTH + HORIZONTAL_PADDING
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

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), ROW_HEIGHT),
        egui::Sense::click(),
    );

    if is_marked {
        ui.painter().rect_filled(rect, 0.0, colors.bg_light);
    } else if is_selected {
        ui.painter().rect_filled(rect, 0.0, colors.selected_bg);
    }

    let mut cursor = rect.left_top();

    // Calculate total fixed width (icon + date + size + paddings) - same as header
    let fixed_width_total =
        ICON_WIDTH
            + HORIZONTAL_PADDING // Padding after icon
            + MODIFIED_DATE_WIDTH
            + INTER_COLUMN_PADDING // Padding between Modified and Size
            + FILE_SIZE_WIDTH
            + HORIZONTAL_PADDING; // Padding at the end

    // Name width takes remaining space
    let name_width = (rect.width() - fixed_width_total).max(0.0);

    cursor.x += draw_icon(ui, cursor, entry.is_dir, is_selected, colors, is_bookmarked);

    // --- Draw Name Column ---
    let name_clip_rect = egui::Rect::from_min_size(cursor, egui::vec2(name_width, ROW_HEIGHT));
    if rename_mode && is_selected {
        ui.painter().with_clip_rect(name_clip_rect).text(
            cursor + egui::vec2(0.0, ROW_HEIGHT / 2.0),
            Align2::LEFT_CENTER,
            "> ",
            egui::FontId::proportional(14.0),
            colors.yellow,
        );
        let rename_cursor_start = cursor.x + 14.0; // Width of "> " prefix

        let text_edit = egui::TextEdit::singleline(new_name)
            .desired_width(name_width - 14.0)
            .font(egui::FontId::proportional(14.0))
            .text_color(colors.yellow)
            .frame(false);

        // Define the rectangle for the text edit
        let text_edit_rect = name_clip_rect.with_min_x(rename_cursor_start);
        
        // Place the TextEdit widget within the specific rectangle using ui.put
        let text_edit_response = ui.put(text_edit_rect, text_edit);

        if rename_focus {
            ui.ctx().memory_mut(|mem| {
                mem.request_focus(text_edit_response.id); // Focus the TextEdit response ID
            });
        }
    } else {
        let name_text = truncate_text(&entry.name, name_width);
        let name_color = if entry.is_dir { colors.blue } else { colors.fg };
        ui.painter().with_clip_rect(name_clip_rect).text(
            cursor + egui::vec2(0.0, ROW_HEIGHT / 2.0),
            Align2::LEFT_CENTER,
            &name_text,
            egui::FontId::proportional(14.0),
            name_color,
        );
    }
    cursor.x += name_width + INTER_COLUMN_PADDING; // Advance cursor including padding

    // --- Draw Modified Column ---
    let modified_date = DateTime::<Local>::from(entry.modified)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let date_color = if is_selected {
        Color32::WHITE
    } else {
        colors.gray
    };
    ui.painter().text(
        cursor + egui::vec2(0.0, ROW_HEIGHT / 2.0),
        Align2::LEFT_CENTER,
        &modified_date,
        egui::FontId::proportional(14.0),
        date_color,
    );
    cursor.x += MODIFIED_DATE_WIDTH + INTER_COLUMN_PADDING; // Advance cursor including padding

    // --- Draw Size Column ---
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
        cursor + egui::vec2(0.0, ROW_HEIGHT / 2.0),
        Align2::LEFT_CENTER,
        &size_str,
        egui::FontId::proportional(14.0),
        size_color,
    );
    // No cursor advance needed after the last column

    response.clicked()
}

pub fn draw_parent_entry_row(
    ui: &mut Ui,
    entry: &DirEntry,
    is_selected: bool,
    colors: &AppColors,
    is_bookmarked: bool,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), ROW_HEIGHT),
        egui::Sense::click(),
    );

    if is_selected {
        ui.painter().rect_filled(rect, 0.0, colors.selected_bg);
    }

    let mut cursor = rect.left_top();

    let name_width = rect.width() - ICON_WIDTH;

    cursor.x += draw_icon(ui, cursor, entry.is_dir, is_selected, colors, is_bookmarked);

    // Name with truncation
    let name_text = truncate_text(&entry.name, name_width);
    let name_color = if entry.is_dir { colors.blue } else { colors.fg };
    ui.painter().text(
        cursor + egui::vec2(0.0, ROW_HEIGHT / 2.0),
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
