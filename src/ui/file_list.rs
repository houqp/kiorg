use egui::{Align2, Ui};

use crate::config::colors::AppColors;
use crate::models::dir_entry::DirEntry;
use crate::models::tab::{SortColumn, SortOrder};
use crate::ui::style::{HEADER_FONT_SIZE, HEADER_ROW_HEIGHT};

const ICON_SIZE: f32 = 14.0;
const ICON_WIDTH: f32 = 22.0;
const HORIZONTAL_PADDING: f32 = 10.0;
const INTER_COLUMN_PADDING: f32 = 10.0; // Explicit padding between columns
const MODIFIED_DATE_WIDTH: f32 = 120.0;
const FILE_SIZE_WIDTH: f32 = 60.0;
const SECONDARY_COLUMN_FONT_SIZE: f32 = 12.0;
pub const ROW_HEIGHT: f32 = 20.0;

pub struct TableHeaderParams<'a> {
    pub colors: &'a AppColors,
    pub sort_column: &'a SortColumn,
    pub sort_order: &'a SortOrder,
    pub on_sort: &'a mut dyn FnMut(SortColumn),
}

pub fn draw_table_header(ui: &mut Ui, params: &mut TableHeaderParams) -> egui::Response {
    ui.style_mut().spacing.item_spacing.y = 2.0;

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), HEADER_ROW_HEIGHT),
        egui::Sense::hover(), // Sense hover on the whole row for potential background effects
    );
    let mut cursor = rect.left_top();

    // Calculate total fixed width (icon + date + size + paddings)
    let fixed_width_total = ICON_WIDTH
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
    let name_col_rect =
        egui::Rect::from_min_size(cursor, egui::vec2(name_width, HEADER_ROW_HEIGHT));
    draw_header_column(ui, params, name_col_rect, "Name", SortColumn::Name);
    cursor.x += name_width + INTER_COLUMN_PADDING; // Advance cursor including padding

    // --- Draw Modified Column ---
    let mod_col_rect =
        egui::Rect::from_min_size(cursor, egui::vec2(MODIFIED_DATE_WIDTH, HEADER_ROW_HEIGHT));
    draw_header_column(
        ui,
        params,
        mod_col_rect,
        "Date Modified",
        SortColumn::Modified,
    );
    cursor.x += MODIFIED_DATE_WIDTH + INTER_COLUMN_PADDING; // Advance cursor including padding

    // --- Draw Size Column ---
    let size_col_rect =
        egui::Rect::from_min_size(cursor, egui::vec2(FILE_SIZE_WIDTH, HEADER_ROW_HEIGHT));
    draw_header_column(ui, params, size_col_rect, "Size", SortColumn::Size);
    // No cursor advance needed after the last column

    ui.separator();

    response
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
        params.colors.highlight
    } else {
        params.colors.link_text
    };

    let header_text = format!("{text}{sort_indicator}");

    // Create a child UI with the exact rectangle we want
    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(col_rect));

    let response = child_ui.add(
        egui::Label::new(
            egui::RichText::new(header_text)
                .color(text_color)
                .size(HEADER_FONT_SIZE),
        )
        .sense(egui::Sense::click()),
    );

    if response.clicked() {
        (params.on_sort)(column);
    }

    // Change cursor on hover
    if response.hovered() {
        response.on_hover_cursor(egui::CursorIcon::PointingHand);
    }
}

#[derive(Debug)]
pub struct EntryRowParams<'a> {
    pub entry: &'a DirEntry,
    pub is_selected: bool,
    pub colors: &'a AppColors,
    pub is_marked: bool,
    pub is_bookmarked: bool,
    pub is_being_opened: bool,
    pub is_in_cut_clipboard: bool,
    pub is_in_copy_clipboard: bool,
    pub is_drag_active: bool,
    pub is_drag_source: bool,
}

fn draw_icon(
    ui: &mut Ui,
    cursor: egui::Pos2,
    is_dir: bool,
    is_selected: bool,
    colors: &AppColors,
    is_bookmarked: bool,
    is_symlink: bool,
) -> f32 {
    // Draw the base icon (folder, file, or symlink)
    let base_icon = if is_symlink {
        // Use a link icon for symlinks
        "🔗"
    } else if is_dir {
        "📁"
    } else {
        "📄"
    };

    let icon_color = if is_selected {
        colors.fg_selected
    } else {
        colors.fg_light
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
                colors.fg_selected
            } else {
                colors.fg_light.gamma_multiply(1.2)
            }, // More subtle color
        );
    }

    ICON_WIDTH + HORIZONTAL_PADDING
}

