pub mod colors;
pub mod shortcuts;

use crate::models::tab::{SortColumn, SortOrder};
use colors::ColorScheme;
use serde::{Deserialize, Serialize};

use std::fs;
use std::io::Read;
use std::path::PathBuf;

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

pub fn load_config_with_override(
    config_dir_override: Option<&PathBuf>,
) -> Result<Config, toml::de::Error> {
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

    toml::from_str(&contents)
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
            let dir = dirs_next::config_dir().unwrap_or_else(|| PathBuf::from("."));
            dir.join("kiorg")
        }
    }
}
