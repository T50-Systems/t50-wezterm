/// Convert from a macOS screen coordinate with the origin in the bottom left
/// to a pixel coordinate with its origin in the top left
fn cartesian_to_screen_point(cartesian: NSPoint) -> ScreenPoint {
    unsafe {
        let screens = NSScreen::screens(nil);
        let primary = screens.objectAtIndex(0);
        let frame = NSScreen::frame(primary);
        let backing_frame = NSScreen::convertRectToBacking_(primary, frame);
        let scale = backing_frame.size.height / frame.size.height;
        ScreenPoint::new(
            (cartesian.x * scale) as isize,
            ((frame.size.height - cartesian.y) * scale) as isize,
        )
    }
}

/// Convert from a pixel coordinate in the top left to a macOS screen
/// coordinate with its origin in the bottom left
fn screen_point_to_cartesian(point: ScreenPoint) -> NSPoint {
    unsafe {
        let screens = NSScreen::screens(nil);
        let primary = screens.objectAtIndex(0);
        let frame = NSScreen::frame(primary);
        let backing_frame = NSScreen::convertRectToBacking_(primary, frame);
        let scale = backing_frame.size.height / frame.size.height;
        NSPoint::new(
            point.x as f64 / scale,
            frame.size.height - (point.y as f64 / scale),
        )
    }
}

impl WindowInner {
    fn enable_opengl(&mut self) -> anyhow::Result<Rc<glium::backend::Context>> {
        if let Some(window_view) = WindowView::get_this(unsafe { &**self.view }) {
            window_view.inner.borrow_mut().enable_opengl()
        } else {
            anyhow::bail!("window invalid");
        }
    }

    fn is_fullscreen(&mut self) -> bool {
        if self.is_native_fullscreen() {
            true
        } else if let Some(window_view) = WindowView::get_this(unsafe { &**self.view }) {
            window_view.inner.borrow().fullscreen.is_some()
        } else {
            false
        }
    }

    fn apply_decorations(&mut self) {
        if !self.is_fullscreen() {
            apply_decorations_to_window(
                &self.window,
                self.config.window_decorations,
                self.config.integrated_title_button_style,
            );
        }
    }

    fn toggle_native_fullscreen(&mut self) {
        unsafe {
            NSWindow::toggleFullScreen_(*self.window, nil);
        }
    }

    fn is_native_fullscreen(&self) -> bool {
        let style_mask = unsafe { NSWindow::styleMask(*self.window) };
        style_mask.contains(NSWindowStyleMask::NSFullScreenWindowMask)
    }

    /// If we were in native full screen mode, exit it and return true.
    /// Otherwise, return false
    fn exit_native_fullscreen(&mut self) -> bool {
        if self.is_native_fullscreen() {
            self.toggle_native_fullscreen();
            true
        } else {
            false
        }
    }

    /// If we were in simple full screen mode, exit it and return true.
    /// Otherwise, return false
    fn exit_simple_fullscreen(&mut self) -> bool {
        if let Some(window_view) = WindowView::get_this(unsafe { &**self.view }) {
            let is_fullscreen = window_view.inner.borrow().fullscreen.is_some();
            if is_fullscreen {
                self.toggle_simple_fullscreen();
            }
            is_fullscreen
        } else {
            false
        }
    }

    fn toggle_simple_fullscreen(&mut self) {
        let current_app = unsafe { NSApplication::sharedApplication(nil) };

        if let Some(window_view) = WindowView::get_this(unsafe { &**self.view }) {
            let fullscreen = window_view.inner.borrow_mut().fullscreen.take();
            match fullscreen {
                Some(saved_rect) => unsafe {
                    // Restore prior dimensions
                    self.window.orderOut_(nil);
                    apply_decorations_to_window(
                        &self.window,
                        self.config.window_decorations,
                        self.config.integrated_title_button_style,
                    );
                    self.window.setFrame_display_(saved_rect, YES);
                    self.window.makeKeyAndOrderFront_(nil);
                    self.window.setOpaque_(NO);
                    current_app.setPresentationOptions_(
                        NSApplicationPresentationOptions::NSApplicationPresentationDefault,
                    );
                },
                None => unsafe {
                    // Go full screen
                    let saved_rect = NSWindow::frame(*self.window);
                    window_view
                        .inner
                        .borrow_mut()
                        .fullscreen
                        .replace(saved_rect);

                    let main_screen = NSScreen::mainScreen(nil);
                    let screen_rect = NSScreen::frame(main_screen);

                    self.window.orderOut_(nil);
                    self.window
                        .setStyleMask_(NSWindowStyleMask::NSBorderlessWindowMask);
                    self.window.setFrame_display_(screen_rect, YES);
                    self.window.makeKeyAndOrderFront_(nil);
                    self.window.setOpaque_(YES);
                    current_app.setPresentationOptions_(
                        NSApplicationPresentationOptions:: NSApplicationPresentationAutoHideMenuBar
                            | NSApplicationPresentationOptions::NSApplicationPresentationAutoHideDock
                    );
                },
            }
        }
    }

