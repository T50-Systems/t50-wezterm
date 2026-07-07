impl HasDisplayHandle for Window {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        unsafe {
            Ok(DisplayHandle::borrow_raw(RawDisplayHandle::AppKit(
                AppKitDisplayHandle::new(),
            )))
        }
    }
}

impl HasWindowHandle for Window {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let handle =
            AppKitWindowHandle::new(NonNull::new(self.ns_view as *mut _).expect("non-null"));
        unsafe { Ok(WindowHandle::borrow_raw(RawWindowHandle::AppKit(handle))) }
    }
}

/// @see https://developer.apple.com/documentation/appkit/nswindow/level
pub type NSWindowLevel = i64;

pub fn nswindow_level_to_window_level(nswindow_level: NSWindowLevel) -> WindowLevel {
    match nswindow_level {
        -1 => WindowLevel::AlwaysOnBottom,
        0 => WindowLevel::Normal,
        3 => WindowLevel::AlwaysOnTop,
        _ => panic!("Invalid window level: {}", nswindow_level),
    }
}

pub fn window_level_to_nswindow_level(level: WindowLevel) -> NSWindowLevel {
    match level {
        WindowLevel::AlwaysOnBottom => -1,
        WindowLevel::Normal => 0,
        WindowLevel::AlwaysOnTop => 3,
    }
}

#[async_trait(?Send)]
impl WindowOps for Window {
    async fn enable_opengl(&self) -> anyhow::Result<Rc<glium::backend::Context>> {
        let window_id = self.id;
        promise::spawn::spawn(async move {
            if let Some(handle) = Connection::get().unwrap().window_by_id(window_id) {
                let mut inner = handle.borrow_mut();
                inner.enable_opengl()
            } else {
                bail!("invalid window");
            }
        })
        .await
    }

    fn notify<T: Any + Send + Sync>(&self, t: T)
    where
        Self: Sized,
    {
        Connection::with_window_inner(self.id, move |inner| {
            if let Some(window_view) = WindowView::get_this(unsafe { &**inner.view }) {
                window_view
                    .inner
                    .borrow_mut()
                    .events
                    .dispatch(WindowEvent::Notification(Box::new(t)));
            }
            Ok(())
        });
    }

    fn close(&self) {
        Connection::with_window_inner(self.id, |inner| {
            inner.close();
            Ok(())
        });
    }

    fn focus(&self) {
        Connection::with_window_inner(self.id, |inner| {
            inner.focus();
            Ok(())
        });
    }

    fn hide(&self) {
        Connection::with_window_inner(self.id, |inner| {
            inner.hide();
            Ok(())
        });
    }

    fn show(&self) {
        Connection::with_window_inner(self.id, |inner| {
            inner.show();
            Ok(())
        });
    }

    fn set_cursor(&self, cursor: Option<MouseCursor>) {
        Connection::with_window_inner(self.id, move |inner| {
            let _ = inner.set_cursor(cursor);
            Ok(())
        });
    }

    fn invalidate(&self) {
        Connection::with_window_inner(self.id, |inner| {
            inner.invalidate();
            Ok(())
        });
    }

    fn set_title(&self, title: &str) {
        let title = title.to_owned();
        Connection::with_window_inner(self.id, move |inner| {
            inner.set_title(&title);
            Ok(())
        });
    }

    fn set_window_level(&self, level: WindowLevel) {
        Connection::with_window_inner(self.id, move |inner| {
            inner.set_window_level(level);
            Ok(())
        });
    }

    fn set_inner_size(&self, width: usize, height: usize) {
        Connection::with_window_inner(self.id, move |inner| {
            inner.set_inner_size(width, height);
            if let Some(window_view) = WindowView::get_this(unsafe { &**inner.view }) {
                window_view
                    .inner
                    .borrow_mut()
                    .events
                    .dispatch(WindowEvent::SetInnerSizeCompleted);
            }
            Ok(())
        });
    }

    fn set_window_position(&self, coords: ScreenPoint) {
        Connection::with_window_inner(self.id, move |inner| {
            inner.set_window_position(coords);
            Ok(())
        });
    }

    fn set_text_cursor_position(&self, cursor: Rect) {
        Connection::with_window_inner(self.id, move |inner| {
            inner.set_text_cursor_position(cursor);
            Ok(())
        });
    }

    fn get_clipboard(&self, _clipboard: Clipboard) -> Future<String> {
        Future::result(
            ClipboardContext::new()
                .read()
                .map_err(|e| anyhow!("Failed to get clipboard:{}", e)),
        )
    }

    fn set_clipboard(&self, _clipboard: Clipboard, text: String) {
        ClipboardContext::new().write(text).ok();
    }

    fn toggle_fullscreen(&self) {
        Connection::with_window_inner(self.id, move |inner| {
            inner.toggle_fullscreen();
            Ok(())
        });
    }

    fn maximize(&self) {
        Connection::with_window_inner(self.id, move |inner| {
            inner.maximize();
            Ok(())
        });
    }

    fn restore(&self) {
        Connection::with_window_inner(self.id, move |inner| {
            inner.restore();
            Ok(())
        });
    }

    fn set_resize_increments(&self, incr: ResizeIncrement) {
        Connection::with_window_inner(self.id, move |inner| {
            inner.set_resize_increments(incr);
            Ok(())
        });
    }

    fn config_did_change(&self, config: &ConfigHandle) {
        let config = config.clone();
        Connection::with_window_inner(self.id, move |inner| {
            inner.config_did_change(&config);
            Ok(())
        });
    }

    fn get_os_parameters(
        &self,
        config: &ConfigHandle,
        window_state: WindowState,
    ) -> anyhow::Result<Option<Parameters>> {
        // We implement this method primarily to provide Notch-avoidance for
        // systems with a notch.
        // We only need this for non-native full screen mode.

        let native_full_screen = {
            let style_mask = unsafe { NSWindow::styleMask(self.ns_window) };
            style_mask.contains(NSWindowStyleMask::NSFullScreenWindowMask)
        };

        let border_dimensions = if window_state.contains(WindowState::FULL_SCREEN)
            && !native_full_screen
            && !config.macos_fullscreen_extend_behind_notch
        {
            let main_screen = unsafe { NSScreen::mainScreen(nil) };
            let has_safe_area_insets: BOOL =
                unsafe { msg_send![main_screen, respondsToSelector: sel!(safeAreaInsets)] };
            if has_safe_area_insets == YES {
                #[derive(Debug)]
                struct NSEdgeInsets {
                    top: CGFloat,
                    left: CGFloat,
                    bottom: CGFloat,
                    right: CGFloat,
                }
                let insets: NSEdgeInsets = unsafe { msg_send![main_screen, safeAreaInsets] };
                log::trace!("{:?}", insets);

                let scale = unsafe {
                    let frame = NSScreen::frame(main_screen);
                    let backing_frame = NSScreen::convertRectToBacking_(main_screen, frame);
                    backing_frame.size.height / frame.size.height
                };

                let top = (insets.top.ceil() * scale) as usize;
                Some(Border {
                    top: ULength::new(top),
                    left: ULength::new(insets.left.ceil() as usize),
                    right: ULength::new(insets.right.ceil() as usize),
                    bottom: ULength::new(insets.bottom.ceil() as usize),
                    color: crate::color::LinearRgba::with_components(0., 0., 0., 1.),
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok(Some(Parameters {
            title_bar: TitleBar {
                padding_left: ULength::new(0),
                padding_right: ULength::new(0),
                height: None,
                font_and_size: None,
            },
            border_dimensions,
        }))
    }
}
