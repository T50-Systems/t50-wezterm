impl WaylandWindowInner {

    pub(crate) fn dispatch_pending_event(&mut self) {
        let mut pending;
        {
            let mut pending_events = self.pending_event.lock().unwrap();
            pending = pending_events.clone();
            *pending_events = PendingEvent::default();
        }

        if pending.close {
            self.events.dispatch(WindowEvent::CloseRequested);
        }

        if let Some(window_state) = pending.window_state.take() {
            log::debug!(
                "dispatch_pending_event self.window_state={:?}, pending:{:?}",
                self.window_state,
                window_state
            );
            self.window_state = window_state;
        }

        if pending.configure.is_none() {
            if pending.dpi.is_some() {
                // Synthesize a pending configure event for the dpi change
                pending.configure.replace((
                    self.pixels_to_surface(self.dimensions.pixel_width as i32) as u32,
                    self.pixels_to_surface(self.dimensions.pixel_height as i32) as u32,
                ));
                log::debug!("synthesize configure with {:?}", pending.configure);
            }
        }

        if let Some(ref window_config) = pending.window_configure {
            self.window_frame.update_state(window_config.state);
            self.window_frame
                .update_wm_capabilities(window_config.capabilities);
        }

        if let Some((mut w, mut h)) = pending.configure.take() {
            log::trace!("Pending configure: w:{w}, h{h} -- {:?}", self.window);
            if self.window.is_some() {
                let surface_udata = SurfaceUserData::from_wl(self.surface());
                let factor = surface_udata.surface_data.scale_factor() as f64;
                let old_dimensions = self.dimensions;

                // FIXME: teach this how to resolve dpi_by_screen
                let dpi = self.config.dpi.unwrap_or(factor * crate::DEFAULT_DPI) as usize;

                // Do this early because this affects surface_to_pixels/pixels_to_surface
                self.dimensions.dpi = dpi;

                let mut pixel_width = self.surface_to_pixels(w.try_into().unwrap());
                let mut pixel_height = self.surface_to_pixels(h.try_into().unwrap());

                if self.window_state.can_resize() {
                    self.window_frame.set_resizable(true);
                    if let Some(incr) = self.resize_increments {
                        let min_width = incr.base_width + incr.x;
                        let min_height = incr.base_height + incr.y;
                        let extra_width = (pixel_width - incr.base_width as i32) % incr.x as i32;
                        let extra_height = (pixel_height - incr.base_height as i32) % incr.y as i32;
                        let desired_pixel_width = max(pixel_width - extra_width, min_width as i32);
                        let desired_pixel_height =
                            max(pixel_height - extra_height, min_height as i32);
                        w = self.pixels_to_surface(desired_pixel_width) as u32;
                        h = self.pixels_to_surface(desired_pixel_height) as u32;
                        pixel_width = self.surface_to_pixels(w.try_into().unwrap());
                        pixel_height = self.surface_to_pixels(h.try_into().unwrap());
                    }
                }

                // Align pixel dimensions to the integer buffer scale factor
                // to satisfy the Wayland protocol requirement that buffer
                // dimensions must be an integer multiple of the buffer_scale.
                let scale = factor as i32;
                pixel_width = (pixel_width / scale) * scale;
                pixel_height = (pixel_height / scale) * scale;

                log::trace!("Resizing frame");
                if !self.window_frame.is_hidden() {
                    // Clamp the size to at least one surface heigh/width.
                    let width = NonZeroU32::new(w).unwrap_or(NonZeroU32::new(1).unwrap());
                    let height = NonZeroU32::new(h).unwrap_or(NonZeroU32::new(1).unwrap());
                    self.window_frame.resize(width, height);
                    pending.refresh_decorations = true
                }
                let (x, y) = self.window_frame.location();
                let surface_width = self.pixels_to_surface(pixel_width);
                let surface_height = self.pixels_to_surface(pixel_height);
                self.window
                    .as_mut()
                    .unwrap()
                    .xdg_surface()
                    .set_window_geometry(x, y, surface_width, surface_height);
                // Compute the new pixel dimensions
                let new_dimensions = Dimensions {
                    pixel_width: pixel_width.try_into().unwrap(),
                    pixel_height: pixel_height.try_into().unwrap(),
                    dpi,
                };

                // Only trigger a resize if the new dimensions are different;
                // this makes things more efficient and a little more smooth
                if new_dimensions != old_dimensions {
                    self.dimensions = new_dimensions;

                    self.events.dispatch(WindowEvent::Resized {
                        dimensions: self.dimensions,
                        window_state: self.window_state,
                        // We don't know if we're live resizing or not, so
                        // assume no.
                        live_resizing: false,
                    });
                    // Avoid blurring by matching the scaling factor of the
                    // compositor; if it is going to double the size then
                    // we render at double the size anyway and tell it that
                    // the buffer is already doubled.
                    // Take care to detach the current buffer (managed by EGL),
                    // so that the compositor doesn't get annoyed by it not
                    // having dimensions that match the scale.
                    // The wegl_surface.resize won't take effect until
                    // we paint later on.
                    // We do this only if the scale has actually changed,
                    // otherwise interactive window resize will keep removing
                    // the window contents!
                    if let Some(wegl_surface) = self.wegl_surface.as_mut() {
                        wegl_surface.resize(pixel_width, pixel_height, 0, 0);
                    }
                    if self.surface_factor != factor {
                        let wayland_conn = Connection::get().unwrap().wayland();
                        let wayland_state = wayland_conn.wayland_state.borrow();
                        let mut pool = wayland_state.mem_pool.borrow_mut();

                        // Make a "fake" buffer with the right dimensions, as
                        // simply detaching the buffer can cause wlroots-derived
                        // compositors consider the window to be unconfigured.
                        if let Ok((buffer, _bytes)) = pool.create_buffer(
                            factor as i32,
                            factor as i32,
                            (factor * 4.0) as i32,
                            wayland_client::protocol::wl_shm::Format::Argb8888,
                        ) {
                            self.surface().attach(Some(buffer.wl_buffer()), 0, 0);
                            self.surface().set_buffer_scale(factor as i32);
                            self.surface_factor = factor;
                        }
                    }
                }
                self.do_paint().unwrap();
            }
        }
        if pending.refresh_decorations && self.window.is_some() {
            self.refresh_frame();
        }
        if pending.had_configure_event && self.window.is_some() {
            log::debug!("Had configured an event");
            if let Some(notify) = self.pending_first_configure.take() {
                // Allow window creation to complete
                notify.try_send(()).ok();
            }
        }
}
