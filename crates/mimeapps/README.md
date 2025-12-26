# mimeapps

Cross-platform library for looking up registered applications for a given file path.

## Features

- **Linux**: Full implementation of the [freedesktop.org MIME associations specification](https://specifications.freedesktop.org/mime-apps-spec/mime-apps-spec-latest.html). It parses `mimeapps.list` files and desktop entries to find associated applications.
- **macOS**: Uses `NSWorkspace` to find applications capable of opening a given file.
- **Windows**: (Pending implementation) Currently returns an empty list.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
mimeapps = "0.1.0"
```

### Example

```rust
use std::path::Path;
use mimeapps::get_apps_for_file;

fn main() {
    let path = Path::new("document.pdf");
    let apps = get_apps_for_file(path);

    for app in apps {
        println!("Application Name: {}", app.name);
        println!("Executable Path: {}", app.path);
    }
}
```

## Platform-specific details

### Linux

The library follows the XDG specification to look up applications:
1. It determines the MIME type of the file.
2. It looks up associations in `mimeapps.list` files in the following order:
   - `$XDG_CONFIG_HOME/mimeapps.list`
   - `$XDG_CONFIG_DIRS/mimeapps.list`
   - `$XDG_DATA_HOME/applications/mimeapps.list`
   - `$XDG_DATA_DIRS/applications/mimeapps.list`
3. It also searches for applications in `defaults.list` (deprecated but still used as fallback).
4. It verifies the existence of `.desktop` files and respects `NoDisplay` and `Hidden` fields.

### macOS

Uses `NSWorkspace`'s `URLsForApplicationsToOpenURL:` to retrieve a list of applications that can open the specified file URL.