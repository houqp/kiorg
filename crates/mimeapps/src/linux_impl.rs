/// Reference:
///  https://specifications.freedesktop.org/mime-apps/latest/index.html
///  https://wiki.archlinux.org/title/XDG_MIME_Applications
use super::AppInfo;
use file_type::FileType;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Parse a mimeapps.list file and return Added/Default/Removed associations
fn parse_mimeapps_file(path: &Path, mimetype: &str) -> (Vec<String>, Vec<String>) {
    let Ok(content) = fs::read_to_string(path) else {
        return (Vec::new(), Vec::new());
    };

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut current_section = None;

    for line in content.lines() {
        let line = line.trim();

        // Skip comments
        if line.starts_with('#') {
            continue;
        }

        // Handle section headers
        if line.starts_with('[') && line.ends_with(']') {
            current_section = match line {
                "[Default Applications]" => Some("default"),
                "[Added Associations]" => Some("added"),
                "[Removed Associations]" => Some("removed"),
                _ => None,
            };
            continue;
        }

        // Parse associations in current section
        let Some(section) = current_section else {
            continue;
        };
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != mimetype {
            continue;
        }

        let apps: Vec<String> = value
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        match section {
            "default" | "added" => added.extend(apps),
            "removed" => removed.extend(apps),
            _ => {}
        }
    }

    (added, removed)
}

/// Get XDG directories from environment or use defaults
fn get_xdg_config_home() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| h.join(".config"))
                .unwrap_or_else(|| PathBuf::from(".config"))
        })
}

fn get_xdg_config_dirs() -> Vec<PathBuf> {
    std::env::var("XDG_CONFIG_DIRS")
        .ok()
        .map(|dirs| {
            dirs.split(':')
                .filter(|d| !d.is_empty())
                .map(PathBuf::from)
                .collect()
        })
        .unwrap_or_else(|| vec![PathBuf::from("/etc/xdg")])
}

fn get_xdg_data_home() -> PathBuf {
    std::env::var("XDG_DATA_HOME")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| h.join(".local/share"))
                .unwrap_or_else(|| PathBuf::from(".local/share"))
        })
}

fn get_xdg_data_dirs() -> Vec<PathBuf> {
    std::env::var("XDG_DATA_DIRS")
        .ok()
        .map(|dirs| {
            dirs.split(':')
                .filter(|d| !d.is_empty())
                .map(PathBuf::from)
                .collect()
        })
        .unwrap_or_else(|| {
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share"),
            ]
        })
}

