use crate::app::Kiorg;

#[cfg(not(target_os = "windows"))]
mod implementation {
    use super::Kiorg;
    use crate::ui::style::section_title_text;
    use egui::Vec2;
    use egui_term::{PtyEvent, TerminalView};

    pub struct TerminalContext {
        pub terminal_backend: egui_term::TerminalBackend,
        pub pty_proxy_receiver: std::sync::mpsc::Receiver<(u64, egui_term::PtyEvent)>,
    }

    impl TerminalContext {
        pub fn new(
            ctx: &egui::Context,
            working_directory: std::path::PathBuf,
        ) -> Result<Self, String> {
            let system_shell = std::env::var("SHELL")
                .map_err(|e| format!("SHELL variable is not defined: {e}"))?;

            // Sometimes, TERM is not set properly on app start, e.g. launching from MacOS Dock
            if std::env::var("TERM").is_err() {
                unsafe {
                    std::env::set_var("TERM", "xterm-256color");
                }
            }

            let (pty_proxy_sender, pty_proxy_receiver) = std::sync::mpsc::channel();

            let terminal_backend = egui_term::TerminalBackend::new(
                0,
                ctx.clone(),
                pty_proxy_sender,
                egui_term::BackendSettings {
                    shell: system_shell,
                    working_directory: Some(working_directory),
                    ..Default::default()
                },
            )
            .map_err(|e| format!("Failed to create terminal backend: {e}"))?;

            Ok(Self {
                terminal_backend,
                pty_proxy_receiver,
            })
        }
    }

    pub fn draw(ctx: &egui::Context, app: &mut Kiorg) {
        if let Some(terminal_ctx) = &mut app.terminal_ctx {
            if let Ok((_, PtyEvent::Exit)) = terminal_ctx.pty_proxy_receiver.try_recv() {
                app.terminal_ctx = None;
                return;
            }

            let mut close_terminal = false;

            // Create a panel at the bottom of the screen
            let screen_height = ctx.content_rect().height();
            egui::TopBottomPanel::bottom("terminal_panel")
                .resizable(true)
                .default_height(screen_height * 0.6)
                .min_height(100.0)
                .max_height(screen_height * 0.9)
                .show(ctx, |ui| {
                    // Add a close button in the top right corner
                    ui.horizontal(|ui| {
                        ui.label(section_title_text("Terminal", &app.colors));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("×").clicked() {
                                close_terminal = true;
                            }
                        });
                    });

                    let terminal = TerminalView::new(ui, &mut terminal_ctx.terminal_backend)
                        .set_focus(true)
                        .set_size(Vec2::new(ui.available_width(), ui.available_height()));
                    ui.add(terminal);
                });

            // Close the terminal if the close button was clicked
            if close_terminal {
                app.terminal_ctx = None;
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod implementation {
    use super::*;
    use crate::ui::popup::PopupType;

    pub struct TerminalContext {}

    impl TerminalContext {
        pub fn new(
            _ctx: &egui::Context,
            _working_directory: std::path::PathBuf,
        ) -> Result<Self, String> {
            Ok(Self {})
        }
    }

    pub fn draw(_ctx: &egui::Context, app: &mut Kiorg) {
        if app.terminal_ctx.is_some() {
            // Show the feature disabled popup
            app.show_popup = Some(PopupType::GenericMessage(
                "Terminal feature disabled".to_string(),
                "Terminal feature disabled for this release".to_string(),
            ));
            app.terminal_ctx = None;
        }
    }
}

pub use implementation::{TerminalContext, draw};
