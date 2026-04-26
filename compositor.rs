// ============================================================
//  compositor.rs — Hlavní smyčka kompozitoru
//  Inicializuje vše a spustí event loop
// ============================================================

use std::time::Duration;

use smithay::{
    backend::{
        drm::{DrmDevice, DrmDeviceFd, DrmEvent, DrmNode, NodeType},
        gbm::{GbmDevice, GbmAllocator, GbmBufferFlags},
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::gles::GlesRenderer,
        session::{libseat::LibSeatSession, Session},
        udev::{UdevBackend, UdevEvent},
        allocator::gbm::GbmAllocator as GbmAllocatorBackend,
        drm::compositor::DrmCompositor,
    },
    input::{Seat, SeatState},
    reexports::{
        calloop::{EventLoop, LoopHandle},
        wayland_server::Display,
    },
    utils::{SERIAL_COUNTER, Point, Logical},
    wayland::{
        compositor::CompositorState,
        shell::xdg::XdgShellState,
        shm::ShmState,
        output::OutputManagerState,
        socket::ListeningSocketSource,
    },
};

use tracing::{info, warn, error};

use crate::{
    config::Config,
    state::State,
    render::DERenderer,
};

// Data přiřazená každému Wayland klientovi
pub struct ClientData {
    pub compositor_state: smithay::wayland::compositor::CompositorClientState,
}

impl smithay::reexports::wayland_server::backend::ClientData for ClientData {
    fn initialized(&self, _: smithay::reexports::wayland_server::backend::ClientId) {}
    fn disconnected(&self, _: smithay::reexports::wayland_server::backend::ClientId, _: smithay::reexports::wayland_server::backend::DisconnectReason) {}
}