/// Get the current desktop(s) from XDG_CURRENT_DESKTOP
fn get_current_desktops() -> Vec<String> {
    std::env::var("XDG_CURRENT_DESKTOP")
        .ok()
        .map(|desktop| {
            desktop
                .split(':')
                .map(|d| d.trim().to_lowercase())
                .filter(|d| !d.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Build list of mimeapps.list paths to check in order
fn get_mimeapps_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let desktops = get_current_desktops();

    let config_home = get_xdg_config_home();
    let config_dirs = get_xdg_config_dirs();
    let data_home = get_xdg_data_home();
    let data_dirs = get_xdg_data_dirs();

    // Desktop-specific config home
    for desktop in &desktops {
        paths.push(config_home.join(format!("{}-mimeapps.list", desktop)));
    }

    // Config home
    paths.push(config_home.join("mimeapps.list"));

    // Desktop-specific config dirs
    for desktop in &desktops {
        for dir in &config_dirs {
            paths.push(dir.join(format!("{}-mimeapps.list", desktop)));
        }
    }

    // Config dirs
    for dir in &config_dirs {
        paths.push(dir.join("mimeapps.list"));
    }

    // Desktop-specific data home (deprecated)
    for desktop in &desktops {
        paths.push(
            data_home
                .join("applications")
                .join(format!("{}-mimeapps.list", desktop)),
        );
    }

    // Data home (deprecated)
    paths.push(data_home.join("applications/mimeapps.list"));

    // Desktop-specific data dirs
    for desktop in &desktops {
        for dir in &data_dirs {
            paths.push(
                dir.join("applications")
                    .join(format!("{}-mimeapps.list", desktop)),
            );
        }
    }

    // Data dirs
    for dir in &data_dirs {
        paths.push(dir.join("applications/mimeapps.list"));
    }

    paths
}

/// Check if a desktop file lists the given mimetype
fn desktop_file_has_mimetype(path: &Path, mimetype: &str) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };

    content
        .lines()
        .find(|line| line.starts_with("MimeType="))
        .map(|line| line[9..].split(';').any(|m| m == mimetype))
        .unwrap_or(false)
}

/// Find a desktop file by name in XDG data directories
/// Handles vendor-prefixed names (e.g., vendor-app.desktop -> vendor/app.desktop)
fn find_desktop_file(desktop_name: &str) -> Option<PathBuf> {
    let data_home = get_xdg_data_home();
    let data_dirs = get_xdg_data_dirs();

    for data_dir in std::iter::once(data_home).chain(data_dirs) {
        let apps_dir = data_dir.join("applications");

        // Try direct path first
        let direct_path = apps_dir.join(desktop_name);
        if direct_path.exists() {
            return Some(direct_path);
        }

        // Try vendor-prefixed path (e.g., foo-bar.desktop -> foo/bar.desktop)
        if let Some((vendor, app)) = desktop_name.split_once('-') {
            let vendor_path = apps_dir.join(vendor).join(app);
            if vendor_path.exists() {
                return Some(vendor_path);
            }
        }

        // Search in subdirectories
        if let Ok(entries) = fs::read_dir(&apps_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let subdir_path = path.join(desktop_name);
                    if subdir_path.exists() {
                        return Some(subdir_path);
                    }
                }
            }
        }
    }

    None
}

/// Parse a desktop file to extract Name and Exec fields
fn parse_desktop_file(path: &Path) -> Option<AppInfo> {
    let content = fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut exec = None;
    let mut in_desktop_entry = false;

    for line in content.lines() {
        let line = line.trim();

        // Skip comments
        if line.starts_with('#') {
            continue;
        }

        // Track if we're in [Desktop Entry] section
        if line.starts_with('[') && line.ends_with(']') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }

        if !in_desktop_entry {
            continue;
        }

        if line.starts_with("Name=") && name.is_none() {
            name = Some(line[5..].to_string());
        }
        if line.starts_with("Exec=") {
            exec = Some(line[5..].to_string());
        }
    }

    match (name, exec) {
        (Some(n), Some(e)) => Some(AppInfo {
            path: e, // Keep full exec command for launching
            name: n,
        }),
        _ => None,
    }
}

/// Process desktop files in a directory and add matching apps to results
fn process_desktop_files(
    dir: &Path,
    mimetype: &str,
    blacklist: &HashSet<String>,
    results: &mut Vec<AppInfo>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.extension().map_or(false, |ext| ext == "desktop") {
            continue;
        }

        let Some(filename) = path.file_name() else {
            continue;
        };
        let filename_str = filename.to_string_lossy();

        if blacklist.contains(filename_str.as_ref()) {
            continue;
        }
        if results.iter().any(|r| r.name == *filename_str) {
            continue;
        }
        if !desktop_file_has_mimetype(&path, mimetype) {
            continue;
        }

        if let Some(app_info) = parse_desktop_file(&path) {
            results.push(app_info);
        }
    }
}

/// Process added associations by finding and parsing their desktop files
fn process_added_associations(
    desktop_names: Vec<String>,
    blacklist: &HashSet<String>,
    results: &mut Vec<AppInfo>,
) {
    for desktop_name in desktop_names {
        if blacklist.contains(&desktop_name) {
            continue;
        }
        if results.iter().any(|r| r.name == desktop_name) {
            continue;
        }

        if let Some(desktop_path) = find_desktop_file(&desktop_name) {
            if let Some(mut app_info) = parse_desktop_file(&desktop_path) {
                // Store the desktop filename for deduplication
                app_info.name = format!("{} ({})", app_info.name, desktop_name);
                results.push(app_info);
            }
        }
    }
}

