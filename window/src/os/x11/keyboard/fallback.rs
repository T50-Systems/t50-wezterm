
impl KeyboardWithFallback {
    pub fn new(selected: Keyboard) -> anyhow::Result<Self> {
        Ok(Self {
            selected,
            fallback: Keyboard::new_default()?,
        })
    }

    pub fn new_from_string(s: String) -> anyhow::Result<Self> {
        let selected = Keyboard::new_from_string(s)?;
        Self::new(selected)
    }

    pub fn process_wayland_key(
        &self,
        code: u32,
        pressed: bool,
        events: &mut WindowEventSender,
    ) -> Option<WindowKeyEvent> {
        let want_repeat = self.selected.wayland_key_repeats(code);
        let raw_modifiers = self.get_key_modifiers();
        self.process_key_event_impl(
            xkb::Keycode::new(code + 8),
            raw_modifiers,
            pressed,
            events,
            want_repeat,
        )
    }

    /// Compute the Modifier mask equivalent from the button mask
    /// provided in an XCB keyboard event
    fn modifiers_from_btn_mask(mask: xcb::x::KeyButMask) -> Modifiers {
        let mut res = Modifiers::default();
        if mask.contains(xcb::x::KeyButMask::SHIFT) {
            res |= Modifiers::SHIFT;
        }
        if mask.contains(xcb::x::KeyButMask::CONTROL) {
            res |= Modifiers::CTRL;
        }
        if mask.contains(xcb::x::KeyButMask::MOD1) {
            res |= Modifiers::ALT;
        }
        if mask.contains(xcb::x::KeyButMask::MOD4) {
            res |= Modifiers::SUPER;
        }
        res
    }

    pub fn process_key_press_event(
        &self,
        xcb_ev: &xcb::x::KeyPressEvent,
        events: &mut WindowEventSender,
    ) {
        let xcode = xkb::Keycode::from(xcb_ev.detail());
        self.process_xcb_key_event_impl(xcode, xcb_ev.state(), true, events);
    }

    pub fn process_key_release_event(
        &self,
        xcb_ev: &xcb::x::KeyReleaseEvent,
        events: &mut WindowEventSender,
    ) {
        let xcode = xkb::Keycode::from(xcb_ev.detail());
        self.process_xcb_key_event_impl(xcode, xcb_ev.state(), false, events);
    }

    // for X11 we always pass down raw_modifiers from the incoming
    // key event to use in preference to whatever is computed by XKB.
    // The reason is that the update_state() call triggered by the XServer
    // doesn't know about state managed by the IME, so we cannot trust
    // that the modifiers are right.
    // <https://github.com/ibus/ibus/issues/2600#issuecomment-1904322441>
    //
    // As part of this, we need to update the mask with the currently
    // known modifiers in order for automation scenarios to work out:
    // <https://github.com/fcitx/fcitx5/issues/893>
    // <https://github.com/wezterm/wezterm/issues/4615>
    fn process_xcb_key_event_impl(
        &self,
        xcode: xkb::Keycode,
        state: KeyButMask,
        pressed: bool,
        events: &mut WindowEventSender,
    ) -> Option<WindowKeyEvent> {
        // extract current modifiers
        let event_modifiers = Self::modifiers_from_btn_mask(state);
        // take the raw modifier mask
        let raw_mod_mask = state.bits();
        // and apply it to the underlying state so that eg: shifted keys
        // are correctly represented
        self.merge_current_xcb_modifiers(raw_mod_mask);

        // now do the regular processing
        let result = self.process_key_event_impl(xcode, event_modifiers, pressed, events, false);

        // and restore the prior modifier state
        self.reapply_last_xcb_state();

        result
    }

