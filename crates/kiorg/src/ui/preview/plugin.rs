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
                    TableBuilder::new(ui)
                        .striped(true)
                        .columns(Column::auto(), table.headers.len())
                        .header(20.0, |mut header| {
                            for h in &table.headers {
                                header.col(|ui| {
                                    ui.strong(RichText::new(h).color(colors.fg));
                                });
                            }
                        })
                        .body(|mut body| {
                            for row in &table.rows {
                                body.row(18.0, |mut row_ui| {
                                    for cell in row {
                                        row_ui.col(|ui| {
                                            ui.label(RichText::new(cell).color(colors.fg));
                                        });
                                    }
                                });
                            }
                        });
                }
            }
            ui.add_space(10.0);
        }
    });
}
