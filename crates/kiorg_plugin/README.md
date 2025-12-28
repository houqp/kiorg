# kiorg-plugin

A Rust crate providing the plugin framework for the kiorg file manager.

## Overview

This crate defines the communication protocol and utilities for building
plugins that integrate with kiorg. Plugins communicate with the main
application using MessagePack-encoded messages over stdin/stdout.

## Usage

Add this crate to your plugin's dependencies:

```toml
[dependencies]
kiorg_plugin = "*"
```

### Basic Plugin Structure

```rust
use kiorg_plugin::{
    Component, PluginCapabilities, PluginHandler, PluginMetadata,
    PluginResponse, PreviewCapability, TextComponent,
};

struct MyPlugin;

impl PluginHandler for MyPlugin {
    fn on_preview(&mut self, path: &str) -> PluginResponse {
        // Return rich preview components
        PluginResponse::Preview {
            components: vec![Component::Text(TextComponent {
                text: format!("Previewing file: {}", path),
            })],
        }
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "my-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "A simple kiorg preview plugin".to_string(),
            homepage: None,
            capabilities: PluginCapabilities {
                preview: Some(PreviewCapability {
                    file_pattern: r"\.txt$".to_string(), // Match .txt files
                }),
            },
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    MyPlugin.run();
    Ok(())
}
```
