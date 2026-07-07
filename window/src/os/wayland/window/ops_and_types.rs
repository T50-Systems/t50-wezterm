#[async_trait(?Send)]
impl WindowOps for WaylandWindow {
    fn show(&self) {
        WaylandConnection::with_window_inner(self.0, |inner| {
            inner.show();
            Ok(())
        });
    }

    fn notify<T: Any + Send + Sync>(&self, t: T)
    where
        Self: Sized,
    {
        WaylandConnection::with_window_inner(self.0, move |inner| {
            inner
                .events
                .dispatch(WindowEvent::Notification(Box::new(t)));
            Ok(())
        });
    }

    async fn enable_opengl(&self) -> anyhow::Result<Rc<glium::backend::Context>> {
        let window = self.0;
        promise::spawn::spawn(async move {
            if let Some(handle) = Connection::get().unwrap().wayland().window_by_id(window) {
                let mut inner = handle.borrow_mut();
                inner.enable_opengl()
            } else {
                anyhow::bail!("invalid window");
            }
        })
        .await
    }

    fn hide(&self) {
        WaylandConnection::with_window_inner(self.0, move |inner| {
            inner.window.as_ref().unwrap().set_minimized();
            Ok(())
        });
    }

    fn close(&self) {
        WaylandConnection::with_window_inner(self.0, |inner| {
            inner.close();
            Ok(())
        });
    }

    fn set_cursor(&self, cursor: Option<MouseCursor>) {
        WaylandConnection::with_window_inner(self.0, move |inner| {
            inner.set_cursor(cursor);
            Ok(())
        });
    }

    fn invalidate(&self) {
        WaylandConnection::with_window_inner(self.0, |inner| {
            inner.invalidate();
            Ok(())
        });
    }

    fn set_text_cursor_position(&self, cursor: Rect) {
        WaylandConnection::with_window_inner(self.0, move |inner| {
            inner.set_text_cursor_position(cursor);
            Ok(())
        });
    }

    fn set_title(&self, title: &str) {
        let title = title.to_owned();
        WaylandConnection::with_window_inner(self.0, |inner| {
            inner.set_title(title);
            Ok(())
        });
    }

    fn set_inner_size(&self, width: usize, height: usize) {
        WaylandConnection::with_window_inner(self.0, move |inner| {
            inner.set_inner_size(width, height);
            Ok(())
        });
    }

    fn set_resize_increments(&self, incr: ResizeIncrement) {
        WaylandConnection::with_window_inner(self.0, move |inner| {
            inner.set_resize_increments(incr)
        });
    }

    fn get_clipboard(&self, clipboard: Clipboard) -> Future<String> {
        let mut promise = Promise::new();
        let future = promise.get_future().unwrap();
        let promise = Arc::new(Mutex::new(promise));
        promise::spawn::spawn_into_main_thread(async move {
            let conn = crate::Connection::get().unwrap().wayland();
            // Clone the Arc before dropping the borrow so get_clipboard_data can re-borrow
            // wayland_state internally (so we don't have to pass all state manually).
            let copy_paste_offer = conn.wayland_state.borrow().copy_paste_offer.clone();
            match copy_paste_offer
                .lock()
                .unwrap()
                .get_clipboard_data(clipboard)
            {
                Ok(read) => {
                    std::thread::spawn(move || {
                        let mut promise = promise.lock().unwrap();
                        match read_pipe_with_timeout(read) {
                            Ok(result) => {
                                // Normalize the text to unix line endings, otherwise
                                // copying from eg: firefox inserts a lot of blank
                                // lines, and that is super annoying.
                                promise.ok(result.replace("\r\n", "\n"));
                            }
                            Err(e) => {
                                log::error!("while reading clipboard: {}", e);
                                promise.err(anyhow!("{}", e));
                            }
                        };
                    });
                }
                Err(e) => {
                    // Report the error on the Promise
                    promise.lock().unwrap().err(e);
                }
            };
        })
        .detach();
        future
    }