    fn update_window_shadow(&mut self) {
        let is_opaque = if self.config.window_background_opacity >= 1.0 {
            YES
        } else {
            NO
        };
        unsafe {
            self.window.setOpaque_(is_opaque);
            // when transparent, also turn off the window shadow,
            // because having the shadow enabled seems to correlate
            // with ghostly remnants see:
            // https://github.com/wezterm/wezterm/issues/310.
            // But allow overriding the shadows independent of opacity as well:
            // <https://github.com/wezterm/wezterm/issues/2669>
            let shadow = if self
                .config
                .window_decorations
                .contains(WindowDecorations::MACOS_FORCE_ENABLE_SHADOW)
            {
                YES
            } else if self
                .config
                .window_decorations
                .contains(WindowDecorations::MACOS_FORCE_DISABLE_SHADOW)
            {
                NO
            } else {
                is_opaque
            };
            self.window.setHasShadow_(shadow);
        }
    }

    fn update_titlebar_background(&self) {
        if !self
            .config
            .window_decorations
            .contains(WindowDecorations::MACOS_USE_BACKGROUND_COLOR_AS_TITLEBAR_COLOR)
        {
            return;
        }

        // Set the titlebar background to the theme color falling back to black if there is no
        // specified color scheme
        let color = self
            .config
            .resolved_palette
            .background
            .unwrap_or(RgbaColor::from(SrgbaTuple(0., 0., 0., 255.)));

        unsafe {
            if let Some(titlebar_view_container) = get_titlebar_view_container(&self.window) {
                let layer: id = msg_send![*titlebar_view_container.load(), layer];

                if layer.is_null() {
                    return;
                }

                // We need to make sure to convert the config color into an sRGB CGColor or the color will be slightly off
                let srgb_cgcolor = objc2_core_graphics::CGColor::new_srgb(
                    color.0.into(),
                    color.1.into(),
                    color.2.into(),
                    color.3.into(),
                );

                let _: () = msg_send![layer, setBackgroundColor: srgb_cgcolor];
            } else {
                log::trace!("failed to get titlebar view container from window");
            }
        }
    }

    fn update_window_background_blur(&mut self) {
        unsafe {
            CGSSetWindowBackgroundBlurRadius(
                CGSMainConnectionID(),
                self.window.windowNumber(),
                self.config.macos_window_background_blur,
            );
        }
    }
}

impl WindowInner {
    fn show(&mut self) {
        unsafe {
            let current_app = NSRunningApplication::currentApplication(nil);
            current_app.activateWithOptions_(NSApplicationActivateIgnoringOtherApps);

            // Stupid hack: adjust the window style mask and set it back
            // to what it was.
            // Without this, the CAMetalLayer used by webgpu seems to get
            // stuck with a scale factor of 2 despite us having configured 1.
            self.window
                .setStyleMask_(NSWindowStyleMask::NSBorderlessWindowMask);

            apply_decorations_to_window(
                &self.window,
                self.config.window_decorations,
                self.config.integrated_title_button_style,
            );

            self.update_titlebar_background();

            self.window.makeKeyAndOrderFront_(nil)
        }
    }

    fn close(&mut self) {
        unsafe {
            self.window.close();
        }
    }

    fn focus(&mut self) {
        unsafe {
            self.window.makeKeyAndOrderFront_(nil);
        }
    }

    fn hide(&mut self) {
        unsafe {
            NSWindow::miniaturize_(*self.window, *self.window);
            // We could literally set it invisible like this, but
            // then there is no UI to make it visible again later.
            //let () = msg_send![*self.window, setIsVisible: NO];
        }
    }

