impl WindowView {
    extern "C" fn dealloc(this: &mut Object, _sel: Sel) {
        Self::drop_inner(this);
        unsafe {
            let superclass = superclass(this);
            let () = msg_send![super(this, superclass), dealloc];
        }
    }

    fn drop_inner(this: &mut Object) {
        unsafe {
            let myself: *mut c_void = *this.get_ivar(VIEW_CLS_NAME);
            this.set_ivar(VIEW_CLS_NAME, std::ptr::null_mut() as *mut c_void);

            if !myself.is_null() {
                let myself = Box::from_raw(myself as *mut Self);
                drop(myself);
            }
        }
    }

    // Called by the inputContext manager when the IME processes events.
    // We need to translate the selector back into appropriate key
    // sequences
    extern "C" fn do_command_by_selector(this: &mut Object, _sel: Sel, a_selector: Sel) {
        let selector = format!("{:?}", a_selector);
        log::trace!("do_command_by_selector {:?}", selector);

        if let Some(myself) = Self::get_this(this) {
            let mut inner = myself.inner.borrow_mut();
            inner.ime_state = ImeDisposition::Continue;
            inner.ime_last_event.take();
        }
    }

    extern "C" fn has_marked_text(this: &mut Object, _sel: Sel) -> BOOL {
        if let Some(myself) = Self::get_this(this) {
            let inner = myself.inner.borrow();
            if inner.ime_text.is_empty() {
                NO
            } else {
                YES
            }
        } else {
            NO
        }
    }

    extern "C" fn marked_range(this: &mut Object, _sel: Sel) -> NSRange {
        if let Some(myself) = Self::get_this(this) {
            let inner = myself.inner.borrow();
            log::trace!("marked_range {:?}", inner.ime_text);
            if inner.ime_text.is_empty() {
                NSRange::new(NSNotFound as _, 0)
            } else {
                NSRange::new(0, inner.ime_text.len() as u64)
            }
        } else {
            NSRange::new(NSNotFound as _, 0)
        }
    }

    extern "C" fn selected_range(_this: &mut Object, _sel: Sel) -> NSRange {
        NSRange::new(NSNotFound as _, 0)
    }

    // Called by the IME when inserting composed text and/or emoji
    extern "C" fn insert_text_replacement_range(
        this: &mut Object,
        _sel: Sel,
        astring: id,
        replacement_range: NSRange,
    ) {
        let s = unsafe { nsstring_to_str(astring) };
        log::trace!(
            "insert_text_replacement_range {} {:?}",
            s,
            replacement_range
        );
        if let Some(myself) = Self::get_this(this) {
            let mut inner = myself.inner.borrow_mut();

            let key_is_down = inner.key_is_down.take().unwrap_or(true);

            let key = KeyCode::composed(s);

            let event = KeyEvent {
                key,
                modifiers: Modifiers::NONE,
                leds: KeyboardLedStatus::empty(),
                repeat_count: 1,
                key_is_down,
                raw: None,
            };

            inner.ime_text.clear();
            inner
                .events
                .dispatch(WindowEvent::AdviseDeadKeyStatus(DeadKeyStatus::None));
            inner.ime_last_event.replace(event.clone());
            inner.events.dispatch(WindowEvent::KeyEvent(event));
            inner.ime_state = ImeDisposition::Acted;
        }
    }

    extern "C" fn set_marked_text_selected_range_replacement_range(
        this: &mut Object,
        _sel: Sel,
        astring: id,
        selected_range: NSRange,
        replacement_range: NSRange,
    ) {
        let s = unsafe { nsstring_to_str(astring) };
        log::trace!(
            "set_marked_text_selected_range_replacement_range {} {:?} {:?}",
            s,
            selected_range,
            replacement_range
        );
        if let Some(myself) = Self::get_this(this) {
            let mut inner = myself.inner.borrow_mut();
            inner.ime_text = s.to_string();

            /*
            let key_is_down = inner.key_is_down.take().unwrap_or(true);

            let key = KeyCode::composed(s);

            let event = KeyEvent {
                key,
                modifiers: Modifiers::NONE,
                repeat_count: 1,
                key_is_down,
            }
            .normalize_shift();

            inner.ime_last_event.replace(event.clone());
            inner.events.dispatch(WindowEvent::KeyEvent(event));
            */
            inner.ime_last_event.take();
            inner.ime_state = ImeDisposition::Acted;
        }
    }

