impl Window {
    pub async fn new_window<F>(
        _class_name: &str,
        name: &str,
        geometry: RequestedWindowGeometry,
        config: Option<&ConfigHandle>,
        _font_config: Rc<FontConfiguration>,
        event_handler: F,
    ) -> anyhow::Result<Window>
    where
        F: 'static + FnMut(WindowEvent, &Window),
    {
        let config = match config {
            Some(c) => c.clone(),
            None => config::configuration(),
        };

        let conn = Connection::get().expect("new_window called on gui thread");
        let ResolvedGeometry {
            width,
            height,
            x,
            y,
        } = conn.resolve_geometry(geometry);

        let scale_factor = (conn.default_dpi() / crate::DEFAULT_DPI) as usize;
        let width = width / scale_factor;
        let height = height / scale_factor;
        let x = x.map(|x| x / scale_factor as i32);
        let y = y.map(|y| y / scale_factor as i32);

        let initial_pos = match (x, y) {
            (Some(x), Some(y)) => Some(ScreenPoint::new(x as isize, y as isize)),
            _ => None,
        };

        unsafe {
            let style_mask = decoration_to_mask(
                config.window_decorations,
                config.integrated_title_button_style,
            );
            let rect = NSRect::new(
                NSPoint::new(0., 0.),
                NSSize::new(width as f64, height as f64),
            );

            let conn = Connection::get().expect("Connection::init has not been called");

            let window_id = conn.next_window_id();
            let events = WindowEventSender::new(event_handler);

            let inner = Rc::new(RefCell::new(Inner {
                events,
                view_id: None,
                window_id,
                window: None,
                screen_changed: false,
                paint_throttled: false,
                invalidated: true,
                gl_context_pair: None,
                text_cursor_position: Rect::new(Point::new(0, 0), Size::new(0, 0)),
                tracking_rect_tag: 0,
                hscroll_remainder: 0.,
                vscroll_remainder: 0.,
                last_wheel: Instant::now(),
                key_is_down: None,
                dead_pending: None,
                fullscreen: None,
                config: config.clone(),
                ime_state: ImeDisposition::None,
                ime_last_event: None,
                live_resizing: false,
                ime_text: String::new(),
            }));

            let window: id = msg_send![get_window_class(), alloc];
            let window = StrongPtr::new(NSWindow::initWithContentRect_styleMask_backing_defer_(
                window,
                rect,
                style_mask,
                NSBackingStoreBuffered,
                NO,
            ));

            apply_decorations_to_window(
                &window,
                config.window_decorations,
                config.integrated_title_button_style,
            );

            // Prevent Cocoa native tabs from being used
            let _: () = msg_send![*window, setTabbingMode:2 /* NSWindowTabbingModeDisallowed */];
            let _: () = msg_send![*window, setRestorable: NO];

            window.setReleasedWhenClosed_(NO);
            window.setBackgroundColor_(cocoa::appkit::NSColor::clearColor(nil));

            // Tell Cocoa that we output in sRGB, so it handles color space
            // conversion for non-sRGB displays.
            window.setColorSpace_(cocoa::appkit::NSColorSpace::sRGBColorSpace(nil));

            // We could set this, but it makes the entire window, including
            // its titlebar, opaque to this fixed degree.
            // window.setAlphaValue_(0.4);

            // Window positioning: the first window opens up in the center of
            // the screen.  Subsequent windows will be offset from the position
            // of the prior window at the time it was created.  It's not a
            // perfect algorithm by any means, and doesn't take in account
            // windows moving and closing since the last creation, but it is
            // better than creating them all centered which is what we used
            // to do here.
            thread_local! {
                static LAST_POSITION: RefCell<Option<NSPoint>> = RefCell::new(None);
            }

            let frame = NSWindow::frame(*window);
            let active_screen = NSScreen::mainScreen(nil);
            let active_screen_frame = NSScreen::frame(active_screen);

            fn point_in_rect(pt: NSPoint, rect: NSRect) -> bool {
                let rect: euclid::Rect<f64, ()> = euclid::rect(
                    rect.origin.x,
                    rect.origin.y,
                    rect.size.width,
                    rect.size.height,
                );
                rect.contains(euclid::point2(pt.x, pt.y))
            }

            LAST_POSITION.with(|last_pos| {
                if let Some(pos) = initial_pos {
                    // Put it where they asked it to be, without influencing
                    // future positioning info
                    set_window_position(*window, pos);
                    return;
                }
                let pos = last_pos.borrow_mut().take();
                let next_pos = match pos {
                    Some(pos) if point_in_rect(pos, active_screen_frame) => {
                        // Only continue the cascade if the prior point is
                        // still within the currently active screen
                        window.cascadeTopLeftFromPoint_(pos)
                    }
                    _ => {
                        // Otherwise, position as if it is the first time
                        // we're displaying on this screen
                        window.center();
                        window.cascadeTopLeftFromPoint_(frame.origin)
                    }
                };
                last_pos.borrow_mut().replace(next_pos);
            });

            window.setTitle_(*nsstring(&name));
            window.setAcceptsMouseMovedEvents_(YES);

            let view = WindowView::init_with_frame(&inner, rect)?;
            view.setAutoresizingMask_(NSViewHeightSizable | NSViewWidthSizable);

            let () = msg_send![
                *view,
                setLayerContentsPlacement: NSViewLayerContentsPlacementTopLeft
            ];

            CGSSetWindowBackgroundBlurRadius(
                CGSMainConnectionID(),
                window.windowNumber(),
                config.macos_window_background_blur,
            );
            window.setContentView_(*view);
            window.setDelegate_(*view);

            view.setWantsLayer(YES);
            let () = msg_send![
                *view,
                setLayerContentsRedrawPolicy: NSViewLayerContentsRedrawDuringViewResize
            ];

            // register for drag and drop operations.
            let () = msg_send![
                *window,
                registerForDraggedTypes:
                    NSArray::arrayWithObject(nil, appkit::NSFilenamesPboardType)
            ];

            let frame = NSView::frame(*view);
            let backing_frame = NSView::convertRectToBacking(*view, frame);
            let width = backing_frame.size.width;
            let height = backing_frame.size.height;

            let dpi = dpi_for_window_screen(*window, &config)
                .unwrap_or(crate::DEFAULT_DPI * (backing_frame.size.width / frame.size.width))
                as usize;

            let weak_window = window.weak();
            let window_handle = Window {
                id: window_id,
                ns_window: *window,
                ns_view: *view,
            };
            let window_inner = Rc::new(RefCell::new(WindowInner {
                window,
                view,
                config: config.clone(),
            }));
            inner.borrow_mut().window.replace(weak_window);
            conn.windows
                .borrow_mut()
                .insert(window_id, Rc::clone(&window_inner));

            inner
                .borrow_mut()
                .events
                .assign_window(window_handle.clone());

            window_handle.config_did_change(&config);

            // Synthesize a resize event immediately; this allows
            // the embedding application an opportunity to discover
            // the dpi and adjust for display scaling
            inner.borrow_mut().events.dispatch(WindowEvent::Resized {
                dimensions: Dimensions {
                    pixel_width: width as usize,
                    pixel_height: height as usize,
                    dpi,
                },
                window_state: WindowState::default(),
                live_resizing: false,
            });

            Ok(window_handle)
        }
    }
}
