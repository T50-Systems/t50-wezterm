impl WindowView {
    extern "C" fn did_resize(this: &mut Object, _sel: Sel, _notification: id) {
        if let Some(this) = Self::get_this(this) {
            let inner = this.inner.borrow_mut();

            if let Some(gl_context_pair) = inner.gl_context_pair.as_ref() {
                gl_context_pair.backend.update();
            }
        }

        let frame = unsafe { NSView::frame(this as *mut _) };
        let backing_frame = unsafe { NSView::convertRectToBacking(this as *mut _, frame) };
        let width = backing_frame.size.width;
        let height = backing_frame.size.height;
        if let Some(this) = Self::get_this(this) {
            let mut inner = this.inner.borrow_mut();

            // This is a little gross; ideally we'd call
            // WindowInner:is_fullscreen to determine this, but
            // we can't get a mutable reference to it from here
            // as we can be called in a context where something
            // higher up the callstack already has a mutable
            // reference and we'd panic.
            let is_full_screen = inner.fullscreen.is_some()
                || inner.window.as_ref().map_or(false, |window| {
                    let window = window.load();
                    let style_mask = unsafe { NSWindow::styleMask(*window) };
                    style_mask.contains(NSWindowStyleMask::NSFullScreenWindowMask)
                });

            let live_resizing = inner.live_resizing;

            // Note: isZoomed can falsely return YES in situations such as
            // the current screen changing. We cannot detect that case here.
            // There is some logic to compensate for this in
            // wezterm-gui/src/termwindow/resize.rs.
            // <https://github.com/wezterm/wezterm/issues/3503>
            let is_zoomed = !is_full_screen
                && inner.window.as_ref().map_or(false, |window| {
                    let window = window.load();
                    unsafe { msg_send![*window, isZoomed] }
                });

            let window_level = inner
                .window
                .as_ref()
                .map(|window| {
                    let level = unsafe { window.load().level() };
                    nswindow_level_to_window_level(level)
                })
                .unwrap_or_default();

            let level_state = match window_level {
                WindowLevel::AlwaysOnBottom => WindowState::ALWAYS_ON_BOTTOM,
                WindowLevel::AlwaysOnTop => WindowState::ALWAYS_ON_TOP,
                WindowLevel::Normal => WindowState::default(),
            };

            let screen_state = match (is_full_screen, is_zoomed) {
                (true, _) => WindowState::FULL_SCREEN,
                (_, true) => WindowState::MAXIMIZED,
                _ => WindowState::default(),
            };

            let dpi = inner
                .window
                .as_ref()
                .and_then(|window| {
                    let window = window.load();
                    dpi_for_window_screen(*window, &inner.config)
                })
                .unwrap_or(crate::DEFAULT_DPI * (backing_frame.size.width / frame.size.width))
                as usize;

            inner.events.dispatch(WindowEvent::Resized {
                dimensions: Dimensions {
                    pixel_width: width as usize,
                    pixel_height: height as usize,
                    dpi,
                },
                window_state: screen_state | level_state,
                live_resizing,
            });
        }
    }

    extern "C" fn update_layer(_view: &mut Object, _sel: Sel) {
        log::trace!("update_layer called");
    }

    extern "C" fn wants_update_layer(_view: &mut Object, _sel: Sel) -> BOOL {
        log::trace!("wants_update_layer called");
        YES
    }

    extern "C" fn display_layer(view: &mut Object, sel: Sel, _layer_id: id) {
        Self::draw_rect(
            view,
            sel,
            NSRect::new(NSPoint::new(0., 0.), NSSize::new(0., 0.)),
        )
    }

    extern "C" fn draw_layer_in_context(
        _view: &mut Object,
        _sel: Sel,
        _layer_id: id,
        _context: id,
    ) {
    }

    extern "C" fn layer_should_inherit_contents_scale_from_window(
        _: &Object,
        _: Sel,
        layer: *mut Object,
        _: CGFloat,
        _: *mut Object,
    ) -> BOOL {
        log::trace!("layer_should_inherit_contents_scale_from_window");
        unsafe {
            let () = msg_send![layer, setContentsScale: 1.0];
        }
        YES
    }

    extern "C" fn make_backing_layer(view: &mut Object, _: Sel) -> id {
        log::trace!("make_backing_layer");
        let class = class!(CAMetalLayer);
        unsafe {
            // Use type method to get a instance of CAMetalLayer.
            // So that we don't have to worry about retaining/releasing it.
            let layer: id = msg_send![class, layer];
            let () = msg_send![layer, setDelegate: view];
            let () = msg_send![layer, setContentsScale: 1.0];
            let () = msg_send![layer, setOpaque: NO];
            layer
        }
    }

    extern "C" fn draw_rect(view: &mut Object, sel: Sel, _dirty_rect: NSRect) {
        if let Some(this) = Self::get_this(view) {
            let mut inner = this.inner.borrow_mut();

            if inner.screen_changed {
                // If the screen resolution changed (which can also
                // happen if the window was dragged to another monitor
                // with different dpi), then we treat this as a resize
                // event that will in turn trigger an invalidation
                // and a repaint.
                inner.screen_changed = false;
                drop(inner);
                Self::did_resize(view, sel, nil);
                return;
            }

            if inner.paint_throttled {
                inner.invalidated = true;
            } else {
                inner.events.dispatch(WindowEvent::NeedRepaint);
                inner.invalidated = false;
                inner.paint_throttled = true;

                let window_id = inner.window_id;
                let max_fps = inner.config.max_fps;
                promise::spawn::spawn(async move {
                    async_io::Timer::after(std::time::Duration::from_millis(1000 / max_fps as u64))
                        .await;
                    Connection::with_window_inner(window_id, move |inner| {
                        if let Some(window_view) = WindowView::get_this(unsafe { &**inner.view }) {
                            let mut state = window_view.inner.borrow_mut();
                            state.paint_throttled = false;
                            if state.invalidated {
                                unsafe {
                                    let () = msg_send![*inner.view, setNeedsDisplay: YES];
                                }
                            }
                        }
                        Ok(())
                    });
                })
                .detach();
            }
        }
    }

    extern "C" fn dragging_entered(this: &mut Object, _: Sel, sender: id) -> BOOL {
        if let Some(this) = Self::get_this(this) {
            let mut inner = this.inner.borrow_mut();

            let pb: id = unsafe { msg_send![sender, draggingPasteboard] };
            if pb.is_null() {
                return NO;
            }

            let filenames =
                unsafe { NSPasteboard::propertyListForType(pb, appkit::NSFilenamesPboardType) };
            if filenames.is_null() {
                return NO;
            }

            let paths = unsafe { filenames.iter() }
                .map(|file| unsafe {
                    let path = nsstring_to_str(file);
                    PathBuf::from(path)
                })
                .collect::<Vec<_>>();
            inner.events.dispatch(WindowEvent::DraggedFile(paths));
        }
        YES
    }

    extern "C" fn perform_drag_operation(this: &mut Object, _: Sel, sender: id) -> BOOL {
        if let Some(this) = Self::get_this(this) {
            let mut inner = this.inner.borrow_mut();

            let pb: id = unsafe { msg_send![sender, draggingPasteboard] };
            if pb.is_null() {
                return NO;
            }

            let filenames =
                unsafe { NSPasteboard::propertyListForType(pb, appkit::NSFilenamesPboardType) };
            if filenames.is_null() {
                return NO;
            }

            let paths = unsafe { filenames.iter() }
                .map(|file| unsafe {
                    let path = nsstring_to_str(file);
                    PathBuf::from(path)
                })
                .collect::<Vec<_>>();
            inner.events.dispatch(WindowEvent::DroppedFile(paths));
        }
        YES
    }

    fn get_this(this: &Object) -> Option<&mut Self> {
        unsafe {
            let myself: *mut c_void = *this.get_ivar(VIEW_CLS_NAME);
            if myself.is_null() {
                None
            } else {
                Some(&mut *(myself as *mut Self))
            }
        }
    }

    fn init_with_frame(inner: &Rc<RefCell<Inner>>, rect: NSRect) -> anyhow::Result<StrongPtr> {
        let cls = Self::get_class();

        let view_id: id = unsafe { msg_send![cls, alloc] };
        let view_id: StrongPtr = unsafe { StrongPtr::new(msg_send![view_id, initWithFrame:rect]) };
        inner.borrow_mut().view_id.replace(view_id.weak());

        let view = Box::into_raw(Box::new(Self {
            inner: Rc::clone(&inner),
        }));

        unsafe {
            (**view_id).set_ivar(VIEW_CLS_NAME, view as *mut c_void);
        }

        Ok(view_id)
    }

    fn get_class() -> &'static Class {
        Class::get(VIEW_CLS_NAME).unwrap_or_else(Self::define_class)
    }
}