    extern "C" fn unmark_text(this: &mut Object, _sel: Sel) {
        log::trace!("unmarkText");
        if let Some(myself) = Self::get_this(this) {
            let mut inner = myself.inner.borrow_mut();
            // FIXME: docs say to insert the text here,
            // but iterm doesn't... and we've never seen
            // this get called so far?
            inner.ime_text.clear();
            inner.ime_last_event.take();
            inner.ime_state = ImeDisposition::Acted;
        }
    }

    extern "C" fn valid_attributes_for_marked_text(_this: &mut Object, _sel: Sel) -> id {
        // FIXME: returns NSArray<NSAttributedStringKey> *
        // log::trace!("valid_attributes_for_marked_text");
        // nil
        unsafe { NSArray::arrayWithObjects(nil, &[]) }
    }

    extern "C" fn attributed_substring_for_proposed_range(
        _this: &mut Object,
        _sel: Sel,
        _proposed_range: NSRange,
        _actual_range: NSRangePointer,
    ) -> id {
        log::trace!(
            "attributedSubstringForProposedRange {:?} {:?}",
            _proposed_range,
            _actual_range
        );
        nil
    }

    extern "C" fn character_index_for_point(
        _this: &mut Object,
        _sel: Sel,
        _point: NSPoint,
    ) -> NSUInteger {
        NSNotFound as _
    }

    extern "C" fn first_rect_for_character_range(
        this: &mut Object,
        _sel: Sel,
        range: NSRange,
        actual: NSRangePointer,
    ) -> NSRect {
        // Returns a rect in screen coordinates; this is used to place
        // the input method editor
        log::trace!(
            "firstRectForCharacterRange: range:{:?} actual:{:?}",
            range,
            actual
        );
        let window: id = unsafe { msg_send![this, window] };
        let frame = unsafe { NSWindow::frame(window) };
        let content: NSRect = unsafe { msg_send![window, contentRectForFrameRect: frame] };
        let backing_frame: NSRect = unsafe { msg_send![this, convertRectToBacking: frame] };
        let scale = frame.size.width / backing_frame.size.width;

        if let Some(this) = Self::get_this(this) {
            let cursor_pos = this
                .inner
                .borrow()
                .text_cursor_position
                .to_f64()
                .scale(scale, scale);

            NSRect::new(
                NSPoint::new(
                    content.origin.x + cursor_pos.min_x(),
                    content.origin.y + content.size.height - cursor_pos.max_y(),
                ),
                NSSize::new(cursor_pos.size.width, cursor_pos.size.height),
            )
        } else {
            frame
        }
    }

    extern "C" fn accepts_first_mouse(_this: &mut Object, _sel: Sel, _nsevent: id) -> BOOL {
        YES
    }

    extern "C" fn accepts_first_responder(_this: &mut Object, _sel: Sel) -> BOOL {
        YES
    }

    extern "C" fn view_did_change_effective_appearance(this: &mut Object, _sel: Sel) {
        if let Some(this) = Self::get_this(this) {
            let appearance = Connection::get().unwrap().get_appearance();
            this.inner
                .borrow_mut()
                .events
                .dispatch(WindowEvent::AppearanceChanged(appearance));
        }
    }

    extern "C" fn update_tracking_areas(this: &mut Object, _sel: Sel) {
        let frame = unsafe { NSView::frame(this as *mut _) };

        if let Some(this) = Self::get_this(this) {
            let mut inner = this.inner.borrow_mut();
            if let Some(ref view) = inner.view_id {
                let view = view.load();
                if view.is_null() {
                    return;
                }

                let tag = inner.tracking_rect_tag;
                if tag != 0 {
                    unsafe {
                        let () = msg_send![*view, removeTrackingRect: tag];
                    }
                }

                let rect = NSRect::new(
                    NSPoint::new(0.0, 0.0),
                    NSSize::new(frame.size.width, frame.size.height),
                );
                inner.tracking_rect_tag = unsafe {
                    msg_send![*view, addTrackingRect: rect owner: *view userData: nil assumeInside: NO]
                };
            }
        }
    }

    extern "C" fn window_should_close(this: &mut Object, _sel: Sel, _id: id) -> BOOL {
        unsafe {
            let () = msg_send![this, setNeedsDisplay: YES];
        }

        if let Some(this) = Self::get_this(this) {
            this.inner
                .borrow_mut()
                .events
                .dispatch(WindowEvent::CloseRequested);
            NO
        } else {
            YES
        }
    }
}
