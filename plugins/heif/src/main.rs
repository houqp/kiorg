//! HEIF/HEIC preview plugin for kiorg
//!
//! This plugin uses libheif-rs to decode HEIF/HEIC images and render them as PNG previews.

use kiorg_plugin::{
    Component, ImageComponent, ImageFormat, ImageSource, PluginCapabilities, PluginHandler,
    PluginMetadata, PluginResponse, PreviewCapability, TableComponent, TitleComponent,
};
use libheif_rs::{Channel, ColorSpace, HeifContext, LibHeif, RgbChroma};
use std::io::Cursor;

struct HeifPlugin {
    metadata: PluginMetadata,
}

struct HeifData {
    filename: String,
    png_data: Vec<u8>,
    metadata_rows: Vec<Vec<String>>,
}

impl PluginHandler for HeifPlugin {
    fn on_preview(&mut self, path: &str, available_width: f32) -> PluginResponse {
        match self.process_heif(path, Some(available_width)) {
            Ok(data) => PluginResponse::Preview {
                components: vec![
                    Component::Title(TitleComponent {
                        text: data.filename,
                    }),
                    Component::Image(ImageComponent {
                        source: ImageSource::Bytes {
                            format: ImageFormat::Png,
                            data: data.png_data,
                            uid: path.to_string(),
                        },
                        interactive: false,
                    }),
                    Component::Table(TableComponent {
                        headers: None,
                        rows: data.metadata_rows,
                    }),
                ],
            },
            Err(e) => PluginResponse::Error {
                message: format!("Failed to process HEIF file: {}", e),
            },
        }
    }

    fn on_preview_popup(&mut self, path: &str, _available_width: f32) -> PluginResponse {
        match self.process_heif(path, None) {
            Ok(data) => PluginResponse::Preview {
                components: vec![Component::Image(ImageComponent {
                    source: ImageSource::Bytes {
                        format: ImageFormat::Png,
                        data: data.png_data,
                        uid: path.to_string(),
                    },
                    interactive: true,
                })],
            },
            Err(e) => PluginResponse::Error {
                message: format!("Failed to process HEIF file for popup: {}", e),
            },
        }
    }

    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
}

impl HeifPlugin {
    fn process_heif(
        &self,
        path: &str,
        available_width: Option<f32>,
    ) -> Result<HeifData, Box<dyn std::error::Error>> {
        let lib_heif = LibHeif::new();
        let ctx = HeifContext::read_from_file(path)?;
        let handle = ctx.primary_image_handle()?;

        // Decode the image
        let image = lib_heif.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgb), None)?;

        let width = image.width();
        let height = image.height();

        let planes = image.planes();
        let interleaved_plane = planes.interleaved.ok_or("No interleaved plane found")?;
        let data = interleaved_plane.data;
        let stride = interleaved_plane.stride;

        // Handle stride if necessary (if stride != width * 3)
        let packed_data = if stride == (width * 3) as usize {
            data.to_vec()
        } else {
            let mut buffer = Vec::with_capacity((width * height * 3) as usize);
            for y in 0..height {
                let start = (y as usize) * stride;
                let end = start + (width as usize) * 3;
                buffer.extend_from_slice(&data[start..end]);
            }
            buffer
        };

        // Create image buffer from raw data
        let buffer = image::RgbImage::from_raw(width, height, packed_data)
            .ok_or("Failed to create image buffer")?;
        let mut dynamic_image = image::DynamicImage::ImageRgb8(buffer);

        // Resize if the image is wider than available width
        if let Some(available_width) = available_width {
            let available_width_u32 = available_width as u32;
            if width > available_width_u32 {
                let new_height = (height as f64 * (available_width as f64 / width as f64)) as u32;
                dynamic_image = dynamic_image.resize(
                    available_width_u32,
                    new_height,
                    image::imageops::FilterType::Triangle,
                );
            }
        }

        // Encode to PNG
        let mut png_data = Vec::new();
        let mut cursor = Cursor::new(&mut png_data);
        dynamic_image.write_to(&mut cursor, image::ImageFormat::Png)?;

        // Get some metadata for the table
        let mut metadata_rows = vec![
            vec!["Width".to_string(), width.to_string()],
            vec!["Height".to_string(), height.to_string()],
        ];

        if let Some(color_space) = image.color_space() {
            metadata_rows.push(vec![
                "Color Space".to_string(),
                format!("{:?}", color_space),
            ]);
        }

        metadata_rows.push(vec![
            "Bit Depth".to_string(),
            image
                .bits_per_pixel(Channel::Interleaved)
                .map(|b| b.to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
        ]);

        // Add metadata info
        for metadata in handle.all_metadata() {
            if metadata.item_type.to_string() != "Exif" {
                continue;
            }

            // The first 4 bytes of the Exif data in HEIF are the offset to the TIFF header
            if metadata.raw_data.len() >= 4 {
                let offset =
                    u32::from_be_bytes(metadata.raw_data[0..4].try_into().unwrap_or([0; 4]))
                        as usize;
                let tiff_data = match 4usize
                    .checked_add(offset)
                    .and_then(|start| metadata.raw_data.get(start..))
                {
                    Some(data) => data,
                    None => {
                        metadata_rows.push(vec![
                            "EXIF Metadata".to_string(),
                            "Invalid EXIF data offset".to_string(),
                        ]);
                        continue;
                    }
                };
                if let Ok((exif_fields, _)) = exif::parse_exif(tiff_data) {
                    for field in &exif_fields {
                        metadata_rows.push(vec![
                            field.tag.to_string(),
                            field.display_value().with_unit(field).to_string(),
                        ]);
                    }
                } else {
                    metadata_rows.push(vec![
                        "EXIF Metadata".to_string(),
                        "Failed to parse EXIF data".to_string(),
                    ]);
                }
            }
        }

        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("HEIF Preview")
            .to_string();

        Ok(HeifData {
            filename,
            png_data,
            metadata_rows,
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    HeifPlugin {
        metadata: PluginMetadata {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "HEIF/HEIC image preview plugin".to_string(),
            homepage: None,
            capabilities: PluginCapabilities {
                preview: Some(PreviewCapability {
                    file_pattern: r"(?i)\.(heif|heic)$".to_string(),
                }),
            },
        },
    }
    .run();
    Ok(())
}
