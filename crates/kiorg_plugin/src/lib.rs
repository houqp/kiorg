//! Plugin protocol definitions and message types
//!
//! This module defines the communication protocol between the host application
//! and plugins using MessagePack serialization.

use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
pub use uuid;
pub use uuid::Uuid;

/// Protocol version for compatibility checking
/// Major version changes indicate incompatible protocol changes
pub const PROTOCOL_VERSION: &str = "0.0.1";

/// Check if the provided engine version is compatible with this plugin library version
pub fn check_compatibility(engine_version: &str) -> bool {
    let engine_major = engine_version.split('.').next().unwrap_or("0");
    let my_major = PROTOCOL_VERSION.split('.').next().unwrap_or("0");

    engine_major == my_major
}

/// Unique identifier for plugin calls - serialized as bytes for efficiency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallId(#[serde(with = "uuid_bytes")] pub Uuid);

impl CallId {
    /// Create a new random CallId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CallId {
    fn default() -> Self {
        Self::new()
    }
}

mod uuid_bytes {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use uuid::Uuid;

    pub fn serialize<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(uuid.as_bytes())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = <[u8; 16]>::deserialize(deserializer)?;
        Ok(Uuid::from_bytes(bytes))
    }
}

/// Unique identifier for streams
pub type StreamId = Uuid;

/// Hello message exchanged during plugin handshake
pub type HelloMessage = PluginMetadata;

/// Plugin capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapabilities {
    /// Preview rendering capabilities
    pub preview: Option<PreviewCapability>,
}

/// Preview rendering capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewCapability {
    /// Regex pattern to match file names/extensions that this plugin can preview
    pub file_pattern: String,
}

/// Commands that can be sent from engine to plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "_T")]
pub enum EngineCommand {
    /// Initial handshake message
    Hello { protocol_version: String },
    /// Preview command - takes a file path
    Preview { path: String },
    /// Preview popup command - takes a file path
    PreviewPopup { path: String },
}

/// Message sent from engine to plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMessage {
    /// Unique identifier for this message
    pub id: CallId,
    /// The command to execute
    pub command: EngineCommand,
}

/// Response from plugin to engine
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "_T")]
pub enum PluginResponse {
    /// Hello response
    Hello(HelloMessage),
    /// Preview response with content to display
    Preview { components: Vec<Component> },
    /// Version incompatible response
    VersionIncompatible {
        protocol_version: String,
        metadata: PluginMetadata,
    },
    /// Error response for reporting issues back to the engine
    Error { message: String },
}

