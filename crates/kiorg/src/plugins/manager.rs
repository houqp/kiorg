//! Simple synchronous plugin manager for discovering and managing plugins
//!
//! The PluginManager is responsible for:
//! - Discovering plugins in specified directories
//! - Managing basic plugin metadata
//! - Simple plugin operations without complex async execution

use kiorg_plugin::{CallId, EngineCommand, EngineMessage, PluginMetadata};
use snafu::Snafu;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Plugin executable prefix
const PLUGIN_PREFIX: &str = "kiorg_plugin_";

/// Error types for plugin management
#[derive(Debug, Snafu)]
pub enum PluginError {
    #[snafu(display("Plugin not found: {}", name))]
    NotFound { name: String },
    #[snafu(display("Plugin execution error: {}", message))]
    ExecutionError { message: String },
    #[snafu(display("Protocol error: {}", message))]
    ProtocolError { message: String },
    #[snafu(display("Incompatible plugin protocol version: {}", protocol_version))]
    Incompatible {
        protocol_version: String,
        metadata: Box<PluginMetadata>,
    },
    #[snafu(display("IO error: {}", source))]
    IoError { source: std::io::Error },
}

/// A failed plugin load attempt
#[derive(Debug, Clone)]
pub struct FailedPlugin {
    /// Plugin executable path
    pub path: PathBuf,
    /// Error message
    pub error: String,
}

/// A simple loaded plugin reference with running process
#[derive(Debug)]
pub struct LoadedPlugin {
    /// Plugin metadata
    pub metadata: PluginMetadata,
    /// Plugin executable path
    pub path: PathBuf,
    /// Running plugin process
    pub process: Mutex<Child>,
    /// Error state if plugin has crashed or failed
    pub error: Option<String>,
    /// Time taken to load the plugin
    pub load_time: std::time::Duration,
    /// Compiled regex for preview file pattern matching
    pub preview_regex: Option<regex::Regex>,
}

impl Drop for LoadedPlugin {
    fn drop(&mut self) {
        let result = self.process.lock();
        let mut child = match result {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        let _ = child.kill();
        let _ = child.wait();
    }
}

impl LoadedPlugin {
    /// Execute preview command on the plugin for the given file path
    pub fn preview_file(
        &mut self,
        file_path: &str,
    ) -> Result<Vec<kiorg_plugin::Component>, PluginError> {
        // Check if plugin already has an error state
        if let Some(error) = &self.error {
            return Err(PluginError::ExecutionError {
                message: format!("Plugin is in error state: {}", error),
            });
        }

        // Create the preview command message
        let engine_message = EngineMessage {
            id: CallId::new(),
            command: EngineCommand::Preview {
                path: file_path.to_string(),
            },
        };

        let plugin_name = &self.metadata.name;
        debug!(
            "Sending preview message to plugin '{}': {:?}",
            plugin_name, engine_message
        );

        // Send message to the running plugin process
        let mut process_guard = match self.process.lock() {
            Ok(guard) => guard,
            Err(_) => {
                let error_msg = "Failed to acquire plugin process lock".to_string();
                self.error = Some(error_msg.clone());
                return Err(PluginError::ExecutionError { message: error_msg });
            }
        };

        // Send the message to plugin stdin with length prefix
        match communicate_with_plugin(
            &mut process_guard,
            engine_message,
            std::time::Duration::from_secs(5),
            plugin_name,
        ) {
            Ok(plugin_response) => {
                // Extract the preview content
                match plugin_response {
                    kiorg_plugin::PluginResponse::Preview { components } => Ok(components),
                    _ => Err(PluginError::ProtocolError {
                        message: "Expected Preview response from plugin".to_string(),
                    }),
                }
            }
            Err(e) => {
                self.error = Some(e.to_string());
                Err(e)
            }
        }
    }
}

/// Helper to handle communication with a plugin process
fn communicate_with_plugin(
    child: &mut std::process::Child,
    message: EngineMessage,
    timeout: std::time::Duration,
    plugin_name: &str,
) -> Result<kiorg_plugin::PluginResponse, PluginError> {
    let mut stdin = child.stdin.take().ok_or(PluginError::ExecutionError {
        message: "Plugin stdin not available".to_string(),
    })?;
    let mut stdout = child.stdout.take().ok_or(PluginError::ExecutionError {
        message: "Plugin stdout not available".to_string(),
    })?;

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        // Send
        if let Err(e) = kiorg_plugin::send_message_to_writer(&mut stdin, &message) {
            let _ = tx.send(Err(format!("Failed to send message: {}", e)));
            return;
        }

        // Read
        let result: Result<kiorg_plugin::PluginResponse, _> =
            kiorg_plugin::read_message_from_reader(&mut stdout);
        match result {
            Ok(response) => {
                let _ = tx.send(Ok((response, stdin, stdout)));
            }
            Err(e) => {
                let _ = tx.send(Err(format!("Failed to read response: {}", e)));
            }
        }
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok((plugin_response, stdin_back, stdout_back))) => {
            debug!(
                "Received response from plugin '{}': {:?}",
                plugin_name, plugin_response
            );
            child.stdin = Some(stdin_back);
            child.stdout = Some(stdout_back);
            Ok(plugin_response)
        }
        other => {
            // Check if the process has exited
            if let Ok(Some(status)) = child.try_wait() {
                let error_msg = format!("Plugin process exited unexpectedly: {}", status);
                error!("Plugin '{}' crashed: {}", plugin_name, error_msg);
                return Err(PluginError::ExecutionError { message: error_msg });
            }

            // If process is still running (or we can't check), kill it
            let _ = child.kill();
            let _ = child.wait();

            match other {
                Ok(Err(msg)) => {
                    let error_msg = format!("Plugin communication error: {}", msg);
                    error!("Plugin '{}' error: {}", plugin_name, error_msg);
                    Err(PluginError::ProtocolError { message: error_msg })
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    let error_msg = format!(
                        "Timed out waiting for response from plugin '{}'",
                        plugin_name
                    );
                    error!("Plugin '{}' error: {}", plugin_name, error_msg);
                    Err(PluginError::ExecutionError { message: error_msg })
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    let error_msg = "Plugin response thread disconnected unexpectedly".to_string();
                    error!("Plugin '{}' error: {}", plugin_name, error_msg);
                    Err(PluginError::ExecutionError { message: error_msg })
                }
                Ok(Ok(_)) => unreachable!(),
            }
        }
    }
}

