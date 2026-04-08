use super::Schematic;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use directories::ProjectDirs;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub log_level: String,
    pub schematics: BTreeMap<String, Schematic>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            log_level: "info".to_string(),
            schematics: BTreeMap::new(),
        }
    }
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let path = get_config_path().ok_or("Could not determine config path")?;

    if !path.exists() {
        let default_config = Config::default();

        let toml_str = toml::to_string_pretty(&default_config)?;
        fs::write(&path, toml_str)?;

        return Ok(default_config);
    }

    let config_content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&config_content)?;

    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_config_path().ok_or("Could not determine config path")?;

    let toml_str = toml::to_string_pretty(config)?;
    fs::write(path, toml_str)?;

    Ok(())
}

fn get_config_path() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "gabrielld06", "schematics") {
        let config_dir = proj_dirs.config_dir();

        fs::create_dir_all(config_dir).ok()?;

        Some(config_dir.join("config.toml"))
    } else {
        None
    }
}
