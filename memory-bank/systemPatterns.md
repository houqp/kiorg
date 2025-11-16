## System Patterns

### System architecture
The application follows a modular architecture, with separate modules for UI, models, configuration, and utilities. The UI is built using the egui framework and is composed of several panels: left panel (bookmarks), center panel (file list), right panel (preview).

The configuration system has been expanded to include user preferences such as column sort order, which are persisted between application sessions using TOML files.

### Key technical decisions
*   Using Rust for performance, safety, and cross-platform compatibility.
*   Using egui for rapid UI development.
*   Using TOML for configuration files.
*   Using serde for serialization and deserialization of configuration data.
*   Implementing user preference persistence for improved user experience.
*   Implementing async operations for long-running tasks to prevent UI blocking.
*   Adding performance benchmarking infrastructure for optimization guidance.

### Design patterns in use
*   Composition over inheritance.
*   Modular design.

### Component relationships
*   The `app.rs` module is the main entry point and orchestrates the other modules.
*   The `ui` module contains the UI components.
*   The `models` module defines the data structures.
*   The `config` module handles the application configuration and user preferences persistence.
*   The `utils` module provides utility functions.
*   The `Tab` model now interacts with the `Config` module to initialize with persisted sort preferences.
*   The center panel UI interacts with the `Config` module to save sort preferences when they change.
*   Platform-specific popup modules (`volumes.rs`, `windows_drives.rs`) handle OS-specific file system browsing.
*   The preview system includes dedicated handlers for different content types, including zoomable image previews.

### Critical implementation paths
*   File navigation: `src/ui/center_panel.rs` handles the display of files and folders, and responds to keyboard input for navigation.
*   Main app entry point: `src/app.rs`.
*   Bookmark management: `src/ui/left_panel.rs` handles the display and management of bookmarks.
*   Configuration management: `src/config/mod.rs` handles loading and saving application configuration.
*   Sort order persistence: Column sorting in `src/ui/file_list.rs` triggers configuration updates through `src/ui/center_panel.rs`.
