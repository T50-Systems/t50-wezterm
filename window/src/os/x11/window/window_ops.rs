impl HasDisplayHandle for XWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let conn = Connection::get()
            .expect("display_handle only callable on main thread")
            .x11();
        let handle = XcbDisplayHandle::new(NonNull::new(conn.get_raw_conn() as _), conn.screen_num);

        unsafe { Ok(DisplayHandle::borrow_raw(RawDisplayHandle::Xcb(handle))) }
    }
}

impl HasWindowHandle for XWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let conn = Connection::get().expect("window_handle only callable on main thread");
        let handle = conn
            .x11()
            .window_by_id(self.0)
            .expect("window handle invalid!?");

        let inner = handle.lock().unwrap();
        let handle = inner.window_handle()?;
        unsafe { Ok(WindowHandle::borrow_raw(handle.as_raw())) }
    }
}

#[async_trait(?Send)]
impl WindowOps for XWindow {
    async fn enable_opengl(&self) -> anyhow::Result<Rc<glium::backend::Context>> {
        let window = self.0;
        promise::spawn::spawn(async move {
            if let Some(handle) = Connection::get().unwrap().x11().window_by_id(window) {
                let mut inner = handle.lock().unwrap();
                inner.enable_opengl()
            } else {
                anyhow::bail!("invalid window");
            }
        })
        .await
    }

    fn notify<T: Any + Send + Sync>(&self, t: T)
    where
        Self: Sized,
    {
        XConnection::with_window_inner(self.0, move |inner| {
            inner
                .events
                .dispatch(WindowEvent::Notification(Box::new(t)));
            Ok(())
        });
    }

    fn close(&self) {
        XConnection::with_window_inner(self.0, |inner| {
            inner.close();
            Ok(())
        });
    }

    fn hide(&self) {
        XConnection::with_window_inner(self.0, |inner| {
            inner.hide();
            Ok(())
        });
    }

    fn toggle_fullscreen(&self) {
        XConnection::with_window_inner(self.0, |inner| {
            inner.toggle_fullscreen();
            Ok(())
        });
    }

    fn maximize(&self) {
        XConnection::with_window_inner(self.0, |inner| {
            inner.maximize();
            Ok(())
        });
    }

    fn restore(&self) {
        XConnection::with_window_inner(self.0, |inner| {
            inner.restore();
            Ok(())
        });
    }

    fn config_did_change(&self, config: &ConfigHandle) {
        let config = config.clone();
        XConnection::with_window_inner(self.0, move |inner| {
            inner.config_did_change(&config);
            Ok(())
        });
    }

    fn focus(&self) {
        XConnection::with_window_inner(self.0, |inner| {
            inner.focus();
            Ok(())
        });
    }

    fn show(&self) {
        XConnection::with_window_inner(self.0, |inner| {
            inner.show();
            Ok(())
        });
    }

    fn set_cursor(&self, cursor: Option<MouseCursor>) {
        XConnection::with_window_inner(self.0, move |inner| {
            let _ = inner.set_cursor(cursor);
            Ok(())
        });
    }

    fn invalidate(&self) {
        XConnection::with_window_inner(self.0, |inner| {
            inner.invalidate();
            Ok(())
        });
    }

    fn set_title(&self, title: &str) {
        let title = title.to_owned();
        XConnection::with_window_inner(self.0, move |inner| {
            inner.set_title(&title);
            Ok(())
        });
    }

    fn set_inner_size(&self, width: usize, height: usize) {
        XConnection::with_window_inner(self.0, move |inner| {
            inner
                .conn()
                .send_request_no_reply_log(&xcb::x::ConfigureWindow {
                    window: inner.window_id,
                    value_list: &[
                        xcb::x::ConfigWindow::Width(width as u32),
                        xcb::x::ConfigWindow::Height(height as u32),
                    ],
                });
            inner.resize_child(width as u32, height as u32);
            inner.outstanding_configure_requests += 1;
            Ok(())
        });
    }

    fn request_drag_move(&self) {
        XConnection::with_window_inner(self.0, move |inner| {
            inner.request_drag_move()?;
            Ok(())
        });
    }

    fn set_window_drag_position(&self, coords: ScreenPoint) {
        XConnection::with_window_inner(self.0, move |inner| {
            inner.window_drag_position.replace(coords);
            Ok(())
        });
    }

    fn set_window_position(&self, coords: ScreenPoint) {
        XConnection::with_window_inner(self.0, move |inner| {
            inner.set_window_position(coords);
            Ok(())
        });
    }

    fn set_text_cursor_position(&self, cursor: Rect) {
        XConnection::with_window_inner(self.0, move |inner| {
            inner.set_text_cursor_position(cursor);
            Ok(())
        });
    }

    fn set_icon(&self, image: Image) {
        XConnection::with_window_inner(self.0, move |inner| {
            inner.set_icon(&image);
            Ok(())
        });
    }

    fn set_resize_increments(&self, incr: ResizeIncrement) {
        XConnection::with_window_inner(self.0, move |inner| {
            if let Err(err) = inner.set_resize_increments(incr) {
                log::error!("set_resize_increments failed: {:#}", err);
            }
            Ok(())
        });
    }

    /// Initiate textual transfer from the clipboard
    fn get_clipboard(&self, clipboard: Clipboard) -> Future<String> {
        let window_id = self.0;
        log::trace!("SEL: window_id={window_id:?} Window::get_clipboard {clipboard:?} called");
        let mut promise = Promise::new();
        let future = promise.get_future().unwrap();
        let mut promise = Some(promise);

        XConnection::with_window_inner(window_id, move |inner| {
            // In theory, we could simply consult inner.copy_and_paste to see
            // if we think we own the clipboard, but there are some situations
            // where the selection owner moves between two wezterm windows
            // where we don't receive a SELECTION_NOTIFY in time to correctly
            // invalidate that state, so we always ask the X server to for
            // the selection, even if it is a little slower.
            // <https://github.com/wezterm/wezterm/issues/2110>
            let promise = promise.take().unwrap();
            log::debug!(
                "SEL: window_id={window_id:?} Window::get_clipboard: \
                        {clipboard:?}, prepare promise, time={}",
                inner.copy_and_paste.time
            );
            inner.copy_and_paste.request_mut(clipboard).replace(promise);
            let conn = inner.conn();
            // Find the owner and ask them to send us the buffer
            conn.send_request_no_reply_log(&xcb::x::ConvertSelection {
                requestor: inner.window_id,
                selection: match clipboard {
                    Clipboard::Clipboard => conn.atom_clipboard,
                    Clipboard::PrimarySelection => xcb::x::ATOM_PRIMARY,
                },
                target: conn.atom_utf8_string,
                property: conn.atom_xsel_data,
                time: inner.copy_and_paste.time,
            });
            Ok(())
        });

        future
    }

    /// Set some text in the clipboard
    fn set_clipboard(&self, clipboard: Clipboard, text: String) {
        let window_id = self.0;
        XConnection::with_window_inner(window_id, move |inner| {
            log::trace!(
                "SEL: window_id={window_id:?} now owns selection \
                for {clipboard:?} {text:?}"
            );
            inner
                .copy_and_paste
                .clipboard_mut(clipboard)
                .replace(text.clone());
            inner.update_selection_owner(clipboard)?;
            Ok(())
        });
    }
}
