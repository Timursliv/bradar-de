// ============================================================
//  input.rs — Zpracování vstupu (klávesnice, myš)
// ============================================================

use smithay::{
    backend::input::{
        InputEvent, KeyboardKeyEvent, PointerMotionEvent,
        PointerButtonEvent, ButtonState, InputBackend,
    },
    input::keyboard::{FilterResult, keysyms},
    utils::SERIAL_COUNTER,
};

use crate::{
    state::State,
    keybinds::Action,
    window::WindowId,
};

impl State {
    // ============================================================
    //  VSTUP Z KLÁVESNICE
    // ============================================================
    pub fn handle_keyboard<B: InputBackend>(
        &mut self,
        event: B::KeyboardKeyEvent,
    ) {
        let serial = SERIAL_COUNTER.next_serial();
        let time = event.time_msec();
        let key_state = event.state();

        // Zpracuj klávesu přes keyboard handler
        // Ten se postará o XKB (layout, shift, caps lock atd.)
        let action = self.keyboard.input(
            self,
            event.key_code(),
            key_state,
            serial,
            time,
            |state, modifiers, handle| {
                let sym = handle.modified_sym();

                // Zkontroluj jestli odpovídá nějaké zkratce
                if let Some(action) = state.keybinds.handle(sym.raw(), modifiers) {
                    return FilterResult::Intercept(Some(action));
                }

                // Jinak pošli klávesu do fokusovaného okna
                FilterResult::Forward
            },
        );

        // Proveď akci
        if let Some(Some(action)) = action {
            self.execute_action(action);
        }
    }

    // ============================================================
    //  POHYB MYŠI
    // ============================================================
    pub fn handle_pointer_motion<B: InputBackend>(
        &mut self,
        event: B::PointerMotionEvent,
    ) {
        // Aktualizuj pozici kurzoru
        let delta = event.delta();
        self.cursor_pos.x = (self.cursor_pos.x + delta.x)
            .clamp(0.0, self.screen_width as f64);
        self.cursor_pos.y = (self.cursor_pos.y + delta.y)
            .clamp(0.0, self.screen_height as f64);

        let cx = self.cursor_pos.x;
        let cy = self.cursor_pos.y;

        // Pokud přesouváme okno, aktualizuj jeho pozici
        if self.cursor_state.dragging {
            if let Some(focused_id) = self.window_manager.focused {
                let (new_x, new_y) = self.cursor_state.drag_window_pos(cx, cy);
                if let Some(state) = self.window_manager.get_mut(focused_id) {
                    // Přímý přesun bez animace (drag musí být okamžitý)
                    state.x = new_x;
                    state.y = new_y.max(self.window_manager.bar_height as i32);
                }
            }
        }

        // Aktualizuj pointer v Smithay (posílá pozici do oken)
        let serial = SERIAL_COUNTER.next_serial();
        let time = event.time_msec();

        // Najdi okno pod kurzorem
        let window_under = self.space.element_under(self.cursor_pos)
            .map(|(w, _)| w.clone());

        self.pointer.motion(
            self,
            window_under,
            &smithay::input::pointer::MotionEvent {
                location: self.cursor_pos,
                serial,
                time,
            },
        );
    }

    // ============================================================
    //  KLIK MYŠÍ
    // ============================================================
    pub fn handle_pointer_button<B: InputBackend>(
        &mut self,
        event: B::PointerButtonEvent,
    ) {
        let serial = SERIAL_COUNTER.next_serial();
        let time = event.time_msec();
        let button = event.button();
        let state = event.state();

        match state {
            ButtonState::Pressed => {
                let cx = self.cursor_pos.x;
                let cy = self.cursor_pos.y;

                // Fokusuj okno pod kurzorem
                if let Some(focused_id) = self.window_manager.focus_at(cx, cy) {
                    // Začni přesun okna (levé tlačítko)
                    if button == 0x110 { // BTN_LEFT
                        if let Some(win_state) = self.window_manager.get(focused_id) {
                            self.cursor_state.start_drag(
                                cx, cy,
                                win_state.x, win_state.y,
                            );
                        }
                    }
                }
            }
            ButtonState::Released => {
                // Ukonči přesun okna
                self.cursor_state.stop_drag();
            }
        }

        // Pošli klik do fokusovaného okna
        let focused_window = self.window_manager.focused
            .and_then(|id| self.window_manager.get(id))
            .map(|s| s.window.clone());

        self.pointer.button(
            self,
            &smithay::input::pointer::ButtonEvent {
                button,
                state,
                serial,
                time,
            },
        );
    }

    // ============================================================
    //  PROVEDENÍ AKCE (z klávesové zkratky)
    // ============================================================
    pub fn execute_action(&mut self, action: Action) {
        match action {
            Action::Quit => {
                self.quit();
            }

            Action::LaunchTerminal => {
                let cmd = self.config.keybinds.terminal.clone();
                self.keybinds.launch(&cmd);
            }

            Action::LaunchLauncher => {
                let cmd = self.config.keybinds.launcher.clone();
                self.keybinds.launch(&cmd);
            }

            Action::CloseWindow => {
                // Zavři fokusované okno
                if let Some(id) = self.window_manager.focused {
                    if let Some(state) = self.window_manager.get(id) {
                        // Pošli close request do Wayland klienta
                        if let Some(toplevel) = state.window.toplevel() {
                            toplevel.send_close();
                        }
                    }
                }
            }

            Action::MaximizeWindow => {
                if let Some(id) = self.window_manager.focused {
                    self.window_manager.maximize(id);
                }
            }

            Action::MinimizeWindow => {
                if let Some(id) = self.window_manager.focused {
                    self.window_manager.minimize(id);
                }
            }

            Action::SwitchWorkspace(n) => {
                self.window_manager.switch_workspace(n);
            }

            Action::MoveWindowToWorkspace(n) => {
                if let Some(id) = self.window_manager.focused {
                    if let Some(state) = self.window_manager.get_mut(id) {
                        state.workspace = n;
                    }
                    self.window_manager.switch_workspace(n);
                }
            }

            Action::FocusNext => {
                // TODO: fokusuj další okno v pořadí
            }

            Action::FocusPrev => {
                // TODO: fokusuj předchozí okno
            }

            Action::Screenshot => {
                tracing::info!("Screenshot TODO");
                // TODO: implementuj screenshot
            }
        }
    }
}
