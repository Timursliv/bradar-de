// ============================================================
//  config.rs — Konfigurace celého DE
//  Soubor: ~/.config/bradar-de/config.toml
// ============================================================

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

// ============================================================
//  HLAVNÍ KONFIGURACE
// ============================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: ThemeConfig,
    pub bar: BarConfig,
    pub window: WindowConfig,
    pub keybinds: KeybindsConfig,
    pub animations: AnimationConfig,
}

// ============================================================
//  TÉMA — barvy celého DE
// ============================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,

    // Pozadí plochy
    pub background: String,         // např. "#1a1a2e"
    pub wallpaper: Option<String>,  // cesta k obrázku

    // Barvy oken
    pub window_border_active: String,    // barva rámečku aktivního okna
    pub window_border_inactive: String,  // barva rámečku neaktivního okna
    pub window_border_width: u32,        // tloušťka rámečku v px

    // Vizuální efekty
    pub rounding: u32,    // zaoblení rohů v px (0 = žádné)
    pub blur: bool,       // rozmazání pozadí pod okny
    pub blur_strength: u32,
    pub shadows: bool,    // stíny pod okny
    pub shadow_size: u32,
    pub shadow_color: String,

    // Barvy lišty
    pub bar_background: String,
    pub bar_text: String,
    pub bar_accent: String,
}

// ============================================================
//  LIŠTA (BAR)
// ============================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarConfig {
    pub enabled: bool,
    pub height: u32,         // výška v px
    pub position: String,    // "top" nebo "bottom"
    pub font: String,
    pub font_size: u32,
    pub show_clock: bool,
    pub show_workspaces: bool,
    pub show_active_window: bool,
    pub clock_format: String, // např. "%H:%M" nebo "%H:%M:%S"
}

// ============================================================
//  OKNA
// ============================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub gaps_inner: u32,  // mezera mezi okny
    pub gaps_outer: u32,  // mezera od okraje obrazovky
    pub layout: String,   // "floating" nebo "tiling"
    pub default_width: u32,
    pub default_height: u32,
}

// ============================================================
//  KLÁVESOVÉ ZKRATKY
// ============================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindsConfig {
    pub modifier: String,    // "super" (Windows klávesa) nebo "alt"
    pub terminal: String,    // příkaz pro terminál
    pub launcher: String,    // příkaz pro launcher aplikací
}

// ============================================================
//  ANIMACE
// ============================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    pub enabled: bool,
    pub duration_ms: u64,   // jak dlouho trvá animace
    pub style: String,      // "ease", "bounce", "linear"
}

// ============================================================
//  VÝCHOZÍ HODNOTY
//  Toto se použije pokud config.toml neexistuje
// ============================================================
impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeConfig {
                name: "dark".into(),
                background: "#0f0f14".into(),
                wallpaper: None,

                // Hezké modré/fialové rámečky
                window_border_active: "#7aa2f7".into(),
                window_border_inactive: "#2a2a3a".into(),
                window_border_width: 2,

                // macOS-like zaoblení
                rounding: 12,
                blur: true,
                blur_strength: 10,
                shadows: true,
                shadow_size: 20,
                shadow_color: "#00000080".into(),

                // Tmavá průhledná lišta
                bar_background: "#0f0f14cc".into(),
                bar_text: "#c0caf5".into(),
                bar_accent: "#7aa2f7".into(),
            },
            bar: BarConfig {
                enabled: true,
                height: 32,
                position: "top".into(),
                font: "monospace".into(),
                font_size: 13,
                show_clock: true,
                show_workspaces: true,
                show_active_window: true,
                clock_format: "%H:%M".into(),
            },
            window: WindowConfig {
                gaps_inner: 8,
                gaps_outer: 16,
                layout: "floating".into(),
                default_width: 800,
                default_height: 600,
            },
            keybinds: KeybindsConfig {
                modifier: "super".into(),
                terminal: "kitty".into(),
                launcher: "launcher".into(),
            },
            animations: AnimationConfig {
                enabled: true,
                duration_ms: 250,
                style: "ease".into(),
            },
        }
    }
}

// ============================================================
//  NAČTENÍ KONFIGURACE
// ============================================================
impl Config {
    pub fn load() -> Self {
        let path = Self::config_path();

        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match toml::from_str(&content) {
                        Ok(config) => {
                            info!("Loaded config from {:?}", path);
                            return config;
                        }
                        Err(e) => {
                            warn!("Config parse error: {} — using defaults", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Cannot read config: {} — using defaults", e);
                }
            }
        } else {
            // Vytvoř výchozí config
            let default = Self::default();
            default.save();
            info!("Created default config at {:?}", path);
            return default;
        }

        Self::default()
    }

    pub fn save(&self) {
        let path = Self::config_path();

        // Vytvoř složku pokud neexistuje
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        match toml::to_string_pretty(self) {
            Ok(content) => {
                let _ = fs::write(&path, content);
            }
            Err(e) => {
                warn!("Cannot save config: {}", e);
            }
        }
    }

    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
        PathBuf::from(home).join(".config/bradar-de/config.toml")
    }
}
