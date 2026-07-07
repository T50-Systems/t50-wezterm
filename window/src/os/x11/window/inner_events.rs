fn configure_notify(&mut self, source: &str, width: u16, height: u16) -> anyhow::Result<()> {
    let conn = self.conn();

    self.update_ime_position();

    let mut dpi = conn.default_dpi();

    if !self.config.dpi_by_screen.is_empty() {
        let coords = conn
            .send_and_wait_request(&xcb::x::TranslateCoordinates {
                src_window: self.window_id,
                dst_window: conn.root,
                src_x: 0,
                src_y: 0,
            })
            .context("querying window coordinates")?;
        let screens = conn.get_cached_screens()?;
        let window_rect: ScreenRect = euclid::rect(
            coords.dst_x().into(),
            coords.dst_y().into(),
            width as isize,
            height as isize,
        );
        let screen = screens
            .by_name
            .values()
            .filter_map(|screen| {
                screen
                    .rect
                    .intersection(&window_rect)
                    .map(|r| (screen, r.area()))
            })
            .max_by_key(|s| s.1)
            .ok_or_else(|| anyhow::anyhow!("window is not in any screen"))?
            .0;

        if let Some(value) = self.config.dpi_by_screen.get(&screen.name).copied() {
            dpi = value;
        } else if let Some(value) = self.config.dpi {
            dpi = value;
        }
    }

    if width == self.width && height == self.height && dpi == self.dpi {
        // Effectively unchanged; perhaps it was simply moved?
        // Do nothing!
        log::trace!(
            "Ignoring {source} ({width}x{height} dpi={dpi}) \
                                 because width,height,dpi are unchanged",
        );
        return Ok(());
    }

    self.resize_child(width as u32, height as u32);

    log::trace!(
        "{source}: width {} -> {}, height {} -> {}, dpi {} -> {}",
        self.width,
        width,
        self.height,
        height,
        self.dpi,
        dpi
    );

    self.width = width;
    self.height = height;
    self.dpi = dpi;
    self.last_wm_state = self.get_window_state().unwrap_or(WindowState::default());

    let dimensions = Dimensions {
        pixel_width: self.width as usize,
        pixel_height: self.height as usize,
        dpi: self.dpi as usize,
    };

    self.queue_pending(WindowEvent::Resized {
        dimensions,
        window_state: self.last_wm_state,
        // Assume that we're live resizing: we don't know for sure,
        // but it seems like a reasonable assumption
        live_resizing: true,
    });
    Ok(())
}