/// Add all desktop filenames in a directory to the blacklist
fn blacklist_desktop_files(dir: &Path, blacklist: &mut HashSet<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "desktop") {
            if let Some(filename) = path.file_name() {
                blacklist.insert(filename.to_string_lossy().to_string());
            }
        }
    }
}

/// Check if a directory is a data directory (not config)
fn is_data_directory(dir: &Path) -> bool {
    let data_home = get_xdg_data_home();
    let data_dirs = get_xdg_data_dirs();

    dir.starts_with(&data_home) || data_dirs.iter().any(|d| dir.starts_with(d))
}

/// Get applications for a file by looking up its MIME type on Linux
///
/// This function implements the XDG MIME apps specification for finding applications
/// associated with a file based on its MIME type.
pub fn get_apps_for_file_linux(path: &Path) -> Vec<AppInfo> {
    // Determine MIME type
    let mimetype = FileType::try_from_file(path)
        .ok()
        .and_then(|ft| ft.media_types().first().map(|s| s.to_string()))
        .unwrap_or_else(|| "application/octet-stream".to_string());

    let mut results = Vec::new();
    let mut blacklist = HashSet::new();
    let mut processed_dirs = HashSet::new();

    for mimeapps_path in get_mimeapps_paths() {
        let Some(mimeapps_dir) = mimeapps_path.parent() else {
            continue;
        };

        // Only process each directory once
        if !processed_dirs.insert(mimeapps_dir.to_path_buf()) {
            continue;
        }

        // Parse the mimeapps.list file
        let (added, removed) = parse_mimeapps_file(&mimeapps_path, &mimetype);

        // Process added/default associations by finding their desktop files
        process_added_associations(added, &blacklist, &mut results);

        // Add "Removed Associations" to blacklist
        blacklist.extend(removed);

        // Look for desktop files in data directories (not config)
        if is_data_directory(mimeapps_dir) {
            process_desktop_files(mimeapps_dir, &mimetype, &blacklist, &mut results);
            blacklist_desktop_files(mimeapps_dir, &mut blacklist);
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_mimeapps_file() {
        let mimeapps_content = r#"[Default Applications]
application/pdf=evince.desktop
image/vnd.djvu=evince.desktop
text/plain=sublime_text.desktop
text/x-markdown=sublime_text.desktop
application/x-shellscript=sublime_text.desktop
text/x-java=sublime_text.desktop
inode/x-empty=sublime_text.desktop
text/x-tex=sublime_text.desktop
text/x-ruby=sublime_text.desktop
text/x-python=sublime_text.desktop
text/x-readme=sublime_text.desktop
application/x-ruby=sublime_text.desktop
text/html=firefox.desktop
text/rhtml=sublime_text.desktop
image/png=gthumb.desktop
image/jpeg=gthumb.desktop
x-scheme-handler/http=firefox.desktop
x-scheme-handler/https=firefox.desktop
x-scheme-handler/ftp=firefox.desktop
x-scheme-handler/chrome=firefox.desktop
application/x-extension-htm=firefox.desktop
application/x-extension-html=firefox.desktop
application/x-extension-shtml=firefox.desktop
application/xhtml+xml=firefox.desktop
application/x-extension-xhtml=firefox.desktop
application/x-extension-xht=firefox.desktop
video/ogg=mplayer.desktop
video/x-msvideo=mplayer.desktop
audio/mpeg=mplayer.desktop
video/quicktime=mplayer.desktop
video/webm=mplayer.desktop
video/x-flv=mplayer.desktop
video/mp4=mplayer.desktop
application/ogg=mplayer.desktop
audio/x-flac=mplayer.desktop
audio/mp4=mplayer.desktop
application/x-flash-video=mplayer.desktop

[Added Associations]
application/pdf=evince.desktop;
x-scheme-handler/http=firefox.desktop;
x-scheme-handler/https=firefox.desktop;
x-scheme-handler/ftp=firefox.desktop;
x-scheme-handler/chrome=firefox.desktop;
text/html=firefox.desktop;
application/x-extension-htm=firefox.desktop;
application/x-extension-html=firefox.desktop;
application/x-extension-shtml=firefox.desktop;
application/xhtml+xml=firefox.desktop;
application/x-extension-xhtml=firefox.desktop;
application/x-extension-xht=firefox.desktop;
"#;

        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_mimeapps.list");
        {
            let mut file = fs::File::create(&temp_file).unwrap();
            file.write_all(mimeapps_content.as_bytes()).unwrap();
        }

        // Test parsing application/pdf - should appear in both Default and Added
        let (added, removed) = parse_mimeapps_file(&temp_file, "application/pdf");
        assert_eq!(added.len(), 2); // Once from Default, once from Added
        assert_eq!(added[0], "evince.desktop");
        assert_eq!(added[1], "evince.desktop");
        assert!(removed.is_empty());

        // Test parsing text/html - appears in both sections
        let (added, removed) = parse_mimeapps_file(&temp_file, "text/html");
        assert_eq!(added.len(), 2);
        assert_eq!(added[0], "firefox.desktop");
        assert_eq!(added[1], "firefox.desktop");
        assert!(removed.is_empty());

        // Test parsing text/plain - only in Default Applications
        let (added, removed) = parse_mimeapps_file(&temp_file, "text/plain");
        assert_eq!(added.len(), 1);
        assert_eq!(added[0], "sublime_text.desktop");
        assert!(removed.is_empty());

        // Test parsing x-scheme-handler/https - only in Added Associations
        let (added, removed) = parse_mimeapps_file(&temp_file, "x-scheme-handler/https");
        assert_eq!(added.len(), 2); // Once from Default, once from Added
        assert_eq!(added[0], "firefox.desktop");
        assert_eq!(added[1], "firefox.desktop");
        assert!(removed.is_empty());

        // Test parsing non-existent mimetype
        let (added, removed) = parse_mimeapps_file(&temp_file, "application/nonexistent");
        assert!(added.is_empty());
        assert!(removed.is_empty());

        // Clean up
        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_parse_mimeapps_file_with_removed_associations() {
        let mimeapps_content = r#"[Default Applications]
application/pdf=evince.desktop

[Added Associations]
application/pdf=okular.desktop;

[Removed Associations]
application/pdf=xpdf.desktop;
"#;

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_mimeapps_removed.list");
        {
            let mut file = fs::File::create(&temp_file).unwrap();
            file.write_all(mimeapps_content.as_bytes()).unwrap();
        }

        let (added, removed) = parse_mimeapps_file(&temp_file, "application/pdf");
        assert_eq!(added.len(), 2);
        assert_eq!(added[0], "evince.desktop");
        assert_eq!(added[1], "okular.desktop");
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], "xpdf.desktop");

        // Clean up
        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_parse_mimeapps_file_multiple_apps() {
        let mimeapps_content = r#"[Added Associations]
image/jpeg=gimp.desktop;gthumb.desktop;eog.desktop;
"#;

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_mimeapps_multiple.list");
        {
            let mut file = fs::File::create(&temp_file).unwrap();
            file.write_all(mimeapps_content.as_bytes()).unwrap();
        }

        let (added, removed) = parse_mimeapps_file(&temp_file, "image/jpeg");
        assert_eq!(added.len(), 3);
        assert_eq!(added[0], "gimp.desktop");
        assert_eq!(added[1], "gthumb.desktop");
        assert_eq!(added[2], "eog.desktop");
        assert!(removed.is_empty());

        // Clean up
        let _ = fs::remove_file(temp_file);
    }
}