pub fn draw_entry_row(ui: &mut Ui, params: EntryRowParams<'_>) -> egui::Response {
    let EntryRowParams {
        entry,
        is_selected,
        colors,
        is_marked,
        is_bookmarked,
        is_being_opened,
        is_in_cut_clipboard,
        is_in_copy_clipboard,
        is_drag_active,
        is_drag_source,
    } = params;

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), ROW_HEIGHT),
        egui::Sense::click_and_drag(), // Use click_and_drag to detect double clicks and drag
    );

    // Provide detailed accessibility information
    response.widget_info(|| {
        let mut state_info = Vec::new();
        if is_marked {
            state_info.push("marked");
        }
        if is_in_cut_clipboard {
            state_info.push("cut to clipboard");
        }
        if is_in_copy_clipboard {
            state_info.push("copied to clipboard");
        }
        if is_drag_source {
            state_info.push("being dragged");
        }
        let accessibility_text = if !state_info.is_empty() {
            format!("{} {}", state_info.join(", "), entry.accessibility_text())
        } else {
            entry.accessibility_text()
        };
        if is_selected {
            egui::WidgetInfo::selected(egui::WidgetType::Button, true, true, accessibility_text)
        } else {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, accessibility_text)
        }
    });

    // Show a subtle pulsing effect for files being opened
    if is_being_opened && !entry.is_dir {
        let time = ui.ctx().input(|i| i.time);
        let pulse = (time * 20.0).sin().mul_add(0.5, 0.5) as f32; // Pulsing effect between 0.0 and 1.0
        let pulse_color = colors.success.gamma_multiply(pulse.mul_add(0.5, 0.3));
        // Draw a pulsing background
        ui.painter().rect_filled(rect, 0.0, pulse_color);
    } else if is_drag_source {
        // Show visual feedback for the file being dragged
        ui.painter()
            .rect_filled(rect, 2.0, colors.highlight.gamma_multiply(0.2));
    } else if is_drag_active && entry.is_dir && response.contains_pointer() {
        // Check hover state directly on the response to highlight folders as valid drop targets
        //
        // NOTE: we need to use response.contains_pointer here because
        // response.hover is always false when dragging is active
        ui.painter()
            .rect_filled(rect, 2.0, colors.success.gamma_multiply(0.2));
    } else if is_marked {
        ui.painter().rect_filled(rect, 0.0, colors.bg_light);
    } else if is_selected {
        ui.painter().rect_filled(rect, 0.0, colors.bg_selected);
    }

    let mut cursor = rect.left_top();

    // Calculate total fixed width (icon + date + size + paddings) - same as header
    let fixed_width_total = ICON_WIDTH
            + HORIZONTAL_PADDING // Padding after icon
            + MODIFIED_DATE_WIDTH
            + INTER_COLUMN_PADDING // Padding between Modified and Size
            + FILE_SIZE_WIDTH
            + HORIZONTAL_PADDING; // Padding at the end

    // Name width takes remaining space
    let name_width = (rect.width() - fixed_width_total).max(0.0);

    cursor.x += draw_icon(
        ui,
        cursor,
        entry.is_dir,
        is_selected,
        colors,
        is_bookmarked,
        entry.is_symlink,
    );

    // --- Draw Name Column ---
    let name_clip_rect = egui::Rect::from_min_size(cursor, egui::vec2(name_width, ROW_HEIGHT));
    let name_text = truncate_text(&entry.name, name_width);
    let name_color = if is_in_cut_clipboard {
        // Use error color (red) for cut files
        colors.error
    } else if is_in_copy_clipboard {
        // Use success color (green) for copied files
        colors.success
    } else if entry.is_dir {
        colors.fg_folder
    } else {
        colors.fg
    };

    let mut job = egui::text::LayoutJob {
        text: name_text.clone(),
        ..Default::default()
    };

    // Just add the whole text with normal color (no highlighting)
    job.append(
        &name_text,
        0.0,
        egui::TextFormat {
            color: name_color,
            ..Default::default()
        },
    );

    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let galley_pos = cursor + egui::vec2(0.0, ROW_HEIGHT / 2.0 - galley.size().y / 2.0); // Center vertically

    ui.painter()
        .with_clip_rect(name_clip_rect)
        .galley(galley_pos, galley, name_color);
    cursor.x += name_width + INTER_COLUMN_PADDING; // Advance cursor including padding

    let secondary_font_color = if is_selected {
        colors.fg_selected
    } else {
        colors.fg_light
    };

    // --- Draw Modified Column ---
    ui.painter().text(
        cursor + egui::vec2(0.0, ROW_HEIGHT / 2.0),
        Align2::LEFT_CENTER,
        &entry.formatted_modified,
        egui::FontId::proportional(SECONDARY_COLUMN_FONT_SIZE),
        secondary_font_color,
    );
    cursor.x += MODIFIED_DATE_WIDTH + INTER_COLUMN_PADDING; // Advance cursor including padding

    // --- Draw Size Column ---
    ui.painter().text(
        cursor + egui::vec2(FILE_SIZE_WIDTH - HORIZONTAL_PADDING, ROW_HEIGHT / 2.0),
        Align2::RIGHT_CENTER,
        &entry.formatted_size,
        egui::FontId::proportional(SECONDARY_COLUMN_FONT_SIZE),
        secondary_font_color,
    );
    // No cursor advance needed after the last column

    response
}

