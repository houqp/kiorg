# Kiorg

<p align="center">
  <img src="assets/icon.png" alt="Kiorg Logo" width="200px">
</p>

Kiorg is a performance focused cross-platform file manager with Vim-inspired key
bindings. It is built using the [egui](https://www.egui.rs/#demo) framework.

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

## Installation

Pre-built binaries for all platforms are available on the [releases page](https://github.com/houqp/kiorg/releases).

Alternatively, you can build it from source using cargo:

```bash
cargo install --git https://github.com/houqp/kiorg --locked
```

## Configuration

Kiorg uses TOML configuration files stored in the user's config directory:

* Linux: `~/.config/kiorg/`
* macOS: `~/.config/kiorg/` (if it exists) or `~/Library/Application Support/kiorg/`
* Windows: `%APPDATA%\kiorg\`

### Sample Configuration

```toml
# Sort preference configuration (optional)
[sort_preference]
column = "Name"             # Sort column: "Name", "Modified", "Size", or "None"
order = "Ascending"         # Sort order: "Ascending" or "Descending"

# Override default shortcuts (optional)
[shortcuts]
MoveDown = [
  { key = "j" },
  { key = "down" }
]
MoveUp = [
  { key = "k" },
  { key = "up" }
]
DeleteEntry = [
  { key = "d" }
]
ActivateSearch = [
  { key = "/" },
  { key = "f", ctrl = true }
]
```