    fn process_key_event_impl(
        &self,
        xcode: xkb::Keycode,
        raw_modifiers: Modifiers,
        pressed: bool,
        events: &mut WindowEventSender,
        want_repeat: bool,
    ) -> Option<WindowKeyEvent> {
        let phys_code = self.selected.phys_code_map.borrow().get(&xcode).copied();

        let leds = self.get_led_status();

        let xsym = self.selected.state.borrow().key_get_one_sym(xcode);
        let fallback_xsym = self.fallback.state.borrow().key_get_one_sym(xcode);
        let handled = Handled::new();

        let raw_key_event = RawKeyEvent {
            key: match phys_code {
                Some(phys) => KeyCode::Physical(phys),
                None => KeyCode::RawCode(xcode.into()),
            },
            phys_code,
            raw_code: xcode.into(),
            modifiers: raw_modifiers,
            leds,
            repeat_count: 1,
            key_is_down: pressed,
            handled: handled.clone(),
        };

        let mut kc = None;

        let ksym = if pressed {
            events.dispatch(WindowEvent::RawKeyEvent(raw_key_event.clone()));
            if handled.is_handled() {
                self.selected.compose_clear();
                self.fallback.compose_clear();
                log::trace!("process_key_event: raw key was handled; not processing further");

                if want_repeat {
                    return Some(WindowKeyEvent::RawKeyEvent(raw_key_event));
                }
                return None;
            }

            let fallback_feed = self.fallback.compose_feed(xcode, fallback_xsym);
            let selected_feed = self.selected.compose_feed(xcode, xsym);

            match selected_feed {
                FeedResult::Composing(composition) => {
                    log::trace!(
                        "process_key_event: RawKeyEvent FeedResult::Composing: {:?}",
                        composition
                    );
                    events.dispatch(WindowEvent::AdviseDeadKeyStatus(DeadKeyStatus::Composing(
                        composition,
                    )));
                    return None;
                }
                FeedResult::Composed(utf8, sym) => {
                    if !utf8.is_empty() {
                        kc.replace(crate::KeyCode::composed(&utf8));
                    }
                    log::trace!(
                        "process_key_event: RawKeyEvent FeedResult::Composed: \
                                {utf8:?}, {sym:?}. kc -> {kc:?}",
                    );
                    events.dispatch(WindowEvent::AdviseDeadKeyStatus(DeadKeyStatus::None));
                    sym
                }
                FeedResult::Nothing(utf8, sym) => {
                    // Composition had no special expansion.
                    // Xkb will return a textual representation of the key even when
                    // it is not generally useful; for example, when CTRL, ALT or SUPER
                    // are held, we don't want its mapping as it can be counterproductive:
                    // CTRL-<ALPHA> is helpfully encoded in the form that we would
                    // send to the terminal, however, we do want the chance to
                    // distinguish between eg: CTRL-i and Tab.
                    //
                    // This logic excludes that textual expansion for this situation.
                    //
                    // <https://github.com/wezterm/wezterm/issues/1851>
                    // <https://github.com/wezterm/wezterm/issues/2845>

                    if !utf8.is_empty()
                        && !raw_modifiers
                            .intersects(Modifiers::CTRL | Modifiers::ALT | Modifiers::SUPER)
                    {
                        kc.replace(crate::KeyCode::composed(&utf8));
                    }

                    log::trace!(
                        "process_key_event: RawKeyEvent FeedResult::Nothing: \
                         {utf8:?}, {sym:?}. kc -> {kc:?} fallback_feed={fallback_feed:?}"
                    );

                    let key_code_from_sym =
                        keysym_to_keycode(sym.into()).or_else(|| keysym_to_keycode(xsym.into()));

                    // If we have a modified key, and its expansion is non-ascii, such as cyrillic
                    // "Es" (which appears visually similar to "c" in latin texts), then consider
                    // this key expansion against the default latin layout.
                    // This allows "CTRL-C" to work for users of cyrillic layouts

                    if kc.is_none()
                        && raw_modifiers
                            .intersects(Modifiers::CTRL | Modifiers::ALT | Modifiers::SUPER)
                    {
                        match key_code_from_sym {
                            Some(crate::KeyCode::Char(c)) if !c.is_ascii() => {
                                // Potentially a Cyrillic or other non-european layout.
                                // Consider shortcuts like CTRL-C against the default
                                // latin layout
                                match fallback_feed {
                                    FeedResult::Nothing(_fb_utf8, fb_sym) => {
                                        log::trace!(
                                            "process_key_event: RawKeyEvent using fallback \
                                             sym {fb_sym:?} because layout would expand to \
                                             non-ascii text {c:?}"
                                        );
                                        fb_sym
                                    }
                                    _ => sym,
                                }
                            }
                            _ => sym,
                        }
                    } else if kc.is_none() && key_code_from_sym.is_none() {
                        // Not sure if this is a good idea, see
                        // <https://github.com/wezterm/wezterm/issues/4910> for context.
                        match fallback_feed {
                            FeedResult::Nothing(_fb_utf8, fb_sym) => {
                                log::trace!(
                                    "process_key_event: RawKeyEvent using fallback \
                                     sym {fb_sym:?} because layout did not expand to \
                                     anything"
                                );
                                fb_sym
                            }
                            _ => sym,
                        }
                    } else {
                        sym
                    }
                }
                FeedResult::Cancelled => {
                    log::trace!("process_key_event: RawKeyEvent FeedResult::Cancelled");
                    events.dispatch(WindowEvent::AdviseDeadKeyStatus(DeadKeyStatus::None));
                    return None;
                }
            }
        } else {
            xsym
        };

        let kc = match kc {
            Some(kc) => kc,
            None => match keysym_to_keycode(ksym.into()).or_else(|| keysym_to_keycode(xsym.into()))
            {
                Some(kc) => kc,
                None => {
                    log::trace!("keysym_to_keycode for {:?} and {:?} -> None", ksym, xsym);
                    return None;
                }
            },
        };

        let event = KeyEvent {
            key: kc,
            leds,
            modifiers: raw_modifiers,
            repeat_count: 1,
            key_is_down: pressed,
            raw: Some(raw_key_event),
        }
        .normalize_shift()
        .resurface_positional_modifier_key();

        if pressed && want_repeat {
            events.dispatch(WindowEvent::KeyEvent(event.clone()));
            // Returns the event that should be repeated later
            Some(WindowKeyEvent::KeyEvent(event))
        } else {
            events.dispatch(WindowEvent::KeyEvent(event));
            None
        }
    }

