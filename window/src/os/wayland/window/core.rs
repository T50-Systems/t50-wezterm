use std::any::Any;
use std::cell::RefCell;
use std::cmp::max;
use std::convert::TryInto;
use std::io::Read;
use std::num::NonZeroU32;
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail};
use async_io::Timer;
use async_trait::async_trait;
use config::ConfigHandle;
use promise::{Future, Promise};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawWindowHandle,
    WaylandWindowHandle, WindowHandle,
};
use smithay_client_toolkit::compositor::{CompositorHandler, SurfaceData, SurfaceDataExt};
use smithay_client_toolkit::data_device_manager::ReadPipe;
use smithay_client_toolkit::globals::GlobalData;
use smithay_client_toolkit::reexports::csd_frame::{
    DecorationsFrame, FrameAction, ResizeEdge, WindowState as SCTKWindowState,
};
use smithay_client_toolkit::reexports::protocols::xdg::shell::client::xdg_toplevel::ResizeEdge as XdgResizeEdge;
use smithay_client_toolkit::seat::pointer::CursorIcon;
use smithay_client_toolkit::shell::xdg::fallback_frame::FallbackFrame;
use smithay_client_toolkit::shell::xdg::window::{
    DecorationMode, Window as XdgWindow, WindowConfigure, WindowDecorations as Decorations,
    WindowHandler,
};
use smithay_client_toolkit::shell::xdg::XdgSurface;
use smithay_client_toolkit::shell::WaylandSurface;
use wayland_client::protocol::wl_callback::WlCallback;
use wayland_client::protocol::wl_keyboard::{Event as WlKeyboardEvent, KeyState};
use wayland_client::protocol::wl_pointer::{ButtonState, WlPointer};
use wayland_client::protocol::wl_region::WlRegion;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection as WConnection, Dispatch, Proxy, QueueHandle};
use wayland_egl::{is_available as egl_is_available, WlEglSurface};
use wayland_protocols_plasma::blur::client::org_kde_kwin_blur::OrgKdeKwinBlur;
use wayland_protocols_plasma::blur::client::org_kde_kwin_blur_manager::OrgKdeKwinBlurManager;
use wezterm_font::FontConfiguration;
use wezterm_input_types::{
    KeyboardLedStatus, Modifiers, MouseButtons, MouseEvent, MouseEventKind, MousePress,
    ScreenPoint, WindowDecorations,
};

use crate::wayland::WaylandConnection;
use crate::x11::KeyboardWithFallback;
use crate::{
    Appearance, Clipboard, Connection, ConnectionOps, Dimensions, MouseCursor, Point, Rect,
    RequestedWindowGeometry, ResizeIncrement, ResolvedGeometry, Window, WindowEvent,
    WindowEventSender, WindowKeyEvent, WindowOps, WindowState,
};

/// Wayland-specific coordinate conversion methods for Dimensions
trait WaylandDimensions {
    fn dpi_factor(&self) -> f64;
    fn pixels_to_surface(&self, pixels: i32) -> i32;
    fn surface_to_pixels(&self, surface: i32) -> i32;
}

impl WaylandDimensions for Dimensions {
    fn dpi_factor(&self) -> f64 {
        self.dpi as f64 / crate::DEFAULT_DPI as f64
    }

    fn pixels_to_surface(&self, pixels: i32) -> i32 {
        // Take care to round up, otherwise we can lose a pixel
        // and that can effectively lose the final row of the terminal
        (pixels as f64 / self.dpi_factor()).ceil() as i32
    }

    fn surface_to_pixels(&self, surface: i32) -> i32 {
        (surface as f64 * self.dpi_factor()).ceil() as i32
    }
}

use super::pointer::{PendingMouse, PointerUserData};
use super::state::WaylandState;

#[derive(Debug)]
pub(super) struct KeyRepeatState {
    pub(super) when: Instant,
    pub(super) event: WindowKeyEvent,
}

impl KeyRepeatState {
    pub(super) fn schedule(state: Arc<Mutex<Self>>, window_id: usize) {
        promise::spawn::spawn_into_main_thread(async move {
            let delay;
            let gap;
            {
                let conn = WaylandConnection::get().unwrap().wayland();
                let (rate, ddelay) = {
                    let wstate = conn.wayland_state.borrow();
                    (
                        wstate.key_repeat_rate as u64,
                        wstate.key_repeat_delay as u64,
                    )
                };
                if rate == 0 {
                    return;
                }
                delay = Duration::from_millis(ddelay);
                gap = Duration::from_millis(1000 / rate);
            }

            let mut initial = true;
            Timer::after(delay).await;
            loop {
                {
                    let handle = {
                        let conn = WaylandConnection::get().unwrap().wayland();
                        match conn.window_by_id(window_id) {
                            Some(handle) => handle,
                            None => return,
                        }
                    };

                    let mut inner = handle.borrow_mut();

                    if inner.key_repeat.as_ref().map(|(_, k)| Arc::as_ptr(k))
                        != Some(Arc::as_ptr(&state))
                    {
                        // Key was released and/or some other key is doing
                        // its own repetition now
                        return;
                    }

                    let mut st = state.lock().unwrap();

                    let mut repeat_count = 1;

                    let mut elapsed = st.when.elapsed();
                    if initial {
                        elapsed -= delay;
                        initial = false;
                    }

                    // If our scheduling interval is longer than the repeat
                    // gap, we need to inflate the repeat count to match
                    // the intended rate
                    while elapsed >= gap {
                        repeat_count += 1;
                        elapsed -= gap;
                    }

                    let event = match st.event.clone() {
                        WindowKeyEvent::KeyEvent(mut key) => {
                            key.repeat_count = repeat_count;
                            WindowEvent::KeyEvent(key)
                        }
                        WindowKeyEvent::RawKeyEvent(mut raw) => {
                            raw.repeat_count = repeat_count;
                            WindowEvent::RawKeyEvent(raw)
                        }
                    };

                    inner.events.dispatch(event);

                    st.when = Instant::now();
                }

                Timer::after(gap).await;
            }
        })
        .detach();
    }
}

