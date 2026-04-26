// ============================================================
//  layout.rs — Rozmístění oken
//  Floating (volné) nebo Tiling (automatické dláždění)
// ============================================================

use crate::window::WindowManager;

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutMode {
    Floating,  // okna jsou volná — přesouvají se myší
    Tiling,    // okna se automaticky rozmísťují
}

pub struct Layout {
    pub mode: LayoutMode,
}

impl Layout {
    pub fn new(mode_str: &str) -> Self {
        let mode = match mode_str {
            "tiling" => LayoutMode::Tiling,
            _ => LayoutMode::Floating,
        };
        Self { mode }
    }

    // Přeuspořádej okna podle aktuálního layoutu
    pub fn arrange(&self, wm: &mut WindowManager) {
        match self.mode {
            LayoutMode::Floating => {
                // Floating — nic nedělej, okna jsou tam kde je uživatel dal
            }
            LayoutMode::Tiling => {
                self.arrange_tiling(wm);
            }
        }
    }

    // --------------------------------------------------------
    //  TILING LAYOUT
    //  Rozdělí obrazovku mezi okna rovnoměrně
    // --------------------------------------------------------
    fn arrange_tiling(&self, wm: &mut WindowManager) {
        let screen_w = wm.screen_width;
        let screen_h = wm.screen_height;
        let bar_h = wm.bar_height;
        let gap = 8u32; // mezera mezi okny

        // Počítej jen viditelná okna na aktivním workspace
        let workspace = wm.active_workspace;
        let window_ids: Vec<_> = wm.visible_windows()
            .iter()
            .map(|s| s.id)
            .collect();

        let count = window_ids.len();
        if count == 0 { return; }

        let usable_h = screen_h - bar_h;

        if count == 1 {
            // Jedno okno = celá obrazovka (s mezerami)
            if let Some(state) = wm.get_mut(window_ids[0]) {
                state.x = gap as i32;
                state.y = bar_h as i32 + gap as i32;
                state.width = screen_w - gap * 2;
                state.height = usable_h - gap * 2;
            }
        } else {
            // Více oken — první zabere levou polovinu,
            // ostatní se rozdělí do pravé poloviny vertikálně

            let half_w = screen_w / 2;
            let right_count = count - 1;
            let right_h = (usable_h - gap * (right_count as u32 + 1)) / right_count as u32;

            // Hlavní okno — levá polovina
            if let Some(state) = wm.get_mut(window_ids[0]) {
                state.x = gap as i32;
                state.y = bar_h as i32 + gap as i32;
                state.width = half_w - gap - gap / 2;
                state.height = usable_h - gap * 2;
            }

            // Ostatní okna — pravá polovina
            for (i, id) in window_ids[1..].iter().enumerate() {
                if let Some(state) = wm.get_mut(*id) {
                    state.x = (half_w + gap / 2) as i32;
                    state.y = bar_h as i32 + gap as i32 + (i as i32 * (right_h as i32 + gap as i32));
                    state.width = half_w - gap - gap / 2;
                    state.height = right_h;
                }
            }
        }
    }

    pub fn toggle(&mut self) {
        self.mode = match self.mode {
            LayoutMode::Floating => LayoutMode::Tiling,
            LayoutMode::Tiling => LayoutMode::Floating,
        };
        tracing::info!("Layout switched to {:?}", self.mode);
    }
}
