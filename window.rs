// ============================================================
//  window.rs — Správa oken
//  Sleduje všechna okna, jejich pozice, fokus, animace
// ============================================================

use std::collections::HashMap;
use smithay::desktop::Window;
use smithay::utils::{Point, Size, Rectangle, Logical};
use crate::animation::WindowAnim;

// ============================================================
//  ID OKNA
// ============================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

// ============================================================
//  STAV JEDNOHO OKNA
// ============================================================
#[derive(Debug)]
pub struct WindowState {
    pub id: WindowId,
    pub window: Window,

    // Pozice a velikost
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,

    // Vizuální stav
    pub opacity: f32,         // 0.0 - 1.0
    pub minimized: bool,
    pub maximized: bool,
    pub fullscreen: bool,

    // Animace
    pub animation: Option<WindowAnim>,

    // Pracovní plocha (workspace)
    pub workspace: usize,

    // Název okna
    pub title: String,
}

impl WindowState {
    pub fn new(id: WindowId, window: Window, x: i32, y: i32, width: u32, height: u32, duration_ms: u64) -> Self {
        // Začni s animací otevření
        let animation = if duration_ms > 0 {
            Some(WindowAnim::open(duration_ms))
        } else {
            None
        };

        Self {
            id,
            window,
            x,
            y,
            width,
            height,
            opacity: 1.0,
            minimized: false,
            maximized: false,
            fullscreen: false,
            animation,
            workspace: 0,
            title: String::from("Window"),
        }
    }

    pub fn geometry(&self) -> Rectangle<i32, Logical> {
        Rectangle::from_loc_and_size(
            Point::from((self.x, self.y)),
            Size::from((self.width as i32, self.height as i32)),
        )
    }

    // Aktualizuj animaci a aplikuj hodnoty
    pub fn update_animation(&mut self) -> bool {
        if let Some(ref mut anim) = self.animation {
            self.opacity = anim.current_opacity() as f32;

            if let (Some(x), Some(y)) = (anim.current_x(), anim.current_y()) {
                self.x = x as i32;
                self.y = y as i32;
            }

            if anim.is_done() {
                self.animation = None;
                return true; // animace dokončena
            }
        }
        false
    }
}

// ============================================================
//  SPRÁVCE OKEN
//  Uchovává všechna okna a jejich stavy
// ============================================================
pub struct WindowManager {
    windows: HashMap<WindowId, WindowState>,
    next_id: u64,
    pub focused: Option<WindowId>,
    pub active_workspace: usize,
    animation_duration_ms: u64,

    // Velikost obrazovky (potřebujeme pro maximalizaci atd.)
    pub screen_width: u32,
    pub screen_height: u32,
    pub bar_height: u32, // výška lišty — okna se nesmí překrývat s lištou
}

impl WindowManager {
    pub fn new(screen_width: u32, screen_height: u32, bar_height: u32, animation_duration_ms: u64) -> Self {
        Self {
            windows: HashMap::new(),
            next_id: 0,
            focused: None,
            active_workspace: 0,
            animation_duration_ms,
            screen_width,
            screen_height,
            bar_height,
        }
    }

    // --------------------------------------------------------
    //  PŘIDÁNÍ NOVÉHO OKNA
    // --------------------------------------------------------
    pub fn add_window(&mut self, window: Window, width: u32, height: u32) -> WindowId {
        let id = WindowId(self.next_id);
        self.next_id += 1;

        // Vycentruj okno na obrazovce
        let x = ((self.screen_width as i32 - width as i32) / 2).max(0);
        let y = ((self.screen_height as i32 - height as i32) / 2)
            .max(self.bar_height as i32);

        let state = WindowState::new(
            id, window, x, y, width, height,
            self.animation_duration_ms,
        );

        self.windows.insert(id, state);
        self.focused = Some(id); // nové okno dostane fokus

        tracing::info!("Window {:?} added at ({}, {})", id, x, y);
        id
    }

    // --------------------------------------------------------
    //  ODEBRÁNÍ OKNA
    // --------------------------------------------------------
    pub fn remove_window(&mut self, id: WindowId) {
        self.windows.remove(&id);

        // Pokud bylo fokusované, fokusuj jiné
        if self.focused == Some(id) {
            self.focused = self.windows.keys().next().copied();
        }

        tracing::info!("Window {:?} removed", id);
    }