    fn set_clipboard(&self, clipboard: Clipboard, text: String) {
        promise::spawn::spawn_into_main_thread(async move {
            let conn = crate::Connection::get().unwrap().wayland();
            // Clone the Arc before dropping the borrow so set_clipboard_data can re-borrow
            // wayland_state internally (so we don't have to pass all state manually).
            let copy_paste_offer = conn.wayland_state.borrow().copy_paste_offer.clone();
            copy_paste_offer
                .lock()
                .unwrap()
                .set_clipboard_data(clipboard, text);
        })
        .detach();
    }

    fn toggle_fullscreen(&self) {
        WaylandConnection::with_window_inner(self.0, move |inner| {
            if inner.window_state.contains(WindowState::FULL_SCREEN) {
                inner.window.as_ref().unwrap().unset_fullscreen();
            } else {
                inner.window.as_ref().unwrap().set_fullscreen(None);
            }
            Ok(())
        });
    }

    fn maximize(&self) {
        WaylandConnection::with_window_inner(self.0, move |inner| Ok(inner.maximize()));
    }

    fn restore(&self) {
        WaylandConnection::with_window_inner(self.0, move |inner| Ok(inner.restore()));
    }

    fn config_did_change(&self, config: &ConfigHandle) {
        let config = config.clone();
        WaylandConnection::with_window_inner(self.0, move |inner| {
            inner.config_did_change(config);
            Ok(())
        });
    }
}
#[derive(Default, Clone, Debug)]
pub(crate) struct PendingEvent {
    pub(crate) close: bool,
    pub(crate) had_configure_event: bool,
    refresh_decorations: bool,
    // XXX: configure and window_configure could probably be combined, but right now configure only
    // queues a new size, so it can be out of sync. Example would be maximizing and minimizing winodw
    pub(crate) configure: Option<(u32, u32)>,
    pub(crate) window_configure: Option<WindowConfigure>,
    pub(crate) dpi: Option<i32>,
    pub(crate) window_state: Option<WindowState>,
}

pub(crate) fn read_pipe_with_timeout(mut file: ReadPipe) -> anyhow::Result<String> {
    let mut result = Vec::new();

    // set non-blocking I/O on the pipe
    // (adapted from FileDescriptor::set_non_blocking_impl in /filedescriptor/src/unix.rs)
    if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK) } != 0 {
        bail!(
            "failed to change non-blocking mode: {}",
            std::io::Error::last_os_error()
        )
    }

    let mut pfd = libc::pollfd {
        fd: file.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };

    let mut buf = [0u8; 8192];

    loop {
        if unsafe { libc::poll(&mut pfd, 1, 3000) == 1 } {
            match file.read(&mut buf) {
                Ok(size) if size == 0 => {
                    break;
                }
                Ok(size) => {
                    result.extend_from_slice(&buf[..size]);
                }
                Err(e) => bail!("error reading from pipe: {}", e),
            }
        } else {
            bail!("timed out reading from pipe");
        }
    }

    Ok(String::from_utf8(result)?)
}

pub struct WaylandWindowInner {
    pub(crate) events: WindowEventSender,
    surface_factor: f64,
    window: Option<XdgWindow>,
    pub(super) window_frame: FallbackFrame<WaylandState>,
    dimensions: Dimensions,
    resize_increments: Option<ResizeIncrement>,
    window_state: WindowState,
    last_mouse_coords: Point,
    mouse_buttons: MouseButtons,
    hscroll_remainder: f64,
    vscroll_remainder: f64,
    modifiers: Modifiers,
    leds: KeyboardLedStatus,
    pub(super) key_repeat: Option<(u32, Arc<Mutex<KeyRepeatState>>)>,
    pub(super) pending_event: Arc<Mutex<PendingEvent>>,
    pub(super) pending_mouse: Arc<Mutex<PendingMouse>>,
    pending_first_configure: Option<async_channel::Sender<()>>,
    frame_callback: Option<WlCallback>,
    invalidated: bool,
    // font_config: Rc<FontConfiguration>,
    text_cursor: Option<Rect>,
    appearance: Appearance,
    config: ConfigHandle,
    // cache the title for comparison to avoid spamming
    // the compositor with updates that don't actually change it
    title: Option<String>,
    // wegl_surface is listed before gl_state because it
    // must be dropped before gl_state otherwise the underlying
    // libraries will segfault on shutdown
    wegl_surface: Option<WlEglSurface>,
    gl_state: Option<Rc<glium::backend::Context>>,
