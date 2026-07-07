impl XConnection {
impl XConnection {
    pub(crate) fn update_xrm(&self) {
        match read_xsettings(
            &self.conn,
            self.atom_xsettings_selection,
            self.atom_xsettings_settings,
        ) {
            Ok(settings) => {
                if *self.xsettings.borrow() != settings {
                    log::trace!("xsettings changed to {:?}", settings);
                    *self.xsettings.borrow_mut() = settings;
                }
            }
            Err(err) => {
                log::trace!("error reading xsettings: {:#}", err);
            }
        }

        let xrm = crate::x11::xrm::parse_root_resource_manager(&self.conn, self.root)
            .unwrap_or(HashMap::new());
        *self.xrm.borrow_mut() = xrm;

        let dpi = compute_default_dpi(&self.xrm.borrow(), &self.xsettings.borrow());
        *self.default_dpi.borrow_mut() = dpi;
        self.update_net_supported();
    }

    fn update_net_supported(&self) {
        if let Ok(reply) = self.send_and_wait_request(&xcb::x::GetProperty {
            delete: false,
            window: self.root,
            property: self.atom_net_supported,
            r#type: xcb::x::ATOM_ATOM,
            long_offset: 0,
            long_length: 1024,
        }) {
            let supported: HashSet<Atom> = reply.value::<Atom>().iter().copied().collect();
            *self.supported.borrow_mut() = supported;
        }
    }

    pub(crate) fn advise_of_appearance_change(&self, appearance: crate::Appearance) {
        for win in self.windows.borrow().values() {
            win.lock().unwrap().appearance_changed(appearance);
        }
    }

    fn process_queued_xcb(&self) -> anyhow::Result<()> {
        if let Some(event) = self
            .conn
            .poll_for_event()
            .context("X11 connection is broken")?
        {
            if let Err(err) = self.process_xcb_event_ime(&event) {
                return Err(err);
            }
        }
        self.conn.flush().context("flushing pending requests")?;

        loop {
            match self
                .conn
                .poll_for_queued_event()
                .context("poll_for_queued_event")?
            {
                None => {
                    self.conn.flush().context("flushing pending requests")?;
                    return Ok(());
                }
                Some(event) => self
                    .process_xcb_event_ime(&event)
                    .context("process_xcb_event_ime")?,
            }
            self.conn.flush().context("flushing pending requests")?;
        }
    }

    fn process_xcb_event_ime(&self, event: &xcb::Event) -> anyhow::Result<()> {
        // check for previous errors produced by the IME forward_event callback
        self.ime_process_event_result.replace(Ok(()))?;

        if config::configuration().use_ime && self.ime.borrow_mut().process_event(event) {
            self.ime_process_event_result.replace(Ok(()))
        } else {
            self.process_xcb_event(event)
        }
    }

    unsafe fn rewire_event(&self, raw_ev: *mut xcb::ffi::xcb_generic_event_t) {
        let ev_type = ((*raw_ev).response_type & 0x7f) as i32;

        if let Some(func) = xlib::XESetWireToEvent(self.conn.get_raw_dpy(), ev_type, None) {
            xlib::XESetWireToEvent(self.conn.get_raw_dpy(), ev_type, Some(func));
            (*raw_ev).sequence = xlib::XLastKnownRequestProcessed(self.conn.get_raw_dpy()) as u16;
            let mut dummy: xlib::XEvent = std::mem::zeroed();
            func(
                self.conn.get_raw_dpy(),
                &mut dummy as *mut xlib::XEvent,
                raw_ev as *mut xlib::xEvent,
            );
        }
    }

    pub(crate) fn get_cached_screens(&self) -> anyhow::Result<Screens> {
        {
            let screens = self.screens.borrow();
            if let Some(cached) = screens.as_ref() {
                return Ok(cached.clone());
            }
        }

        let screens = self.screens()?;

        self.screens.borrow_mut().replace(screens.clone());

        Ok(screens)
    }

    fn process_xcb_event(&self, event: &xcb::Event) -> anyhow::Result<()> {
        match event {
            // Following stuff is not obvious at all.
            // This was necessary in the past to handle GL when XCB owns the event queue.
            // It may not be necessary anymore, but it is included here
            // because <https://github.com/wezterm/wezterm/issues/1992> is a resize related
            // issue and it might possibly be related to these dri2 related issues:
            // <https://bugs.freedesktop.org/show_bug.cgi?id=35945#c4>
            // and mailing thread starting here:
            // <http://lists.freedesktop.org/archives/xcb/2015-November/010556.html>
            xcb::Event::Dri2(dri2::Event::BufferSwapComplete(ev)) => unsafe {
                self.rewire_event(ev.as_raw())
            },
            xcb::Event::Dri2(dri2::Event::InvalidateBuffers(ev)) => unsafe {
                self.rewire_event(ev.as_raw())
            },
            xcb::Event::RandR(randr) => {
                log::trace!("{randr:?}");
                // Clear our cache
                self.screens.borrow_mut().take();
            }
            _ => {}
        }

        if let Some(window_id) = window_id_from_event(event) {
            self.process_window_event(window_id, event)?;
        } else if matches!(event, xcb::Event::Xkb(_)) {
            // key press/release are not processed here.
            // xkbcommon depends on those events in order to:
            //    - update modifiers state
            //    - update keymap/state on keyboard changes
            if let Some((mods, leds)) = self.keyboard.process_xkb_event(&self.conn, event)? {
                // route changed state to the window with focus
                for window in self.windows.borrow().values() {
                    let mut window = window.lock().unwrap();
                    if window.has_focus == Some(true) {
                        window
                            .events
                            .dispatch(crate::WindowEvent::AdviseModifiersLedStatus(mods, leds));
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn window_by_id(
        &self,
        window_id: xcb::x::Window,
    ) -> Option<Arc<Mutex<XWindowInner>>> {
        self.windows.borrow().get(&window_id).map(Arc::clone)
    }

    fn parent_id_by_child_id(&self, child_id: xcb::x::Window) -> Option<xcb::x::Window> {
        self.child_to_parent_id.borrow().get(&child_id).copied()
    }

    fn dispatch_pending_events(&self) -> anyhow::Result<()> {
        for window in self.windows.borrow().values() {
            let mut inner = window.lock().unwrap();
            inner.dispatch_pending_events()?;
        }

        Ok(())
    }

    fn process_window_event(
        &self,
        window_id: xcb::x::Window,
        event: &xcb::Event,
    ) -> anyhow::Result<()> {
        if let Some(window) = self.window_by_id(window_id) {
            let mut inner = window.lock().unwrap();
            inner.dispatch_event(event)?;
        } else if let Some(parent_id) = self.parent_id_by_child_id(window_id) {
            if let Some(window) = self.window_by_id(parent_id) {
                let mut inner = window.lock().unwrap();
                inner.dispatch_event(event)?;
            }
        }
        Ok(())
    }

    fn intern_atom(conn: &xcb::Connection, name: &str) -> anyhow::Result<Atom> {
        let cookie = conn.send_request(&xcb::x::InternAtom {
            only_if_exists: false,
            name: name.as_bytes(),
        });
        let reply = conn.wait_for_reply(cookie)?;
        Ok(reply.atom())
}
