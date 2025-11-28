use crate::config::colors::AppColors;
use crate::config::shortcuts::{ShortcutAction, Shortcuts, shortcuts_helpers};
use egui::{self, RichText};

use super::popup::window_utils::show_center_popup_window;

pub fn show_help_window(
    ctx: &egui::Context,
    shortcuts: &Shortcuts,
    show_help: &mut bool,
    colors: &AppColors,
) {
    let mut keep_open = *show_help; // Use a temporary variable for the open state

    let response = show_center_popup_window("Help", ctx, &mut keep_open, |ui| {
        ui.horizontal(|ui| {
            // Column 1: Navigation and Popups
            ui.vertical(|ui| {
                ui.heading(RichText::new("Navigation").color(colors.fg_light));
                let table = egui::Grid::new("help_grid");
                table.show(ui, |ui| {
                    let navigation_actions = [
                        (ShortcutAction::MoveDown, "Move down"),
                        (ShortcutAction::MoveUp, "Move up"),
                        (ShortcutAction::PageDown, "Move down by page"),
                        (ShortcutAction::PageUp, "Move up by page"),
                        (
                            ShortcutAction::GoToParentDirectory,
                            "Go to parent directory",
                        ),
                        (ShortcutAction::OpenDirectory, "Open directory"),
                        (ShortcutAction::GoToFirstEntry, "Jump to the first entry"),
                        (ShortcutAction::GoToLastEntry, "Jump to the last entry"),
                        (ShortcutAction::GoBackInHistory, "Go back in history"),
                        (ShortcutAction::GoForwardInHistory, "Go forward in history"),
                        (ShortcutAction::ToggleHiddenFiles, "Toggle hidden files"),
                    ];

                    for (action, description) in navigation_actions {
                        let shortcut_display =
                            shortcuts_helpers::get_shortcut_display(shortcuts, action);
                        ui.label(RichText::new(shortcut_display).color(colors.highlight));
                        ui.label(description);
                        ui.end_row();
                    }
                });

                ui.add_space(10.0); // Space between sections

                ui.heading(RichText::new("Popups").color(colors.fg_light));
                let table = egui::Grid::new("popup_help_grid");
                table.show(ui, |ui| {
                    let popup_actions = [
                        (
                            ShortcutAction::ShowTeleport,
                            "Teleport with history fuzzy search",
                        ),
                        (ShortcutAction::ShowBookmarks, "Show bookmark popup"),
                        #[cfg(target_os = "windows")]
                        (ShortcutAction::ShowWindowsDrives, "Show drives popup"),
                        #[cfg(target_os = "macos")]
                        (ShortcutAction::ShowVolumes, "Show volumes popup"),
                        (
                            ShortcutAction::ShowFilePreview,
                            "Preview file in a popup window",
                        ),
                        (ShortcutAction::ShowSortToggle, "Show sort toggle popup"),
                        (
                            ShortcutAction::ShowActionHistory,
                            "Show action history popup",
                        ),
                    ];

                    for (action, description) in popup_actions {
                        let shortcut_display =
                            shortcuts_helpers::get_shortcut_display(shortcuts, action);
                        ui.label(RichText::new(shortcut_display).color(colors.highlight));
                        ui.label(description);
                        ui.end_row();
                    }
                });

                ui.add_space(10.0); // Space between sections

                ui.heading(RichText::new("Tabs").color(colors.fg_light));
                let table = egui::Grid::new("tab_help_grid");
                table.show(ui, |ui| {
                    let tab_actions = [
                        (ShortcutAction::CreateTab, "Create new tab"),
                        (ShortcutAction::CloseCurrentTab, "Close current tab"),
                        (
                            ShortcutAction::SwitchToPreviousTab,
                            "Switch to previous tab",
                        ),
                        (ShortcutAction::SwitchToNextTab, "Switch to next tab"),
                    ];

                    for (action, description) in tab_actions {
                        let shortcut_display =
                            shortcuts_helpers::get_shortcut_display(shortcuts, action);
                        ui.label(RichText::new(shortcut_display).color(colors.highlight));
                        ui.label(description);
                        ui.end_row();
                    }

                    // Add tab switching shortcuts
                    #[cfg(target_os = "macos")]
                    ui.label(RichText::new("Cmd+1-9").color(colors.highlight));
                    #[cfg(not(target_os = "macos"))]
                    ui.label(RichText::new("Ctrl+1-9").color(colors.highlight));
                    ui.label("Switch to tab by number");
                    ui.end_row();
                });
            });

            ui.separator(); // Vertical separator between columns

            // Column 2
            ui.vertical(|ui| {
                // Section: File Operations
                ui.heading(RichText::new("File Operations").color(colors.fg_light));
                let table = egui::Grid::new("file_op_help_grid");
                table.show(ui, |ui| {
                    let file_actions = [
                        (ShortcutAction::OpenDirectoryOrFile, "Open file"),
                        (
                            ShortcutAction::OpenWithCommand,
                            "Open file with custom command",
                        ),
                        (
                            ShortcutAction::DeleteEntry,
                            "Delete selected file/directory",
                        ),
                        (
                            ShortcutAction::RenameEntry,
                            "Rename selected file/directory",
                        ),
                        (ShortcutAction::AddEntry, "Add file/directory"),
                        (ShortcutAction::SelectEntry, "Mark/unmark entry"),
                        (
                            ShortcutAction::ToggleRangeSelection,
                            "Toggle range selection mode",
                        ),
                        (ShortcutAction::SelectAllEntries, "Select all entries"),
                        (ShortcutAction::CopyEntry, "Copy selected entry"),
                        (ShortcutAction::CutEntry, "Cut selected entry"),
                        (ShortcutAction::PasteEntry, "Paste copied/cut entries"),
                        (
                            ShortcutAction::ToggleBookmark,
                            "Add/remove bookmark for current directory",
                        ),
                        (ShortcutAction::CopyPath, "Copy full path"),
                        (ShortcutAction::CopyName, "Copy name"),
                        (ShortcutAction::Undo, "Undo last action"),
                        (ShortcutAction::Redo, "Redo last action"),
                    ];
                    for (action, description) in file_actions {
                        let shortcut_display =
                            shortcuts_helpers::get_shortcut_display(shortcuts, action);
                        ui.label(RichText::new(shortcut_display).color(colors.highlight));
                        ui.label(description);
                        ui.end_row();
                    }
                });
                ui.add_space(10.0); // Space between sections

                // Section: Search
                ui.heading(RichText::new("Search").color(colors.fg_light));
                let table = egui::Grid::new("search_help_grid");
                table.show(ui, |ui| {
                    let search_actions =
                        [(ShortcutAction::ActivateSearch, "Activate search filter")];
                    for (action, description) in search_actions {
                        let shortcut_display =
                            shortcuts_helpers::get_shortcut_display(shortcuts, action);
                        ui.label(RichText::new(shortcut_display).color(colors.highlight));
                        ui.label(description);
                        ui.end_row();
                    }

                    // Add search-specific shortcuts
                    ui.label(RichText::new("Enter (in search)").color(colors.highlight));
                    ui.label("Apply filter");
                    ui.end_row();

                    ui.label(RichText::new("Esc (in search)").color(colors.highlight));
                    ui.label("Clear filter");
                    ui.end_row();
                });

                ui.add_space(10.0); // Space between sections

                // Section: Utils
                ui.heading(RichText::new("Utils").color(colors.fg_light));
                let table = egui::Grid::new("utils_help_grid");
                table.show(ui, |ui| {
                    let util_actions = [
                        (
                            ShortcutAction::OpenTerminal,
                            "Open terminal panel at current directory",
                        ),
                        (ShortcutAction::Exit, "Exit Kiorg or close popups"),
                        (ShortcutAction::ShowHelp, "Toggle this help window"),
                    ];
                    for (action, description) in util_actions {
                        let shortcut_display =
                            shortcuts_helpers::get_shortcut_display(shortcuts, action);
                        ui.label(RichText::new(shortcut_display).color(colors.highlight));
                        ui.label(description);
                        ui.end_row();
                    }
                });
            });
        });

        ui.add_space(10.0);
        ui.separator(); // Horizontal separator below columns

        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Press ? or Enter to close").color(colors.fg_light))
        });
    });

    if response.is_some() {
        *show_help = keep_open;
    }
}
