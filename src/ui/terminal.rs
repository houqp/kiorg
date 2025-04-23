use egui::Vec2;
use egui_term::{PtyEvent, TerminalView};

use crate::app::Kiorg;
use crate::ui::style::section_title_text;

pub struct TerminalContext {
    pub terminal_backend: egui_term::TerminalBackend,
    pub pty_proxy_receiver: std::sync::mpsc::Receiver<(u64, egui_term::PtyEvent)>,
}

impl TerminalContext {
    pub fn new(ctx: &egui::Context, working_directory: std::path::PathBuf) -> Self {
        let system_shell = std::env::var("SHELL")
            .expect("SHELL variable is not defined")
            .to_string();
        let (pty_proxy_sender, pty_proxy_receiver) = std::sync::mpsc::channel();
        let terminal_backend = egui_term::TerminalBackend::new(
            0,
            ctx.clone(),
            pty_proxy_sender.clone(),
            egui_term::BackendSettings {
                shell: system_shell,
                working_directory: Some(working_directory),
                ..Default::default()
            },
        )
        // TODO: propagate error to UI
        .expect("Failed to create terminal backend");

        Self {
            terminal_backend,
            pty_proxy_receiver,
        }
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
                    ui.label(section_title_text("Terminal", &app.state.colors));
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
