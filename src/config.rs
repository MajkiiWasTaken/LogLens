/************************************************
* File: config.rs
* Author: Michal Švrček
*
* Device profile configuration management.
*
* ver. 0.5.0
*************************************************/

use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, env, fs, path::PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub devices: BTreeMap<String, Device>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub host: String,

    #[serde(default)]
    pub user: Option<String>,

    #[serde(default)]
    pub port: Option<u16>,

    #[serde(default)]
    pub description: String,
}

pub fn path() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        let base = env::var_os("APPDATA").ok_or("APPDATA environment variable is missing")?;

        Ok(PathBuf::from(base).join("LogLens").join("config.toml"))
    }

    #[cfg(not(windows))]
    {
        let base = env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .ok_or("Cannot locate configuration directory")?;

        Ok(base.join("loglens").join("config.toml"))
    }
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let path = path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content =
            fs::read_to_string(&path).map_err(|e| format!("Cannot read configuration: {e}"))?;

        toml::from_str(&content).map_err(|e| format!("Invalid TOML configuration: {e}"))
    }

    pub fn save(&self) -> Result<(), String> {
        let path = path()?;

        let directory = path.parent().ok_or("Invalid configuration path")?;

        fs::create_dir_all(directory)
            .map_err(|e| format!("Cannot create config directory: {e}"))?;

        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Cannot serialize configuration: {e}"))?;

        if path.exists() {
            fs::copy(&path, path.with_extension("toml.bak"))
                .map_err(|e| format!("Cannot back up configuration: {e}"))?;
        }

        fs::write(&path, content).map_err(|e| format!("Cannot save configuration: {e}"))
    }
}
