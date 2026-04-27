mod state;
mod input;
mod config;

use smithay::{
    backend::{
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        session::{libseat::LibSeatSession, Session},
        udev::UdevBackend,
    },
    reexports::{
        calloop::EventLoop,
        wayland_server::Display,
    },
    wayland::{
        compositor::CompositorState,
        shell::xdg::XdgShellState,
        shm::ShmState,
        output::OutputManagerState,
        socket::ListeningSocketSource,
    },
    input::{SeatState, Seat},
};

use tracing::info;
use tracing_subscriber::EnvFilter;
use std::time::Duration;

use state::State;
use config::Config;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    info!("Starting Bradar DE...");

    let config = Config::load();

    // Display
    let mut display: Display<State> = Display::new().unwrap();
    let dh = display.handle();

    // Event loop
    let mut event_loop: EventLoop<State> = EventLoop::try_new().unwrap();
    let loop_handle = event_loop.handle();
    let loop_signal = event_loop.get_signal();

    // Session
    let (session, notifier) = LibSeatSession::new().expect(
        "Could not create libseat session. Run: sudo systemctl start seatd"
    );
    info!("Seat: {}", session.seat());

    loop_handle.insert_source(notifier, |_, _, _| {}).unwrap();

    // Protocols
    let compositor_state = CompositorState::new::<State>(&dh);
    let xdg_shell_state = XdgShellState::new::<State>(&dh);
    let shm_state = ShmState::new::<State>(&dh, vec![]);
    let output_manager_state = OutputManagerState::new_with_xdg_output::<State>(&dh);
    let mut seat_state = SeatState::new();
    let mut seat = seat_state.new_wl_seat(&dh, "seat0");
    let keyboard = seat.add_keyboard(Default::default(), 200, 25).unwrap();
    let pointer = seat.add_pointer();

    // Wayland socket
    let socket = ListeningSocketSource::new_auto().unwrap();
    let socket_name = socket.socket_name().to_string_lossy().into_owned();
    std::env::set_var("WAYLAND_DISPLAY", &socket_name);
    info!("WAYLAND_DISPLAY={}", socket_name);

    loop_handle.insert_source(socket, |stream, _, state: &mut State| {
        state.display_handle
            .insert_client(stream, std::sync::Arc::new(state::ClientData::default()))
            .unwrap();
        info!("Client connected");
    }).unwrap();

    // libinput
    let mut libinput_ctx = input::Libinput::new_with_udev(
        LibinputSessionInterface::from(session.clone()),
    );
    libinput_ctx.udev_assign_seat(session.seat()).unwrap();
    let input_backend = LibinputInputBackend::new(libinput_ctx);

    loop_handle.insert_source(input_backend, |event, _, state: &mut State| {
        state.process_input_event(event);
    }).unwrap();

    // Build state
    let mut state = State::new(
        dh,
        loop_handle,
        loop_signal,
        seat,
        keyboard,
        pointer,
        compositor_state,
        xdg_shell_state,
        shm_state,
        output_manager_state,
        seat_state,
        config,
    );

    info!("Bradar DE running!");
    info!("Super+T = terminal | Super+Q = close | Super+Escape = quit");

    while state.running {
        display.dispatch_clients(&mut state).unwrap();
        event_loop
            .dispatch(Some(Duration::from_millis(16)), &mut state)
            .unwrap();
        display.flush_clients(&mut state).unwrap();
    }

    info!("Bradar DE stopped.");
}
