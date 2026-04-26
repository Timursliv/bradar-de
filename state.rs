// ============================================================
//  state.rs — Hlavní stav celého DE
//  Tento struct drží VŠE co DE potřebuje
// ============================================================

use smithay::{
    desktop::{Space, Window},
    input::{Seat, SeatState, keyboard::KeyboardHandle, pointer::PointerHandle},
    reexports::{
        calloop::{LoopHandle, LoopSignal},
        wayland_server::DisplayHandle,
    },
    utils::{Clock, Monotonic, Point, Logical},
    wayland::{
        compositor::CompositorState,
        shell::xdg::XdgShellState,
        shm::ShmState,
        output::OutputManagerState,
    },
};

use crate::{
    config::Config,
    window::WindowManager,
    bar::Bar,
    keybinds::Keybinds,
    cursor::CursorState,
};

// ============================================================
//  HLAVNÍ STAV
// ============================================================
pub struct State {
    // --------------------------------------------------------
    //  WAYLAND INFRASTRUKTURA
    // --------------------------------------------------------
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, State>,
    pub loop_signal: LoopSignal,
    pub clock: Clock<Monotonic>,

    // --------------------------------------------------------
    //  WAYLAND PROTOKOLY
    // --------------------------------------------------------
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub seat_state: SeatState<State>,

    // --------------------------------------------------------
    //  INPUT
    // --------------------------------------------------------
    pub seat: Seat<State>,
    pub keyboard: KeyboardHandle<State>,
    pub pointer: PointerHandle<State>,

    // Pozice kurzoru na obrazovce
    pub cursor_pos: Point<f64, Logical>,

    // --------------------------------------------------------
    //  SPRÁVA OKEN
    // --------------------------------------------------------
    pub space: Space<Window>,        // Smithay space (interní)
    pub window_manager: WindowManager,

    // --------------------------------------------------------
    //  DE KOMPONENTY
    // --------------------------------------------------------
    pub bar: Bar,
    pub keybinds: Keybinds,
    pub cursor_state: CursorState,

    // --------------------------------------------------------
    //  KONFIGURACE
    // --------------------------------------------------------
    pub config: Config,

    // --------------------------------------------------------
    //  ROZLIŠENÍ OBRAZOVKY
    // --------------------------------------------------------
    pub screen_width: u32,
    pub screen_height: u32,

    // --------------------------------------------------------
    //  OVLÁDÁNÍ
    // --------------------------------------------------------
    pub running: bool,  // false = ukončit DE
}

impl State {
    pub fn new(
        display_handle: DisplayHandle,
        loop_handle: LoopHandle<'static, State>,
        loop_signal: LoopSignal,
        seat: Seat<State>,
        keyboard: KeyboardHandle<State>,
        pointer: PointerHandle<State>,
        compositor_state: CompositorState,
        xdg_shell_state: XdgShellState,
        shm_state: ShmState,
        output_manager_state: OutputManagerState,
        seat_state: SeatState<State>,
        config: Config,
        screen_width: u32,
        screen_height: u32,
    ) -> Self {
        let bar_height = if config.bar.enabled { config.bar.height } else { 0 };

        let window_manager = WindowManager::new(
            screen_width,
            screen_height,
            bar_height,
            if config.animations.enabled { config.animations.duration_ms } else { 0 },
        );

        let bar = Bar::new(config.bar.clone(), screen_width);

        let keybinds = Keybinds::new(
            config.keybinds.terminal.clone(),
            config.keybinds.launcher.clone(),
        );

        Self {
            display_handle,
            loop_handle,
            loop_signal,
            clock: Clock::new(),
            compositor_state,
            xdg_shell_state,
            shm_state,
            output_manager_state,
            seat_state,
            seat,
            keyboard,
            pointer,
            cursor_pos: Point::from((0.0, 0.0)),
            space: Space::default(),
            window_manager,
            bar,
            keybinds,
            cursor_state: CursorState::new(),
            config,
            screen_width,
            screen_height,
            running: true,
        }
    }

    // --------------------------------------------------------
    //  AKTUALIZACE (každý snímek)
    // --------------------------------------------------------
    pub fn update(&mut self) {
        // Aktualizuj animace oken
        self.window_manager.update();

        // Aktualizuj lištu
        let workspace = self.window_manager.active_workspace;
        let title = self.window_manager.focused_title().map(|s| s.to_string());
        self.bar.update(workspace, title.as_deref());
    }

    // --------------------------------------------------------
    //  QUIT
    // --------------------------------------------------------
    pub fn quit(&mut self) {
        tracing::info!("Quitting DE...");
        self.running = false;
        self.loop_signal.stop();
    }
}

// ============================================================
//  SMITHAY TRAITS
//  Smithay vyžaduje implementaci těchto traits pro State
// ============================================================

// Compositor trait
impl smithay::wayland::compositor::CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(
        &self,
        client: &'a smithay::reexports::wayland_server::Client,
    ) -> &'a smithay::wayland::compositor::CompositorClientState {
        &client.get_data::<crate::compositor::ClientData>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface) {
        // Klient commitnul nový obsah — překresli
        smithay::desktop::on_commit_buffer_handler::<Self>(surface);
    }
}

// XDG Shell trait — pro otevírání/zavírání oken
impl smithay::wayland::shell::xdg::XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        // Nové okno se otevírá!
        let window = Window::new_wayland_window(surface);
        let default_w = self.config.window.default_width;
        let default_h = self.config.window.default_height;
        self.window_manager.add_window(window.clone(), default_w, default_h);
        self.space.map_element(window, (0, 0), true);
        tracing::info!("New window opened");
    }

    fn toplevel_destroyed(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        // Okno se zavírá
        tracing::info!("Window closed");
    }

    fn new_popup(
        &mut self,
        _surface: smithay::wayland::shell::xdg::PopupSurface,
        _positioner: smithay::wayland::shell::xdg::PositionerState,
    ) {}

    fn grab(
        &mut self,
        _surface: smithay::wayland::shell::xdg::PopupSurface,
        _seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat,
        _serial: smithay::utils::Serial,
    ) {}
}

// SHM trait — sdílená paměť pro obsah oken
impl smithay::wayland::shm::ShmHandler for State {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

// Seat trait — vstupní zařízení
impl smithay::input::SeatHandler for State {
    type KeyboardFocus = smithay::desktop::Window;
    type PointerFocus = smithay::desktop::Window;
    type TouchFocus = smithay::desktop::Window;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(
        &mut self,
        _seat: &Seat<Self>,
        _focused: Option<&Self::KeyboardFocus>,
    ) {}

    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        _image: smithay::input::pointer::CursorImageStatus,
    ) {}
}

smithay::delegate_compositor!(State);
smithay::delegate_xdg_shell!(State);
smithay::delegate_shm!(State);
smithay::delegate_seat!(State);
smithay::delegate_output!(State);
