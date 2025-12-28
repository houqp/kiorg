use crate::config::colors::AppColors;
use crate::models::preview_content::RenderedComponent;
use crate::ui::preview;
use egui::{RichText, Ui};

pub fn render(
    ui: &mut Ui,
    components: &[RenderedComponent],
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    for (i, component) in components.iter().enumerate() {
        ui.push_id(i, |ui| {
            if i > 0 {
                ui.add_space(10.0);
            }
            match component {
                RenderedComponent::Title(title) => {
                    ui.heading(RichText::new(&title.text).color(colors.fg));
                }
                RenderedComponent::Text(text) => {
                    preview::text::render(ui, &text.text, colors);
                }
                RenderedComponent::Image(image) => {
                    if image.interactive {
                        crate::ui::preview::image::render_interactive(
                            ui,
                            &image.image,
                            available_width,
                            available_height,
                        );
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add(
                                image
                                    .image
                                    .clone()
                                    .max_size(egui::vec2(available_width, available_height * 0.6))
                                    .maintain_aspect_ratio(true),
                            );
                        });
                    }
                }
                RenderedComponent::Table(table) => {
                    use egui_extras::{Column, TableBuilder};
                    let num_columns = if let Some(headers) = &table.headers {
                        headers.len()
                    } else if let Some(first_row) = table.rows.first() {
                        first_row.len()
                    } else {
                        return;
                    };

                    let mut builder = TableBuilder::new(ui).striped(true).vscroll(false);
                    for _ in 0..(num_columns - 1) {
                        builder = builder
                            .column(Column::auto_with_initial_suggestion(150.0).resizable(true));
                    }
                    builder = builder.column(Column::remainder());

                    let body_cb = |mut body: egui_extras::TableBody| {
                        for row in &table.rows {
                            body.row(18.0, |mut row_ui| {
                                for cell in row {
                                    row_ui.col(|ui| {
                                        ui.label(RichText::new(cell).color(colors.fg));
                                    });
                                }
                            });
                        }
                    };

                    if let Some(headers) = &table.headers {
                        builder
                            .header(20.0, |mut header| {
                                for h in headers {
                                    header.col(|ui| {
                                        ui.strong(RichText::new(h).color(colors.fg));
                                    });
                                }
                            })
                            .body(body_cb);
                    } else {
                        builder.body(body_cb);
                    }
                }
            }
        });
    }
}
