// ============================================================
//  BRADAR DE — main.rs
//  Entry point — spouští celé DE
// ============================================================

mod state;       // Hlavní stav DE
mod compositor;  // Wayland compositor logika
mod input;       // Klávesnice a myš
mod render;      // Kreslení na obrazovku
mod window;      // Správa oken
mod bar;         // Horní lišta (jako macOS menu bar)
mod config;      // Konfigurace (~/.config/bradar-de/config.toml)
mod keybinds;    // Klávesové zkratky
mod layout;      // Rozmístění oken (floating / tiling)
mod cursor;      // Kurzor myši
mod animation;   // Animace oken

use std::time::Duration;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn main() {
    // Nastav logování
    // Spusť s: RUST_LOG=debug cargo run  pro více výpisů
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("===========================================");
    info!("  BRADAR DE — starting up");
    info!("===========================================");

    // Načti konfiguraci
    let config = config::Config::load();
    info!("Config loaded: theme={}", config.theme.name);

    // Spusť compositor
    compositor::run(config);
}
