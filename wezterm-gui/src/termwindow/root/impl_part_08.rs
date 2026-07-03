impl TermWindow {
    fn do_open_link_at_mouse_cursor(&self, pane: &Arc<dyn Pane>) {
        // They clicked on a link, so let's open it!
        // We need to ensure that we spawn the `open` call outside of the context
        // of our window loop; on Windows it can cause a panic due to
        // triggering our WndProc recursively.
        // We get that assurance for free as part of the async dispatch that we
        // perform below; here we allow the user to define an `open-uri` event
        // handler that can bypass the normal `open_url` functionality.
        if let Some(link) = self.current_highlight.as_ref().cloned() {
            let window = GuiWin::new(self);
            let pane = MuxPane(pane.pane_id());

            async fn open_uri(
                lua: Option<Rc<mlua::Lua>>,
                window: GuiWin,
                pane: MuxPane,
                link: String,
            ) -> anyhow::Result<()> {
                let default_click = match lua {
                    Some(lua) => {
                        let args = lua.pack_multi((window, pane, link.clone()))?;
                        config::lua::emit_event(&lua, ("open-uri".to_string(), args))
                            .await
                            .map_err(|e| {
                                log::error!("while processing open-uri event: {:#}", e);
                                e
                            })?
                    }
                    None => true,
                };
                if default_click {
                    log::info!("clicking {}", link);
                    wezterm_open_url::open_url(&link);
                }
                Ok(())
            }

            promise::spawn::spawn(config::with_lua_config_on_main_thread(move |lua| {
                open_uri(lua, window, pane, link.uri().to_string())
            }))
            .detach();
        }
    }

    fn close_current_pane(&mut self, confirm: bool) {
        let mux_window_id = self.mux_window_id;
        let mux = Mux::get();
        let tab = match mux.get_active_tab_for_window(mux_window_id) {
            Some(tab) => tab,
            None => return,
        };
        let pane = match tab.get_active_pane() {
            Some(p) => p,
            None => return,
        };

        let pane_id = pane.pane_id();
        if confirm && !pane.can_close_without_prompting(CloseReason::Pane) {
            let window = self.window.clone().unwrap();
            let (overlay, future) = start_overlay_pane(self, &pane, move |pane_id, term| {
                confirm_close_pane(pane_id, term, mux_window_id, window)
            });
            self.assign_overlay_for_pane(pane_id, overlay);
            promise::spawn::spawn(future).detach();
        } else {
            mux.remove_pane(pane_id);
        }
    }

    fn close_specific_tab(&mut self, tab_idx: usize, confirm: bool) {
        let mux = Mux::get();
        let mux_window_id = self.mux_window_id;
        let mux_window = match mux.get_window(mux_window_id) {
            Some(w) => w,
            None => return,
        };

        let tab = match mux_window.get_by_idx(tab_idx) {
            Some(tab) => Arc::clone(tab),
            None => return,
        };
        drop(mux_window);

        let tab_id = tab.tab_id();
        if confirm && !tab.can_close_without_prompting(CloseReason::Tab) {
            if self.activate_tab(tab_idx as isize).is_err() {
                return;
            }

            let window = self.window.clone().unwrap();
            let (overlay, future) = start_overlay(self, &tab, move |tab_id, term| {
                confirm_close_tab(tab_id, term, mux_window_id, window)
            });
            self.assign_overlay(tab_id, overlay);
            promise::spawn::spawn(future).detach();
        } else {
            mux.remove_tab(tab_id);
        }
    }

    fn close_current_tab(&mut self, confirm: bool) {
        let mux = Mux::get();
        let tab = match mux.get_active_tab_for_window(self.mux_window_id) {
            Some(tab) => tab,
            None => return,
        };
        let tab_id = tab.tab_id();
        let mux_window_id = self.mux_window_id;
        if confirm && !tab.can_close_without_prompting(CloseReason::Tab) {
            let window = self.window.clone().unwrap();
            let (overlay, future) = start_overlay(self, &tab, move |tab_id, term| {
                confirm_close_tab(tab_id, term, mux_window_id, window)
            });
            self.assign_overlay(tab_id, overlay);
            promise::spawn::spawn(future).detach();
        } else {
            mux.remove_tab(tab_id);
        }
    }

    pub fn pane_state(&self, pane_id: PaneId) -> RefMut<'_, PaneState> {
        RefMut::map(self.pane_state.borrow_mut(), |state| {
            state.entry(pane_id).or_insert_with(PaneState::default)
        })
    }

    pub fn tab_state(&self, tab_id: TabId) -> RefMut<'_, TabState> {
        RefMut::map(self.tab_state.borrow_mut(), |state| {
            state.entry(tab_id).or_insert_with(TabState::default)
        })
    }

    /// Resize overlays to match their corresponding tab/pane dimensions

    pub fn resize_overlays(&self) {
        let mux = Mux::get();
        for (_, state) in self.tab_state.borrow().iter() {
            if let Some(overlay) = state.overlay.as_ref().map(|o| &o.pane) {
                overlay.resize(self.terminal_size).ok();
            }
        }
        for (pane_id, state) in self.pane_state.borrow().iter() {
            if let Some(overlay) = state.overlay.as_ref().map(|o| &o.pane) {
                if let Some(pane) = mux.get_pane(*pane_id) {
                    let dims = pane.get_dimensions();
                    overlay
                        .resize(TerminalSize {
                            cols: dims.cols,
                            rows: dims.viewport_rows,
                            dpi: self.terminal_size.dpi,
                            pixel_height: (self.terminal_size.pixel_height
                                / self.terminal_size.rows)
                                * dims.viewport_rows,
                            pixel_width: (self.terminal_size.pixel_width / self.terminal_size.cols)
                                * dims.cols,
                        })
                        .ok();
                }
            }
        }
    }

    pub fn get_viewport(&self, pane_id: PaneId) -> Option<StableRowIndex> {
        self.pane_state(pane_id).viewport
    }

    pub fn set_viewport(
        &mut self,
        pane_id: PaneId,
        position: Option<StableRowIndex>,
        dims: RenderableDimensions,
    ) {
        let pos = match position {
            Some(pos) => {
                // Drop out of scrolling mode if we're off the bottom
                if pos >= dims.physical_top {
                    None
                } else {
                    Some(pos.max(dims.scrollback_top))
                }
            }
            None => None,
        };

        let mut state = self.pane_state(pane_id);
        if pos != state.viewport {
            state.viewport = pos;

            // This is a bit gross.  If we add other overlays that need this information,
            // this should get extracted out into a trait
            if let Some(overlay) = state.overlay.as_ref() {
                if let Some(copy) = overlay.pane.downcast_ref::<CopyOverlay>() {
                    copy.viewport_changed(pos);
                } else if let Some(qs) = overlay.pane.downcast_ref::<QuickSelectOverlay>() {
                    qs.viewport_changed(pos);
                }
            }
        }
        self.window.as_ref().unwrap().invalidate();
    }

    fn maybe_scroll_to_bottom_for_input(&mut self, pane: &Arc<dyn Pane>) {
        if self.config.scroll_to_bottom_on_input {
            self.scroll_to_bottom(pane);
        }
    }

    fn scroll_to_top(&mut self, pane: &Arc<dyn Pane>) {
        let dims = pane.get_dimensions();
        self.set_viewport(pane.pane_id(), Some(dims.scrollback_top), dims);
    }

    fn scroll_to_bottom(&mut self, pane: &Arc<dyn Pane>) {
        self.pane_state(pane.pane_id()).viewport = None;
    }

    fn get_active_pane_no_overlay(&self) -> Option<Arc<dyn Pane>> {
        let mux = Mux::get();
        mux.get_active_tab_for_window(self.mux_window_id)
            .and_then(|tab| tab.get_active_pane())
    }

    /// Returns a Pane that we can interact with; this will typically be
    /// the active tab for the window, but if the window has a tab-wide
    /// overlay (such as the launcher / tab navigator),
    /// then that will be returned instead.  Otherwise, if the pane has
    /// an active overlay (such as search or copy mode) then that will
    /// be returned.

    pub fn get_active_pane_or_overlay(&self) -> Option<Arc<dyn Pane>> {
        let mux = Mux::get();
        let tab = match mux.get_active_tab_for_window(self.mux_window_id) {
            Some(tab) => tab,
            None => return None,
        };

        let tab_id = tab.tab_id();

        if let Some(tab_overlay) = self
            .tab_state(tab_id)
            .overlay
            .as_ref()
            .map(|overlay| overlay.pane.clone())
        {
            Some(tab_overlay)
        } else {
            let pane = tab.get_active_pane()?;
            let pane_id = pane.pane_id();
            self.pane_state(pane_id)
                .overlay
                .as_ref()
                .map(|overlay| overlay.pane.clone())
                .or_else(|| Some(pane))
        }
    }

    fn get_splits(&mut self) -> Vec<PositionedSplit> {
        let mux = Mux::get();
        let tab = match mux.get_active_tab_for_window(self.mux_window_id) {
            Some(tab) => tab,
            None => return vec![],
        };

        let tab_id = tab.tab_id();

        if self.tab_state(tab_id).overlay.is_some() {
            vec![]
        } else {
            tab.iter_splits()
        }
    }

    fn pos_pane_to_pane_info(pos: &PositionedPane) -> PaneInformation {
        PaneInformation {
            pane_id: pos.pane.pane_id(),
            pane_index: pos.index,
            is_active: pos.is_active,
            is_zoomed: pos.is_zoomed,
            has_unseen_output: pos.pane.has_unseen_output(),
            left: pos.left,
            top: pos.top,
            width: pos.width,
            height: pos.height,
            pixel_width: pos.pixel_width,
            pixel_height: pos.pixel_height,
            title: pos.pane.get_title(),
            user_vars: pos.pane.copy_user_vars(),
            progress: pos.pane.get_progress(),
        }
    }

    fn get_tab_information(&mut self) -> Vec<TabInformation> {
        let mux = Mux::get();
        let window = match mux.get_window(self.mux_window_id) {
            Some(window) => window,
            _ => return vec![],
        };
        let tab_index = window.get_active_idx();

        window
            .iter()
            .enumerate()
            .map(|(idx, tab)| {
                let panes = self.get_pos_panes_for_tab(tab);

                TabInformation {
                    tab_index: idx,
                    tab_id: tab.tab_id(),
                    is_active: tab_index == idx,
                    is_last_active: window
                        .get_last_active_idx()
                        .map(|last_active| last_active == idx)
                        .unwrap_or(false),
                    window_id: self.mux_window_id,
                    tab_title: tab.get_title(),
                    active_pane: panes
                        .iter()
                        .find(|p| p.is_active)
                        .map(Self::pos_pane_to_pane_info),
                }
            })
            .collect()
    }

    fn get_pane_information(&self) -> Vec<PaneInformation> {
        self.get_panes_to_render()
            .iter()
            .map(Self::pos_pane_to_pane_info)
            .collect()
    }

    fn get_pos_panes_for_tab(&self, tab: &Arc<Tab>) -> Vec<PositionedPane> {
        let tab_id = tab.tab_id();

        if let Some(pane) = self
            .tab_state(tab_id)
            .overlay
            .as_ref()
            .map(|overlay| overlay.pane.clone())
        {
            let size = tab.get_size();
            vec![PositionedPane {
                index: 0,
                is_active: true,
                is_zoomed: false,
                left: 0,
                top: 0,
                width: size.cols as _,
                height: size.rows as _,
                pixel_width: size.cols as usize * self.render_metrics.cell_size.width as usize,
                pixel_height: size.rows as usize * self.render_metrics.cell_size.height as usize,
                pane,
            }]
        } else {
            let mut panes = tab.iter_panes();
            for p in &mut panes {
                if let Some(overlay) = self.pane_state(p.pane.pane_id()).overlay.as_ref() {
                    p.pane = Arc::clone(&overlay.pane);
                }
            }
            panes
        }
    }

    fn get_panes_to_render(&self) -> Vec<PositionedPane> {
        let mux = Mux::get();
        let tab = match mux.get_active_tab_for_window(self.mux_window_id) {
            Some(tab) => tab,
            None => return vec![],
        };

        self.get_pos_panes_for_tab(&tab)
    }

    /// if pane_id.is_none(), removes any overlay for the specified tab.
    /// Otherwise: if the overlay is the specified pane for that tab, remove it.

    fn cancel_overlay_for_tab(&mut self, tab_id: TabId, pane_id: Option<PaneId>) {
        if pane_id.is_some() {
            let current = self
                .tab_state(tab_id)
                .overlay
                .as_ref()
                .map(|o| o.pane.pane_id());
            if current != pane_id {
                return;
            }
        }
        if let Some(overlay) = self.tab_state(tab_id).overlay.take() {
            Mux::get().remove_pane(overlay.pane.pane_id());
        }
        if let Some(window) = self.window.as_ref() {
            window.invalidate();
        }
    }

    pub fn schedule_cancel_overlay(window: Window, tab_id: TabId, pane_id: Option<PaneId>) {
        window.notify(TermWindowNotif::CancelOverlayForTab { tab_id, pane_id });
    }

    fn cancel_overlay_for_pane(&mut self, pane_id: PaneId) {
        if let Some(overlay) = self.pane_state(pane_id).overlay.take() {
            // Ungh, when I built the CopyOverlay, its pane doesn't get
            // added to the mux and instead it reports the overlaid
            // pane id.  Take care to avoid killing ourselves off
            // when closing the CopyOverlay
            if pane_id != overlay.pane.pane_id() {
                Mux::get().remove_pane(overlay.pane.pane_id());
            }
        }
        if let Some(window) = self.window.as_ref() {
            window.invalidate();
        }
    }

    pub fn schedule_cancel_overlay_for_pane(window: Window, pane_id: PaneId) {
        window.notify(TermWindowNotif::CancelOverlayForPane(pane_id));
    }

    pub fn assign_overlay_for_pane(&mut self, pane_id: PaneId, pane: Arc<dyn Pane>) {
        self.cancel_overlay_for_pane(pane_id);
        self.pane_state(pane_id).overlay.replace(OverlayState {
            pane,
            key_table_state: KeyTableState::default(),
        });
        self.update_title();
    }

    pub fn assign_overlay(&mut self, tab_id: TabId, overlay: Arc<dyn Pane>) {
        self.cancel_overlay_for_tab(tab_id, None);
        self.tab_state(tab_id).overlay.replace(OverlayState {
            pane: overlay,
            key_table_state: KeyTableState::default(),
        });
        self.update_title();
    }
}
