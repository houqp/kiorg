use crate::config::colors::AppColors;
use crate::ui::preview;
use egui::{RichText, Ui};

pub fn render(
    ui: &mut Ui,
    components: &[kiorg_plugin::Component],
    colors: &AppColors,
    available_width: f32,
) {
    ui.vertical(|ui| {
        for component in components {
            match component {
                kiorg_plugin::Component::Title(title) => {
                    ui.heading(RichText::new(&title.text).color(colors.fg));
                }
                kiorg_plugin::Component::Text(text) => {
                    preview::text::render(ui, &text.text, colors);
                }
                kiorg_plugin::Component::Image(image) => match &image.source {
                    kiorg_plugin::ImageSource::Path(path) => {
                        let uri = format!("file://{}", path);
                        ui.add(egui::Image::new(uri).max_width(available_width));
                    }
                    kiorg_plugin::ImageSource::Url(url) => {
                        ui.add(egui::Image::new(url).max_width(available_width));
                    }
                    kiorg_plugin::ImageSource::Bytes { format: _, data } => {
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut hasher = DefaultHasher::new();
                        data.hash(&mut hasher);
                        let hash = hasher.finish();
                        let uri = format!("bytes://{}", hash);
                        ui.add(
                            egui::Image::from_bytes(uri, data.clone()).max_width(available_width),
                        );
                    }
                },
                kiorg_plugin::Component::Table(table) => {
                    use egui_extras::{Column, TableBuilder};
                    let num_columns = if let Some(headers) = &table.headers {
                        headers.len()
                    } else if let Some(first_row) = table.rows.first() {
                        first_row.len()
                    } else {
                        0
                    };
                    if num_columns == 0 {
                        continue;
                    }

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
            ui.add_space(10.0);
        }
    });
}