fn xdnd_event(&mut self, msgtype: Atom, data: &[u32]) -> anyhow::Result<()> {
    use xcb::XidNew;
    let conn = self.conn();
    let msgtype_name = conn.atom_name(msgtype);
    let srcwin = unsafe { xcb::x::Window::new(data[0]) };
    if msgtype == conn.atom_xdndenter {
        self.drag_and_drop.src_window = Some(srcwin);
        let moretypes = data[1] & 0x01 != 0;
        let xdndversion = data[1] >> 24 as u8;
        log::trace!(
            "ClientMessage {msgtype_name}, Version {xdndversion}, more than 3 types: {moretypes}"
        );
        if !moretypes {
            self.drag_and_drop.src_types = data[2..]
                .into_iter()
                .filter(|&&x| x != 0)
                .map(|&x| unsafe { Atom::new(x) })
                .collect();
        } else {
            self.drag_and_drop.src_types = match conn.send_and_wait_request(&xcb::x::GetProperty {
                delete: false,
                window: srcwin,
                property: conn.atom_xdndtypelist,
                r#type: xcb::x::ATOM_ATOM,
                long_offset: 0,
                long_length: u32::max_value(),
            }) {
                Ok(prop) => prop.value::<Atom>().to_vec(),
                Err(err) => {
                    log::error!(
                        "xdnd: unable to get type list from source window: {:?}",
                        err
                    );
                    Vec::<Atom>::new()
                }
            };
        }
        self.drag_and_drop.target_type = xcb::x::ATOM_NONE;
        for t in [
            conn.atom_texturilist,
            conn.atom_xmozurl,
            conn.atom_utf8_string,
        ] {
            if self.drag_and_drop.src_types.contains(&t) {
                self.drag_and_drop.target_type = t;
                break;
            }
        }
        for t in &self.drag_and_drop.src_types {
            log::trace!("types offered: {}", conn.atom_name(*t));
        }
        log::trace!(
            "selected: {}",
            conn.atom_name(self.drag_and_drop.target_type)
        );
    } else if self.drag_and_drop.src_window != Some(srcwin) {
        log::error!("ClientMessage {msgtype_name} received, but no Xdnd in progress or source window mismatch");
    } else if msgtype == conn.atom_xdndposition {
        self.drag_and_drop.time = data[3];
        let (x, y) = (data[2] >> 16 as u16, data[2] as u16);
        self.drag_and_drop.src_action = unsafe { Atom::new(data[4]) };
        self.drag_and_drop.target_action = conn.atom_xdndactioncopy;
        log::trace!(
            "ClientMessage {msgtype_name}, ({x}, {y}), timestamp: {}, action: {}",
            self.drag_and_drop.time,
            conn.atom_name(self.drag_and_drop.src_action)
        );
        conn.send_request_no_reply_log(&xcb::x::SendEvent {
            propagate: false,
            destination: xcb::x::SendEventDest::Window(srcwin),
            event_mask: xcb::x::EventMask::empty(),
            event: &xcb::x::ClientMessageEvent::new(
                srcwin,
                conn.atom_xdndstatus,
                xcb::x::ClientMessageData::Data32([
                    self.window_id.resource_id(),
                    2 | (self.drag_and_drop.target_type != xcb::x::ATOM_NONE) as u32,
                    0,
                    0,
                    self.drag_and_drop.target_action.resource_id(),
                ]),
            ),
        });
    } else if msgtype == conn.atom_xdndleave {
        self.drag_and_drop.src_window = None;
        log::trace!("ClientMessage {msgtype_name}");
    } else if msgtype == conn.atom_xdnddrop {
        self.drag_and_drop.time = data[2];
        log::trace!(
            "ClientMessage {msgtype_name}, timestamp: {}",
            self.drag_and_drop.time
        );
        if self.drag_and_drop.target_type != xcb::x::ATOM_NONE {
            conn.send_request_no_reply_log(&xcb::x::ConvertSelection {
                requestor: self.window_id,
                selection: conn.atom_xdndselection,
                target: self.drag_and_drop.target_type,
                property: conn.atom_xsel_data,
                time: self.drag_and_drop.time,
            });
        } else {
            log::warn!("XdndDrop received, but no target type selected. Ignoring.");
            conn.send_request_no_reply_log(&xcb::x::SendEvent {
                propagate: false,
                destination: xcb::x::SendEventDest::Window(srcwin),
                event_mask: xcb::x::EventMask::empty(),
                event: &xcb::x::ClientMessageEvent::new(
                    srcwin,
                    conn.atom_xdndfinished,
                    xcb::x::ClientMessageData::Data32([self.window_id.resource_id(), 0, 0, 0, 0]),
                ),
            });
        }
    }
    return Ok(());
}