    fn mod_is_active(&self, modifier: &str) -> bool {
        // [TODO] consider state  Depressed & consumed mods
        self.selected
            .state
            .borrow()
            .mod_name_is_active(modifier, xkb::STATE_MODS_EFFECTIVE)
    }
    fn led_is_active(&self, led: &str) -> bool {
        self.selected.state.borrow().led_name_is_active(led)
    }

    pub fn get_led_status(&self) -> KeyboardLedStatus {
        let mut leds = KeyboardLedStatus::empty();

        if self.led_is_active(xkb::LED_NAME_NUM) {
            leds |= KeyboardLedStatus::NUM_LOCK;
        }
        if self.led_is_active(xkb::LED_NAME_CAPS) {
            leds |= KeyboardLedStatus::CAPS_LOCK;
        }

        leds
    }

    pub fn get_key_modifiers(&self) -> Modifiers {
        let mut res = Modifiers::default();

        if self.mod_is_active(xkb::MOD_NAME_SHIFT) {
            res |= Modifiers::SHIFT;
        }
        if self.mod_is_active(xkb::MOD_NAME_CTRL) {
            res |= Modifiers::CTRL;
        }
        if self.mod_is_active(xkb::MOD_NAME_ALT) {
            // Mod1
            res |= Modifiers::ALT;
        }
        if self.mod_is_active(xkb::MOD_NAME_LOGO) {
            // Mod4
            res |= Modifiers::SUPER;
        }
        res
    }

    pub fn process_xkb_event(
        &self,
        connection: &xcb::Connection,
        event: &xcb::Event,
    ) -> anyhow::Result<Option<(Modifiers, KeyboardLedStatus)>> {
        let before = self.selected.mods_leds.borrow().clone();

        match event {
            xcb::Event::Xkb(xcb::xkb::Event::StateNotify(e)) => {
                self.update_state(e);
            }
            xcb::Event::Xkb(
                xcb::xkb::Event::MapNotify(_) | xcb::xkb::Event::NewKeyboardNotify(_),
            ) => {
                self.update_keymap(connection)?;
            }
            _ => {}
        }

        let after = (self.get_key_modifiers(), self.get_led_status());
        if after != before {
            *self.selected.mods_leds.borrow_mut() = after.clone();
            Ok(Some(after))
        } else {
            Ok(None)
        }
    }

    pub fn update_modifier_state(
        &self,
        mods_depressed: u32,
        mods_latched: u32,
        mods_locked: u32,
        group: u32,
    ) {
        self.selected
            .update_modifier_state(mods_depressed, mods_latched, mods_locked, group);
        self.fallback
            .update_modifier_state(mods_depressed, mods_latched, mods_locked, group);
    }

    pub fn update_state(&self, ev: &xcb::xkb::StateNotifyEvent) {
        self.selected.update_state(ev);
        self.fallback.update_state(ev);
    }

    pub fn reapply_last_xcb_state(&self) {
        self.selected.reapply_last_xcb_state();
        self.fallback.reapply_last_xcb_state();
    }

    pub fn merge_current_xcb_modifiers(&self, mods: ModMask) {
        self.selected.merge_current_xcb_modifiers(mods);
        self.fallback.merge_current_xcb_modifiers(mods);
    }

    pub fn update_keymap(&self, connection: &xcb::Connection) -> anyhow::Result<()> {
        self.selected.update_keymap(connection)
    }