    fn set_cursor(&mut self, cursor: Option<MouseCursor>) {
        unsafe {
            let ns_cursor_cls = class!(NSCursor);
            if let Some(cursor) = cursor {
                // Unconditionally apply the requested cursor, as there are
                // cases where macOS can decide to change the cursor to something
                // that we don't know about.
                let instance: id = match cursor {
                    MouseCursor::Arrow => msg_send![ns_cursor_cls, arrowCursor],
                    MouseCursor::Text => msg_send![ns_cursor_cls, IBeamCursor],
                    MouseCursor::Hand => msg_send![ns_cursor_cls, pointingHandCursor],
                    MouseCursor::SizeUpDown => msg_send![ns_cursor_cls, resizeUpDownCursor],
                    MouseCursor::SizeLeftRight => msg_send![ns_cursor_cls, resizeLeftRightCursor],
                };
                let () = msg_send![ns_cursor_cls, setHiddenUntilMouseMoves: NO];
                let () = msg_send![instance, set];
            } else {
                let () = msg_send![ns_cursor_cls, setHiddenUntilMouseMoves: YES];
            }
        }
    }

    fn invalidate(&mut self) {
        unsafe {
            let () = msg_send![*self.view, setNeedsDisplay: YES];
            if let Some(window_view) = WindowView::get_this(&**self.view) {
                window_view.inner.borrow_mut().invalidated = true;
            }
        }
    }
    fn set_title(&mut self, title: &str) {
        let title = nsstring(title);
        unsafe {
            NSWindow::setTitle_(*self.window, *title);
        }
    }

    fn set_window_level(&mut self, level: WindowLevel) {
        unsafe {
            NSWindow::setLevel_(*self.window, window_level_to_nswindow_level(level));
            // Dispatch a resize event with the updated window state
            WindowView::did_resize(&mut **self.view, sel!(windowDidResize:), nil);
        }
    }

    fn set_inner_size(&mut self, width: usize, height: usize) {
        unsafe {
            let frame = NSView::frame(*self.view as *mut _);
            let backing_frame = NSView::convertRectToBacking(*self.view as *mut _, frame);
            let scale = backing_frame.size.width / frame.size.width;

            NSWindow::setContentSize_(
                *self.window,
                NSSize::new(width as f64 / scale, height as f64 / scale),
            );

            // setContentSize_ doesn't explicitly invalidate,
            // so we need to do it ourselves
            self.invalidate();
        }
    }

    fn set_window_position(&self, coords: ScreenPoint) {
        set_window_position(*self.window, coords);
    }

    fn set_text_cursor_position(&mut self, cursor: Rect) {
        if let Some(window_view) = WindowView::get_this(unsafe { &**self.view }) {
            window_view.inner.borrow_mut().text_cursor_position = cursor;
        }
        if self.config.use_ime {
            unsafe {
                let input_context: id = msg_send![&**self.view, inputContext];
                let () = msg_send![input_context, invalidateCharacterCoordinates];
            }
        }
    }

    fn is_zoomed(&self) -> bool {
        unsafe { msg_send![*self.window, isZoomed] }
    }

    fn maximize(&mut self) {
        if !self.is_zoomed() {
            unsafe {
                NSWindow::zoom_(*self.window, nil);
            }
        }
    }

    fn restore(&mut self) {
        if self.is_zoomed() {
            unsafe {
                NSWindow::zoom_(*self.window, nil);
            }
        }
    }

    fn toggle_fullscreen(&mut self) {
        let native_fullscreen = self.config.native_macos_fullscreen_mode;

        // If they changed their config since going full screen, be sure
        // to undo whichever fullscreen mode they had active rather than
        // trying to undo the one they have configured.

        if native_fullscreen {
            if !self.exit_simple_fullscreen() {
                self.toggle_native_fullscreen();
            }
        } else {
            if !self.exit_native_fullscreen() {
                self.toggle_simple_fullscreen();
            }
        }
    }

    fn set_resize_increments(&self, incr: ResizeIncrement) {
        let min_width = incr.base_width + incr.x;
        let min_height = incr.base_height + incr.y;
        unsafe {
            self.window
                .setResizeIncrements_(NSSize::new(incr.x.into(), incr.y.into()));
            let () = msg_send![
                *self.window,
                setContentMinSize: NSSize::new(min_width.into(), min_height.into())
            ];
        }
    }

    fn config_did_change(&mut self, config: &ConfigHandle) {
        let dpi_changed =
            self.config.dpi != config.dpi || self.config.dpi_by_screen != config.dpi_by_screen;

        self.config = config.clone();
        if let Some(window_view) = WindowView::get_this(unsafe { &**self.view }) {
            let mut inner = window_view.inner.borrow_mut();
            inner.config = config.clone();
            if dpi_changed {
                inner.screen_changed = true;
            }
        }
        self.update_window_shadow();
        self.update_window_background_blur();
        self.update_titlebar_background();
        self.apply_decorations();
    }
}
