// ============================================================
//  keybinds.rs — Klávesové zkratky
// ============================================================

use smithay::input::keyboard::ModifiersState;
use tracing::info;

// ============================================================
//  AKCE KTERÉ KLÁVESY SPOUŠTĚJÍ
// ============================================================
#[derive(Debug, Clone)]
pub enum Action {
    Quit,                          // ukončí DE
    LaunchTerminal,                // Super+T
    LaunchLauncher,                // Super+Space — spouštěč aplikací
    CloseWindow,                   // Super+Q — zavře aktivní okno
    MaximizeWindow,                // Super+M
    MinimizeWindow,                // Super+H
    SwitchWorkspace(usize),        // Super+1-9
    MoveWindowToWorkspace(usize),  // Super+Shift+1-9
    FocusNext,                     // Super+Tab
    FocusPrev,                     // Super+Shift+Tab
    Screenshot,                    // Print Screen
}

// ============================================================
//  HANDLER KLÁVESOVÝCH ZKRATEK
// ============================================================
pub struct Keybinds {
    pub terminal_cmd: String,
    pub launcher_cmd: String,
}

impl Keybinds {
    pub fn new(terminal: String, launcher: String) -> Self {
        Self {
            terminal_cmd: terminal,
            launcher_cmd: launcher,
        }
    }

    // Zkontroluj jestli klávesa + modifikátory odpovídají zkratce
    // Vrátí akci pokud ano
    pub fn handle(
        &self,
        keysym: u32,
        modifiers: &ModifiersState,
    ) -> Option<Action> {
        use smithay::input::keyboard::keysyms;

        let super_key = modifiers.logo;   // Windows klávesa
        let shift = modifiers.shift;
        let ctrl = modifiers.ctrl;

        match (super_key, shift, ctrl, keysym) {
            // --------------------------------------------------
            //  SYSTÉM
            // --------------------------------------------------

            // Super+Escape = ukončit DE
            (true, false, false, keysyms::KEY_Escape) => {
                info!("Keybind: Quit");
                Some(Action::Quit)
            }

            // Super+T = terminál
            (true, false, false, keysyms::KEY_t) => {
                info!("Keybind: Launch terminal ({})", self.terminal_cmd);
                Some(Action::LaunchTerminal)
            }

            // Super+Space = launcher
            (true, false, false, keysyms::KEY_space) => {
                info!("Keybind: Launch launcher");
                Some(Action::LaunchLauncher)
            }

            // --------------------------------------------------
            //  OKNA
            // --------------------------------------------------

            // Super+Q = zavři okno
            (true, false, false, keysyms::KEY_q) => {
                info!("Keybind: Close window");
                Some(Action::CloseWindow)
            }

            // Super+M = maximalizuj
            (true, false, false, keysyms::KEY_m) => {
                info!("Keybind: Maximize");
                Some(Action::MaximizeWindow)
            }

            // Super+H = minimalizuj
            (true, false, false, keysyms::KEY_h) => {
                info!("Keybind: Minimize");
                Some(Action::MinimizeWindow)
            }

            // Super+Tab = další okno
            (true, false, false, keysyms::KEY_Tab) => {
                Some(Action::FocusNext)
            }

            // Super+Shift+Tab = předchozí okno
            (true, true, false, keysyms::KEY_Tab) => {
                Some(Action::FocusPrev)
            }

            // --------------------------------------------------
            //  WORKSPACES — Super+1 až Super+9
            // --------------------------------------------------
            (true, false, false, keysyms::KEY_1) => Some(Action::SwitchWorkspace(0)),
            (true, false, false, keysyms::KEY_2) => Some(Action::SwitchWorkspace(1)),
            (true, false, false, keysyms::KEY_3) => Some(Action::SwitchWorkspace(2)),
            (true, false, false, keysyms::KEY_4) => Some(Action::SwitchWorkspace(3)),
            (true, false, false, keysyms::KEY_5) => Some(Action::SwitchWorkspace(4)),
            (true, false, false, keysyms::KEY_6) => Some(Action::SwitchWorkspace(5)),
            (true, false, false, keysyms::KEY_7) => Some(Action::SwitchWorkspace(6)),
            (true, false, false, keysyms::KEY_8) => Some(Action::SwitchWorkspace(7)),
            (true, false, false, keysyms::KEY_9) => Some(Action::SwitchWorkspace(8)),

            // Super+Shift+1-9 = přesuň okno na workspace
            (true, true, false, keysyms::KEY_1) => Some(Action::MoveWindowToWorkspace(0)),
            (true, true, false, keysyms::KEY_2) => Some(Action::MoveWindowToWorkspace(1)),
            (true, true, false, keysyms::KEY_3) => Some(Action::MoveWindowToWorkspace(2)),

            // --------------------------------------------------
            //  SCREENSHOT
            // --------------------------------------------------
            (false, false, false, keysyms::KEY_Print) => {
                info!("Keybind: Screenshot");
                Some(Action::Screenshot)
            }

            _ => None,
        }
    }

    // Spusť externí příkaz
    pub fn launch(&self, cmd: &str) {
        info!("Launching: {}", cmd);
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() { return; }

        let mut command = std::process::Command::new(parts[0]);
        if parts.len() > 1 {
            command.args(&parts[1..]);
        }

        match command.spawn() {
            Ok(_) => info!("Launched: {}", cmd),
            Err(e) => tracing::warn!("Failed to launch '{}': {}", cmd, e),
        }
    }

    pub fn launch_terminal(&self) {
        self.launch(&self.terminal_cmd);
    }

    pub fn launch_launcher(&self) {
        self.launch(&self.launcher_cmd);
    }
}