pub fn draw_parent_entry_row(
    ui: &mut Ui,
    entry: &DirEntry,
    is_selected: bool,
    colors: &AppColors,
    is_bookmarked: bool,
    is_in_cut_clipboard: bool,
    is_in_copy_clipboard: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), ROW_HEIGHT),
        // Use click_and_drag to detect double clicks
        egui::Sense::click_and_drag(),
    );

    // Provide detailed accessibility information
    response.widget_info(|| {
        let accessibility_text = entry.accessibility_text();
        if is_selected {
            egui::WidgetInfo::selected(egui::WidgetType::Button, true, true, accessibility_text)
        } else {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, accessibility_text)
        }
    });

    if is_selected {
        ui.painter().rect_filled(rect, 0.0, colors.bg_selected);
    }

    let mut cursor = rect.left_top();

    let name_width = rect.width() - ICON_WIDTH;

    cursor.x += draw_icon(
        ui,
        cursor,
        entry.is_dir,
        is_selected,
        colors,
        is_bookmarked,
        entry.is_symlink,
    );

    // Name with truncation
    let name_text = truncate_text(&entry.name, name_width);
    let name_color = if is_in_cut_clipboard {
        // Use error color (red) for cut files
        colors.error
    } else if is_in_copy_clipboard {
        // Use success color (green) for copied files
        colors.success
    } else if entry.is_dir {
        colors.fg_folder
    } else {
        colors.fg
    };
    ui.painter().text(
        cursor + egui::vec2(0.0, ROW_HEIGHT / 2.0),
        Align2::LEFT_CENTER,
        &name_text,
        egui::FontId::proportional(14.0),
        name_color,
    );

    response
}

#[must_use]
pub fn truncate_text(text: &str, available_width: f32) -> String {
    // Approximate width of a character in pixels
    let char_width = 8.0;
    let max_chars = (available_width / char_width) as usize;
    let chars_cnt = text.chars().count();

    if chars_cnt <= max_chars {
        return text.replace('\n', "?");
    }

    let half_chars = max_chars / 2;

    let start = text.chars().take(half_chars).collect::<String>();
    let end = text
        .chars()
        .skip(chars_cnt - half_chars)
        .take(half_chars)
        .map(|c| if c == '\n' { '?' } else { c })
        .collect::<String>();

    format!("{start}...{end}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("short", 100.0), "short");
        assert_eq!(
            truncate_text("this_is_a_very_long_filename.txt", 80.0),
            "this_...e.txt"
        );
    }

    #[test]
    fn test_truncate_unicode() {
        assert_eq!(
            // set to 136.0 so it tries to subindex at the Narrow No-Break Space char
            truncate_text("Screenshot 2025-04-21 at 2.45.13 AM.png", 136.0),
            "Screensh...3\u{202f}AM.png"
        );
    }

    #[test]
    fn test_truncate_with_newline() {
        assert_eq!(truncate_text("file\nname", 100.0), "file?name");
        assert_eq!(
            truncate_text("a_very_long_file\nname_that_will_be_truncated.txt", 80.0),
            "a_ver...d.txt"
        );
    }
}
