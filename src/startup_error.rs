use eframe::egui;

/// A minimal egui app that displays startup errors
pub struct StartupErrorApp {
    error_message: String,
    title: String,
    additional_info: Option<String>,
}

impl StartupErrorApp {
    pub fn new(error_message: String, title: String) -> Self {
        Self {
            error_message,
            title,
            additional_info: None,
        }
    }

    /// Create a new startup error app with additional context information
    pub fn with_info(error_message: String, title: String, additional_info: String) -> Self {
        Self {
            error_message,
            title,
            additional_info: Some(additional_info),
        }
    }

    /// Convenience method to show a configuration error dialog
    pub fn show_config_error(error_message: String) -> Result<(), eframe::Error> {
        Self::show_error_dialog(error_message, "Configuration Error".to_string(), None)
    }

    /// Convenience method to show a configuration error dialog with config path
    pub fn show_config_error_with_path(
        error_message: String,
        config_path: &str,
    ) -> Result<(), eframe::Error> {
        Self::show_error_dialog(
            error_message,
            "Configuration Error".to_string(),
            Some(format!("Config file: {}", config_path)),
        )
    }

    /// Show a startup error dialog using eframe
    pub fn show_error_dialog(
        error_message: String,
        title: String,
        additional_info: Option<String>,
    ) -> Result<(), eframe::Error> {
        let icon_data = crate::utils::icon::load_app_icon();
        let window_title = format!("Kiorg - {}", title);

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_resizable(true)
                .with_title(&window_title)
                .with_inner_size([600.0, 400.0])
                .with_icon(icon_data),
            centered: true,
            ..Default::default()
        };

        eframe::run_native(
            &window_title,
            options,
            Box::new(move |_cc| {
                let app = if let Some(info) = additional_info {
                    StartupErrorApp::with_info(error_message, title, info)
                } else {
                    StartupErrorApp::new(error_message, title)
                };
                Ok(Box::new(app))
            }),
        )
    }
}

impl eframe::App for StartupErrorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(20.0);

                let visuals = &ctx.style().visuals;

                // Error icon and title
                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                    ui.label(egui::RichText::new("❗").size(30.0));
                    ui.add_space(5.0);
                    ui.label(
                        egui::RichText::new(&self.title)
                            .size(16.0)
                            .strong()
                            .color(visuals.error_fg_color),
                    );
                });
                ui.add_space(10.0);

                // Error details in a scrollable frame
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Frame::default().inner_margin(15.0).show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(&self.error_message)
                                .size(12.0)
                                .family(egui::FontFamily::Monospace),
                        );

                        ui.add_space(10.0);

                        // Additional info if provided
                        if let Some(info) = &self.additional_info {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(info));
                            });
                            ui.add_space(10.0);
                        }
                    });
                });

                // OK button to close - centered and prominent
                ui.vertical_centered(|ui| {
                    let button = egui::Button::new(egui::RichText::new("OK").size(14.0).strong());
                    if ui.add(button).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });
    }
}
