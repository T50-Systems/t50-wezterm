
impl WaylandState {
    pub(super) fn window_by_id(&self, window_id: usize) -> Option<Rc<RefCell<WaylandWindowInner>>> {
        self.windows.borrow().get(&window_id).map(Rc::clone)
    }

    fn handle_window_event(&self, window: &XdgWindow, event: WaylandWindowEvent) {
        let surface_data = SurfaceUserData::from_wl(window.wl_surface());
        let window_id = surface_data.window_id;

        let window_inner = self
            .window_by_id(window_id)
            .expect("Inner Window should exist");

        let p = window_inner.borrow().pending_event.clone();
        let mut pending_event = p.lock().unwrap();

        let changed = match event {
            WaylandWindowEvent::Close => {
                // TODO: This should the new queue function
                // p.queue_close()
                if !pending_event.close {
                    pending_event.close = true;
                    true
                } else {
                    false
                }
            }
            WaylandWindowEvent::Request(configure) => {
                pending_event.window_configure.replace(configure.clone());
                // TODO: This should the new queue function
                // p.queue_configure(&configure)
                //
                let mut changed;
                pending_event.had_configure_event = true;
                if let (Some(w), Some(h)) = configure.new_size {
                    changed = pending_event.configure.is_none();
                    pending_event.configure.replace((w.get(), h.get()));
                } else {
                    changed = true;
                }

                let mut state = WindowState::default();
                if configure.state.contains(SCTKWindowState::FULLSCREEN) {
                    state |= WindowState::FULL_SCREEN;
                }
                if configure.state.contains(SCTKWindowState::MAXIMIZED) {
                    state |= WindowState::MAXIMIZED;
                }

                log::debug!(
                    "Config: self.window_state={:?}, states: {:?} {:?}",
                    pending_event.window_state,
                    state,
                    configure.state
                );

                if pending_event.window_state.is_none() && state != WindowState::default() {
                    changed = true;
                }

                pending_event.window_state.replace(state);
                changed
            }
        };
        if changed {
            WaylandConnection::with_window_inner(window_id, move |inner| {
                inner.dispatch_pending_event();
                Ok(())
            });
        }
    }
}

impl CompositorHandler for WaylandState {
    fn scale_factor_changed(
        &mut self,
        _conn: &WConnection,
        _qh: &wayland_client::QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        // We do nothing, we get the scale_factor from surface_data
    }

    fn frame(
        &mut self,
        _conn: &WConnection,
        _qh: &wayland_client::QueueHandle<Self>,
        surface: &wayland_client::protocol::wl_surface::WlSurface,
        _time: u32,
    ) {
        log::trace!("frame: CompositorHandler");
        let surface_data = SurfaceUserData::from_wl(surface);
        let window_id = surface_data.window_id;

        WaylandConnection::with_window_inner(window_id, |inner| {
            inner.next_frame_is_ready();
            Ok(())
        });
    }

    fn transform_changed(
        &mut self,
        _conn: &WConnection,
        _qh: &wayland_client::QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _new_transform: wayland_client::protocol::wl_output::Transform,
    ) {
        // TODO: do we need to do anything here?
    }

    fn surface_enter(
        &mut self,
        _conn: &WConnection,
        _qh: &wayland_client::QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _output: &wayland_client::protocol::wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &WConnection,
        _qh: &wayland_client::QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _output: &wayland_client::protocol::wl_output::WlOutput,
    ) {
    }
}

impl WindowHandler for WaylandState {
    fn request_close(
        &mut self,
        _conn: &WConnection,
        _qh: &wayland_client::QueueHandle<Self>,
        window: &XdgWindow,
    ) {
        self.handle_window_event(window, WaylandWindowEvent::Close);
    }

    fn configure(
        &mut self,
        _conn: &WConnection,
        _qh: &wayland_client::QueueHandle<Self>,
        window: &XdgWindow,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        self.handle_window_event(window, WaylandWindowEvent::Request(configure));
    }
}

impl Dispatch<OrgKdeKwinBlurManager, GlobalData> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &OrgKdeKwinBlurManager,
        _event: <OrgKdeKwinBlurManager as Proxy>::Event,
        _data: &GlobalData,
        _conn: &WConnection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        // No events from OrgKdeKwinBlurManager...
    }
}

impl Dispatch<OrgKdeKwinBlur, GlobalData> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &OrgKdeKwinBlur,
        _event: <OrgKdeKwinBlur as Proxy>::Event,
        _data: &GlobalData,
        _conn: &WConnection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        // No events from OrgKdeKwinBlur...
    }
}

impl Dispatch<WlRegion, GlobalData> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlRegion,
        _event: <WlRegion as Proxy>::Event,
        _data: &GlobalData,
        _conn: &WConnection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

pub(super) struct SurfaceUserData {
    surface_data: SurfaceData,
    pub(super) window_id: usize,
}

impl SurfaceUserData {
    pub(super) fn from_wl(wl: &WlSurface) -> &Self {
        wl.data()
            .expect("User data should be associated with WlSurface")
    }
    pub(super) fn try_from_wl(wl: &WlSurface) -> Option<&SurfaceUserData> {
        wl.data()
    }
}

impl SurfaceDataExt for SurfaceUserData {
    fn surface_data(&self) -> &SurfaceData {
        &self.surface_data
    }
}

impl HasDisplayHandle for WaylandWindowInner {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let conn = WaylandConnection::get().unwrap().wayland();
        let backend = conn.connection.backend();
        let handle = backend.display_handle()?;
        Ok(unsafe { DisplayHandle::borrow_raw(handle.as_raw()) })
    }
}

impl HasWindowHandle for WaylandWindowInner {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let handle = WaylandWindowHandle::new(
            NonNull::new(self.surface().id().as_ptr() as _).expect("non-null"),
        );
        unsafe { Ok(WindowHandle::borrow_raw(RawWindowHandle::Wayland(handle))) }
    }
}

impl HasDisplayHandle for WaylandWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let conn = WaylandConnection::get().unwrap().wayland();
        let backend = conn.connection.backend();
        let handle = backend.display_handle()?;
        Ok(unsafe { DisplayHandle::borrow_raw(handle.as_raw()) })
    }
}

impl HasWindowHandle for WaylandWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let conn = Connection::get().expect("raw_window_handle only callable on main thread");
        let handle = conn
            .wayland()
            .window_by_id(self.0)
            .expect("window handle invalid!?");

        let inner = handle.borrow();
        let handle = inner.window_handle()?;
        unsafe { Ok(WindowHandle::borrow_raw(handle.as_raw())) }
    }
