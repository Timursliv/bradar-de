use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub keybinds: Keybinds,
    pub theme: Theme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinds {
    pub terminal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub background: String,
    pub border_active: String,
    pub border_inactive: String,
    pub rounding: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keybinds: Keybinds {
                terminal: "kitty".into(),
            },
            theme: Theme {
                background: "#0f0f14".into(),
                border_active: "#7aa2f7".into(),
                border_inactive: "#2a2a3a".into(),
                rounding: 12,
            },
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            match fs::read_to_string(&path).map(|s| toml::from_str(&s)) {
                Ok(Ok(cfg)) => {
                    info!("Config loaded from {:?}", path);
                    return cfg;
                }
                _ => warn!("Config parse error, using defaults"),
            }
        }
        let cfg = Self::default();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&path, toml::to_string_pretty(&cfg).unwrap_or_default());
        cfg
    }

    fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
        PathBuf::from(home).join(".config/bradar-de/config.toml")
    }
}
