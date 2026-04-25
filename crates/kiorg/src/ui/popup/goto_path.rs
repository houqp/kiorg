use crate::app::Kiorg;
use crate::ui::popup::PopupType;
use egui::{Align, Color32, Key, Layout, TextEdit, Vec2};
use nucleo::{Config as NucleoConfig, Matcher, Utf32Str};
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

const MAX_SUGGESTIONS: usize = 10;

#[derive(Debug, Clone)]
pub struct GoToPathState {
    pub input: String,
    pub selected_index: usize,
    pub focus_input: bool,
    pub suggestions: Vec<PathBuf>,
    pub parent_exists: bool,
    last_input: Option<String>,
}

impl GoToPathState {
    pub fn new(initial_path: String) -> Self {
        Self {
            input: initial_path,
            selected_index: 0,
            focus_input: true,
            suggestions: Vec::new(),
            parent_exists: true,
            last_input: None, // Force update_suggestions to run on first call
        }
    }

    pub fn update_suggestions(&mut self) {
        #[cfg(unix)]
        if !self.input.is_empty() && !self.input.starts_with('/') {
            self.input.insert(0, '/');
        }
        #[cfg(windows)]
        if !self.input.is_empty() && !self.input.starts_with('\\') && !self.input.contains(':') {
            self.input.insert(0, '\\');
        }

        if Some(&self.input) == self.last_input.as_ref() {
            return;
        }

        self.last_input = Some(self.input.clone());

        if self.input.is_empty() {
            #[cfg(unix)]
            {
                let mut dirs = Vec::new();
                if let Ok(entries) = std::fs::read_dir("/") {
                    for entry in entries.flatten() {
                        if let Ok(file_type) = entry.file_type() {
                            if file_type.is_dir() || (file_type.is_symlink() && entry.path().is_dir()) {
                                dirs.push(entry.path());
                            }
                        }
                    }
                }
                dirs.sort();
                self.suggestions = dirs.into_iter().take(MAX_SUGGESTIONS).collect();
                self.parent_exists = true;
            }
            #[cfg(windows)]
            {
                // SAFETY: GetLogicalDrives is a pure FFI call with no invariants to maintain.
                unsafe {
                    let drives_mask = windows_sys::Win32::Storage::FileSystem::GetLogicalDrives();
                    let mut drives = Vec::new();
                    for i in 0..26 {
                        if (drives_mask & (1 << i)) != 0 {
                            let drive_letter = (b'A' + i) as char;
                            let drive_path = format!("{}:\\", drive_letter);
                            drives.push(std::path::PathBuf::from(drive_path));
                        }
                    }
                    self.suggestions = drives;
                }
                self.parent_exists = true;
            }
            self.selected_index = 0;
            return;
        }

        // Split input into parent and stem (part after last slash)
        let path = Path::new(&self.input);
        let mut components = path.components();
        let (parent, stem) = if self.input.ends_with(MAIN_SEPARATOR) {
            (path, "")
        } else {
            let stem = components
                .next_back()
                .and_then(|c| c.as_os_str().to_str())
                .unwrap_or("");
            (components.as_path(), stem)
        };

        if !parent.is_dir() {
            self.suggestions = Vec::new();
            self.parent_exists = false;
            self.selected_index = 0;
            return;
        }
        self.parent_exists = true;

        let mut dirs = Vec::new();
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() || (file_type.is_symlink() && entry.path().is_dir()) {
                        dirs.push(entry.path());
                    }
                }
            }
        }

        if stem.is_empty() {
            dirs.sort();
            self.suggestions = dirs.into_iter().take(MAX_SUGGESTIONS).collect();
        } else {
            let mut config = NucleoConfig::DEFAULT;
            config.ignore_case = true;
            let mut matcher = Matcher::new(config);

            let mut needle_buf = Vec::new();
            let needle = stem.to_lowercase();
            let needle_utf32 = Utf32Str::new(&needle, &mut needle_buf);

            let mut scored_dirs: Vec<(PathBuf, u16)> = dirs
                .into_iter()
                .filter_map(|path| {
                    let name = path.file_name()?.to_str()?;
                    let mut haystack_buf = Vec::new();
                    let haystack_utf32 = Utf32Str::new(name, &mut haystack_buf);
                    matcher
                        .fuzzy_match(haystack_utf32, needle_utf32)
                        .map(|score| (path, score))
                })
                .collect();

            scored_dirs.sort_by(|a, b| b.1.cmp(&a.1));
            self.suggestions = scored_dirs
                .into_iter()
                .map(|(p, _)| p)
                .take(MAX_SUGGESTIONS)
                .collect();
        }

        self.selected_index = 0;
    }
}

