// ============================================================
//  animation.rs — Animační systém
//  Plynulé animace oken (otevření, zavření, přesun)
// ============================================================

use std::time::{Duration, Instant};

// ============================================================
//  STAV ANIMACE
// ============================================================
#[derive(Debug, Clone, PartialEq)]
pub enum AnimState {
    Idle,      // žádná animace
    Running,   // animace běží
    Finished,  // animace dokončena
}

// ============================================================
//  TYP ANIMACE
// ============================================================
#[derive(Debug, Clone)]
pub enum AnimKind {
    Open,    // okno se otevírá (fade + scale in)
    Close,   // okno se zavírá (fade + scale out)
    Move,    // okno se přesouvá
    Resize,  // okno se mění velikost
}

// ============================================================
//  EASING FUNKCE
//  Určují jak rychle animace začíná/končí
// ============================================================
#[derive(Debug, Clone)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,  // nejhezčí — pomalý začátek i konec
    Bounce,     // odrazí se na konci
}

impl Easing {
    // t = 0.0 až 1.0 (pokrok animace)
    // vrací upravený pokrok
    pub fn apply(&self, t: f64) -> f64 {
        match self {
            Easing::Linear => t,

            Easing::EaseIn => t * t,

            Easing::EaseOut => t * (2.0 - t),

            // Nejpoužívanější — hladké jak macOS
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }

            // Odrazí se na konci jako spring
            Easing::Bounce => {
                if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    7.5625 * t * t + 0.75
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    7.5625 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / 2.75;
                    7.5625 * t * t + 0.984375
                }
            }
        }
    }
}

// ============================================================
//  ANIMACE HODNOTY (f64)
//  Animuje libovolnou číselnou hodnotu z A do B
// ============================================================
#[derive(Debug, Clone)]
pub struct Anim {
    pub from: f64,
    pub to: f64,
    pub duration: Duration,
    pub easing: Easing,
    pub start_time: Option<Instant>,
    pub state: AnimState,
}

impl Anim {
    pub fn new(from: f64, to: f64, duration_ms: u64, easing: Easing) -> Self {
        Self {
            from,
            to,
            duration: Duration::from_millis(duration_ms),
            easing,
            start_time: None,
            state: AnimState::Idle,
        }
    }

    // Spusť animaci
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.state = AnimState::Running;
    }

    // Vrať aktuální hodnotu
    pub fn value(&mut self) -> f64 {
        match self.state {
            AnimState::Idle => self.from,
            AnimState::Finished => self.to,
            AnimState::Running => {
                let elapsed = self.start_time
                    .map(|t| t.elapsed())
                    .unwrap_or(Duration::ZERO);

                // Pokrok 0.0 - 1.0
                let t = (elapsed.as_secs_f64() / self.duration.as_secs_f64())
                    .clamp(0.0, 1.0);

                if t >= 1.0 {
                    self.state = AnimState::Finished;
                    return self.to;
                }

                // Aplikuj easing
                let eased = self.easing.apply(t);

                // Interpoluj mezi from a to
                self.from + (self.to - self.from) * eased
            }
        }
    }

    pub fn is_done(&self) -> bool {
        self.state == AnimState::Finished
    }
}

// ============================================================
//  ANIMACE OKNA
//  Kombinuje více animací pro jedno okno
// ============================================================
#[derive(Debug, Clone)]
pub struct WindowAnim {
    pub kind: AnimKind,

    // Průhlednost (0.0 = neviditelné, 1.0 = plně viditelné)
    pub opacity: Anim,

    // Měřítko (1.0 = normální, 0.9 = trochu zmenšené)
    pub scale: Anim,

    // Pozice X a Y (pro animaci přesunu)
    pub x: Option<Anim>,
    pub y: Option<Anim>,
}

impl WindowAnim {
    // Animace otevření okna — fade in + scale up
    pub fn open(duration_ms: u64) -> Self {
        let mut opacity = Anim::new(0.0, 1.0, duration_ms, Easing::EaseOut);
        let mut scale = Anim::new(0.9, 1.0, duration_ms, Easing::EaseOut);
        opacity.start();
        scale.start();
        Self {
            kind: AnimKind::Open,
            opacity,
            scale,
            x: None,
            y: None,
        }
    }

    // Animace zavření okna — fade out + scale down
    pub fn close(duration_ms: u64) -> Self {
        let mut opacity = Anim::new(1.0, 0.0, duration_ms, Easing::EaseIn);
        let mut scale = Anim::new(1.0, 0.9, duration_ms, Easing::EaseIn);
        opacity.start();
        scale.start();
        Self {
            kind: AnimKind::Close,
            opacity,
            scale,
            x: None,
            y: None,
        }
    }

    // Animace přesunu okna
    pub fn move_to(from_x: f64, to_x: f64, from_y: f64, to_y: f64, duration_ms: u64) -> Self {
        let mut x = Anim::new(from_x, to_x, duration_ms, Easing::EaseInOut);
        let mut y = Anim::new(from_y, to_y, duration_ms, Easing::EaseInOut);
        x.start();
        y.start();
        Self {
            kind: AnimKind::Move,
            opacity: Anim::new(1.0, 1.0, duration_ms, Easing::Linear),
            scale: Anim::new(1.0, 1.0, duration_ms, Easing::Linear),
            x: Some(x),
            y: Some(y),
        }
    }

    // Je animace hotová?
    pub fn is_done(&mut self) -> bool {
        self.opacity.is_done() && self.scale.is_done()
    }

    // Aktuální hodnoty
    pub fn current_opacity(&mut self) -> f64 {
        self.opacity.value()
    }

    pub fn current_scale(&mut self) -> f64 {
        self.scale.value()
    }

    pub fn current_x(&mut self) -> Option<f64> {
        self.x.as_mut().map(|a| a.value())
    }

    pub fn current_y(&mut self) -> Option<f64> {
        self.y.as_mut().map(|a| a.value())
    }
}