pub fn dispatch_event(&mut self, event: &Event) -> anyhow::Result<()> {
    let conn = self.conn();
    match event {
        Event::X(xcb::x::Event::Expose(expose)) => {
            self.expose(
                expose.x(),
                expose.y(),
                expose.width(),
                expose.height(),
                expose.count(),
            );
        }
        Event::Present(xcb::present::Event::ConfigureNotify(cfg)) => {
            self.configure_notify("Present::ConfigureNotify", cfg.width(), cfg.height())?;
        }
        Event::X(xcb::x::Event::ConfigureNotify(cfg)) => {
            self.configure_notify("X::ConfigureNotify", cfg.width(), cfg.height())?;
            if self.outstanding_configure_requests > 0 {
                self.outstanding_configure_requests -= 1;
                self.pending_finished_resizes += 1;
            }
        }
        Event::X(xcb::x::Event::KeyPress(key_press)) => {
            self.copy_and_paste.time = key_press.time();
            conn.keyboard
                .process_key_press_event(key_press, &mut self.events);
        }
        Event::X(xcb::x::Event::KeyRelease(key_release)) => {
            self.copy_and_paste.time = key_release.time();
            conn.keyboard
                .process_key_release_event(key_release, &mut self.events);
        }
        Event::X(xcb::x::Event::MotionNotify(motion)) => {
            let event = MouseEvent {
                kind: MouseEventKind::Move,
                coords: Point::new(
                    motion.event_x().try_into().unwrap(),
                    motion.event_y().try_into().unwrap(),
                ),
                screen_coords: ScreenPoint::new(
                    motion.root_x().try_into().unwrap(),
                    motion.root_y().try_into().unwrap(),
                ),
                modifiers: xkeysyms::modifiers_from_state(motion.state().bits()),
                mouse_buttons: MouseButtons::default(),
            };
            self.do_mouse_event(event)?;
        }
        Event::X(xcb::x::Event::ButtonPress(e)) => {
            self.button_event(
                true,
                e.time(),
                e.detail(),
                e.event_x(),
                e.event_y(),
                e.root_x(),
                e.root_y(),
                e.state(),
            )?;
        }
        Event::X(xcb::x::Event::ButtonRelease(e)) => {
            self.button_event(
                false,
                e.time(),
                e.detail(),
                e.event_x(),
                e.event_y(),
                e.root_x(),
                e.root_y(),
                e.state(),
            )?;
        }
        Event::X(xcb::x::Event::ClientMessage(msg)) => {
            let type_atom_name = conn.atom_name(msg.r#type());
            use xcb::x::ClientMessageData;
            use xcb::XidNew;
            let xdnd_msgtype_atoms = [
                conn.atom_xdndenter,
                conn.atom_xdndposition,
                conn.atom_xdndstatus,
                conn.atom_xdndleave,
                conn.atom_xdnddrop,
                conn.atom_xdndfinished,
            ];
            if xdnd_msgtype_atoms.contains(&msg.r#type()) {
                if let ClientMessageData::Data32(data) = msg.data() {
                    self.xdnd_event(msg.r#type(), &data)?;
                } else {
                    log::warn!("Received ClientMessage {type_atom_name} with wrong format");
                }
            } else if msg.r#type() == conn.atom_protocols {
                if let ClientMessageData::Data32(data) = msg.data() {
                    let protocol_atom = unsafe { Atom::new(data[0]) };
                    log::trace!(
                        "ClientMessage {type_atom_name}/{}",
                        conn.atom_name(protocol_atom)
                    );
                    if protocol_atom == conn.atom_delete {
                        self.events.dispatch(WindowEvent::CloseRequested);
                    }
                } else {
                    log::warn!("Received ClientMessage {type_atom_name} with wrong format");
                }
            }
        }
        Event::X(xcb::x::Event::DestroyNotify(_)) => {
            self.events.dispatch(WindowEvent::Destroyed);
            conn.windows.borrow_mut().remove(&self.window_id);
            conn.child_to_parent_id.borrow_mut().remove(&self.child_id);
        }
        Event::X(xcb::x::Event::SelectionClear(e)) => {
            if let Err(err) = self.selection_clear(e) {
                log::error!("Error handling SelectionClear: {err:#}");
            }
        }
        Event::X(xcb::x::Event::SelectionRequest(e)) => {
            if let Err(err) = self.selection_request(e) {
                // Don't propagate this, as it is not worth exiting the program over it.
                // <https://github.com/wezterm/wezterm/pull/6135>
                log::error!("Error handling SelectionRequest: {err:#}");
            }
        }
        Event::X(xcb::x::Event::SelectionNotify(e)) => {
            if let Err(err) = self.selection_notify(e) {
                log::error!("Error handling SelectionNotify: {err:#}");
            }
        }
        Event::X(xcb::x::Event::PropertyNotify(msg)) => {
            let atom_name = conn.atom_name(msg.atom());
            log::trace!("PropertyNotifyEvent {atom_name}");

            if msg.atom() == conn.atom_gtk_edge_constraints {
                // "_GTK_EDGE_CONSTRAINTS" property is changed when the
                // accessibility settings change the text size and thus
                // the dpi.  We use this as a way to detect dpi changes
                // when running under gnome.
                conn.update_xrm();
                self.check_dpi_and_synthesize_resize();
                let appearance = conn.get_appearance();
                self.appearance_changed(appearance);
            }

            if msg.atom() == conn.atom_net_wm_state {
                // Change in window state should be accompanied by
                // a Configure Notify but not all WMs send these
                // events consistently/at all/in the same order.
                self.sure_about_geometry = false;
                self.verify_focus = true;
            }
        }
        Event::X(xcb::x::Event::FocusIn(e)) => {
            if !matches!(e.detail(), xcb::x::NotifyDetail::Pointer) {
                self.focus_changed(true);
            }
        }
        Event::X(xcb::x::Event::FocusOut(e)) => {
            if !matches!(e.detail(), xcb::x::NotifyDetail::Pointer) {
                self.focus_changed(false);
            }
        }
        Event::X(xcb::x::Event::LeaveNotify(_)) => {
            self.events.dispatch(WindowEvent::MouseLeave);
        }
        _ => {
            log::warn!("unhandled: {:?}", event);
        }
    }

    Ok(())
}
