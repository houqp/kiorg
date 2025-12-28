//! Demo plugin demonstrating the simplified kiorg plugin system
//!
//! This plugin demonstrates the basic Hello/Preview protocol, always
//! returning "hello world" for preview requests.

use kiorg_plugin::{
    PluginCapabilities, PluginHandler, PluginMetadata, PluginResponse, PreviewCapability,
};

const ICON_BYTES: &[u8] = include_bytes!("../../../../../assets/icons/1024x1024@2x.png");

struct DemoPlugin {
    metadata: PluginMetadata,
}

impl PluginHandler for DemoPlugin {
    fn on_preview(&mut self, path: &str) -> PluginResponse {
        // Return preview content that includes the file path
        PluginResponse::Preview {
            components: vec![
                kiorg_plugin::Component::Title(kiorg_plugin::TitleComponent {
                    text: "Demo Plugin Preview".to_string(),
                }),
                kiorg_plugin::Component::Text(kiorg_plugin::TextComponent {
                    text: format!("Hello from demo plugin!\n\nFile: {}", path),
                }),
                kiorg_plugin::Component::Image(kiorg_plugin::ImageComponent::from_source(
                    kiorg_plugin::ImageSource::Bytes {
                        format: kiorg_plugin::ImageFormat::Png,
                        data: ICON_BYTES.to_vec(),
                        uid: kiorg_plugin::uuid::Uuid::new_v4().to_string(),
                    },
                )),
                kiorg_plugin::Component::Table(kiorg_plugin::TableComponent {
                    headers: Some(vec!["Property".to_string(), "Value".to_string()]),
                    rows: vec![
                        vec![
                            "Plugin Name".to_string(),
                            env!("CARGO_PKG_NAME").to_string(),
                        ],
                        vec!["Plugin Version".to_string(), self.metadata.version.clone()],
                    ],
                }),
            ],
        }
    }

    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    DemoPlugin {
        metadata: PluginMetadata {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description:
                "A demo plugin demonstrating kiorg plugin capabilities for preview rendering"
                    .to_string(),
            homepage: None,
            capabilities: PluginCapabilities {
                preview: Some(PreviewCapability {
                    file_pattern: r"^kiorg$".to_string(), // Match files named "kiorg"
                }),
            },
        },
    }
    .run();
    Ok(())
}
