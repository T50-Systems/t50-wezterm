impl WindowView {
    /// Ensure that the menubar is shown when we transition from a fullscreen window
    /// to either a non-fullscreen window or no windows.
    /// Without this, we can end up in a state where the menu bar is invisible when
    /// it should otherwise be visible, and it is especially confusing when there
    /// are no windows.
    fn update_application_presentation(&self, is_key: bool) {
        let is_simple_full_screen;
        let native_full_screen;

        {
            let inner = self.inner.borrow();
            native_full_screen = inner.config.native_macos_fullscreen_mode;
            is_simple_full_screen = inner.fullscreen.is_some();
        }

        if !native_full_screen {
            let current_app = unsafe { NSApplication::sharedApplication(nil) };
            let target_options = match (is_key, is_simple_full_screen) {
                (true, true) => {
                    NSApplicationPresentationOptions::NSApplicationPresentationAutoHideMenuBar
                        | NSApplicationPresentationOptions::NSApplicationPresentationAutoHideDock
                }
                (true, false) | (false, _) => {
                    NSApplicationPresentationOptions::NSApplicationPresentationDefault
                }
            };
            unsafe {
                let current_options: NSApplicationPresentationOptions =
                    msg_send![current_app, presentationOptions];
                if current_options != target_options {
                    current_app.setPresentationOptions_(target_options);
                }
            }
        }
    }

    extern "C" fn did_become_key(this: &mut Object, _sel: Sel, _id: id) {
        if let Some(this) = Self::get_this(this) {
            this.inner
                .borrow_mut()
                .events
                .dispatch(WindowEvent::FocusChanged(true));
            this.update_application_presentation(true);
        }
    }

    extern "C" fn did_resign_key(this: &mut Object, _sel: Sel, _id: id) {
        if let Some(this) = Self::get_this(this) {
            this.inner
                .borrow_mut()
                .events
                .dispatch(WindowEvent::FocusChanged(false));
            this.update_application_presentation(true);
        }
    }

    // Switch the coordinate system to have 0,0 in the top left
    extern "C" fn is_flipped(_this: &Object, _sel: Sel) -> BOOL {
        YES
    }

    // Tell the window/view/layer stuff that we only have a single opaque
    // thing in the window so that it can optimize rendering
    extern "C" fn is_opaque(_this: &Object, _sel: Sel) -> BOOL {
        NO
    }

    // Don't use Cocoa native window tabbing
    extern "C" fn allow_automatic_tabbing(_this: &Object, _sel: Sel) -> BOOL {
        NO
    }

    extern "C" fn wezterm_perform_key_assignment(
        this: &mut Object,
        _sel: Sel,
        menu_item: *mut Object,
    ) {
        let menu_item = MenuItem::with_menu_item(menu_item);
        // Safe because weztermPerformKeyAssignment: is only used with KeyAssignment
        let action = menu_item.get_represented_item();
        log::debug!("wezterm_perform_key_assignment {action:?}",);
        match action {
            Some(RepresentedItem::KeyAssignment(action)) => {
                if let Some(this) = Self::get_this(this) {
                    this.inner
                        .borrow_mut()
                        .events
                        .dispatch(WindowEvent::PerformKeyAssignment(action));
                }
            }
            None => {}
        }
    }

    extern "C" fn window_will_close(this: &mut Object, _sel: Sel, _id: id) {
        if let Some(this) = Self::get_this(this) {
            // Advise the window of its impending death
            this.inner
                .borrow_mut()
                .events
                .dispatch(WindowEvent::Destroyed);
            this.update_application_presentation(false);
            let conn = Connection::get().unwrap();
            let window_id = this.inner.borrow_mut().window_id;
            conn.windows.borrow_mut().remove(&window_id);
        }
    }

    fn mouse_common(this: &mut Object, nsevent: id, kind: MouseEventKind) {
        let view = this as id;
        let coords;
        let mouse_buttons;
        let modifiers;
        let screen_coords;
        unsafe {
            let point = NSView::convertPoint_fromView_(view, nsevent.locationInWindow(), nil);
            let rect = NSRect::new(NSPoint::new(0., 0.), NSSize::new(point.x, point.y));
            let backing_rect = NSView::convertRectToBacking(view, rect);
            // backing_rect computes abs() values, so we need to restore the sign
            // from the original point
            coords = NSPoint::new(
                f64::copysign(backing_rect.size.width, point.x),
                f64::copysign(backing_rect.size.height, point.y),
            );
            mouse_buttons = decode_mouse_buttons(NSEvent::pressedMouseButtons(nsevent));
            modifiers = key_modifiers(nsevent.modifierFlags());
            screen_coords = NSEvent::mouseLocation(nsevent);
        }
        let event = MouseEvent {
            kind,
            coords: Point::new(coords.x as isize, coords.y as isize),
            screen_coords: cartesian_to_screen_point(screen_coords),
            mouse_buttons,
            modifiers,
        };

        if let Some(myself) = Self::get_this(this) {
            let mut inner = myself.inner.borrow_mut();
            inner.events.dispatch(WindowEvent::MouseEvent(event));
        }
    }

    extern "C" fn mouse_up(this: &mut Object, _sel: Sel, nsevent: id) {
        Self::mouse_common(this, nsevent, MouseEventKind::Release(MousePress::Left));
    }

    extern "C" fn mouse_down(this: &mut Object, _sel: Sel, nsevent: id) {
        Self::mouse_common(this, nsevent, MouseEventKind::Press(MousePress::Left));
    }
    extern "C" fn right_mouse_up(this: &mut Object, _sel: Sel, nsevent: id) {
        Self::mouse_common(this, nsevent, MouseEventKind::Release(MousePress::Right));
    }

    extern "C" fn other_mouse_up(this: &mut Object, _sel: Sel, nsevent: id) {
        // Safety: We know this is an button event
        unsafe {
            let button_number = NSEvent::buttonNumber(nsevent);
            // Button 2 is the middle mouse button (scroll wheel)
            // but is the dedicated middle mouse button on 4 button mouses
            if button_number == 2 {
                Self::mouse_common(this, nsevent, MouseEventKind::Release(MousePress::Middle));
            }
        }
    }

    extern "C" fn scroll_wheel(this: &mut Object, _sel: Sel, nsevent: id) {
        let precise = unsafe { nsevent.hasPreciseScrollingDeltas() } == YES;
        let scale = if precise {
            // Devices with precise deltas report number of pixels scrolled.
            // At this layer we don't know how many pixels comprise a cell
            // in the terminal widget, and our abstraction doesn't allow being
            // told what that amount should be, so we come up with a hard
            // coded factor based on the likely default font size and dpi
            // to make the scroll speed feel a bit better.
            15.0
        } else {
            // Whereas imprecise deltas report the number of lines scrolled,
            // so we want to report those lines here wholesale.
            1.0
        };
        let mut vert_delta = unsafe { nsevent.scrollingDeltaY() } / scale;
        let mut horz_delta = unsafe { nsevent.scrollingDeltaX() } / scale;

        if let Some(myself) = Self::get_this(this) {
            let mut inner = myself.inner.borrow_mut();

            let elapsed = inner.last_wheel.elapsed();

            // If it's been a while since the last wheel movement,
            // we want to clear out any accumulated fractional amount
            // and round this event up to 1 line so that we get an
            // immediate scroll on the first move.
            let stale = std::time::Duration::from_millis(250);
            if elapsed >= stale {
                if vert_delta != 0.0 && vert_delta.abs() < 1.0 {
                    vert_delta = round_away_from_zerof(vert_delta);
                }
                if horz_delta != 0.0 && horz_delta.abs() < 1.0 {
                    horz_delta = round_away_from_zerof(horz_delta);
                }
                inner.vscroll_remainder = 0.;
                inner.hscroll_remainder = 0.;
            }

            inner.last_wheel = Instant::now();

            // Reset remainder when changing scroll direction
            if vert_delta.signum() != inner.vscroll_remainder.signum() {
                inner.vscroll_remainder = 0.;
            }
            if horz_delta.signum() != inner.hscroll_remainder.signum() {
                inner.hscroll_remainder = 0.;
            }

            vert_delta += inner.vscroll_remainder;
            horz_delta += inner.hscroll_remainder;

            inner.vscroll_remainder = vert_delta.fract();
            inner.hscroll_remainder = horz_delta.fract();

            vert_delta = vert_delta.trunc();
            horz_delta = horz_delta.trunc();
        }

        if vert_delta.abs() < 1.0 && horz_delta.abs() < 1.0 {
            return;
        }

        let kind = if vert_delta.abs() > horz_delta.abs() {
            MouseEventKind::VertWheel(round_away_from_zero(vert_delta))
        } else {
            MouseEventKind::HorzWheel(round_away_from_zero(horz_delta))
        };
        Self::mouse_common(this, nsevent, kind);
    }

    extern "C" fn right_mouse_down(this: &mut Object, _sel: Sel, nsevent: id) {
        Self::mouse_common(this, nsevent, MouseEventKind::Press(MousePress::Right));
    }

    extern "C" fn other_mouse_down(this: &mut Object, _sel: Sel, nsevent: id) {
        // Safety: See `other_mouse_up`
        unsafe {
            let button_number = NSEvent::buttonNumber(nsevent);
            // See `other_mouse_up`
            if button_number == 2 {
                Self::mouse_common(this, nsevent, MouseEventKind::Press(MousePress::Middle));
            }
        }
    }

    extern "C" fn mouse_moved_or_dragged(this: &mut Object, _sel: Sel, nsevent: id) {
        Self::mouse_common(this, nsevent, MouseEventKind::Move);
    }

    extern "C" fn mouse_exited(this: &mut Object, _sel: Sel, _nsevent: id) {
        if let Some(myself) = Self::get_this(this) {
            myself
                .inner
                .borrow_mut()
                .events
                .dispatch(WindowEvent::MouseLeave);
        }
    }
}
