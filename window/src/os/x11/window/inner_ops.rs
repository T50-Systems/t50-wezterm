impl XWindowInner {
    fn close(&mut self) {
        let conn = self.conn();
        conn.flush()
            .context("flush pending requests prior to issuing DestroyWindow")
            .ok();
        // Remove the window from the map now, as GL state
        // requires that it is able to make_current() in its
        // Drop impl, and that cannot succeed after we've
        // destroyed the window at the X11 level.
        self.conn().windows.borrow_mut().remove(&self.window_id);
        self.conn()
            .child_to_parent_id
            .borrow_mut()
            .remove(&self.child_id);

        // Unmap the window first: calling DestroyWindow here may race
        // with some requests made either by EGL or the IME, but I haven't
        // been able to pin down the source.
        // We'll destroy the window in a couple of seconds
        conn.send_request_no_reply_log(&xcb::x::UnmapWindow {
            window: self.window_id,
        });

        conn.send_request_no_reply_log(&xcb::x::DestroyWindow {
            window: self.child_id,
        });

        // Arrange to destroy the window after a couple of seconds; that
        // should give whatever stuff is still referencing the window
        // to finish and avoid triggering a protocol error.
        // I don't really like this as a solution :-/
        // <https://github.com/wezterm/wezterm/issues/2198>
        let window = self.window_id;
        promise::spawn::spawn(async move {
            async_io::Timer::after(std::time::Duration::from_secs(2)).await;
            let conn = Connection::get().unwrap().x11();
            log::trace!("close sending DestroyWindow for {:?}", window);
            conn.send_request_no_reply_log(&xcb::x::DestroyWindow { window });
        })
        .detach();
        // Ensure that we don't try to destroy the window twice,
        // otherwise the rust xcb bindings will generate a
        // fatal error!
        log::trace!("clear out self.window_id");
        self.window_id = xcb::x::Window::none();
    }
    fn hide(&mut self) {}
    fn show(&mut self) {
        self.conn().send_request_no_reply_log(&xcb::x::MapWindow {
            window: self.window_id,
        });
    }

    fn focus(&mut self) {
        let conn = self.conn();
        conn.send_request_no_reply_log(&xcb::x::SendEvent {
            propagate: true,
            destination: xcb::x::SendEventDest::Window(conn.root),
            event_mask: xcb::x::EventMask::SUBSTRUCTURE_REDIRECT
                | xcb::x::EventMask::SUBSTRUCTURE_NOTIFY,
            event: &xcb::x::ClientMessageEvent::new(
                self.window_id,
                conn.atom_net_active_window,
                xcb::x::ClientMessageData::Data32([
                    1,
                    // You'd think that self.copy_and_paste.time would
                    // be the thing to use, but Mutter ignored this request
                    // until I switched to CURRENT_TIME
                    xcb::x::CURRENT_TIME,
                    0,
                    0,
                    0,
                ]),
            ),
        });

        if let Err(err) = conn.flush() {
            log::error!("Error flushing: {err:#}");
        }
    }

    fn invalidate(&mut self) {
        self.queue_pending(WindowEvent::NeedRepaint);
        self.dispatch_pending_events().ok();
    }

    fn maximize(&mut self) {
        if let Err(err) = self.set_maximized_hint(true) {
            log::error!("Failed to maximize: {err:#}");
        }
    }

    fn restore(&mut self) {
        if let Err(err) = self.set_maximized_hint(false) {
            log::error!("Failed to restore: {err:#}");
        }
    }

    fn toggle_fullscreen(&mut self) {
        let fullscreen = match self.get_window_state() {
            Ok(f) => f.contains(WindowState::FULL_SCREEN),
            Err(err) => {
                log::error!("Failed to determine fullscreen state: {}", err);
                return;
            }
        };
        self.set_fullscreen_hint(!fullscreen).ok();
    }

    fn config_did_change(&mut self, config: &ConfigHandle) {
        let dpi_changed =
            self.config.dpi != config.dpi || self.config.dpi_by_screen != config.dpi_by_screen;
        self.config = config.clone();
        let _ = self.adjust_decorations(config.window_decorations);

        if dpi_changed {
            let _ = self.configure_notify("config reload", self.width, self.height);
        }
    }

    fn net_wm_moveresize(&mut self, x_root: u32, y_root: u32, direction: u32, button: u32) {
        let source_indication = 1;
        let conn = self.conn();

        if !conn
            .supported
            .borrow()
            .contains(&conn.atom_net_wm_moveresize)
        {
            log::debug!("WM doesn't support _NET_WM_MOVERESIZE");
            return;
        }

        log::debug!("net_wm_moveresize {x_root},{y_root} direction={direction} button={button}");

        if direction != _NET_WM_MOVERESIZE_CANCEL {
            // Tell the server to ungrab. Even though we haven't explicitly
            // grabbed it in our application code, there's an implicit grab
            // as part of a mouse drag and the moveresize will do nothing
            // if we don't ungrab it.
            conn.send_request_no_reply_log(&xcb::x::UngrabPointer {
                time: self.copy_and_paste.time,
            });
            // Flag to ourselves that we are dragging.
            // This is also used to gate the fallback of calling
            // set_window_position in case the WM doesn't support
            // _NET_WM_MOVERESIZE and we returned early above.
            self.dragging = true;
        }

        conn.send_request_no_reply_log(&xcb::x::SendEvent {
            propagate: true,
            destination: xcb::x::SendEventDest::Window(conn.root),
            event_mask: xcb::x::EventMask::SUBSTRUCTURE_REDIRECT
                | xcb::x::EventMask::SUBSTRUCTURE_NOTIFY,
            event: &xcb::x::ClientMessageEvent::new(
                self.window_id,
                conn.atom_net_wm_moveresize,
                xcb::x::ClientMessageData::Data32([
                    x_root,
                    y_root,
                    direction,
                    button,
                    source_indication,
                ]),
            ),
        });
        conn.flush().context("flush moveresize").ok();
    }

    fn request_drag_move(&mut self) -> anyhow::Result<()> {
        let pos = self.window_drag_position.unwrap_or_default();

        let x_root = pos.x as u32;
        let y_root = pos.y as u32;
        let button = 1; // Left

        self.net_wm_moveresize(x_root, y_root, _NET_WM_MOVERESIZE_MOVE, button);
        Ok(())
    }

    fn set_window_position(&mut self, coords: ScreenPoint) {
        if self.dragging {
            return;
        }

        // We ask the window manager to move the window for us so that
        // we don't have to deal with adjusting for the frame size.
        // Note that neither this technique or the configure_window
        // approach below will successfully move a window running
        // under the crostini environment on a chromebook :-(
        let conn = self.conn();

        conn.send_request_no_reply_log(&xcb::x::SendEvent {
            propagate: true,
            destination: xcb::x::SendEventDest::Window(conn.root),
            event_mask: xcb::x::EventMask::SUBSTRUCTURE_REDIRECT
                | xcb::x::EventMask::SUBSTRUCTURE_NOTIFY,
            event: &xcb::x::ClientMessageEvent::new(
                self.window_id,
                conn.atom_net_move_resize_window,
                xcb::x::ClientMessageData::Data32([
                    xcb::x::Gravity::Static as u32 |
            1<<12 | // normal program
            xcb_util::MOVE_RESIZE_MOVE
                | xcb_util::MOVE_RESIZE_WINDOW_X
                | xcb_util::MOVE_RESIZE_WINDOW_Y,
                    coords.x as u32,
                    coords.y as u32,
                    self.width as u32,
                    self.height as u32,
                ]),
            ),
        });
    }

    /// Change the title for the window manager
    fn set_title(&mut self, title: &str) {
        if title == self.title {
            return;
        }
        self.title = title.to_string();

        let conn = self.conn();

        conn.send_request_no_reply_log(&xcb::x::ChangeProperty {
            mode: PropMode::Replace,
            window: self.window_id,
            property: xcb::x::ATOM_WM_NAME,
            r#type: conn.atom_utf8_string,
            data: title.as_bytes(),
        });

        // Also set EWMH _NET_WM_NAME, as some clients don't correctly
        // fall back to reading WM_NAME
        conn.send_request_no_reply_log(&xcb::x::ChangeProperty {
            mode: PropMode::Replace,
            window: self.window_id,
            property: conn.atom_net_wm_name,
            r#type: conn.atom_utf8_string,
            data: title.as_bytes(),
        });
    }

    fn set_text_cursor_position(&mut self, cursor: Rect) {
        if self.last_cursor_position == cursor {
            return;
        }
        self.last_cursor_position = cursor;
        self.update_ime_position();
    }

    fn update_ime_position(&mut self) {
        if !self.has_focus.unwrap_or(false) {
            return;
        }
        self.conn().ime.borrow_mut().update_pos(
            self.window_id,
            self.last_cursor_position.min_x() as i16,
            self.last_cursor_position.max_y() as i16,
        );
    }

    fn set_icon(&mut self, image: &dyn BitmapImage) {
        let (width, height) = image.image_dimensions();

        // https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm44927025355360
        // says that this is an array of 32bit ARGB data.
        // The first two elements are width, height, with the remainder
        // being the row data, left-to-right, top-to-bottom.
        let mut icon_data = Vec::with_capacity((2 + (width * height)) * 4);
        icon_data.push(width as u32);
        icon_data.push(height as u32);
        // `BitmapImage` is rgba32, so we need to munge to get argb32.
        // We also need to put the data into big endian format.
        for pixel in image.pixels() {
            let [r, g, b, a] = pixel.to_ne_bytes();
            icon_data.push(u32::from_be_bytes([a, r, g, b]));
        }

        self.conn()
            .send_request_no_reply_log(&xcb::x::ChangeProperty {
                mode: PropMode::Replace,
                window: self.window_id,
                property: self.conn().atom_net_wm_icon,
                r#type: xcb::x::ATOM_CARDINAL,
                data: &icon_data,
            });
    }

    fn set_resize_increments(&mut self, incr: ResizeIncrement) -> anyhow::Result<()> {
        use xcb_util::*;
        let hints = xcb_size_hints_t {
            flags: XCB_ICCCM_SIZE_HINT_P_MIN_SIZE
                | XCB_ICCCM_SIZE_HINT_P_RESIZE_INC
                | XCB_ICCCM_SIZE_HINT_BASE_SIZE,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            min_width: (incr.base_width + incr.x).into(),
            min_height: (incr.base_height + incr.y).into(),
            max_width: 0,
            max_height: 0,
            width_inc: incr.x.into(),
            height_inc: incr.y.into(),
            min_aspect_num: 0,
            min_aspect_den: 0,
            max_aspect_num: 0,
            max_aspect_den: 0,
            base_width: incr.base_width.into(),
            base_height: incr.base_height.into(),
            win_gravity: 0,
        };

        let data = unsafe {
            std::slice::from_raw_parts(
                &hints as *const _ as *const u32,
                std::mem::size_of::<xcb_size_hints_t>() / 4,
            )
        };

        self.conn().send_request_no_reply(&xcb::x::ChangeProperty {
            mode: PropMode::Replace,
            window: self.window_id,
            property: xcb::x::ATOM_WM_NORMAL_HINTS,
            r#type: xcb::x::ATOM_WM_SIZE_HINTS,
            data,
        })?;

        Ok(())
    }
}