pub fn draw(ctx: &egui::Context, app: &mut Kiorg) {
    let mut state = if let Some(PopupType::GoToPath(state)) = app.show_popup.take() {
        state
    } else {
        return;
    };

    let colors = &app.colors;
    let mut close = false;
    let mut navigate_to = None;

    crate::ui::popup::frameless_popup::new_frameless_popup_window("Go To Path", colors).show(
        ctx,
        |ui| {
            let popup_height = ui.available_height() - 60.0;
            ui.set_min_width(600.0);
            ui.set_max_width(ui.available_width() - 100.0);
            ui.set_min_height(popup_height);
            ui.set_max_height(popup_height);

            // Handle keyboard input
            let mut consume_tab = false;
            ctx.input_mut(|i| {
                i.events.retain(|event| {
                    if let egui::Event::Key { key, pressed, .. } = event {
                        match *key {
                            Key::Escape => {
                                if *pressed {
                                    close = true;
                                }
                                false
                            }
                            Key::Enter => {
                                if *pressed {
                                    if !state.suggestions.is_empty() {
                                        if state.selected_index >= state.suggestions.len() {
                                            tracing::error!("GoToPath selected_index {} out of bounds (len {}), resetting to 0", state.selected_index, state.suggestions.len());
                                            state.selected_index = 0;
                                        }
                                        navigate_to =
                                            Some(state.suggestions[state.selected_index].clone());
                                    } else {
                                        navigate_to = Some(PathBuf::from(&state.input));
                                    }
                                }
                                false
                            }
                            Key::ArrowDown => {
                                if *pressed && !state.suggestions.is_empty() {
                                    if state.selected_index >= state.suggestions.len() {
                                        tracing::error!("GoToPath selected_index {} out of bounds (len {}), resetting to 0", state.selected_index, state.suggestions.len());
                                        state.selected_index = 0;
                                    } else {
                                        state.selected_index =
                                            (state.selected_index + 1) % state.suggestions.len();
                                    }
                                }
                                false
                            }
                            Key::ArrowUp => {
                                if *pressed && !state.suggestions.is_empty() {
                                    if state.selected_index >= state.suggestions.len() {
                                        tracing::error!("GoToPath selected_index {} out of bounds (len {}), resetting to 0", state.selected_index, state.suggestions.len());
                                        state.selected_index = 0;
                                    } else {
                                        state.selected_index = if state.selected_index == 0 {
                                            state.suggestions.len() - 1
                                        } else {
                                            state.selected_index - 1
                                        };
                                    }
                                }
                                false
                            }
                            Key::Tab => {
                                if *pressed && !state.suggestions.is_empty() {
                                    if state.selected_index >= state.suggestions.len() {
                                        tracing::error!("GoToPath selected_index {} out of bounds (len {}), resetting to 0", state.selected_index, state.suggestions.len());
                                        state.selected_index = 0;
                                    }
                                    let path = &state.suggestions[state.selected_index];
                                    let mut path_str = path.to_string_lossy().to_string();
                                    if path.is_dir() && !path_str.ends_with(MAIN_SEPARATOR) {
                                        path_str.push(MAIN_SEPARATOR);
                                    }
                                    state.input = path_str;
                                    state.focus_input = true;
                                    state.update_suggestions();
                                }
                                consume_tab = true;
                                false
                            }
                            _ => true,
                        }
                    } else {
                        true
                    }
                });
                if consume_tab {
                    i.consume_key(egui::Modifiers::default(), Key::Tab);
                }
            });

            ui.horizontal(|ui| {
                let text_edit = TextEdit::singleline(&mut state.input)
                    .id(egui::Id::new("goto_path_input"))
                    .hint_text("Enter path...")
                    .desired_width(f32::INFINITY)
                    .frame(false);

                let response = ui.add(text_edit);
                // Always request focus to ensure the input box is ready for typing.
                // This prevents focus loss when Tab is pressed for completion.
                response.request_focus();

                if ui.button("×").clicked() {
                    close = true;
                }

                if state.focus_input {
                    // move cursor to end
                    if let Some(mut text_state) = TextEdit::load_state(ui.ctx(), response.id) {
                        let len = state.input.chars().count();
                        text_state
                            .cursor
                            .set_char_range(Some(egui::text::CCursorRange::one(
                                egui::text::CCursor::new(len),
                            )));
                        text_state.store(ui.ctx(), response.id);
                    }
                    state.focus_input = false;
                }

                if response.changed() || state.last_input.is_none() {
                    let prev_input = state.input.clone();
                    state.update_suggestions();
                    // keep cursor to the end when we prepend content to user input automatically, e.g. `/`.
                    if response.changed() && state.input != prev_input {
                        if let Some(mut text_state) = TextEdit::load_state(ui.ctx(), response.id) {
                            let new_len = state.input.chars().count();
                            text_state
                                .cursor
                                .set_char_range(Some(egui::text::CCursorRange::one(
                                    egui::text::CCursor::new(new_len),
                                )));
                            text_state.store(ui.ctx(), response.id);
                        }
                    }
                }
            });

            ui.separator();

            if state.suggestions.is_empty() {
                ui.centered_and_justified(|ui| {
                    let msg = if state.parent_exists {
                        "No suggestions"
                    } else {
                        "Directory not found"
                    };
                    ui.label(egui::RichText::new(msg).weak());
                });
            } else {
                egui::ScrollArea::vertical()
                    .max_height(ui.available_height())
                    .show(ui, |ui| {
                        for (index, path) in state.suggestions.iter().enumerate() {
                            let is_selected = index == state.selected_index;
                            let (bg_color, text_color) = if is_selected {
                                (colors.bg_selected, colors.fg_selected)
                            } else {
                                (Color32::TRANSPARENT, colors.fg)
                            };

                            let response = ui.allocate_response(
                                Vec2::new(ui.available_width(), 30.0),
                                egui::Sense::click(),
                            );
                            if response.clicked() {
                                navigate_to = Some(path.clone());
                            }

                            if is_selected {
                                ui.painter().rect_filled(response.rect, 0.0, bg_color);
                            }

                            let mut content_ui = ui.new_child(
                                egui::UiBuilder::new()
                                    .max_rect(response.rect)
                                    .layout(Layout::left_to_right(Align::Center)),
                            );
                            content_ui.horizontal(|ui| {
                                ui.add_space(8.0);
                                let display_name = path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or_else(|| path.to_str().unwrap_or(""));
                                ui.label(egui::RichText::new(display_name).color(text_color))
                                    .on_hover_text(path.to_string_lossy());
                            });
                        }
                    });
            }
        },
    );

    if close {
        app.show_popup = None;
    } else if let Some(path) = navigate_to {
        app.show_popup = None;
        app.navigate_to_dir(path);
    } else {
        app.show_popup = Some(PopupType::GoToPath(state));
    }
}
