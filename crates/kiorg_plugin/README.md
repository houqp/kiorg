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
    read_message, send_message, EngineCommand,
    PluginResponse, HelloMessage
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        match read_message() {
            Ok(message) => {
                let response = match message.command {
                    EngineCommand::Hello(_) => {
                        PluginResponse::Hello(HelloMessage {
                            version: "1.0.0".to_string(),
                        })
                    }
                    EngineCommand::Preview { path } => {
                        // Generate preview content for the file
                        let content = generate_preview(&path)?;
                        PluginResponse::Preview { content }
                    }
                };
                send_message(&response)?;
            }
            Err(e) => {
                eprintln!("Error reading message: {}", e);
                break;
            }
        }
    }
    Ok(())
}
```