/// Simple plugin manager for basic discovery and management
pub struct PluginManager {
    /// Plugin directory path
    plugin_dir: PathBuf,
    /// Loaded plugins
    loaded: HashMap<String, LoadedPlugin>,
    /// Failed plugins
    failed: Vec<FailedPlugin>,
}

impl PluginManager {
    /// Create a new plugin manager with config directory override
    pub fn new(config_dir_override: Option<&PathBuf>) -> Self {
        let config_dir = crate::config::get_kiorg_config_dir(config_dir_override);
        let plugin_dir = config_dir.join("plugins");

        Self {
            plugin_dir,
            loaded: HashMap::new(),
            failed: Vec::new(),
        }
    }

    /// Load all plugins found in configured directories
    pub fn load_plugins(&mut self) -> Result<(), PluginError> {
        if !self.plugin_dir.exists() {
            debug!("Plugin directory does not exist: {:?}", self.plugin_dir);
            return Ok(());
        }

        let entries =
            std::fs::read_dir(&self.plugin_dir).map_err(|e| PluginError::IoError { source: e })?;
        for entry in entries {
            let entry = entry.map_err(|e| PluginError::IoError { source: e })?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if let Some(filename) = path.file_name().and_then(|n| n.to_str())
                && filename.starts_with(PLUGIN_PREFIX)
            {
                info!("Discovered plugin file: {:?}", path);

                match self.load_single_plugin(&path) {
                    Ok(plugin) => {
                        let name = plugin.metadata.name.clone();

                        // Skip if already loaded
                        if self.loaded.contains_key(&name) {
                            debug!("Plugin '{}' already loaded, skipping", name);
                            continue;
                        }

                        info!(
                            "Plugin '{}' loaded successfully in {:?}",
                            name, plugin.load_time
                        );
                        self.loaded.insert(name.clone(), plugin);

                        // Remove from failed if it was there previously (by path)
                        self.failed
                            .retain(|failed_plugin| failed_plugin.path != path);
                    }
                    Err(e) => {
                        warn!("Failed to load plugin from '{:?}': {}", path, e);
                        // Remove existing failure for this path to avoid duplicates
                        self.failed.retain(|p| p.path != path);
                        self.failed.push(FailedPlugin {
                            path: path.clone(),
                            error: e.to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a single plugin from the given path
    fn load_single_plugin(&self, path: &PathBuf) -> Result<LoadedPlugin, PluginError> {
        // Start the plugin process
        let mut cmd = Command::new(path);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let start_time = std::time::Instant::now();

        let mut child = cmd.spawn().map_err(|e| PluginError::ExecutionError {
            message: format!("Failed to spawn plugin process: {}", e),
        })?;

        // Perform hello handshake to get plugin metadata
        let (metadata, error) = match self.perform_hello_handshake(&mut child) {
            Ok(meta) => (meta, None),
            Err(PluginError::Incompatible {
                protocol_version,
                metadata,
            }) => {
                let major_version = protocol_version.split('.').next().unwrap_or("0");
                (
                    *metadata,
                    Some(format!(
                        "Incompatible protocol version. Plugin built for protocol major version: {}",
                        major_version
                    )),
                )
            }
            Err(e) => {
                let _ = child.kill();
                return Err(e);
            }
        };

        let load_time = start_time.elapsed();

        // Compile preview regex if available
        let preview_regex = if let Some(preview_cap) = &metadata.capabilities.preview {
            match regex::Regex::new(&preview_cap.file_pattern) {
                Ok(regex) => Some(regex),
                Err(e) => {
                    let _ = child.kill();
                    return Err(PluginError::ExecutionError {
                        message: format!("Invalid regex pattern: {}", e),
                    });
                }
            }
        } else {
            None
        };

        Ok(LoadedPlugin {
            metadata,
            path: path.clone(),
            process: Mutex::new(child),
            error,
            load_time,
            preview_regex,
        })
    }

    /// Perform hello handshake with a plugin to get metadata and capabilities
    fn perform_hello_handshake(&self, child: &mut Child) -> Result<PluginMetadata, PluginError> {
        let hello_message = EngineMessage {
            id: CallId::new(),
            command: EngineCommand::Hello {
                protocol_version: kiorg_plugin::PROTOCOL_VERSION.to_string(),
            },
        };

        match communicate_with_plugin(
            child,
            hello_message,
            std::time::Duration::from_secs(2),
            "initializing...",
        )? {
            kiorg_plugin::PluginResponse::Hello(hello_response) => Ok(hello_response),
            kiorg_plugin::PluginResponse::VersionIncompatible {
                protocol_version,
                metadata,
            } => Err(PluginError::Incompatible {
                protocol_version,
                metadata: Box::new(metadata),
            }),
            _ => Err(PluginError::ProtocolError {
                message: "Expected Hello response from plugin".to_string(),
            }),
        }
    }

    /// Unload a plugin by name
    fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        // Remove from loaded plugins and terminate process
        if self.loaded.remove(name).is_some() {
            info!("Plugin '{}' unloaded successfully", name);
            Ok(())
        } else {
            Err(PluginError::NotFound {
                name: name.to_string(),
            })
        }
    }

    /// List loaded plugins
    pub fn list_loaded(&self) -> &HashMap<String, LoadedPlugin> {
        &self.loaded
    }

    /// List failed plugins
    pub fn list_failed(&self) -> &Vec<FailedPlugin> {
        &self.failed
    }

    /// Get the first plugin that can preview the given file name (mutable reference)
    pub fn get_preview_plugin_for_file_mut(
        &mut self,
        file_name: &str,
    ) -> Option<&mut LoadedPlugin> {
        self.loaded.values_mut().find(|plugin| {
            plugin
                .preview_regex
                .as_ref()
                .is_some_and(|regex| regex.is_match(file_name))
        })
    }

    /// Shutdown plugin manager
    pub fn shutdown(&mut self) -> Result<(), PluginError> {
        // Unload all plugins
        let plugin_names: Vec<String> = self.loaded.keys().cloned().collect();

        for name in plugin_names {
            if let Err(e) = self.unload_plugin(&name) {
                warn!("Failed to unload plugin '{}' during shutdown: {}", name, e);
            }
        }

        info!("Plugin manager shutdown complete");
        Ok(())
    }
}
