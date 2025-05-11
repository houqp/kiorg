# Kiorg

![Kiorg Logo](assets/icon.png)

Kiorg is a performance focused cross-platform file management app with Vim-inspired key bindings, built using the [egui](https://www.egui.rs/#demo) framework.

## Key Features

* Lightingly fast rendering and navigation
* Multi-tab support
* Vim-inspired keyboard shortcuts
* Content preview for various file formats
* Customizable shortcuts and color themes through TOML config files
* Cross-platform support (Linux, macOS, Windows)
* Bookmarks for quick access to frequently used directories
* Single binary with battery included
* Builtin terminal emulator
* App state persistence

## Configuration

Kiorg uses TOML configuration files stored in the user's config directory:

* Linux: `~/.config/kiorg/`
* macOS: `~/.config/kiorg/` (if it exists) or `~/Library/Application Support/kiorg/`
* Windows: `%APPDATA%\kiorg\`