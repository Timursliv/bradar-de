// ============================================================
//  bar.rs — Horní lišta (jako macOS menu bar)
//  Zobrazuje: čas | workspaces | název aktivního okna
// ============================================================

use chrono::Local;
use crate::config::BarConfig;

// ============================================================
//  WIDGET NA LIŠTĚ
//  Každá část lišty je widget
// ============================================================
#[derive(Debug, Clone)]
pub enum Widget {
    Clock,
    Workspaces { active: usize, total: usize },
    ActiveWindow { title: String },
    Spacer,
}

// ============================================================
//  LIŠTA
// ============================================================
pub struct Bar {
    pub config: BarConfig,
    pub width: u32,

    // Části lišty
    pub left: Vec<Widget>,    // levá strana
    pub center: Vec<Widget>,  // střed
    pub right: Vec<Widget>,   // pravá strana
}

impl Bar {
    pub fn new(config: BarConfig, width: u32) -> Self {
        Self {
            config,
            width,
            left: vec![
                Widget::Workspaces { active: 0, total: 9 },
            ],
            center: vec![
                Widget::ActiveWindow { title: String::new() },
            ],
            right: vec![
                Widget::Clock,
            ],
        }
    }

    // --------------------------------------------------------
    //  AKTUALIZACE (voláno každý snímek)
    // --------------------------------------------------------
    pub fn update(&mut self, active_workspace: usize, active_window_title: Option<&str>) {
        // Aktualizuj workspace widget
        for widget in &mut self.left {
            if let Widget::Workspaces { active, .. } = widget {
                *active = active_workspace;
            }
        }

        // Aktualizuj název aktivního okna
        for widget in &mut self.center {
            if let Widget::ActiveWindow { title } = widget {
                *title = active_window_title
                    .unwrap_or("")
                    .to_string();
            }
        }
    }

    // --------------------------------------------------------
    //  ZÍSKÁNÍ ČASU
    // --------------------------------------------------------
    pub fn current_time(&self) -> String {
        Local::now().format(&self.config.clock_format).to_string()
    }

    // --------------------------------------------------------
    //  RENDER DAT PRO LIŠTU
    //  Vrátí co má být vykresleno kde
    // --------------------------------------------------------
    pub fn render_data(&self) -> BarRenderData {
        let time = self.current_time();

        let workspace_dots = self.get_workspace_dots();
        let active_title = self.get_active_title();

        BarRenderData {
            height: self.config.height,
            time,
            workspace_dots,
            active_title,
        }
    }

    fn get_workspace_dots(&self) -> Vec<WorkspaceDot> {
        let mut dots = Vec::new();
        for widget in &self.left {
            if let Widget::Workspaces { active, total } = widget {
                for i in 0..*total {
                    dots.push(WorkspaceDot {
                        index: i,
                        active: i == *active,
                    });
                }
            }
        }
        dots
    }

    fn get_active_title(&self) -> String {
        for widget in &self.center {
            if let Widget::ActiveWindow { title } = widget {
                return title.clone();
            }
        }
        String::new()
    }
}

// --------------------------------------------------------
//  DATA PRO VYKRESLENÍ
// --------------------------------------------------------
#[derive(Debug)]
pub struct BarRenderData {
    pub height: u32,
    pub time: String,
    pub workspace_dots: Vec<WorkspaceDot>,
    pub active_title: String,
}

#[derive(Debug)]
pub struct WorkspaceDot {
    pub index: usize,
    pub active: bool,
}