    // --------------------------------------------------------
    //  FOKUS
    // --------------------------------------------------------
    pub fn focus(&mut self, id: WindowId) {
        if self.windows.contains_key(&id) {
            self.focused = Some(id);
        }
    }

    // Fokusuj okno pod kurzorem
    pub fn focus_at(&mut self, x: f64, y: f64) -> Option<WindowId> {
        // Projdi okna od posledního (nejvýše) po první
        let mut top_window: Option<WindowId> = None;

        for (id, state) in &self.windows {
            if state.workspace != self.active_workspace { continue; }
            if state.minimized { continue; }

            let geo = state.geometry();
            if geo.contains(Point::from((x as i32, y as i32))) {
                top_window = Some(*id);
            }
        }

        if let Some(id) = top_window {
            self.focused = Some(id);
        }

        top_window
    }

    // --------------------------------------------------------
    //  PŘESUN OKNA
    // --------------------------------------------------------
    pub fn move_window(&mut self, id: WindowId, new_x: i32, new_y: i32) {
        if let Some(state) = self.windows.get_mut(&id) {
            let from_x = state.x as f64;
            let from_y = state.y as f64;

            // Spusť animaci přesunu
            if self.animation_duration_ms > 0 {
                state.animation = Some(crate::animation::WindowAnim::move_to(
                    from_x, new_x as f64,
                    from_y, new_y as f64,
                    self.animation_duration_ms / 2, // přesun je rychlejší
                ));
            } else {
                state.x = new_x;
                state.y = new_y;
            }
        }
    }

    // --------------------------------------------------------
    //  MAXIMALIZACE
    // --------------------------------------------------------
    pub fn maximize(&mut self, id: WindowId) {
        if let Some(state) = self.windows.get_mut(&id) {
            if state.maximized {
                // Obnovit původní velikost
                state.maximized = false;
                // TODO: uložit původní pozici/velikost před maximalizací
            } else {
                state.maximized = true;
                state.x = 0;
                state.y = self.bar_height as i32;
                state.width = self.screen_width;
                state.height = self.screen_height - self.bar_height;
            }
        }
    }

    // --------------------------------------------------------
    //  MINIMALIZACE
    // --------------------------------------------------------
    pub fn minimize(&mut self, id: WindowId) {
        if let Some(state) = self.windows.get_mut(&id) {
            state.minimized = true;
            if self.focused == Some(id) {
                self.focused = self.windows
                    .iter()
                    .filter(|(_, s)| !s.minimized && s.workspace == self.active_workspace)
                    .map(|(id, _)| *id)
                    .next();
            }
        }
    }

    // --------------------------------------------------------
    //  PŘEPNUTÍ WORKSPACE
    // --------------------------------------------------------
    pub fn switch_workspace(&mut self, workspace: usize) {
        self.active_workspace = workspace;
        self.focused = self.windows
            .iter()
            .filter(|(_, s)| s.workspace == workspace && !s.minimized)
            .map(|(id, _)| *id)
            .next();

        tracing::info!("Switched to workspace {}", workspace);
    }

    // --------------------------------------------------------
    //  AKTUALIZACE (voláno každý snímek)
    // --------------------------------------------------------
    pub fn update(&mut self) {
        for state in self.windows.values_mut() {
            state.update_animation();
        }
    }

    // --------------------------------------------------------
    //  GETTERY
    // --------------------------------------------------------
    pub fn get(&self, id: WindowId) -> Option<&WindowState> {
        self.windows.get(&id)
    }

    pub fn get_mut(&mut self, id: WindowId) -> Option<&mut WindowState> {
        self.windows.get_mut(&id)
    }

    // Všechna viditelná okna na aktivním workspace
    pub fn visible_windows(&self) -> Vec<&WindowState> {
        let mut windows: Vec<&WindowState> = self.windows
            .values()
            .filter(|s| s.workspace == self.active_workspace && !s.minimized)
            .collect();

        // Fokusované okno vždy nahoře
        windows.sort_by(|a, b| {
            let a_focused = self.focused == Some(a.id);
            let b_focused = self.focused == Some(b.id);
            b_focused.cmp(&a_focused)
        });

        windows
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    pub fn focused_title(&self) -> Option<&str> {
        self.focused
            .and_then(|id| self.windows.get(&id))
            .map(|s| s.title.as_str())
    }
}
