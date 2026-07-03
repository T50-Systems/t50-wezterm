impl TermWindow {
    fn mux_pane_output_event_callback(
        n: MuxNotification,
        window: &Window,
        mux_window_id: MuxWindowId,
        dead: &Arc<AtomicBool>,
    ) -> bool {
        if dead.load(Ordering::Relaxed) {
            // Subscription cancelled asynchronously
            return false;
        }

        match n {
            MuxNotification::Alert {
                pane_id,
                alert:
                    Alert::OutputSinceFocusLost
                    | Alert::CurrentWorkingDirectoryChanged
                    | Alert::WindowTitleChanged(_)
                    | Alert::TabTitleChanged(_)
                    | Alert::IconTitleChanged(_)
                    | Alert::Progress(_)
                    | Alert::SetUserVar { .. }
                    | Alert::Bell,
            }
            | MuxNotification::PaneFocused(pane_id)
            | MuxNotification::PaneRemoved(pane_id)
            | MuxNotification::PaneOutput(pane_id) => {
                // Ideally we'd check to see if pane_id is part of this window,
                // but overlays may not be 100% associated with the window
                // in the mux and we don't want to lose the invalidation
                // signal for that case, so we just check window validity
                // here and propagate to the window event handler that
                // will then do the check with full context.
                let mux = Mux::get();
                if mux.get_window(mux_window_id).is_none() {
                    // Something inconsistent: cancel subscription
                    log::debug!(
                        "PaneOutput: wanted mux_window_id={} from mux, but \
                         was not found, cancel mux subscription",
                        mux_window_id
                    );
                    return false;
                }
                let _ = pane_id;
            }
            MuxNotification::PaneAdded(_pane_id) => {
                // If some other client spawns a pane inside this window, this
                // gives us an opportunity to attach it to the clipboard.
                let mux = Mux::get();
                return mux.get_window(mux_window_id).is_some();
            }
            MuxNotification::TabAddedToWindow { window_id, .. }
            | MuxNotification::WindowTitleChanged { window_id, .. }
            | MuxNotification::WindowInvalidated(window_id) => {
                if window_id != mux_window_id {
                    return true;
                }
            }
            MuxNotification::WindowRemoved(window_id) => {
                if window_id != mux_window_id {
                    return true;
                }
                // Set the window as dead to unsubscribe from further notifications
                dead.store(true, Ordering::Relaxed);
                return false;
            }
            MuxNotification::TabResized(tab_id)
            | MuxNotification::TabTitleChanged { tab_id, .. } => {
                let mux = Mux::get();
                if mux.window_containing_tab(tab_id) == Some(mux_window_id) {
                    // fall through
                } else {
                    return true;
                }
            }
            MuxNotification::Alert {
                alert: Alert::ToastNotification { .. },
                ..
            }
            | MuxNotification::AssignClipboard { .. }
            | MuxNotification::SaveToDownloads { .. }
            | MuxNotification::WindowCreated(_)
            | MuxNotification::ActiveWorkspaceChanged(_)
            | MuxNotification::WorkspaceRenamed { .. }
            | MuxNotification::Empty
            | MuxNotification::WindowWorkspaceChanged(_) => return true,
            MuxNotification::Alert {
                alert: Alert::PaletteChanged { .. },
                ..
            } => {
                // fall through
            }
        }

        window.notify(TermWindowNotif::MuxNotification(n));

        true
    }

    fn subscribe_to_pane_updates(&self) {
        let window = self.window.clone().expect("window to be valid on startup");
        let mux_window_id = Arc::clone(&self.mux_window_id_for_subscriptions);
        let mux = Mux::get();
        let dead = Arc::new(AtomicBool::new(false));
        mux.subscribe(move |n| {
            if dead.load(Ordering::Relaxed) {
                return false;
            }
            let mux_window_id = *mux_window_id.lock().unwrap();
            let window = window.clone();
            let dead = dead.clone();
            promise::spawn::spawn_into_main_thread(async move {
                Self::mux_pane_output_event_callback(n, &window, mux_window_id, &dead)
            })
            .detach();
            true
        });
    }

    fn emit_status_event(&mut self) {
        self.emit_window_event("update-right-status", None);
        self.emit_window_event("update-status", None);
    }

    fn schedule_window_event(&mut self, name: &str, pane_id: Option<PaneId>) {
        let window = GuiWin::new(self);
        let pane = match pane_id {
            Some(pane_id) => Mux::get().get_pane(pane_id),
            None => None,
        };
        let pane = match pane {
            Some(pane) => pane,
            None => match self.get_active_pane_or_overlay() {
                Some(pane) => pane,
                None => return,
            },
        };
        let pane = MuxPane(pane.pane_id());
        let name = name.to_string();

        async fn do_event(
            lua: Option<Rc<mlua::Lua>>,
            name: String,
            window: GuiWin,
            pane: MuxPane,
        ) -> anyhow::Result<()> {
            let again = if let Some(lua) = lua {
                let args = lua.pack_multi((window.clone(), pane))?;

                if let Err(err) = config::lua::emit_event(&lua, (name.clone(), args)).await {
                    log::error!("while processing {} event: {:#}", name, err);
                }
                true
            } else {
                false
            };

            window
                .window
                .notify(TermWindowNotif::FinishWindowEvent { name, again });

            Ok(())
        }

        promise::spawn::spawn(config::with_lua_config_on_main_thread(move |lua| {
            do_event(lua, name, window, pane)
        }))
        .detach();
    }

    /// Called as part of finishing up a callout to lua.
    /// If again==false it means that there isn't a lua config
    /// to execute against, so we should just mark as done.
    /// Otherwise, if there is a queued item, schedule it now.

    fn finish_window_event(&mut self, name: &str, again: bool) {
        let state = self
            .event_states
            .entry(name.to_string())
            .or_insert(EventState::None);
        if again {
            match state {
                EventState::InProgress => {
                    *state = EventState::None;
                }
                EventState::InProgressWithQueued(pane) => {
                    let pane = *pane;
                    *state = EventState::InProgress;
                    self.schedule_window_event(name, pane);
                }
                EventState::None => {}
            }
        } else {
            *state = EventState::None;
        }
    }

    pub fn emit_window_event(&mut self, name: &str, pane_id: Option<PaneId>) {
        if self.get_active_pane_or_overlay().is_none() || self.window.is_none() {
            return;
        }

        let state = self
            .event_states
            .entry(name.to_string())
            .or_insert(EventState::None);
        match state {
            EventState::InProgress => {
                // Flag that we want to run again when the currently
                // executing event calls finish_window_event().
                *state = EventState::InProgressWithQueued(pane_id);
                return;
            }
            EventState::InProgressWithQueued(other_pane) => {
                // We've already got one copy executing and another
                // pending dispatch, so don't queue another.
                if pane_id != *other_pane {
                    log::warn!(
                        "Cannot queue {} event for pane {:?}, as \
                         there is already an event queued for pane {:?} \
                         in the same window",
                        name,
                        pane_id,
                        other_pane
                    );
                }
                return;
            }
            EventState::None => {
                // Nothing pending, so schedule a call now
                *state = EventState::InProgress;
                self.schedule_window_event(name, pane_id);
            }
        }
    }

    fn check_for_dirty_lines_and_invalidate_selection(&mut self, pane: &Arc<dyn Pane>) {
        let dims = pane.get_dimensions();
        let viewport = self
            .get_viewport(pane.pane_id())
            .unwrap_or(dims.physical_top);
        let visible_range = viewport..viewport + dims.viewport_rows as StableRowIndex;
        let seqno = self.selection(pane.pane_id()).seqno;
        let dirty = pane.get_changed_since(visible_range, seqno);

        if dirty.is_empty() {
            return;
        }
        if pane.downcast_ref::<CopyOverlay>().is_none()
            && pane.downcast_ref::<QuickSelectOverlay>().is_none()
        {
            // If any of the changed lines intersect with the
            // selection, then we need to clear the selection, but not
            // when the search overlay is active; the search overlay
            // marks lines as dirty to force invalidate them for
            // highlighting purpose but also manipulates the selection
            // and we want to allow it to retain the selection it made!

            let clear_selection =
                if let Some(selection_range) = self.selection(pane.pane_id()).range.as_ref() {
                    let selection_rows = selection_range.rows();
                    selection_rows.into_iter().any(|row| dirty.contains(row))
                } else {
                    false
                };

            if clear_selection {
                self.selection(pane.pane_id()).range.take();
                self.selection(pane.pane_id()).origin.take();
                self.selection(pane.pane_id()).seqno = pane.get_current_seqno();
            }
        }
    }

    fn palette(&mut self) -> &ColorPalette {
        if self.palette.is_none() {
            self.palette
                .replace(config::TermConfig::new().color_palette());
        }
        self.palette.as_ref().unwrap()
    }

    pub fn config_was_reloaded(&mut self) {
        log::debug!(
            "config was reloaded, overrides: {:?}",
            self.config_overrides
        );
        self.key_table_state.clear_stack();
        self.connection_name = Connection::get().unwrap().name();
        let config = match config::overridden_config(&self.config_overrides) {
            Ok(config) => config,
            Err(err) => {
                log::error!(
                    "Failed to apply config overrides to window: {:#}: {:?}",
                    err,
                    self.config_overrides
                );
                configuration()
            }
        };
        self.config = config.clone();
        self.palette.take();

        let mux = Mux::get();
        let window = match mux.get_window(self.mux_window_id) {
            Some(window) => window,
            _ => return,
        };
        if window.len() == 1 {
            self.show_tab_bar = config.enable_tab_bar && !config.hide_tab_bar_if_only_one_tab;
        } else {
            self.show_tab_bar = config.enable_tab_bar;
        }
        *self.cursor_blink_state.borrow_mut() = ColorEase::new(
            config.cursor_blink_rate,
            config.cursor_blink_ease_in,
            config.cursor_blink_rate,
            config.cursor_blink_ease_out,
            None,
        );
        *self.blink_state.borrow_mut() = ColorEase::new(
            config.text_blink_rate,
            config.text_blink_ease_in,
            config.text_blink_rate,
            config.text_blink_ease_out,
            None,
        );
        *self.rapid_blink_state.borrow_mut() = ColorEase::new(
            config.text_blink_rate_rapid,
            config.text_blink_rapid_ease_in,
            config.text_blink_rate_rapid,
            config.text_blink_rapid_ease_out,
            None,
        );

        self.show_scroll_bar = config.enable_scroll_bar;
        self.shape_generation += 1;
        {
            let mut shape_cache = self.shape_cache.borrow_mut();
            shape_cache.update_config(&config);
            shape_cache.clear();
        }
        self.line_state_cache.borrow_mut().update_config(&config);
        self.line_quad_cache.borrow_mut().update_config(&config);
        self.line_to_ele_shape_cache
            .borrow_mut()
            .update_config(&config);
        self.fancy_tab_bar.take();
        self.invalidate_fancy_tab_bar();
        self.invalidate_modal();
        self.input_map = InputMap::new(&config);
        self.leader_is_down = None;
        self.render_state.as_mut().map(|rs| rs.config_changed());
        let dimensions = self.dimensions;

        if let Err(err) = self.fonts.config_changed(&config) {
            log::error!("Failed to load font configuration: {:#}", err);
        }

        if let Some(window) = mux.get_window(self.mux_window_id) {
            let term_config: Arc<dyn TerminalConfiguration> =
                Arc::new(TermConfig::with_config(config.clone()));
            for tab in window.iter() {
                for pane in tab.iter_panes_ignoring_zoom() {
                    pane.pane.set_config(Arc::clone(&term_config));
                }
            }
            for state in self.pane_state.borrow().values() {
                if let Some(overlay) = &state.overlay {
                    overlay.pane.set_config(Arc::clone(&term_config));
                }
            }
            for state in self.tab_state.borrow().values() {
                if let Some(overlay) = &state.overlay {
                    overlay.pane.set_config(Arc::clone(&term_config));
                }
            }
        }

        if let Some(window) = self.window.as_ref().map(|w| w.clone()) {
            self.load_os_parameters();
            self.apply_scale_change(&dimensions, self.fonts.get_font_scale());
            self.apply_dimensions(&dimensions, None, &window);
            window.config_did_change(&config);
            window.invalidate();
        }

        // Do this after we've potentially adjusted scaling based on config/padding
        // and window size
        self.window_background = reload_background_image(
            &config,
            &self.window_background,
            &self.dimensions,
            &self.render_metrics,
        );

        self.invalidate_modal();
        self.emit_window_event("window-config-reloaded", None);
    }

    fn invalidate_modal(&mut self) {
        if let Some(modal) = self.get_modal() {
            modal.reconfigure(self);
            if let Some(window) = self.window.as_ref() {
                window.invalidate();
            }
        }
    }

    pub fn cancel_modal(&self) {
        self.modal.borrow_mut().take();
        if let Some(window) = self.window.as_ref() {
            window.invalidate();
        }
    }

    pub fn set_modal(&self, modal: Rc<dyn Modal>) {
        self.modal.borrow_mut().replace(modal);
        if let Some(window) = self.window.as_ref() {
            window.invalidate();
        }
    }

    fn get_modal(&self) -> Option<Rc<dyn Modal>> {
        self.modal.borrow().as_ref().map(|m| Rc::clone(&m))
    }
}
