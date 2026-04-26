// ============================================================
//  cursor.rs — Správa kurzoru myši
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum CursorShape {
    Default,   // normální šipka
    Move,      // křížek pro přesun okna
    Resize,    // šipky pro změnu velikosti
    Text,      // I-beam pro text
    Pointer,   // ruka pro klikatelné prvky
}

pub struct CursorState {
    pub shape: CursorShape,
    pub visible: bool,

    // Je uživatel momentálně přesouváno okno?
    pub dragging: bool,
    pub drag_start_x: f64,
    pub drag_start_y: f64,
    pub drag_window_start_x: i32,
    pub drag_window_start_y: i32,
}

impl CursorState {
    pub fn new() -> Self {
        Self {
            shape: CursorShape::Default,
            visible: true,
            dragging: false,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
            drag_window_start_x: 0,
            drag_window_start_y: 0,
        }
    }

    // Začni přesun okna
    pub fn start_drag(&mut self, cursor_x: f64, cursor_y: f64, window_x: i32, window_y: i32) {
        self.dragging = true;
        self.drag_start_x = cursor_x;
        self.drag_start_y = cursor_y;
        self.drag_window_start_x = window_x;
        self.drag_window_start_y = window_y;
        self.shape = CursorShape::Move;
    }

    // Ukonči přesun okna
    pub fn stop_drag(&mut self) {
        self.dragging = false;
        self.shape = CursorShape::Default;
    }

    // Vypočítej novou pozici okna při přesunu
    pub fn drag_window_pos(&self, cursor_x: f64, cursor_y: f64) -> (i32, i32) {
        let dx = cursor_x - self.drag_start_x;
        let dy = cursor_y - self.drag_start_y;
        (
            self.drag_window_start_x + dx as i32,
            self.drag_window_start_y + dy as i32,
        )
    }
}
