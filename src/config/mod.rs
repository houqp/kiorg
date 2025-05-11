pub mod colors;
pub mod shortcuts;

use crate::models::tab::{SortColumn, SortOrder};
use colors::ColorScheme;
use serde::{Deserialize, Serialize};

use std::error::Error;
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

// Custom error type for shortcut conflicts
#[derive(Debug)]
pub struct ShortcutConflictError {
    pub shortcut: shortcuts::KeyboardShortcut,
    pub action1: shortcuts::ShortcutAction,
    pub action2: shortcuts::ShortcutAction,
}

impl fmt::Display for ShortcutConflictError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} is assigned to both {:?} and {:?}",
            self.shortcut, self.action1, self.action2
        )
    }
}

impl Error for ShortcutConflictError {}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct SortPreference {
    pub column: SortColumn,
    pub order: SortOrder,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Config {
    pub colors: Option<ColorScheme>,
    pub sort_preference: Option<SortPreference>,
    pub shortcuts: Option<shortcuts::Shortcuts>,
}

impl Config {
    fn default() -> Self {
        Self {
            colors: None,
            sort_preference: None,
            shortcuts: Some(shortcuts::default_shortcuts()),
        }
    }
}

// Define a custom error type that can represent both TOML parsing errors and shortcut conflicts
#[derive(Debug)]
pub enum ConfigError {
    TomlError(toml::de::Error),
    ShortcutConflict(ShortcutConflictError),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::TomlError(e) => write!(f, "Invalid config: {}", e),
            ConfigError::ShortcutConflict(e) => write!(f, "Shortcut conflict: {}", e),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::TomlError(e) => Some(e),
            ConfigError::ShortcutConflict(e) => Some(e),
        }
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::TomlError(err)
    }
}

impl From<ShortcutConflictError> for ConfigError {
    fn from(err: ShortcutConflictError) -> Self {
        ConfigError::ShortcutConflict(err)
    }
}

pub fn load_config_with_override(
    config_dir_override: Option<&PathBuf>,
) -> Result<Config, ConfigError> {
    let config_dir = get_kiorg_config_dir(config_dir_override);
    if !config_dir.exists() {
        let _ = fs::create_dir_all(&config_dir);
    }

    let config_path = config_dir.join("config.toml");

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let mut file = match fs::File::open(&config_path) {
        Ok(file) => file,
        Err(_) => return Ok(Config::default()),
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return Ok(Config::default());
    }

    // Parse the user config
    let mut user_config: Config = toml::from_str(&contents)?;

    // Merge user shortcuts with default shortcuts if user provided any
    if let Some(user_shortcuts) = user_config.shortcuts.take() {
        let default_shortcuts = shortcuts::default_shortcuts();
        // Use ? to propagate the ShortcutConflictError
        let merged_shortcuts = merge_shortcuts(default_shortcuts, user_shortcuts)?;
        user_config.shortcuts = Some(merged_shortcuts);
    } else {
        // If no shortcuts provided, use defaults
        user_config.shortcuts = Some(shortcuts::default_shortcuts());
    }

    Ok(user_config)
}

pub fn save_config(config: &Config) -> Result<(), std::io::Error> {
    save_config_with_override(config, None)
}

pub fn save_config_with_override(
    config: &Config,
    config_dir_override: Option<&PathBuf>,
) -> Result<(), std::io::Error> {
    let config_dir = get_kiorg_config_dir(config_dir_override);

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    let config_path = config_dir.join("config.toml");
    let toml_str = toml::to_string_pretty(config).unwrap_or_default();
    fs::write(&config_path, toml_str)
}

pub fn get_config_path_with_override(config_dir_override: Option<&PathBuf>) -> PathBuf {
    let config_dir = get_kiorg_config_dir(config_dir_override);
    config_dir.join("config.toml")
}

pub fn get_kiorg_config_dir(override_path: Option<&PathBuf>) -> PathBuf {
    match override_path {
        Some(dir) => dir.clone(),
        None => {
            // For macOS, prioritize ~/.config/kiorg for easier config management and terminal access
            #[cfg(target_os = "macos")]
            {
                if let Some(home_dir) = dirs::home_dir() {
                    let xdg_config_dir = home_dir.join(".config").join("kiorg");
                    if xdg_config_dir.exists() {
                        return xdg_config_dir;
                    }
                }
            }

            // Fall back to the standard config directory
            let dir = dirs::config_dir().expect("Failed to look up config directory");
            dir.join("kiorg")
        }
    }
}

// Merge default shortcuts with user-provided shortcuts
// User shortcuts take precedence for any actions they define
// Returns an error if there are conflicting shortcuts
fn merge_shortcuts(
    mut default_shortcuts: shortcuts::Shortcuts,
    user_shortcuts: shortcuts::Shortcuts,
) -> Result<shortcuts::Shortcuts, ShortcutConflictError> {
    // Create a map to track which action each shortcut is assigned to
    let mut shortcut_map: std::collections::HashMap<
        &shortcuts::KeyboardShortcut,
        shortcuts::ShortcutAction,
    > = std::collections::HashMap::new();

    // First, add all default shortcuts to the map
    for (action, shortcuts_list) in &default_shortcuts {
        for shortcut in shortcuts_list {
            shortcut_map.insert(shortcut, *action);
        }
    }

    // Check user shortcuts for conflicts with default shortcuts
    for (action, shortcuts_list) in &user_shortcuts {
        for shortcut in shortcuts_list {
            // If this shortcut is already assigned to a different action, it's a conflict
            if let Some(&existing_action) = shortcut_map.get(shortcut) {
                if existing_action != *action {
                    return Err(ShortcutConflictError {
                        shortcut: shortcut.clone(),
                        action1: existing_action,
                        action2: *action,
                    });
                }
            }

            // Otherwise, add or update the mapping
            shortcut_map.insert(shortcut, *action);
        }
    }

    // If we get here, there are no conflicts, so apply the user shortcuts
    for (action, shortcuts_list) in user_shortcuts {
        // Replace the default shortcuts for this action with the user's shortcuts
        default_shortcuts.insert(action, shortcuts_list);
    }

    Ok(default_shortcuts)
}
