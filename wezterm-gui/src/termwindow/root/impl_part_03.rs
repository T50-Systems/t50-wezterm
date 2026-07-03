impl TermWindow {
    fn dispatch_notif(&mut self, notif: TermWindowNotif, window: &Window) -> anyhow::Result<()> {
        fn chan_err<T>(e: smol::channel::TrySendError<T>) -> anyhow::Error {
            anyhow::anyhow!("{}", e)
        }

        match notif {
            TermWindowNotif::InvalidateShapeCache => {
                self.shape_generation += 1;
                self.shape_cache.borrow_mut().clear();
                self.invalidate_modal();
                window.invalidate();
            }
            TermWindowNotif::PerformAssignment {
                pane_id,
                assignment,
                tx,
            } => {
                let mux = Mux::get();
                let result = || -> anyhow::Result<()> {
                    // The CopyMode overlay doesn't exist in the mux, but aliases
                    // itself with the overlaid pane's pane_id.
                    // So we do a bit of fancy footwork here to resolve the overlay
                    // and use that if it has the same pane_id, but otherwise fall
                    // back to what we get from the mux.
                    // <https://github.com/wezterm/wezterm/issues/3209>
                    let active_pane = self
                        .get_active_pane_or_overlay()
                        .ok_or_else(|| anyhow!("there is no active pane!?"))?;
                    let pane = if active_pane.pane_id() == pane_id {
                        active_pane
                    } else {
                        mux.get_pane(pane_id)
                            .ok_or_else(|| anyhow!("pane id {} is not valid", pane_id))?
                    };
                    self.perform_key_assignment(&pane, &assignment)
                        .context("perform_key_assignment")?;
                    Ok(())
                }();
                window.invalidate();
                if let Some(tx) = tx {
                    tx.try_send(result).ok();
                }
            }
            TermWindowNotif::SetRightStatus(status) => {
                if status != self.right_status {
                    self.right_status = status;
                    self.update_title_post_status();
                } else {
                    self.schedule_next_status_update();
                }
            }
            TermWindowNotif::SetLeftStatus(status) => {
                if status != self.left_status {
                    self.left_status = status;
                    self.update_title_post_status();
                } else {
                    self.schedule_next_status_update();
                }
            }
            TermWindowNotif::SetSecondaryBar {
                left,
                center,
                right,
            } => {
                if left != self.secondary_left_status
                    || center != self.secondary_center_status
                    || right != self.secondary_right_status
                {
                    self.secondary_left_status = left;
                    self.secondary_center_status = center;
                    self.secondary_right_status = right;
                    self.update_title_post_status();
                } else {
                    self.schedule_next_status_update();
                }
            }
            TermWindowNotif::GetDimensions(tx) => {
                tx.try_send((self.dimensions, self.window_state))
                    .map_err(chan_err)
                    .context("send GetDimensions response")?;
            }
            TermWindowNotif::GetEffectiveConfig(tx) => {
                tx.try_send(self.config.clone())
                    .map_err(chan_err)
                    .context("send GetEffectiveConfig response")?;
            }
            TermWindowNotif::FinishWindowEvent { name, again } => {
                self.finish_window_event(&name, again);
            }
            TermWindowNotif::GetConfigOverrides(tx) => {
                tx.try_send(self.config_overrides.clone())
                    .map_err(chan_err)
                    .context("send GetConfigOverrides response")?;
            }
            TermWindowNotif::SetConfigOverrides(value) => {
                if value != self.config_overrides {
                    self.config_overrides = value;
                    self.config_was_reloaded();
                }
            }
            TermWindowNotif::CancelOverlayForPane(pane_id) => {
                self.cancel_overlay_for_pane(pane_id);
            }
            TermWindowNotif::CancelOverlayForTab { tab_id, pane_id } => {
                self.cancel_overlay_for_tab(tab_id, pane_id);
            }
            TermWindowNotif::MuxNotification(n) => match n {
                MuxNotification::Alert {
                    alert: Alert::SetUserVar { name, value },
                    pane_id,
                } => {
                    self.emit_user_var_event(pane_id, name, value);
                }
                MuxNotification::WindowTitleChanged { .. }
                | MuxNotification::Alert {
                    alert:
                        Alert::OutputSinceFocusLost
                        | Alert::CurrentWorkingDirectoryChanged
                        | Alert::WindowTitleChanged(_)
                        | Alert::TabTitleChanged(_)
                        | Alert::IconTitleChanged(_)
                        | Alert::Progress(_),
                    ..
                } => {
                    self.update_title();
                }
                MuxNotification::Alert {
                    alert: Alert::PaletteChanged,
                    pane_id,
                } => {
                    // Shape cache includes color information, so
                    // ensure that we invalidate that as part of
                    // this overall invalidation for the palette
                    self.dispatch_notif(TermWindowNotif::InvalidateShapeCache, window)?;
                    self.mux_pane_output_event(pane_id);
                }
                MuxNotification::Alert {
                    alert: Alert::Bell,
                    pane_id,
                } => {
                    if !self.window_contains_pane(pane_id) {
                        return Ok(());
                    }

                    match self.config.audible_bell {
                        AudibleBell::SystemBeep => {
                            Connection::get().expect("on main thread").beep();
                        }
                        AudibleBell::Disabled => {}
                    }

                    log::trace!("Ding! (this is the bell) in pane {}", pane_id);
                    self.emit_window_event("bell", Some(pane_id));

                    let mut per_pane = self.pane_state(pane_id);
                    per_pane.bell_start.replace(Instant::now());
                    window.invalidate();
                }
                MuxNotification::Alert {
                    alert: Alert::ToastNotification { .. },
                    ..
                } => {}
                MuxNotification::TabAddedToWindow {
                    window_id: _,
                    tab_id,
                } => {
                    let mux = Mux::get();
                    let mut size = self.terminal_size;
                    if let Some(tab) = mux.get_tab(tab_id) {
                        // If we attached to a remote domain and loaded in
                        // a tab async, we need to fixup its size, either
                        // by resizing it or resizes ourselves.
                        // The strategy here is to adjust both by taking
                        // the maximal size in both horizontal and vertical
                        // dimensions and applying that. In practice that
                        // means that a new local client will resize larger
                        // to adjust to the size of an existing client.
                        let tab_size = tab.get_size();
                        size.rows = size.rows.max(tab_size.rows);
                        size.cols = size.cols.max(tab_size.cols);

                        if size.rows != self.terminal_size.rows
                            || size.cols != self.terminal_size.cols
                            || size.pixel_width != self.terminal_size.pixel_width
                            || size.pixel_height != self.terminal_size.pixel_height
                        {
                            self.set_window_size(size, window)?;
                        } else if tab_size.dpi == 0 {
                            log::debug!("fixup dpi in newly added tab");
                            tab.resize(self.terminal_size);
                        }
                    }
                }
                MuxNotification::PaneOutput(pane_id) => {
                    self.mux_pane_output_event(pane_id);
                }
                MuxNotification::WindowInvalidated(_) => {
                    window.invalidate();
                    self.update_title_post_status();
                }
                MuxNotification::WindowRemoved(_window_id) => {
                    // Handled by frontend
                }
                MuxNotification::AssignClipboard { .. } => {
                    // Handled by frontend
                }
                MuxNotification::SaveToDownloads { .. } => {
                    // Handled by frontend
                }
                MuxNotification::PaneFocused(_) => {
                    // Also handled by clientpane
                    self.update_title_post_status();
                }
                MuxNotification::TabResized(_) => {
                    // Also handled by wezterm-client
                    self.update_title_post_status();
                }
                MuxNotification::TabTitleChanged { .. } => {
                    self.update_title_post_status();
                }
                MuxNotification::PaneAdded(_)
                | MuxNotification::WorkspaceRenamed { .. }
                | MuxNotification::PaneRemoved(_)
                | MuxNotification::WindowWorkspaceChanged(_)
                | MuxNotification::ActiveWorkspaceChanged(_)
                | MuxNotification::Empty
                | MuxNotification::WindowCreated(_) => {}
            },
            TermWindowNotif::EmitStatusUpdate => {
                self.emit_status_event();
            }
            TermWindowNotif::GetSelectionForPane { pane_id, tx } => {
                let mux = Mux::get();
                let pane = mux
                    .get_pane(pane_id)
                    .ok_or_else(|| anyhow!("pane id {} is not valid", pane_id))?;

                tx.try_send(self.selection_text(&pane))
                    .map_err(chan_err)
                    .context("send GetSelectionForPane response")?;
            }
            TermWindowNotif::Apply(func) => {
                func(self);
            }
            TermWindowNotif::SwitchToMuxWindow(mux_window_id) => {
                self.mux_window_id = mux_window_id;
                *self.mux_window_id_for_subscriptions.lock().unwrap() = mux_window_id;

                self.clear_all_overlays();
                self.current_highlight.take();
                self.invalidate_fancy_tab_bar();
                self.invalidate_modal();

                let mux = Mux::get();
                if let Some(window) = mux.get_window(self.mux_window_id) {
                    for tab in window.iter() {
                        tab.resize(self.terminal_size);
                    }
                };
                self.update_title();
                window.invalidate();
            }
            TermWindowNotif::SetInnerSize { width, height } => {
                self.set_inner_size(window, width, height);
            }
        }

        Ok(())
    }

    fn set_inner_size(&mut self, window: &Window, width: usize, height: usize) {
        self.resizes_pending += 1;
        window.set_inner_size(width, height);
    }

    /// Take care to remove our panes from the mux, otherwise
    /// we can leave the mux with no windows but some panes
    /// and it won't believe that we are empty.

    fn clear_all_overlays(&mut self) {
        let overlay_panes_to_cancel = self
            .pane_state
            .borrow()
            .iter()
            .filter_map(|(_, state)| state.overlay.as_ref().map(|overlay| overlay.pane.pane_id()))
            .collect::<Vec<_>>();

        for pane_id in overlay_panes_to_cancel {
            self.cancel_overlay_for_pane(pane_id);
        }

        let tab_overlays_to_cancel = self
            .tab_state
            .borrow()
            .iter()
            .filter_map(|(tab_id, state)| state.overlay.as_ref().map(|_| *tab_id))
            .collect::<Vec<_>>();

        for tab_id in tab_overlays_to_cancel {
            self.cancel_overlay_for_tab(tab_id, None);
        }

        self.pane_state.borrow_mut().clear();
        self.tab_state.borrow_mut().clear();
    }

    fn apply_icon(window: &Window) -> anyhow::Result<()> {
        let image = image::load_from_memory(ICON_DATA)?.into_rgba8();
        let (width, height) = image.dimensions();
        window.set_icon(Image::with_rgba32(
            width as usize,
            height as usize,
            width as usize * 4,
            image.as_raw(),
        ));
        Ok(())
    }

    fn schedule_status_update(&self) {
        if let Some(window) = self.window.as_ref() {
            window.notify(TermWindowNotif::EmitStatusUpdate);
        }
    }

    fn is_pane_visible(&mut self, pane_id: PaneId) -> bool {
        let mux = Mux::get();
        let tab = match mux.get_active_tab_for_window(self.mux_window_id) {
            Some(tab) => tab,
            None => return false,
        };

        let tab_id = tab.tab_id();
        if let Some(tab_overlay) = self
            .tab_state(tab_id)
            .overlay
            .as_ref()
            .map(|overlay| overlay.pane.clone())
        {
            return tab_overlay.pane_id() == pane_id;
        }

        tab.contains_pane(pane_id)
    }

    fn mux_pane_output_event(&mut self, pane_id: PaneId) {
        metrics::histogram!("mux.pane_output_event.rate").record(1.);
        if self.is_pane_visible(pane_id) {
            if let Some(ref win) = self.window {
                win.invalidate();
            }
        }
    }
}