/// Component types for rich preview
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Component {
    Title(TitleComponent),
    Text(TextComponent),
    Image(ImageComponent),
    Table(TableComponent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleComponent {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextComponent {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageComponent {
    pub source: ImageSource,
    pub interactive: bool,
}

impl ImageComponent {
    /// Create a non-interactive image component from an image source
    pub fn from_source(source: ImageSource) -> Self {
        Self {
            source,
            interactive: false,
        }
    }

    /// Create an interactive image component from an image source
    pub fn from_source_interactive(source: ImageSource) -> Self {
        Self {
            source,
            interactive: true,
        }
    }
}

pub use image::ImageFormat;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ImageSource {
    Path(String),
    Bytes {
        format: ImageFormat,
        /// image data, must conform to the format specified
        data: Vec<u8>,
        /// unique identifier for the image
        uid: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableComponent {
    pub headers: Option<Vec<String>>,
    pub rows: Vec<Vec<String>>,
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin homepage URL
    pub homepage: Option<String>,
    /// Plugin capabilities
    pub capabilities: PluginCapabilities,
}

/// Trait for implementing a plugin
pub trait PluginHandler {
    fn on_hello(&mut self, protocol_version: &str) -> PluginResponse {
        if !check_compatibility(protocol_version) {
            return PluginResponse::VersionIncompatible {
                protocol_version: PROTOCOL_VERSION.to_string(),
                metadata: self.metadata(),
            };
        }
        PluginResponse::Hello(self.metadata())
    }
    fn on_preview(&mut self, path: &str) -> PluginResponse;
    fn on_preview_popup(&mut self, path: &str) -> PluginResponse {
        self.on_preview(path)
    }
    fn metadata(&self) -> PluginMetadata;

    fn run(mut self)
    where
        Self: std::marker::Sized,
    {
        if !self.parse_args() {
            return; // Exit if help was shown
        }
        let metadata = self.metadata();
        eprintln!("Starting {} v{}", metadata.name, metadata.version);
        self.run_plugin_loop();
    }

    /// Run the main loop for a plugin
    ///
    /// This function will read messages from stdin and dispatch them to the handler.
    /// It will exit when stdin is closed (host process exited) or on communication error.
    fn run_plugin_loop(&mut self) {
        loop {
            match read_message() {
                Ok(message) => {
                    let response = match message.command {
                        EngineCommand::Hello { protocol_version } => {
                            self.on_hello(&protocol_version)
                        }
                        EngineCommand::Preview { path } => self.on_preview(&path),
                        EngineCommand::PreviewPopup { path } => self.on_preview_popup(&path),
                    };

                    if send_message(&response).is_err() {
                        // Failed to send response, host probably disconnected
                        break;
                    }
                }
                Err(e) => {
                    // Check if the error is a clean shutdown (UnexpectedEof)
                    if let Some(io_err) = e.downcast_ref::<io::Error>() {
                        if io_err.kind() == io::ErrorKind::UnexpectedEof {
                            break;
                        }
                    }

                    let error_msg = format!("Invalid command received: {}", e);
                    eprintln!("{}", error_msg);

                    // Try to send the error back to the engine
                    let error_response = PluginResponse::Error { message: error_msg };
                    if send_message(&error_response).is_err() {
                        eprintln!("Failed to send error response to engine");
                        std::process::exit(-2);
                    }
                }
            }
        }
    }

    /// Helper function to parse command line arguments and print help information
    ///
    /// This function should be called at the start of main() to handle --help argument.
    /// It will print plugin information and capabilities, then exit if --help was provided
    /// or if unsupported arguments are passed.
    ///
    /// # Returns
    /// * `true` if the plugin should continue running (normal mode)
    /// * `false` if the plugin should exit (help was printed)
    fn parse_args(&self) -> bool {
        let args: Vec<String> = std::env::args().collect();
        let metadata = self.metadata();

        // Function to print help message
        let print_help = || {
            println!("{} v{}", metadata.name, metadata.version);
            println!("{}\n", metadata.description);
            println!("Capabilities:");
            if let Some(preview_cap) = &metadata.capabilities.preview {
                println!("  Preview Support:");
                println!("    File Pattern: {}", preview_cap.file_pattern);
                println!(
                    "    Description: Provides preview content for files matching the pattern"
                );
            } else {
                println!("  No preview support");
            }
            println!();
            println!("To install this plugin:");
            println!("  1. Copy the plugin binary into the plugins directory under kiorg's config directory.");
            println!("  2. Make sure its name starts with 'kiorg_plugin_'");
        };

        if args.len() > 1 {
            if args[1] == "--help" || args[1] == "-h" {
                print_help();
                false
            } else {
                // Unsupported argument
                println!("Unsupported argument: {}", args[1]);
                println!();
                print_help();
                false
            }
        } else {
            true
        }
    }
}

/// Read a MessagePack message from stdin
pub fn read_message() -> Result<EngineMessage, Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    read_message_from_reader(&mut handle)
}

/// Read a MessagePack message from any reader
pub fn read_message_from_reader<R: Read, T: serde::de::DeserializeOwned>(
    reader: &mut R,
) -> Result<T, Box<dyn std::error::Error>> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let msg_len = u32::from_be_bytes(len_buf) as usize;

    let mut msg_buf = vec![0u8; msg_len];
    reader.read_exact(&mut msg_buf)?;

    let message: T = rmp_serde::from_slice(&msg_buf)?;
    Ok(message)
}

/// Send a MessagePack message to stdout
pub fn send_message(response: &PluginResponse) -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    send_message_to_writer(&mut handle, response)
}

/// Send a MessagePack message to any writer
pub fn send_message_to_writer<W: Write, T: Serialize>(
    writer: &mut W,
    message: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    // Use to_vec_named to ensure structs/enums are serialized as maps,
    // which is required for internally tagged enums (#[serde(tag = "_T")])
    let bytes = rmp_serde::to_vec_named(message)?;
    let len = bytes.len() as u32;
    writer.write_all(&len.to_be_bytes())?;
    writer.write_all(&bytes)?;
    writer.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_engine_message_serialization() {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let cmd = EngineMessage {
            id: CallId(id),
            command: EngineCommand::Hello {
                protocol_version: "1.0.0".to_string(),
            },
        };

        let bytes = rmp_serde::to_vec_named(&cmd).unwrap();

        // Expected MessagePack serialization:
        // Map(2)
        //   "id": Bin(16) <uuid bytes>
        //   "command": Map(2)
        //     "_T": "Hello"
        //     "protocol_version": "1.0.0"

        let expected = vec![
            0x82, // Map(2)
            // Key "id"
            0xa2, 0x69, 0x64, // Value: Bin(16) + bytes
            0xc4, 0x10, 0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66,
            0x55, 0x44, 0x00, 0x00, // Key "command"
            0xa7, 0x63, 0x6f, 0x6d, 0x6d, 0x61, 0x6e, 0x64, // Value: Map(2)
            0x82, // Key "_T"
            0xa2, 0x5f, 0x54, // Value "Hello"
            0xa5, 0x48, 0x65, 0x6c, 0x6c, 0x6f, // Key "protocol_version"
            0xb0, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x63, 0x6f, 0x6c, 0x5f, 0x76, 0x65, 0x72, 0x73,
            0x69, 0x6f, 0x6e, // Value "1.0.0"
            0xa5, 0x31, 0x2e, 0x30, 0x2e, 0x30,
        ];

        assert_eq!(
            bytes, expected,
            "Serialized bytes do not match expected format"
        );
    }

    #[test]
    fn test_plugin_response_serialization() {
        let resp = PluginResponse::Preview {
            components: vec![Component::Text(TextComponent {
                text: "hello".to_string(),
            })],
        };
        let bytes = rmp_serde::to_vec_named(&resp).unwrap();

        // Expected:
        // Map(2)
        //   "_T": "Preview"
        //   "components": Array(1)
        //     Map(2)
        //       "type": "Text"
        //       "text": "hello"

        let expected = vec![
            0x82, // Map(2)
            // Key "_T"
            0xa2, 0x5f, 0x54, // Value "Preview"
            0xa7, 0x50, 0x72, 0x65, 0x76, 0x69, 0x65, 0x77, // Key "components"
            0xaa, 0x63, 0x6f, 0x6d, 0x70, 0x6f, 0x6e, 0x65, 0x6e, 0x74,
            0x73, // Value Array(1)
            0x91, // Array(1)
            0x82, // Map(2)
            // Key "type"
            0xa4, 0x74, 0x79, 0x70, 0x65, // Value "Text"
            0xa4, 0x54, 0x65, 0x78, 0x74, // Key "text"
            0xa4, 0x74, 0x65, 0x78, 0x74, // Value "hello"
            0xa5, 0x68, 0x65, 0x6c, 0x6c, 0x6f,
        ];

        assert_eq!(bytes, expected, "PluginResponse bytes mismatch");
    }

    #[test]
    fn test_plugin_hello_response_serialization() {
        let caps = PluginCapabilities { preview: None };
        let msg = PluginMetadata {
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test Plugin".to_string(),
            homepage: Some("https://example.com".to_string()),
            capabilities: caps,
        };
        let resp = PluginResponse::Hello(msg);
        let bytes = rmp_serde::to_vec_named(&resp).unwrap();

        // Expected:
        // Map(6)
        //   "_T": "Hello"
        //   "name": "Test Plugin"
        //   "version": "1.0.0"
        //   "description": "Test Plugin"
        //   "homepage": "https://example.com"
        //   "capabilities": Map(1)
        //     "preview": Nil

        let expected = vec![
            0x86, // Map(6)
            // Key "_T"
            0xa2, 0x5f, 0x54, // Value "Hello"
            0xa5, 0x48, 0x65, 0x6c, 0x6c, 0x6f, // Key "name"
            0xa4, 0x6e, 0x61, 0x6d, 0x65, // Value "Test Plugin"
            0xab, 0x54, 0x65, 0x73, 0x74, 0x20, 0x50, 0x6c, 0x75, 0x67, 0x69, 0x6e,
            // Key "version"
            0xa7, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, // Value "1.0.0"
            0xa5, 0x31, 0x2e, 0x30, 0x2e, 0x30, // Key "description"
            0xab, 0x64, 0x65, 0x73, 0x63, 0x72, 0x69, 0x70, 0x74, 0x69, 0x6f, 0x6e,
            // Value "Test Plugin"
            0xab, 0x54, 0x65, 0x73, 0x74, 0x20, 0x50, 0x6c, 0x75, 0x67, 0x69, 0x6e,
            // Key "homepage"
            0xa8, 0x68, 0x6f, 0x6d, 0x65, 0x70, 0x61, 0x67, 0x65,
            // Value "https://example.com"
            0xb3, 0x68, 0x74, 0x74, 0x70, 0x73, 0x3a, 0x2f, 0x2f, 0x65, 0x78, 0x61, 0x6d, 0x70,
            0x6c, 0x65, 0x2e, 0x63, 0x6f, 0x6d, // Key "capabilities"
            0xac, 0x63, 0x61, 0x70, 0x61, 0x62, 0x69, 0x6c, 0x69, 0x74, 0x69, 0x65, 0x73,
            // Value Map(1)
            0x81, // Key "preview"
            0xa7, 0x70, 0x72, 0x65, 0x76, 0x69, 0x65, 0x77, // Value Nil
            0xc0,
        ];

        assert_eq!(bytes, expected, "PluginResponse::Hello bytes mismatch");
    }

    #[test]
    fn test_plugin_version_incompatible_response_serialization() {
        let caps = PluginCapabilities { preview: None };
        let meta = PluginMetadata {
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test Plugin".to_string(),
            homepage: Some("https://example.com".to_string()),
            capabilities: caps,
        };
        let resp = PluginResponse::VersionIncompatible {
            protocol_version: "0.0.2".to_string(),
            metadata: meta,
        };
        let bytes = rmp_serde::to_vec_named(&resp).unwrap();

        // Expected:
        // Map(3)
        //   "_T": "VersionIncompatible"
        //   "protocol_version": "0.0.2"
        //   "metadata": Map(6) ...

        let expected = vec![
            0x83, // Map(3)
            // Key "_T"
            0xa2, 0x5f, 0x54, // Value "VersionIncompatible"
            0xb3, 0x56, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x49, 0x6e, 0x63, 0x6f, 0x6d, 0x70,
            0x61, 0x74, 0x69, 0x62, 0x6c, 0x65, // Key "protocol_version"
            0xb0, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x63, 0x6f, 0x6c, 0x5f, 0x76, 0x65, 0x72, 0x73,
            0x69, 0x6f, 0x6e, // Value "0.0.2"
            0xa5, 0x30, 0x2e, 0x30, 0x2e, 0x32, // Key "metadata"
            0xa8, 0x6d, 0x65, 0x74, 0x61, 0x64, 0x61, 0x74, 0x61, // Value Map(5)
            0x85, // Key "name"
            0xa4, 0x6e, 0x61, 0x6d, 0x65, // Value "Test Plugin"
            0xab, 0x54, 0x65, 0x73, 0x74, 0x20, 0x50, 0x6c, 0x75, 0x67, 0x69, 0x6e,
            // Key "version"
            0xa7, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, // Value "1.0.0"
            0xa5, 0x31, 0x2e, 0x30, 0x2e, 0x30, // Key "description"
            0xab, 0x64, 0x65, 0x73, 0x63, 0x72, 0x69, 0x70, 0x74, 0x69, 0x6f, 0x6e,
            // Value "Test Plugin"
            0xab, 0x54, 0x65, 0x73, 0x74, 0x20, 0x50, 0x6c, 0x75, 0x67, 0x69, 0x6e,
            // Key "homepage"
            0xa8, 0x68, 0x6f, 0x6d, 0x65, 0x70, 0x61, 0x67, 0x65,
            // Value "https://example.com"
            0xb3, 0x68, 0x74, 0x74, 0x70, 0x73, 0x3a, 0x2f, 0x2f, 0x65, 0x78, 0x61, 0x6d, 0x70,
            0x6c, 0x65, 0x2e, 0x63, 0x6f, 0x6d, // Key "capabilities"
            0xac, 0x63, 0x61, 0x70, 0x61, 0x62, 0x69, 0x6c, 0x69, 0x74, 0x69, 0x65, 0x73,
            // Value Map(1)
            0x81, // Key "preview"
            0xa7, 0x70, 0x72, 0x65, 0x76, 0x69, 0x65, 0x77, // Value Nil
            0xc0,
        ];

        assert_eq!(
            bytes, expected,
            "PluginResponse::VersionIncompatible bytes mismatch"
        );
    }
}