// ============================================================
//  RUN — spustí celé DE
// ============================================================
pub fn run(config: Config) {
    // ----------------------------------------------------------
    // 1. Wayland display
    // ----------------------------------------------------------
    let mut display: Display<State> = Display::new().unwrap();
    let display_handle = display.handle();

    // ----------------------------------------------------------
    // 2. Event loop
    // ----------------------------------------------------------
    let mut event_loop: EventLoop<State> = EventLoop::try_new().unwrap();
    let loop_handle = event_loop.handle();
    let loop_signal = event_loop.get_signal();

    // ----------------------------------------------------------
    // 3. TTY session (libseat)
    // ----------------------------------------------------------
    let (session, notifier) = LibSeatSession::new()
        .expect("Failed to create libseat session. Is seatd running? sudo systemctl start seatd");

    info!("Session: seat={}", session.seat());

    // Přidej session notifier do event loopu
    loop_handle
        .insert_source(notifier, |event, _, state| {
            // TTY přepínání
            use smithay::backend::session::SessionEvent;
            match event {
                SessionEvent::PauseSession => {
                    info!("TTY: session paused (switched away)");
                }
                SessionEvent::ActivateSession => {
                    info!("TTY: session activated (switched back)");
                }
            }
        })
        .unwrap();

    // ----------------------------------------------------------
    // 4. udev — najdi GPU
    // ----------------------------------------------------------
    let udev = UdevBackend::new(session.seat()).unwrap();

    // ----------------------------------------------------------
    // 5. Wayland protokoly
    // ----------------------------------------------------------
    let compositor_state = CompositorState::new::<State>(&display_handle);
    let xdg_shell_state = XdgShellState::new::<State>(&display_handle);
    let shm_state = ShmState::new::<State>(&display_handle, vec![]);
    let output_manager_state = OutputManagerState::new_with_xdg_output::<State>(&display_handle);

    // ----------------------------------------------------------
    // 6. Seat (klávesnice + myš)
    // ----------------------------------------------------------
    let mut seat_state = SeatState::new();
    let mut seat = seat_state.new_wl_seat(&display_handle, "seat0");

    let keyboard = seat
        .add_keyboard(Default::default(), 200, 25)
        .expect("Failed to add keyboard");

    let pointer = seat.add_pointer();

    // ----------------------------------------------------------
    // 7. Wayland socket — kde se klienti připojí
    // ----------------------------------------------------------
    let socket = ListeningSocketSource::new_auto().unwrap();
    let socket_name = socket.socket_name().to_string_lossy().into_owned();
    std::env::set_var("WAYLAND_DISPLAY", &socket_name);
    info!("Wayland socket: {} (export WAYLAND_DISPLAY={})", socket_name, socket_name);

    loop_handle
        .insert_source(socket, |stream, _, state| {
            state.display_handle
                .insert_client(stream, std::sync::Arc::new(ClientData {
                    compositor_state: Default::default(),
                }))
                .unwrap();
            info!("New Wayland client connected");
        })
        .unwrap();

    // ----------------------------------------------------------
    // 8. libinput — vstup na TTY
    // ----------------------------------------------------------
    let mut libinput = input::Libinput::new_with_udev(
        LibinputSessionInterface::from(session.clone()),
    );
    libinput.udev_assign_seat(session.seat()).unwrap();

    let input_backend = LibinputInputBackend::new(libinput);

    loop_handle
        .insert_source(input_backend, |event, _, state| {
            use smithay::backend::input::{InputEvent, InputBackend};
            use smithay::backend::libinput::LibinputInputBackend as LIB;

            match event {
                InputEvent::Keyboard { event } => {
                    state.handle_keyboard::<LIB>(event);
                }
                InputEvent::PointerMotion { event } => {
                    state.handle_pointer_motion::<LIB>(event);
                }
                InputEvent::PointerButton { event } => {
                    state.handle_pointer_button::<LIB>(event);
                }
                _ => {}
            }
        })
        .unwrap();

    // ----------------------------------------------------------
    // 9. Rozlišení obrazovky
    //    TODO: detekuj automaticky z DRM výstupu
    // ----------------------------------------------------------
    let screen_width = 1920u32;
    let screen_height = 1080u32;

    // ----------------------------------------------------------
    // 10. Vytvoř State
    // ----------------------------------------------------------
    let mut state = State::new(
        display_handle,
        loop_handle.clone(),
        loop_signal,
        seat,
        keyboard,
        pointer,
        compositor_state,
        xdg_shell_state,
        shm_state,
        output_manager_state,
        seat_state,
        config.clone(),
        screen_width,
        screen_height,
    );

    // ----------------------------------------------------------
    // 11. Renderer
    // ----------------------------------------------------------
    let renderer = DERenderer::new(&config, screen_width, screen_height);

    // ----------------------------------------------------------
    // 12. Hlavní smyčka
    // ----------------------------------------------------------
    info!("DE is running!");
    info!("Keybinds:");
    info!("  Super+T       = terminál");
    info!("  Super+Space   = launcher");
    info!("  Super+Q       = zavři okno");
    info!("  Super+M       = maximalizuj");
    info!("  Super+H       = minimalizuj");
    info!("  Super+1-9     = přepni workspace");
    info!("  Super+Escape  = ukončit DE");
    info!("  Ctrl+Alt+F2   = zpět na Mint desktop");

    while state.running {
        // Zpracuj Wayland požadavky od klientů
        display.dispatch_clients(&mut state).unwrap();

        // Aktualizuj stav DE (animace, lišta atd.)
        state.update();

        // Event loop — zpracuj eventy s timeoutem 16ms (60fps)
        event_loop
            .dispatch(Some(Duration::from_millis(16)), &mut state)
            .unwrap();

        // Pošli odpovědi klientům
        display.flush_clients(&mut state).unwrap();

        // TODO: zde by byl DRM render (kreslení na fyzickou obrazovku)
        // Pro plný TTY render je potřeba DrmCompositor + GbmAllocator
        // Viz smithay/examples/drm.rs
    }

    info!("DE stopped. Goodbye!");
}
