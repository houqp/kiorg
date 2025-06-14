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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct SortPreference {
    pub column: SortColumn,
    pub order: SortOrder,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Config {
    pub colors: Option<ColorScheme>,
    pub theme: Option<String>,
    pub sort_preference: Option<SortPreference>,
    pub shortcuts: Option<shortcuts::Shortcuts>,
}

impl Config {
    fn default() -> Self {
        Self {
            colors: None,
            theme: None,
            sort_preference: None,
            shortcuts: None,
        }
    }
}

// Define a custom error type that can represent both TOML parsing errors and shortcut conflicts
#[derive(Debug)]
pub enum ConfigError {
    TomlError(toml::de::Error, PathBuf),
    ShortcutConflict(ShortcutConflictError, PathBuf),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TomlError(e, _) => write!(f, "Invalid config: {e}"),
            Self::ShortcutConflict(e, _) => write!(f, "Shortcut conflict: {e}"),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TomlError(e, _) => Some(e),
            Self::ShortcutConflict(e, _) => Some(e),
        }
    }
}

impl ConfigError {
    #[must_use]
    pub const fn config_path(&self) -> &PathBuf {
        match self {
            Self::TomlError(_, path) => path,
            Self::ShortcutConflict(_, path) => path,
        }
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        // This From implementation is problematic since we don't have the path here
        // We'll handle the path in the load_config_with_override function directly
        Self::TomlError(err, PathBuf::new())
    }
}

impl From<ShortcutConflictError> for ConfigError {
    fn from(err: ShortcutConflictError) -> Self {
        // This From implementation is problematic since we don't have the path here
        // We'll handle the path in the load_config_with_override function directly
        Self::ShortcutConflict(err, PathBuf::new())
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

    let Ok(mut file) = fs::File::open(&config_path) else {
        return Ok(Config::default());
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return Ok(Config::default());
    }

    // Parse the user config
    let user_config: Config = match toml::from_str(&contents) {
        Ok(config) => config,
        Err(e) => return Err(ConfigError::TomlError(e, config_path)),
    };

    // Validate user shortcuts for conflicts
    if let Some(ref user_shortcuts) = user_config.shortcuts {
        if let Err(conflict_error) = validate_user_shortcuts(user_shortcuts) {
            return Err(ConfigError::ShortcutConflict(conflict_error, config_path));
        }
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

#[must_use]
pub fn get_config_path_with_override(config_dir_override: Option<&PathBuf>) -> PathBuf {
    let config_dir = get_kiorg_config_dir(config_dir_override);
    config_dir.join("config.toml")
}

#[must_use]
pub fn get_kiorg_config_dir(override_path: Option<&PathBuf>) -> PathBuf {
    if let Some(dir) = override_path {
        dir.clone()
    } else {
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

/// Validate user shortcuts for conflicts
/// Returns an error if any shortcut is assigned to multiple different actions
fn validate_user_shortcuts(
    user_shortcuts: &shortcuts::Shortcuts,
) -> Result<(), ShortcutConflictError> {
    use std::collections::HashMap;

    // Create a map from shortcut to actions that use it
    let mut shortcut_to_actions: HashMap<
        shortcuts::KeyboardShortcut,
        Vec<shortcuts::ShortcutAction>,
    > = HashMap::new();

    for (action, shortcuts_list) in user_shortcuts {
        for shortcut in shortcuts_list {
            // Only add unique actions (don't count duplicates of the same action)
            let actions_for_shortcut = shortcut_to_actions.entry(shortcut.clone()).or_default();

            if !actions_for_shortcut.contains(action) {
                actions_for_shortcut.push(*action);
            }
        }
    }

    // Check for conflicts (shortcuts assigned to multiple different actions)
    for (shortcut, actions) in shortcut_to_actions {
        if actions.len() > 1 {
            // Found a conflict - return error with the first two conflicting actions
            return Err(ShortcutConflictError {
                shortcut,
                action1: actions[0],
                action2: actions[1],
            });
        }
    }

    Ok(())
}
