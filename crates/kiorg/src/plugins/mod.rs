//! Simple plugin system for basic plugin discovery and management
//!
//! This module provides a simplified plugin system for discovering and managing
//! external plugin executables.

pub mod manager;

pub use manager::PluginManager;

// Re-export types from the kiorg_plugin crate
pub use kiorg_plugin::{
    CallId, EngineCommand, EngineMessage, HelloMessage, PluginMetadata, PluginResponse,
};
