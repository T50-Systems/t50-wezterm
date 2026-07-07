impl WaylandWindowInner {

    fn set_cursor(&mut self, cursor: Option<MouseCursor>) {
        if !PendingMouse::in_window(&self.pending_mouse) {
            return;
        }

        let conn = Connection::get().unwrap().wayland();
        let state = conn.wayland_state.borrow_mut();
        let pointer = match &state.pointer {
            Some(pointer) => pointer,
            None => return,
        };

        match cursor {
            Some(cursor) => {
                if let Err(err) = pointer.set_cursor(
                    &conn.connection,
                    match cursor {
                        MouseCursor::Arrow => CursorIcon::Default,
                        MouseCursor::Hand => CursorIcon::Pointer,
                        MouseCursor::SizeUpDown => CursorIcon::NsResize,
                        MouseCursor::SizeLeftRight => CursorIcon::EwResize,
                        MouseCursor::Text => CursorIcon::Text,
                    },
                ) {
                    log::error!("set_cursor: {}", err);
                }
            }
            None => {
                if let Err(err) = pointer.hide_cursor() {
                    log::error!("hide_cursor: {}", err)
                }
            }
        }
    }

    fn invalidate(&mut self) {
        if self.frame_callback.is_some() {
            self.invalidated = true;
            return;
        }
        self.do_paint().unwrap();
    }

    fn set_text_cursor_position(&mut self, rect: Rect) {
        let conn = WaylandConnection::get().unwrap().wayland();
        let state = conn.wayland_state.borrow();
        let surface = self.surface().clone();
        let active_surface_id = state.active_surface_id.borrow();
        let surface_id = surface.id();

        if let Some(active_surface_id) = active_surface_id.as_ref() {
            if surface_id == active_surface_id.clone() {
                if self.text_cursor.map(|prior| prior != rect).unwrap_or(true) {
                    self.text_cursor.replace(rect);

                    let surface_udata = SurfaceUserData::from_wl(&surface);
                    let factor = surface_udata.surface_data().scale_factor();

                    if let Some(text_input) = &state.text_input {
                        if let Some(input) = text_input.get_text_input_for_surface(&surface) {
                            input.set_cursor_rectangle(
                                rect.min_x() as i32 / factor,
                                rect.min_y() as i32 / factor,
                                rect.width() as i32 / factor,
                                rect.height() as i32 / factor,
                            );
                            input.commit();
                        }
                    }
                }
            }
        }
    }

    fn set_title(&mut self, title: String) {
        if let Some(last_title) = self.title.as_ref() {
            if last_title == &title {
                return;
            }
        }
        if let Some(window) = self.window.as_ref() {
            window.set_title(title.clone());
        }
        self.refresh_frame();
        self.title = Some(title);
    }

    fn set_resize_increments(&mut self, incr: ResizeIncrement) -> anyhow::Result<()> {
        self.resize_increments.replace(incr);
        Ok(())
    }

    fn set_inner_size(&mut self, width: usize, height: usize) {
        let pixel_width = width as i32;
        let pixel_height = height as i32;
        let surface_width = self.pixels_to_surface(pixel_width) as u32;
        let surface_height = self.pixels_to_surface(pixel_height) as u32;
        // window.resize() doesn't generate a configure event,
        // so we're going to fake one up, otherwise the window
        // contents don't reflect the real size until eg:
        // the focus is changed.
        self.pending_event
            .lock()
            .unwrap()
            .configure
            .replace((surface_width, surface_height));
        // apply the synthetic configure event to the inner surfaces
        self.dispatch_pending_event();

        self.events.dispatch(WindowEvent::SetInnerSizeCompleted);
    }

    fn do_paint(&mut self) -> anyhow::Result<()> {
        if self.window.is_none() {
            // We're likely in the middle of closing/destroying
            // the window; we've nothing to do here.
            return Ok(());
        }

        if self.frame_callback.is_some() {
            // Painting now won't be productive, so skip it but
            // remember that we need to be painted so that when
            // the compositor is ready for us, we can paint then.
            self.invalidated = true;
            return Ok(());
        }

        self.invalidated = false;

        // Ask the compositor to wake us up when its time to paint the next frame,
        // note that this only happens _after_ the next commit
        let conn = WaylandConnection::get().unwrap().wayland();
        let qh = conn.event_queue.borrow().handle();

        let callback = self.surface().frame(&qh, self.surface().clone());

        log::trace!("do_paint - callback: {:?}", callback);
        self.frame_callback.replace(callback);

        // The repaint has the side of effect of committing the surface,
        // which is necessary for the frame callback to get triggered.
        // Ordering the repaint after requesting the callback ensures that
        // we will get woken at the appropriate time.
        // <https://github.com/wezterm/wezterm/issues/3468>
        // <https://github.com/wezterm/wezterm/issues/3126>
        self.events.dispatch(WindowEvent::NeedRepaint);

        Ok(())
    }

    fn surface(&self) -> &WlSurface {
        self.window
            .as_ref()
            .expect("Window should exist")
            .wl_surface()
    }

    pub(crate) fn next_frame_is_ready(&mut self) {
        self.frame_callback.take();
        if self.invalidated {
            self.do_paint().ok();
        }
    }

    pub(crate) fn emit_focus(&mut self, mapper: &mut KeyboardWithFallback, focused: bool) {
        // Clear the modifiers when we change focus, otherwise weird
        // things can happen.  For instance, if we lost focus because
        // CTRL+SHIFT+N was pressed to spawn a new window, we'd be
        // left stuck with CTRL+SHIFT held down and the window would
        // be left in a broken state.

        self.modifiers = Modifiers::NONE;
        mapper.update_modifier_state(0, 0, 0, 0);
        self.key_repeat.take();
        self.events.dispatch(WindowEvent::FocusChanged(focused));
        self.text_cursor.take();
    }

    pub(crate) fn appearance_changed(&mut self, appearance: Appearance) {
        if appearance != self.appearance {
            self.appearance = appearance;
            self.events
                .dispatch(WindowEvent::AppearanceChanged(appearance));
        }
    }

    pub(super) fn keyboard_event(
        &mut self,
        mapper: &mut KeyboardWithFallback,
        event: WlKeyboardEvent,
    ) {
        match event {
            WlKeyboardEvent::Enter { keys, .. } => {
                let key_codes = keys
                    .chunks_exact(4)
                    .map(|c| u32::from_ne_bytes(c.try_into().unwrap()))
                    .collect::<Vec<_>>();
                log::trace!("keyboard event: Enter with keys: {:?}", key_codes);
                self.emit_focus(mapper, true);
            }
            WlKeyboardEvent::Leave { .. } => {
                self.emit_focus(mapper, false);
            }
            WlKeyboardEvent::Key { key, state, .. } => {
                if let Some(event) = mapper.process_wayland_key(
                    key,
                    state.into_result().unwrap() == KeyState::Pressed,
                    &mut self.events,
                ) {
                    let rep = Arc::new(Mutex::new(KeyRepeatState {
                        when: Instant::now(),
                        event,
                    }));
                    self.key_repeat.replace((key, Arc::clone(&rep)));
                    let window_id = SurfaceUserData::from_wl(
                        self.window
                            .as_ref()
                            .expect("window should exist")
                            .wl_surface(),
                    )
                    .window_id;
                    KeyRepeatState::schedule(rep, window_id);
                } else if let Some((cur_key, _)) = self.key_repeat.as_ref() {
                    // important to check that it's the same key, because the release of the previously
                    // repeated key can come right after the press of the newly held key
                    if *cur_key == key {
                        self.key_repeat.take();
                    }
                }
            }
            WlKeyboardEvent::Modifiers {
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
                ..
            } => {
                mapper.update_modifier_state(mods_depressed, mods_latched, mods_locked, group);

                let mods = mapper.get_key_modifiers();
                let leds = mapper.get_led_status();

                let changed = (mods != self.modifiers) || (leds != self.leds);

                self.modifiers = mapper.get_key_modifiers();
                self.leds = mapper.get_led_status();

                if changed {
                    self.events
                        .dispatch(WindowEvent::AdviseModifiersLedStatus(mods, leds));
                }
            }
            _ => {}
        }
    }

    pub(super) fn frame_action(&mut self, pointer: &WlPointer, serial: u32, action: FrameAction) {
        let pointer_data = pointer.data::<PointerUserData>().unwrap();
        let seat = pointer_data.pdata.seat();
        match action {
            FrameAction::Close => self.events.dispatch(WindowEvent::CloseRequested),
            FrameAction::Minimize => self.window.as_ref().unwrap().set_minimized(),
            FrameAction::Maximize => self.window.as_ref().unwrap().set_maximized(),
            FrameAction::UnMaximize => self.window.as_ref().unwrap().unset_maximized(),
            FrameAction::ShowMenu(x, y) => {
                self.window
                    .as_ref()
                    .unwrap()
                    .show_window_menu(seat, serial, (x, y))
            }
            FrameAction::Resize(edge) => {
                let edge = match edge {
                    ResizeEdge::None => XdgResizeEdge::None,
                    ResizeEdge::Top => XdgResizeEdge::Top,
                    ResizeEdge::Bottom => XdgResizeEdge::Bottom,
                    ResizeEdge::Left => XdgResizeEdge::Left,
                    ResizeEdge::TopLeft => XdgResizeEdge::TopLeft,
                    ResizeEdge::BottomLeft => XdgResizeEdge::BottomLeft,
                    ResizeEdge::Right => XdgResizeEdge::Right,
                    ResizeEdge::TopRight => XdgResizeEdge::TopRight,
                    ResizeEdge::BottomRight => XdgResizeEdge::BottomRight,
                    _ => return, // Realistically, there probably won't be any new edges added.
                };
                self.window.as_ref().unwrap().resize(seat, serial, edge)
            }
            FrameAction::Move => self.window.as_ref().unwrap().move_(seat, serial),
            _ => log::warn!("unhandled FrameAction: {:?}", action),
        }
    }

    fn maximize(&mut self) {
        if let Some(window) = self.window.as_mut() {
            window.set_maximized();
        }
    }

    fn restore(&mut self) {
        if let Some(window) = self.window.as_mut() {
            window.unset_maximized();
        }
    }

    fn config_did_change(&mut self, config: ConfigHandle) {
        self.config = config;
        self.update_window_background_blur();
    }

    fn update_window_background_blur(&self) {
        let conn = WaylandConnection::get().unwrap().wayland();
        let qh = conn.event_queue.borrow().handle();
        let wayland_state = conn.wayland_state.borrow();
        if let Some(manager) = &wayland_state.kde_blur_manager {
            let kde_blur = manager.create(self.surface(), &qh, GlobalData);
            if self.config.kde_window_background_blur {
                kde_blur.set_region(None);
            } else {
                kde_blur.release();
            }
            kde_blur.commit();
        }
}
