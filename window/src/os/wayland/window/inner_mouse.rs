impl WaylandWindowInner {
impl WaylandWindowInner {
    fn close(&mut self) {
        self.events.dispatch(WindowEvent::Destroyed);
        self.window.take();
    }

    fn show(&mut self) {
        log::trace!("WaylandWindowInner show: {:?}", self.window);
        if self.window.is_none() {
            return;
        }

        // If the do_paint function has been called previously, calling it again will not
        // send the NeedRepaint event. This results in the window not being displayed
        // correctly.
        // Therefore, when frame_callback is set to some, we need to send the NeedRepaint
        // event again to ensure the window is displayed.
        // Fix: https://github.com/wezterm/wezterm/issues/5103
        if self.frame_callback.is_some() {
            self.events.dispatch(WindowEvent::NeedRepaint);
        }

        self.do_paint().unwrap();
    }

    fn refresh_frame(&mut self) {
        if self.window_frame.is_dirty() && !self.window_frame.is_hidden() {
            self.window_frame.draw();
        }
    }

    fn enable_opengl(&mut self) -> anyhow::Result<Rc<glium::backend::Context>> {
        let wayland_conn = Connection::get().unwrap().wayland();
        let mut wegl_surface = None;

        log::trace!("Enable opengl");

        let gl_state = if !egl_is_available() {
            Err(anyhow!("!egl_is_available"))
        } else {
            let window = self
                .window
                .as_ref()
                .ok_or(anyhow!("Window does not exist"))?;
            let object_id = window.wl_surface().id();

            // Align pixel dimensions to the integer buffer scale factor
            // to satisfy the Wayland protocol requirement that buffer
            // dimensions must be an integer multiple of the buffer_scale.
            let surface_udata = SurfaceUserData::from_wl(window.wl_surface());
            let scale = surface_udata.surface_data.scale_factor();
            let pixel_width = (self.dimensions.pixel_width as i32 / scale) * scale;
            let pixel_height = (self.dimensions.pixel_height as i32 / scale) * scale;

            wegl_surface = Some(WlEglSurface::new(object_id, pixel_width, pixel_height)?);

            log::trace!("WEGL Surface here {:?}", wegl_surface);

            match wayland_conn.gl_connection.borrow().as_ref() {
                Some(glconn) => crate::egl::GlState::create_wayland_with_existing_connection(
                    glconn,
                    wegl_surface.as_ref().unwrap(),
                ),
                None => crate::egl::GlState::create_wayland(
                    Some(wayland_conn.connection.backend().display_ptr() as *const _),
                    wegl_surface.as_ref().unwrap(),
                ),
            }
        };
        let gl_state = gl_state.map(Rc::new).and_then(|state| unsafe {
            wayland_conn
                .gl_connection
                .borrow_mut()
                .replace(Rc::clone(state.get_connection()));
            Ok(glium::backend::Context::new(
                Rc::clone(&state),
                true,
                if cfg!(debug_assertions) {
                    glium::debug::DebugCallbackBehavior::DebugMessageOnError
                } else {
                    glium::debug::DebugCallbackBehavior::Ignore
                },
            )?)
        })?;

        self.gl_state.replace(gl_state.clone());
        self.wegl_surface = wegl_surface;

        Ok(gl_state)
    }

    fn get_dpi_factor(&self) -> f64 {
        self.dimensions.dpi_factor()
    }

    fn surface_to_pixels(&self, surface: i32) -> i32 {
        self.dimensions.surface_to_pixels(surface)
    }

    fn pixels_to_surface(&self, pixels: i32) -> i32 {
        self.dimensions.pixels_to_surface(pixels)
    }

    pub(super) fn dispatch_dropped_files(&mut self, paths: Vec<PathBuf>) {
        self.events.dispatch(WindowEvent::DroppedFile(paths));
    }

    pub(crate) fn dispatch_pending_mouse(&mut self) {
        let pending_mouse = Arc::clone(&self.pending_mouse);

        if let Some((x, y)) = PendingMouse::coords(&pending_mouse) {
            let coords = Point::new(
                self.surface_to_pixels(x as i32) as isize,
                self.surface_to_pixels(y as i32) as isize,
            );
            self.last_mouse_coords = coords;
            let event = MouseEvent {
                kind: MouseEventKind::Move,
                coords,
                screen_coords: ScreenPoint::new(
                    coords.x + self.dimensions.pixel_width as isize,
                    coords.y + self.dimensions.pixel_height as isize,
                ),
                mouse_buttons: self.mouse_buttons,
                modifiers: self.modifiers,
            };
            self.events.dispatch(WindowEvent::MouseEvent(event));
            self.refresh_frame();
        }

        while let Some((button, state)) = PendingMouse::next_button(&pending_mouse) {
            let button_mask = match button {
                MousePress::Left => MouseButtons::LEFT,
                MousePress::Right => MouseButtons::RIGHT,
                MousePress::Middle => MouseButtons::MIDDLE,
            };

            if state == ButtonState::Pressed {
                self.mouse_buttons |= button_mask;
            } else {
                self.mouse_buttons -= button_mask;
            }

            let event = MouseEvent {
                kind: match state {
                    ButtonState::Pressed => MouseEventKind::Press(button),
                    ButtonState::Released => MouseEventKind::Release(button),
                    _ => continue,
                },
                coords: self.last_mouse_coords,
                screen_coords: ScreenPoint::new(
                    self.last_mouse_coords.x + self.dimensions.pixel_width as isize,
                    self.last_mouse_coords.y + self.dimensions.pixel_height as isize,
                ),
                mouse_buttons: self.mouse_buttons,
                modifiers: self.modifiers,
            };
            self.events.dispatch(WindowEvent::MouseEvent(event));
        }

        if let Some((value_x, value_y)) = PendingMouse::scroll(&pending_mouse) {
            let factor = self.get_dpi_factor() as f64;

            if value_x.signum() != self.hscroll_remainder.signum() {
                // reset accumulator when changing scroll direction
                self.hscroll_remainder = 0.0;
            }
            let scaled_x = (value_x * factor) + self.hscroll_remainder;
            let discrete_x = scaled_x.trunc();
            self.hscroll_remainder = scaled_x - discrete_x;
            if discrete_x != 0. {
                let event = MouseEvent {
                    kind: MouseEventKind::HorzWheel(-discrete_x as i16),
                    coords: self.last_mouse_coords,
                    screen_coords: ScreenPoint::new(
                        self.last_mouse_coords.x + self.dimensions.pixel_width as isize,
                        self.last_mouse_coords.y + self.dimensions.pixel_height as isize,
                    ),
                    mouse_buttons: self.mouse_buttons,
                    modifiers: self.modifiers,
                };
                self.events.dispatch(WindowEvent::MouseEvent(event));
            }

            if value_y.signum() != self.vscroll_remainder.signum() {
                self.vscroll_remainder = 0.0;
            }
            let scaled_y = (value_y * factor) + self.vscroll_remainder;
            let discrete_y = scaled_y.trunc();
            self.vscroll_remainder = scaled_y - discrete_y;
            if discrete_y != 0. {
                let event = MouseEvent {
                    kind: MouseEventKind::VertWheel(-discrete_y as i16),
                    coords: self.last_mouse_coords,
                    screen_coords: ScreenPoint::new(
                        self.last_mouse_coords.x + self.dimensions.pixel_width as isize,
                        self.last_mouse_coords.y + self.dimensions.pixel_height as isize,
                    ),
                    mouse_buttons: self.mouse_buttons,
                    modifiers: self.modifiers,
                };
                self.events.dispatch(WindowEvent::MouseEvent(event));
            }
        }

        if !PendingMouse::in_window(&pending_mouse) {
            self.events.dispatch(WindowEvent::MouseLeave);
            self.refresh_frame();
        }
}
