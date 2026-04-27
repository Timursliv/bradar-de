// ============================================================
//  render.rs — Vykreslování na obrazovku
//  Kreslí: pozadí, okna, lištu, kurzor
// ============================================================

use smithay::{
    backend::renderer::{
        Color32F,
        Frame, Renderer,
    },
    utils::{Rectangle, Size, Transform, Physical},
};

use crate::state::State;

// ============================================================
//  BARVA (R, G, B, A) — hodnoty 0.0 - 1.0
// ============================================================
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_color32f(&self) -> Color32F {
        Color32F::new(self.r, self.g, self.b, self.a)
    }

    // Parsuj hex barvu "#rrggbb" nebo "#rrggbbaa"
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let parse = |s: &str| u8::from_str_radix(s, 16).unwrap_or(0) as f32 / 255.0;

        match hex.len() {
            6 => Self::new(
                parse(&hex[0..2]),
                parse(&hex[2..4]),
                parse(&hex[4..6]),
                1.0,
            ),
            8 => Self::new(
                parse(&hex[0..2]),
                parse(&hex[2..4]),
                parse(&hex[4..6]),
                parse(&hex[6..8]),
            ),
            _ => Self::new(0.0, 0.0, 0.0, 1.0),
        }
    }
}

// ============================================================
//  RENDERER
// ============================================================
pub struct DERenderer {
    pub screen_width: u32,
    pub screen_height: u32,
    pub bg_color: Color,
    pub bar_color: Color,
    pub bar_height: u32,
    pub border_active_color: Color,
    pub border_inactive_color: Color,
    pub border_width: u32,
}

impl DERenderer {
    pub fn new(config: &crate::config::Config, screen_width: u32, screen_height: u32) -> Self {
        Self {
            screen_width,
            screen_height,
            bg_color: Color::from_hex(&config.theme.background),
            bar_color: Color::from_hex(&config.theme.bar_background),
            bar_height: config.bar.height,
            border_active_color: Color::from_hex(&config.theme.window_border_active),
            border_inactive_color: Color::from_hex(&config.theme.window_border_inactive),
            border_width: config.theme.window_border_width,
        }
    }

    // --------------------------------------------------------
    //  HLAVNÍ RENDER FUNKCE
    //  Voláno každý snímek (cca 60x za sekundu)
    // --------------------------------------------------------
    pub fn render_frame<F: Frame>(
        &self,
        frame: &mut F,
        state: &mut State,
    ) -> Result<(), F::Error> {
        let size = Size::<i32, Physical>::from((self.screen_width as i32, self.screen_height as i32));
        let full_rect = Rectangle::new((0, 0).into(), size);

        // 1. Vymaž obrazovku barvou pozadí
        frame.clear(
            self.bg_color.to_color32f(),
            &[full_rect],
        )?;

        // 2. Vykresli okna (od spodního po horní)
        self.render_windows(frame, state)?;

        // 3. Vykresli lištu navrchu
        if state.config.bar.enabled {
            self.render_bar(frame, state)?;
        }

        // 4. Vykresli kurzor
        self.render_cursor(frame, state)?;

        Ok(())
    }

    // --------------------------------------------------------
    //  VYKRESLENÍ OKEN
    // --------------------------------------------------------
    fn render_windows<F: Frame>(
        &self,
        frame: &mut F,
        state: &mut State,
    ) -> Result<(), F::Error> {
        let focused_id = state.window_manager.focused;

        // Získej seznam viditelných oken
        let window_data: Vec<_> = state.window_manager.visible_windows()
            .iter()
            .map(|s| (s.id, s.x, s.y, s.width, s.height, s.opacity))
            .collect();

        for (id, x, y, w, h, opacity) in window_data {
            let is_focused = focused_id == Some(id);

            // Rámeček okna
            let border_color = if is_focused {
                self.border_active_color
            } else {
                self.border_inactive_color
            };

            let bw = self.border_width as i32;

            // Vykresli rámeček (4 obdélníky kolem okna)
            // Horní
            frame.clear(
                border_color.to_color32f(),
                &[Rectangle::new((x - bw, y - bw).into(), (w as i32 + bw * 2, bw).into())],
            )?;
            // Spodní
            frame.clear(
                border_color.to_color32f(),
                &[Rectangle::new((x - bw, y + h as i32).into(), (w as i32 + bw * 2, bw).into())],
            )?;
            // Levý
            frame.clear(
                border_color.to_color32f(),
                &[Rectangle::new((x - bw, y).into(), (bw, h as i32).into())],
            )?;
            // Pravý
            frame.clear(
                border_color.to_color32f(),
                &[Rectangle::new((x + w as i32, y).into(), (bw, h as i32).into())],
            )?;

            // Obsah okna vykreslí Smithay automaticky přes Space
            // (render_output funkce)
        }

        Ok(())
    }

    // --------------------------------------------------------
    //  VYKRESLENÍ LIŠTY
    // --------------------------------------------------------
    fn render_bar<F: Frame>(
        &self,
        frame: &mut F,
        state: &mut State,
    ) -> Result<(), F::Error> {
        // Pozadí lišty
        let bar_rect = Rectangle::new(
            (0, 0).into(),
            (self.screen_width as i32, self.bar_height as i32).into(),
        );

        frame.clear(self.bar_color.to_color32f(), &[bar_rect])?;

        // Text v liště se renderuje pomocí knihovny pro text
        // (např. cosmic-text nebo rusttype)
        // Pro začátek je lišta jen barevný obdélník
        // TODO: přidej text rendering

        Ok(())
    }

    // --------------------------------------------------------
    //  VYKRESLENÍ KURZORU
    // --------------------------------------------------------
    fn render_cursor<F: Frame>(
        &self,
        frame: &mut F,
        state: &mut State,
    ) -> Result<(), F::Error> {
        let cx = state.cursor_pos.x as i32;
        let cy = state.cursor_pos.y as i32;

        // Jednoduchý kurzor — bílý čtverec 8x8 px
        // TODO: nahradit obrázkem kurzoru
        let cursor_rect = Rectangle::new((cx, cy).into(), (8, 8).into());
        frame.clear(Color32F::new(1.0, 1.0, 1.0, 1.0), &[cursor_rect])?;

        Ok(())
    }
}
