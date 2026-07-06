impl super::TermWindow {
    fn encode_win32_input(&self, pane: &Arc<dyn Pane>, key: &KeyEvent) -> Option<String> {
        if !self.config.allow_win32_input_mode
            || pane.get_keyboard_encoding() != KeyboardEncoding::Win32
        {
            return None;
        }
        key.encode_win32_input_mode()
    }

    fn encode_kitty_input(&self, pane: &Arc<dyn Pane>, key: &KeyEvent) -> Option<String> {
        if !self.config.enable_kitty_keyboard {
            return None;
        }
        if let KeyboardEncoding::Kitty(flags) = pane.get_keyboard_encoding() {
            Some(key.encode_kitty(flags))
        } else {
            None
        }
    }

    fn lookup_key(
        &mut self,
        pane: &Arc<dyn Pane>,
        keycode: &KeyCode,
        mods: Modifiers,
        only_key_bindings: OnlyKeyBindings,
    ) -> Option<(KeyTableEntry, Option<String>)> {
        if let Some(overlay) = self.pane_state(pane.pane_id()).overlay.as_mut() {
            if let Some((entry, table_name)) = overlay.key_table_state.lookup_key(
                &self.input_map,
                keycode,
                mods,
                only_key_bindings,
            ) {
                return Some((entry, table_name.map(|s| s.to_string())));
            }
        }
        if let Some((entry, table_name)) =
            self.key_table_state
                .lookup_key(&self.input_map, keycode, mods, only_key_bindings)
        {
            return Some((entry, table_name.map(|s| s.to_string())));
        }
        self.input_map
            .lookup_key(keycode, mods, None)
            .map(|entry| (entry, None))
    }

    fn process_key(
        &mut self,
        pane: &Arc<dyn Pane>,
        context: &dyn WindowOps,
        keycode: &KeyCode,
        raw_modifiers: Modifiers,
        leader_active: bool,
        leader_mod: Modifiers,
        only_key_bindings: OnlyKeyBindings,
        is_down: bool,
        key_event: Option<&KeyEvent>,
    ) -> bool {
        if is_down && !leader_active {
            // Check to see if this key-press is the leader activating
            if let Some(duration) = self.input_map.is_leader(&keycode, raw_modifiers) {
                // Yes; record its expiration
                let target = std::time::Instant::now() + duration;
                self.leader_is_down.replace(target);
                self.update_title();
                // schedule an invalidation so that the cursor or status
                // area will be repainted at the right time
                if let Some(window) = self.window.clone() {
                    promise::spawn::spawn(async move {
                        Timer::at(target).await;
                        window.invalidate();
                    })
                    .detach();
                }
                return true;
            }
        }

        if is_down {
            if only_key_bindings == OnlyKeyBindings::No {
                if let Some(modal) = self.get_modal() {
                    if let Key::Code(term_key) = self.win_key_code_to_termwiz_key_code(keycode) {
                        match modal.key_down(term_key, raw_modifiers.remove_positional_mods(), self)
                        {
                            Ok(true) => return true,
                            Ok(false) => {}
                            Err(err) => {
                                log::error!("Error dispatching key to modal: {err:#}");
                                return true;
                            }
                        }
                    }
                }
            }

            if let Some((entry, table_name)) = self.lookup_key(
                pane,
                &keycode,
                raw_modifiers | leader_mod,
                only_key_bindings,
            ) {
                if self.config.debug_key_events {
                    log::info!(
                        "{}{:?} {:?} -> perform {:?}",
                        match table_name {
                            Some(name) => format!("table:{} ", name),
                            None => String::new(),
                        },
                        keycode,
                        raw_modifiers | leader_mod,
                        entry.action,
                    );
                }

                self.key_table_state.did_process_key();
                let handled = match self.perform_key_assignment(&pane, &entry.action) {
                    Ok(PerformAssignmentResult::Handled) => true,
                    Err(_) => true,
                    Ok(_) => false,
                };

                if handled {
                    context.invalidate();

                    if leader_active {
                        // A successful leader key-lookup cancels the leader
                        // virtual modifier state
                        self.leader_done();
                    }

                    return true;
                }
            }
        }

        // While the leader modifier is active, only registered
        // keybindings are recognized.
        let only_key_bindings = match (only_key_bindings, leader_active) {
            (OnlyKeyBindings::Yes, _) => OnlyKeyBindings::Yes,
            (_, true) => OnlyKeyBindings::Yes,
            _ => OnlyKeyBindings::No,
        };

        if only_key_bindings == OnlyKeyBindings::No {
            let config = &self.config;

            // This is a bit ugly.
            // Not all of our platforms report LEFT|RIGHT ALT; most report just ALT.
            // For those that do distinguish between them we want to respect the left vs.
            // right settings for the compose behavior.
            // Otherwise, if the event didn't include left vs. right then we want to
            // respect the generic compose behavior.
            let bypass_compose =
                    // Left ALT and they disabled compose
                    (raw_modifiers.contains(Modifiers::LEFT_ALT)
                    && !config.send_composed_key_when_left_alt_is_pressed)
                    // Right ALT and they disabled compose
                    || (raw_modifiers.contains(Modifiers::RIGHT_ALT)
                        && !config.send_composed_key_when_right_alt_is_pressed)
                    // Generic ALT and they disabled generic compose
                    || (!raw_modifiers.contains(Modifiers::RIGHT_ALT)
                        && !raw_modifiers.contains(Modifiers::LEFT_ALT)
                        && raw_modifiers.contains(Modifiers::ALT)
                        && !(config.send_composed_key_when_left_alt_is_pressed
                             || config.send_composed_key_when_right_alt_is_pressed));

            if bypass_compose {
                if let Key::Code(term_key) = self.win_key_code_to_termwiz_key_code(keycode) {
                    let tw_raw_modifiers = raw_modifiers;

                    let mut did_encode = false;
                    if let Some(key_event) = key_event {
                        if let Some(encoded) = self.encode_win32_input(&pane, &key_event) {
                            if self.config.debug_key_events {
                                log::info!("win32: Encoded input as {:?}", encoded);
                            }
                            pane.writer()
                                .write_all(encoded.as_bytes())
                                .context("sending win32-input-mode encoded data")
                                .ok();
                            did_encode = true;
                        } else if let Some(encoded) = self.encode_kitty_input(&pane, &key_event) {
                            if self.config.debug_key_events {
                                log::info!("kitty: Encoded input as {:?}", encoded);
                            }
                            pane.writer()
                                .write_all(encoded.as_bytes())
                                .context("sending kitty encoded data")
                                .ok();
                            did_encode = true;
                        }
                    };
                    if !did_encode {
                        if self.config.debug_key_events {
                            log::info!(
                                "{:?} {:?} -> send to pane {:?} {:?}",
                                keycode,
                                raw_modifiers,
                                term_key,
                                tw_raw_modifiers
                            );
                        }

                        did_encode = if is_down {
                            pane.key_down(term_key, tw_raw_modifiers)
                        } else {
                            pane.key_up(term_key, tw_raw_modifiers)
                        }
                        .is_ok();
                    };

                    if did_encode {
                        if is_down
                            && !keycode.is_modifier()
                            && self.pane_state(pane.pane_id()).overlay.is_none()
                        {
                            self.maybe_scroll_to_bottom_for_input(&pane);
                        }
                        if is_down
                            && self.config.hide_mouse_cursor_when_typing
                            && !keycode.is_modifier()
                        {
                            context.set_cursor(None);
                        }
                        if !keycode.is_modifier() {
                            context.invalidate();
                        }

                        return true;
                    }
                }
            }
        }

        false
    }
}
