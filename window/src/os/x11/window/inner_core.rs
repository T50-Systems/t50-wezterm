fn enable_opengl(&mut self) -> anyhow::Result<Rc<glium::backend::Context>> {
    let conn = self.conn();

    let gl_state = match conn.gl_connection.borrow().as_ref() {
        None => crate::egl::GlState::create(
            Some(conn.conn.get_raw_dpy() as *const _),
            self.child_id.resource_id() as *mut _,
        ),
        Some(glconn) => crate::egl::GlState::create_with_existing_connection(
            glconn,
            self.child_id.resource_id() as *mut _,
        ),
    };

    // Don't chain on the end of the above to avoid borrowing gl_connection twice.
    let gl_state = gl_state.map(Rc::new).and_then(|state| unsafe {
        conn.gl_connection
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

    Ok(gl_state)
}

/// Add a region to the list of exposed/damaged/dirty regions.
/// Note that a window resize will likely invalidate the entire window.
/// If the new region intersects with the prior region, then we expand
/// it to encompass both.  This avoids bloating the list with a series
/// of increasing rectangles when resizing larger or smaller.
fn expose(&mut self, x: u16, y: u16, width: u16, height: u16, count: u16) {
    log::trace!("expose: {x},{y} {width}x{height} ({count} expose events follow this one)");
    let max_x = x.saturating_add(width);
    let max_y = y.saturating_add(height);
    if max_x > self.width || max_y > self.height {
        log::trace!("flagging geometry as unsure because exposed region is larger than known geom");
        self.sure_about_geometry = false;
    }
    self.queue_pending(WindowEvent::NeedRepaint);
}

fn cancel_drag(&mut self) -> bool {
    if self.dragging {
        log::debug!("cancel_drag");
        self.net_wm_moveresize(0, 0, _NET_WM_MOVERESIZE_CANCEL, 0);
        self.dragging = false;
        if let Some(event) = self.current_mouse_event.take() {
            self.do_mouse_event(MouseEvent {
                kind: MouseEventKind::Release(MousePress::Left),
                ..event
            })
            .ok();
        }
        return true;
    }
    false
}

fn do_mouse_event(&mut self, event: MouseEvent) -> anyhow::Result<()> {
    if self.cancel_drag() {
        return Ok(());
    }
    self.current_mouse_event.replace(event.clone());
    self.events.dispatch(WindowEvent::MouseEvent(event));
    Ok(())
}

fn set_cursor(&mut self, cursor: Option<MouseCursor>) -> anyhow::Result<()> {
    self.cursors.set_cursor(self.window_id, cursor)
}

fn check_dpi_and_synthesize_resize(&mut self) {
    let conn = self.conn();
    let dpi = conn.default_dpi();

    if dpi != self.dpi {
        log::trace!(
            "dpi changed from {} -> {}, so synthesize a resize",
            dpi,
            self.dpi
        );
        self.dpi = dpi;
        self.last_wm_state = self.get_window_state().unwrap_or(WindowState::default());
        self.events.dispatch(WindowEvent::Resized {
            dimensions: Dimensions {
                pixel_width: self.width as usize,
                pixel_height: self.height as usize,
                dpi: self.dpi as usize,
            },
            window_state: self.last_wm_state,
            live_resizing: false,
        });
    }
}

fn queue_pending(&mut self, event: WindowEvent) {
    self.pending.push(event);
}

fn resize_child(&self, width: u32, height: u32) {
    self.conn()
        .send_request_no_reply_log(&xcb::x::ConfigureWindow {
            window: self.child_id,
            value_list: &[
                xcb::x::ConfigWindow::Width(width as u32),
                xcb::x::ConfigWindow::Height(height as u32),
            ],
        });
    // send_request_no_reply_log() is synchronous, so no further synchronization required
}

pub fn dispatch_pending_events(&mut self) -> anyhow::Result<()> {
    if self.pending.is_empty() {
        return Ok(());
    }

    let mut need_paint = false;
    let mut resize = None;

    for event in self.pending.drain(..) {
        match event {
            WindowEvent::NeedRepaint => {
                if need_paint {
                    log::trace!("coalesce a repaint");
                }
                need_paint = true;
            }
            e @ WindowEvent::Resized { .. } => {
                if resize.is_some() {
                    log::trace!("coalesce a resize");
                }
                resize.replace(e);
            }
            e => {
                self.events.dispatch(e);
            }
        }
    }

    if let Some(resize) = resize.take() {
        self.sure_about_geometry = true;
        self.events.dispatch(resize);
    }

    // These SetInnerSizeCompleted events need to be dispatched after the
    // above Resized events because a resize cannot finish before it occurs.
    while self.pending_finished_resizes > 0 {
        self.events.dispatch(WindowEvent::SetInnerSizeCompleted);
        self.pending_finished_resizes -= 1;
    }

    if need_paint {
        if self.paint_throttled {
            self.invalidated = true;
        } else {
            self.invalidated = false;

            if self.verify_focus || self.has_focus.is_none() {
                log::trace!("About to paint, but we're unsure about focus; querying!");

                let focus = self
                    .conn()
                    .send_and_wait_request(&xcb::x::GetInputFocus {})?;
                let focused = focus.focus() == self.window_id;
                log::trace!(
                    "Do I {:?} have focus? result={}, I thought {:?}",
                    self.window_id,
                    focused,
                    self.has_focus
                );
                if Some(focused) != self.has_focus {
                    self.has_focus.replace(focused);
                    self.events.dispatch(WindowEvent::FocusChanged(focused));
                }

                self.verify_focus = false;
            }

            if !self.sure_about_geometry {
                self.sure_about_geometry = true;

                log::trace!(
                    "About to paint, but we're unsure about geometry; querying window_id {:?}!",
                    self.window_id
                );
                let geom = self
                    .conn()
                    .send_and_wait_request(&xcb::x::GetGeometry {
                        drawable: xcb::x::Drawable::Window(self.window_id),
                    })
                    .context("querying geometry")?;
                log::trace!(
                    "geometry is {}x{} vs. our initial {}x{}",
                    geom.width(),
                    geom.height(),
                    self.width,
                    self.height
                );

                let window_state = self.get_window_state().unwrap_or(WindowState::default());

                if self.width != geom.width()
                    || self.height != geom.height()
                    || self.last_wm_state != window_state
                {
                    self.resize_child(geom.width() as u32, geom.height() as u32);

                    self.width = geom.width();
                    self.height = geom.height();
                    self.last_wm_state = window_state;

                    self.events.dispatch(WindowEvent::Resized {
                        dimensions: Dimensions {
                            pixel_width: self.width as usize,
                            pixel_height: self.height as usize,
                            dpi: self.dpi as usize,
                        },
                        window_state,
                        live_resizing: false,
                    });
                }
            }

            self.events.dispatch(WindowEvent::NeedRepaint);

            self.paint_throttled = true;
            let window_id = self.window_id;
            let max_fps = self.config.max_fps;
            promise::spawn::spawn(async move {
                async_io::Timer::after(std::time::Duration::from_millis(1000 / max_fps as u64))
                    .await;
                XConnection::with_window_inner(window_id, move |inner| {
                    inner.paint_throttled = false;
                    if inner.invalidated {
                        inner.invalidate();
                    }
                    Ok(())
                });
            })
            .detach();
        }
    }

    Ok(())
}

fn button_event(
    &mut self,
    pressed: bool,
    time: xcb::x::Timestamp,
    detail: xcb::x::Button,
    event_x: i16,
    event_y: i16,
    root_x: i16,
    root_y: i16,
    state: xcb::x::KeyButMask,
) -> anyhow::Result<()> {
    self.copy_and_paste.time = time;

    if self.cancel_drag() {
        log::debug!("cancel drag due to button {detail} {state:?}");
        return Ok(());
    }

    let kind = match detail {
        b @ 1..=3 => {
            let button = match b {
                1 => MousePress::Left,
                2 => MousePress::Middle,
                3 => MousePress::Right,
                _ => unreachable!(),
            };
            if pressed {
                MouseEventKind::Press(button)
            } else {
                MouseEventKind::Release(button)
            }
        }
        b @ 4..=5 => {
            if !pressed {
                return Ok(());
            }

            // Ideally this would be configurable, but it's currently a bit
            // awkward to configure this layer, so let's just improve the
            // default for now!
            const LINES_PER_TICK: i16 = 5;

            MouseEventKind::VertWheel(if b == 4 {
                LINES_PER_TICK
            } else {
                -LINES_PER_TICK
            })
        }
        _ => {
            log::trace!("button {} is not implemented", detail);
            return Ok(());
        }
    };

    let event = MouseEvent {
        kind,
        coords: Point::new(event_x.try_into().unwrap(), event_y.try_into().unwrap()),
        screen_coords: ScreenPoint::new(root_x.try_into().unwrap(), root_y.try_into().unwrap()),
        modifiers: xkeysyms::modifiers_from_state(state.bits()),
        mouse_buttons: MouseButtons::default(),
    };
    self.do_mouse_event(event)
}
