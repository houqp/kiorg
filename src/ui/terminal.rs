use crate::app::Kiorg;

#[cfg(feature = "terminal")]
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
            let screen_height = ctx.screen_rect().height();
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

#[cfg(not(feature = "terminal"))]
mod implementation {
    use super::*;
    use crate::ui::style::section_title_text;

    pub struct TerminalContext {
        show_disabled_popup: bool,
    }

    impl TerminalContext {
        pub fn new(
            _ctx: &egui::Context,
            _working_directory: std::path::PathBuf,
        ) -> Result<Self, String> {
            Ok(Self {
                show_disabled_popup: true,
            })
        }
    }

    pub fn draw(ctx: &egui::Context, app: &mut Kiorg) {
        if let Some(terminal_ctx) = &mut app.terminal_ctx {
            if terminal_ctx.show_disabled_popup {
                egui::Window::new("Terminal Disabled")
                    .open(&mut terminal_ctx.show_disabled_popup)
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(10.0);
                            ui.label(section_title_text(
                                "Terminal feature disabled for this release",
                                &app.colors,
                            ));
                            ui.add_space(10.0);
                        });
                    });

                if !terminal_ctx.show_disabled_popup {
                    app.terminal_ctx = None;
                }
            }
        }
    }
}

pub use implementation::{TerminalContext, draw};
