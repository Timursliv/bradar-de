pub use input::Libinput;

use smithay::{
    backend::{
        input::{
            AbsolutePositionEvent, Axis, AxisSource, ButtonState,
            Event, InputEvent, KeyboardKeyEvent, PointerAxisEvent,
            PointerButtonEvent, PointerMotionAbsoluteEvent, PointerMotionEvent,
        },
        libinput::LibinputInputBackend,
    },
    input::{
        keyboard::{keysyms, FilterResult},
        pointer::{AxisFrame, ButtonEvent, MotionEvent, RelativeMotionEvent},
    },
    utils::SERIAL_COUNTER,
};

use crate::state::State;

impl State {
    pub fn process_input_event(&mut self, event: InputEvent<LibinputInputBackend>) {
        match event {
            // ---- KEYBOARD ----
            InputEvent::Keyboard { event } => {
                let serial = SERIAL_COUNTER.next_serial();
                let time = event.time_msec();
                let state_val = event.state();

                let action = self.keyboard.input(
                    self,
                    event.key_code(),
                    state_val,
                    serial,
                    time,
                    |state, modifiers, handle| {
                        let sym = handle.modified_sym();
                        let super_key = modifiers.logo;
                        let shift = modifiers.shift;

                        // Super+Escape = quit
                        if super_key && sym.raw() == keysyms::KEY_Escape {
                            return FilterResult::Intercept(1u32);
                        }
                        // Super+T = terminal
                        if super_key && sym.raw() == keysyms::KEY_t {
                            return FilterResult::Intercept(2u32);
                        }
                        // Super+Q = close window
                        if super_key && sym.raw() == keysyms::KEY_q {
                            return FilterResult::Intercept(3u32);
                        }
                        // Super+1..9 = workspaces (future)
                        FilterResult::Forward
                    },
                );

                match action {
                    Some(1) => {
                        tracing::info!("Quit!");
                        self.quit();
                    }
                    Some(2) => {
                        let term = self.config.keybinds.terminal.clone();
                        tracing::info!("Launch: {}", term);
                        let _ = std::process::Command::new(&term).spawn();
                    }
                    Some(3) => {
                        // Close focused window
                        if let Some(win) = self.space.elements().last().cloned() {
                            if let Some(toplevel) = win.toplevel() {
                                toplevel.send_close();
                            }
                        }
                    }
                    _ => {}
                }
            }

            // ---- MOUSE MOVE ----
            InputEvent::PointerMotion { event } => {
                let delta = event.delta();
                self.cursor_pos.x = (self.cursor_pos.x + delta.x)
                    .clamp(0.0, 4096.0);
                self.cursor_pos.y = (self.cursor_pos.y + delta.y)
                    .clamp(0.0, 4096.0);

                let serial = SERIAL_COUNTER.next_serial();
                let under = self.surface_under();
                self.pointer.motion(
                    self,
                    under,
                    &MotionEvent {
                        location: self.cursor_pos,
                        serial,
                        time: event.time_msec(),
                    },
                );
                self.pointer.frame(self);
            }

            // ---- MOUSE CLICK ----
            InputEvent::PointerButton { event } => {
                let serial = SERIAL_COUNTER.next_serial();
                let button = event.button_code();
                let state_val = event.state();

                // Focus window under cursor on click
                if state_val == ButtonState::Pressed {
                    let pos = self.cursor_pos;
                    if let Some((window, _)) = self.space.element_under(pos) {
                        let window = window.clone();
                        self.space.raise_element(&window, true);
                        if let Some(surface) = window.wl_surface() {
                            let keyboard = self.keyboard.clone();
                            keyboard.set_focus(self, Some(surface), serial);
                        }
                    }
                }

                self.pointer.button(
                    self,
                    &ButtonEvent {
                        button,
                        state: state_val,
                        serial,
                        time: event.time_msec(),
                    },
                );
                self.pointer.frame(self);
            }

            _ => {}
        }
    }

    // Find the Wayland surface under the cursor
    fn surface_under(&self) -> Option<(smithay::reexports::wayland_server::protocol::wl_surface::WlSurface, smithay::utils::Point<f64, smithay::utils::Logical>)> {
        let pos = self.cursor_pos;
        self.space.element_under(pos).and_then(|(window, location)| {
            window.surface_under(pos - location.to_f64(), smithay::desktop::WindowSurfaceType::ALL)
                .map(|(surface, point)| (surface, point.to_f64()))
        })
    }
}
