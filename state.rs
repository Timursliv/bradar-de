use smithay::{
    desktop::{Space, Window, PopupManager},
    input::{
        Seat, SeatState, SeatHandler,
        keyboard::KeyboardHandle,
        pointer::{PointerHandle, CursorImageStatus},
    },
    reexports::{
        calloop::{LoopHandle, LoopSignal},
        wayland_server::{
            DisplayHandle, Client,
            backend::{ClientId, DisconnectReason, ClientData as ClientDataTrait},
            protocol::wl_surface::WlSurface,
        },
    },
    utils::{Clock, Monotonic, Point, Logical, Serial},
    wayland::{
        compositor::{CompositorState, CompositorClientState, CompositorHandler, self},
        shell::xdg::{
            XdgShellState, XdgShellHandler, ToplevelSurface, PopupSurface,
            PositionerState, ToplevelConfigure, PopupConfigure,
        },
        shm::{ShmState, ShmHandler},
        output::OutputManagerState,
        selection::data_device::{
            ClientDndGrabHandler, ServerDndGrabHandler, DataDeviceHandler, DataDeviceState,
        },
        selection::SelectionHandler,
    },
};

use crate::config::Config;

// ---- Client Data ----
#[derive(Default)]
pub struct ClientData {
    pub compositor_state: CompositorClientState,
}

impl ClientDataTrait for ClientData {
    fn initialized(&self, _: ClientId) {}
    fn disconnected(&self, _: ClientId, _: DisconnectReason) {}
}

// ---- Main State ----
pub struct State {
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, State>,
    pub loop_signal: LoopSignal,
    pub clock: Clock<Monotonic>,

    // Protocols
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub seat_state: SeatState<State>,
    pub popup_manager: PopupManager,

    // Input
    pub seat: Seat<State>,
    pub keyboard: KeyboardHandle<State>,
    pub pointer: PointerHandle<State>,
    pub cursor_pos: Point<f64, Logical>,

    // Windows
    pub space: Space<Window>,

    // Config
    pub config: Config,

    // Control
    pub running: bool,
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
    ) -> Self {
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
            popup_manager: PopupManager::default(),
            seat,
            keyboard,
            pointer,
            cursor_pos: Point::from((0.0, 0.0)),
            space: Space::default(),
            config,
            running: true,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
        self.loop_signal.stop();
    }
}

// ---- Compositor Handler ----
impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientData>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        compositor::on_commit_buffer_handler::<Self>(surface);
        self.popup_manager.commit(surface);
    }
}

// ---- XDG Shell Handler ----
impl XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface);
        self.space.map_element(window, (100, 100), true);
        tracing::info!("New window opened");
    }

    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        let _ = self.popup_manager.track_popup(surface.into());
    }

    fn reposition_request(&mut self, surface: PopupSurface, positioner: PositionerState, token: u32) {
        surface.with_pending_state(|state| {
            state.geometry = positioner.get_geometry();
            state.positioner = positioner;
        });
        surface.send_repositioned(token);
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        tracing::info!("Window closed");
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat, _serial: Serial) {}

    fn configure_request(&mut self, _surface: ToplevelSurface) {}

    fn configure_done(&mut self, _surface: ToplevelSurface) {}
}

// ---- SHM Handler ----
impl ShmHandler for State {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

// ---- Seat Handler ----
impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}
    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {}
}

// ---- Delegate macros ----
smithay::delegate_compositor!(State);
smithay::delegate_xdg_shell!(State);
smithay::delegate_shm!(State);
smithay::delegate_seat!(State);
smithay::delegate_output!(State);
