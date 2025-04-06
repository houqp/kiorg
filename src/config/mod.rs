pub mod colors;

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

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pub colors: ColorScheme,
    pub sort_preference: Option<SortPreference>,
}

impl Config {
    fn default() -> Self {
        Self {
            colors: ColorScheme::default(),
            sort_preference: None,
        }
    }
}

pub fn load_config_with_override(config_dir_override: Option<&PathBuf>) -> Config {
    let config_dir = get_kiorg_config_dir(config_dir_override);

    if !config_dir.exists() {
        let _ = fs::create_dir_all(&config_dir);
    }

    let config_path = config_dir.join("config.toml");

    if !config_path.exists() {
        let default_config = Config::default();
        let toml_str = toml::to_string_pretty(&default_config).unwrap_or_default();
        let _ = fs::write(&config_path, toml_str);
        return default_config;
    }

    let mut file = match fs::File::open(&config_path) {
        Ok(file) => file,
        Err(_) => return Config::default(),
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return Config::default();
    }

    toml::from_str(&contents).unwrap_or_default()
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

fn get_kiorg_config_dir(override_path: Option<&PathBuf>) -> PathBuf {
    match override_path {
        Some(dir) => dir.clone(),
        None => {
            let dir = dirs_next::config_dir().unwrap_or_else(|| PathBuf::from("."));
            dir.join("kiorg")
        }
    }
}
