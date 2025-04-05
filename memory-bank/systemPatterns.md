## System Patterns

### System architecture
The application follows a modular architecture, with separate modules for UI, models, configuration, and utilities. The UI is built using the egui framework and is composed of several panels: left panel (bookmarks), center panel (file list), right panel (preview).

### Key technical decisions
*   Using Rust for performance, safety, and cross-platform compatibility.
*   Using egui for rapid UI development.
*   Using TOML for configuration files.

### Design patterns in use
*   Composition over inheritance.
*   Modular design.

### Component relationships
*   The `app.rs` module is the main entry point and orchestrates the other modules.
*   The `ui` module contains the UI components.
*   The `models` module defines the data structures.
*   The `config` module handles the application configuration.
*   The `utils` module provides utility functions.

### Critical implementation paths
*   File navigation: `src/ui/center_panel.rs` handles the display of files and folders, and responds to keyboard input for navigation.
*   Main app entry point: `src/app.rs`.
*   Bookmark management: `src/ui/left_panel.rs` handles the display and management of bookmarks.
