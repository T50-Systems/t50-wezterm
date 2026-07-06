impl super::TermWindow {
    pub fn raw_key_event_impl(&mut self, key: RawKeyEvent, context: &dyn WindowOps) {
        // The leader key is a kind of modal modifier key.
        // It is allowed to be active for up to the leader timeout duration,
        // after which it auto-deactivates.
        let (leader_active, leader_mod) = if self.leader_is_active_mut() {
            // Currently active
            (true, Modifiers::LEADER)
        } else {
            (false, Modifiers::NONE)
        };

        if self.config.debug_key_events {
            log::info!(
                "key_event {:?} {}",
                key,
                if leader_active { "LEADER" } else { "" }
            );
        } else {
            log::trace!(
                "key_event {:?} {}",
                key,
                if leader_active { "LEADER" } else { "" }
            );
        }

        let modifier_and_leds = (key.modifiers, key.leds);
        if self.current_modifier_and_leds != modifier_and_leds {
            self.current_modifier_and_leds = modifier_and_leds;
            self.schedule_next_status_update();
        }

        let pane = match self.get_active_pane_or_overlay() {
            Some(pane) => pane,
            None => return,
        };

        // First, try to match raw physical key
        let phys_key = match &key.key {
            phys @ KeyCode::Physical(_) => Some(phys.clone()),
            _ => key.phys_code.map(KeyCode::Physical),
        };

        if let Some(phys_key) = &phys_key {
            if self.process_key(
                &pane,
                context,
                &phys_key,
                key.modifiers,
                leader_active,
                leader_mod,
                OnlyKeyBindings::Yes,
                key.key_is_down,
                None,
            ) {
                key.set_handled();
                return;
            }
        }

        // Then try the raw code
        let raw_key = match &key.key {
            raw @ KeyCode::RawCode(_) => raw.clone(),
            _ => KeyCode::RawCode(key.raw_code),
        };
        if self.process_key(
            &pane,
            context,
            &raw_key,
            key.modifiers,
            leader_active,
            leader_mod,
            OnlyKeyBindings::Yes,
            key.key_is_down,
            None,
        ) {
            key.set_handled();
            return;
        }

        if phys_key.as_ref() == Some(&key.key) || raw_key == key.key {
            // We already matched against whatever key.key is, so no need
            // to do it again below
            return;
        }

        if self.process_key(
            &pane,
            context,
            &key.key,
            key.modifiers,
            leader_active,
            leader_mod,
            OnlyKeyBindings::Yes,
            key.key_is_down,
            None,
        ) {
            key.set_handled();
        }
    }

    pub fn current_modifier_and_led_state(&self) -> (Modifiers, KeyboardLedStatus) {
        self.current_modifier_and_leds
    }

    pub fn leader_is_active(&self) -> bool {
        match self.leader_is_down.as_ref() {
            Some(expiry) if *expiry > std::time::Instant::now() => {
                self.update_next_frame_time(Some(*expiry));
                true
            }
            Some(_) => false,
            None => false,
        }
    }

    pub fn leader_is_active_mut(&mut self) -> bool {
        match self.leader_is_down.as_ref() {
            Some(expiry) if *expiry > std::time::Instant::now() => {
                self.update_next_frame_time(Some(*expiry));
                true
            }
            Some(_) => {
                self.leader_done();
                false
            }
            None => false,
        }
    }

    pub fn current_key_table_name(&mut self) -> Option<String> {
        let mut name = None;

        if let Some(pane) = self.get_active_pane_or_overlay() {
            if let Some(overlay) = self.pane_state(pane.pane_id()).overlay.as_mut() {
                name = overlay
                    .key_table_state
                    .current_table()
                    .map(|s| s.to_string());

                if let Some(entry) = overlay.key_table_state.stack.last() {
                    if let Some(expiry) = entry.expiration {
                        self.update_next_frame_time(Some(expiry));
                    }
                }
            }
        }
        if name.is_none() {
            name = self.key_table_state.current_table().map(|s| s.to_string());
        }
        if let Some(entry) = self.key_table_state.stack.last() {
            if let Some(expiry) = entry.expiration {
                self.update_next_frame_time(Some(expiry));
            }
        }
        name
    }

    pub fn composition_status(&self) -> &DeadKeyStatus {
        &self.dead_key_status
    }

    fn leader_done(&mut self) {
        self.leader_is_down.take();
        self.update_title();
        if let Some(window) = &self.window {
            window.invalidate();
        }
    }

    pub fn key_event_impl(&mut self, window_key: KeyEvent, context: &dyn WindowOps) {
        let pane = match self.get_active_pane_or_overlay() {
            Some(pane) => pane,
            None => return,
        };

        // The leader key is a kind of modal modifier key.
        // It is allowed to be active for up to the leader timeout duration,
        // after which it auto-deactivates.
        let (leader_active, leader_mod) = if self.leader_is_active_mut() {
            // Currently active
            (true, Modifiers::LEADER)
        } else {
            (false, Modifiers::NONE)
        };

        if self.config.debug_key_events {
            log::info!(
                "key_event {:?} {}",
                window_key,
                if leader_active { "LEADER" } else { "" }
            );
        } else {
            log::trace!(
                "key_event {:?} {}",
                window_key,
                if leader_active { "LEADER" } else { "" }
            );
        }

        let modifiers = window_key.modifiers;

        if self.process_key(
            &pane,
            context,
            &window_key.key,
            window_key.modifiers,
            leader_active,
            leader_mod,
            OnlyKeyBindings::No,
            window_key.key_is_down,
            Some(&window_key),
        ) {
            return;
        }

        // If we get here, then none of the keys matched
        // any key table rules. Therefore, we should pop all `until_unknown`
        // entries from the stack.
        if window_key.key_is_down {
            self.key_table_state.pop_until_unknown();
        }

        let key = self.win_key_code_to_termwiz_key_code(&window_key.key);

        match key {
            Key::Code(key) => {
                if window_key.key_is_down && !key.is_modifier() {
                    if leader_active {
                        // Leader was pressed and this non-modifier keypress isn't
                        // a registered key binding; swallow this event and cancel
                        // the leader modifier.
                        self.leader_done();
                        return;
                    }
                    self.key_table_state.did_process_key();
                }

                if let Some(modal) = self.get_modal() {
                    if window_key.key_is_down {
                        modal.key_down(key, modifiers, self).ok();
                    }
                    return;
                }

                let res = if let Some(encoded) = self.encode_win32_input(&pane, &window_key) {
                    if self.config.debug_key_events {
                        log::info!("win32: Encoded input as {:?}", encoded);
                    }
                    pane.writer()
                        .write_all(encoded.as_bytes())
                        .context("sending win32-input-mode encoded data")
                } else if let Some(encoded) = self.encode_kitty_input(&pane, &window_key) {
                    if self.config.debug_key_events {
                        log::info!("kitty: Encoded input as {:?}", encoded);
                    }
                    pane.writer()
                        .write_all(encoded.as_bytes())
                        .context("sending kitty encoded data")
                } else {
                    if self.config.debug_key_events {
                        log::info!(
                            "send to pane {} key={:?} mods={:?}",
                            if window_key.key_is_down { "DOWN" } else { "UP" },
                            key,
                            modifiers
                        );
                    }

                    if window_key.key_is_down {
                        pane.key_down(key, modifiers)
                    } else {
                        pane.key_up(key, modifiers)
                    }
                };

                if res.is_ok() {
                    if window_key.key_is_down
                        && !key.is_modifier()
                        && self.pane_state(pane.pane_id()).overlay.is_none()
                    {
                        self.maybe_scroll_to_bottom_for_input(&pane);
                    }
                    if window_key.key_is_down
                        && self.config.hide_mouse_cursor_when_typing
                        && !key.is_modifier()
                    {
                        context.set_cursor(None);
                    }
                    if !key.is_modifier() {
                        context.invalidate();
                    }
                }
            }
            Key::Composed(s) => {
                if !window_key.key_is_down {
                    return;
                }
                if leader_active {
                    // Leader was pressed and this non-modifier keypress isn't
                    // a registered key binding; swallow this event and cancel
                    // the leader modifier.
                    self.leader_done();
                    return;
                }
                self.key_table_state.did_process_key();
                if self.config.debug_key_events {
                    log::info!("send to pane string={:?}", s);
                }
                pane.writer().write_all(s.as_bytes()).ok();
                self.maybe_scroll_to_bottom_for_input(&pane);
                context.invalidate();
            }
            Key::None => {}
        }
    }
}