enum WaylandWindowEvent {
    Close,
    Request(WindowConfigure),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct WaylandWindow(usize);

impl WaylandWindow {
    pub async fn new_window<F>(
        class_name: &str,
        name: &str,
        geometry: RequestedWindowGeometry,
        config: Option<&ConfigHandle>,
        _font_config: Rc<FontConfiguration>,
        event_handler: F,
    ) -> anyhow::Result<Window>
    where
        F: 'static + FnMut(WindowEvent, &Window),
    {
        let config = match config {
            Some(c) => c.clone(),
            None => config::configuration(),
        };

        let conn = WaylandConnection::get()
            .ok_or_else(|| {
                anyhow!(
                    "new_window must be called on the gui thread after Connection:init has succeed",
                )
            })?
            .wayland();

        let window_id = conn.next_window_id();
        let pending_event = Arc::new(Mutex::new(PendingEvent::default()));

        let (pending_first_configure, wait_configure) = async_channel::bounded(1);

        let qh = conn.event_queue.borrow().handle();

        // We need user data so we can get the window_id => WaylandWindowInner during a handler
        let surface_data = SurfaceUserData {
            surface_data: SurfaceData::default(),
            window_id,
        };
        let surface = {
            let compositor = &conn.wayland_state.borrow().compositor;
            compositor.create_surface_with_data(&qh, surface_data)
        };

        let ResolvedGeometry {
            x: _,
            y: _,
            width,
            height,
        } = conn.resolve_geometry(geometry);

        let dimensions = Dimensions {
            pixel_width: width,
            pixel_height: height,
            dpi: config.dpi.unwrap_or(crate::DEFAULT_DPI) as usize,
        };

        let window = {
            let xdg_shell = &conn.wayland_state.borrow().xdg;
            xdg_shell.create_window(surface.clone(), Decorations::RequestServer, &qh)
        };

        window.set_app_id(class_name.to_string());
        window.set_title(name.to_string());
        let decorations = config.window_decorations;

        let decor_mode = if decorations == WindowDecorations::NONE {
            None
        } else if decorations == WindowDecorations::default() {
            Some(DecorationMode::Server)
        } else {
            Some(DecorationMode::Client)
        };
        window.request_decoration_mode(decor_mode);

        let mut window_frame = {
            let wayland_state = &conn.wayland_state.borrow();
            let shm = &wayland_state.shm;
            let subcompositor = wayland_state.subcompositor.clone();
            FallbackFrame::new(&window, shm, subcompositor, qh.clone())
                .expect("failed to create csd frame")
        };
        let hidden = match decor_mode {
            Some(DecorationMode::Client) => false,
            _ => true,
        };
        window_frame.set_hidden(hidden);
        if !hidden {
            window_frame.resize(
                NonZeroU32::new(dimensions.pixel_width as u32)
                    .ok_or_else(|| anyhow!("dimensions {dimensions:?} are invalid"))?,
                NonZeroU32::new(dimensions.pixel_height as u32)
                    .ok_or_else(|| anyhow!("dimensions {dimensions:?} are invalid"))?,
            );
        }

        window.set_min_size(Some((32, 32)));
        let (x, y) = window_frame.location();
        let surface_width = dimensions.pixels_to_surface(dimensions.pixel_width as i32);
        let surface_height = dimensions.pixels_to_surface(dimensions.pixel_height as i32);
        window
            .xdg_surface()
            .set_window_geometry(x, y, surface_width, surface_height);
        window.commit();

        let pending_mouse = PendingMouse::create(window_id);

        {
            let surface_to_pending = &mut conn.wayland_state.borrow_mut().surface_to_pending;
            surface_to_pending.insert(surface.id(), Arc::clone(&pending_mouse));
        }

        let appearance = conn.get_appearance();

        let inner = Rc::new(RefCell::new(WaylandWindowInner {
            events: WindowEventSender::new(event_handler),
            surface_factor: 1.0,
            invalidated: false,
            window: Some(window),
            window_frame,
            dimensions,
            resize_increments: None,
            window_state: WindowState::default(),
            last_mouse_coords: Point::new(0, 0),
            mouse_buttons: MouseButtons::NONE,
            hscroll_remainder: 0.0,
            vscroll_remainder: 0.0,

            modifiers: Modifiers::NONE,
            leds: KeyboardLedStatus::empty(),

            key_repeat: None,
            pending_event,
            pending_mouse,

            pending_first_configure: Some(pending_first_configure),
            frame_callback: None,

            text_cursor: None,
            appearance,

            config,

            title: None,

            wegl_surface: None,
            gl_state: None,
        }));

        let window_handle = Window::Wayland(WaylandWindow(window_id));

        inner
            .borrow_mut()
            .events
            .assign_window(window_handle.clone());

        inner.borrow().update_window_background_blur();

        {
            let windows = &conn.wayland_state.borrow().windows;
            windows.borrow_mut().insert(window_id, inner.clone());
        };

        wait_configure.recv().await?;

        Ok(window_handle)
    }
