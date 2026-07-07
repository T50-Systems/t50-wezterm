impl TmuxDomainState {
    fn sync_window_state(&self, windows: &[WindowItem], new_window: bool) -> anyhow::Result<()> {
        let Some(current_session) = *self.tmux_session.lock() else {
            return Ok(());
        };
        let mux = Mux::get();

        self.create_gui_window();
        let mut gui_window = self.gui_window.lock();
        let gui_window_id = match gui_window.as_mut() {
            Some(x) => x,
            None => {
                anyhow::bail!("No tmux gui created");
            }
        };

        for window in windows.iter() {
            if window.session_id != current_session {
                continue;
            }

            let size = TerminalSize {
                rows: window.window_height as usize,
                cols: window.window_width as usize,
                pixel_width: 0,
                pixel_height: 0,
                dpi: 0,
            };

            let tab = Arc::new(Tab::new(&size));
            tab.set_title(&format!("{}", &window.window_name));
            mux.add_tab_no_panes(&tab);

            let _ = self.add_attached_window(window, &tab.tab_id())?;

            let mut split_stack;
            let mut split_direction;

            let mut split_pane_index = 1;
            for l in &window.layout {
                match l {
                    WindowLayout::SinglePane(x) => {
                        let p = PaneItem {
                            session_id: window.session_id,
                            window_id: window.window_id,
                            _pane_index: 0,
                            cursor_x: 0,
                            cursor_y: 0,
                            pane_active: false,
                            pane_id: x.pane_id,
                            pane_width: x.pane_width,
                            pane_height: x.pane_height,
                            pane_left: x.pane_left,
                            pane_top: x.pane_top,
                        };
                        let local_pane = self.create_pane(&p).context("failed to create pane")?;
                        tab.assign_pane(&local_pane);
                        self.add_attached_pane(p.window_id, p.pane_id)?;
                        let _ = mux.add_pane(&local_pane);
                        break;
                    }

                    WindowLayout::SplitHorizontal(x) => {
                        split_direction = SplitDirection::Horizontal;
                        split_stack = x;
                    }

                    WindowLayout::SplitVertical(x) => {
                        split_direction = SplitDirection::Vertical;
                        split_stack = x;
                    }
                }

                for x in split_stack {
                    let p = PaneItem {
                        session_id: window.session_id,
                        window_id: window.window_id,
                        _pane_index: 0,
                        cursor_x: 0,
                        cursor_y: 0,
                        pane_active: false,
                        pane_id: x.pane_id,
                        pane_width: x.pane_width,
                        pane_height: x.pane_height,
                        pane_left: x.pane_left,
                        pane_top: x.pane_top,
                    };
                    let local_pane;
                    if !self.check_pane_attached(p.window_id, p.pane_id) {
                        local_pane = self.create_pane(&p).context("failed to create pane")?;
                        self.add_attached_pane(p.window_id, p.pane_id)?;
                        let _ = mux.add_pane(&local_pane);
                        if let None = tab.get_active_pane() {
                            tab.assign_pane(&local_pane);
                            split_pane_index = tab.get_active_idx();
                            continue;
                        }

                        split_pane_index = tab.split_and_insert(
                            split_pane_index,
                            SplitRequest {
                                direction: split_direction,
                                target_is_second: false,
                                top_level: false,
                                size: SplitSize::Cells(
                                    if split_direction == SplitDirection::Horizontal {
                                        p.pane_width as usize
                                    } else {
                                        p.pane_height as usize
                                    },
                                ),
                            },
                            local_pane.clone(),
                        )? + 1;
                    } else {
                        let pane_map = self.remote_panes.lock();
                        let local_pane_id = match pane_map.get(&p.pane_id) {
                            Some(x) => x.lock().local_pane_id,
                            None => anyhow::bail!("cannot find the local pane for {}", p.pane_id),
                        };

                        split_pane_index = match tab
                            .iter_panes_ignoring_zoom()
                            .iter()
                            .find(|x| x.pane.pane_id() == local_pane_id)
                        {
                            Some(x) => x.index,
                            None => {
                                log::info!("invalid pane id {local_pane_id}");
                                continue;
                            }
                        };
                        continue;
                    }
                }
            }

            mux.add_tab_to_window(&tab, **gui_window_id)?;
            gui_window_id.notify();

            let gui_tabs = self.gui_tabs.lock();
            let local_tab = match gui_tabs.get(&window.window_id) {
                Some(x) => x,
                None => {
                    log::info!(
                        "cannot find the local tab for tmux window {}",
                        window.window_id
                    );
                    continue;
                }
            };

            // For new window, we wait for nature ouput instead of capturing
            if !new_window {
                for p in local_tab.panes.iter() {
                    self.cmd_queue.lock().push_back(Box::new(CapturePane {
                        pane_id: *p,
                        history_limit: window.history_limit,
                    }));
                }
            }

            // To keep the active window last one to make it active after set the focus pane
            if !window.window_active {
                self.cmd_queue.lock().push_back(Box::new(ListAllPanes {
                    window_id: window.window_id,
                    prune: false,
                    layout_csum: window.layout_csum.clone(),
                }));
            }
        }

        // To keep the active window last one to make it active after set the focus pane
        match windows.iter().find(|w| w.window_active) {
            Some(window) => {
                self.cmd_queue.lock().push_back(Box::new(ListAllPanes {
                    window_id: window.window_id,
                    prune: false,
                    layout_csum: window.layout_csum.clone(),
                }));
            }
            None => {}
        }

        if *self.attach_state.lock() == AttachState::Init {
            self.cmd_queue.lock().push_back(Box::new(AttachDone));
        }

        TmuxDomainState::schedule_send_next_command(self.domain_id);

        Ok(())
    }

    pub fn subscribe_notification(&self) {
        let mux = Mux::get();
        let domain_id = self.domain_id;
        mux.subscribe(move |n| {
            promise::spawn::spawn_into_main_thread(async move {
                let mux = Mux::get();
                let domain = match mux.get_domain(domain_id) {
                    Some(d) => d,
                    None => return,
                };
                let tmux_domain = match domain.downcast_ref::<TmuxDomain>() {
                    Some(t) => t,
                    None => return,
                };

                if *tmux_domain.inner.attach_state.lock() == AttachState::Init {
                    return;
                }

                match n {
                    MuxNotification::PaneFocused(pane_id) => {
                        let tmux_pane_id = match tmux_domain
                            .inner
                            .remote_panes
                            .lock()
                            .iter()
                            .find(|(_, p)| p.lock().local_pane_id == pane_id)
                        {
                            Some((_, p)) => Some(p.lock().pane_id),
                            None => None,
                        };

                        if let Some(pane_id) = tmux_pane_id {
                            tmux_domain
                                .inner
                                .cmd_queue
                                .lock()
                                .push_back(Box::new(SelectPane { pane_id: pane_id }));
                            TmuxDomainState::schedule_send_next_command(domain_id);
                        }
                    }
                    MuxNotification::WindowInvalidated(window_id) => {
                        if let Some(window) = mux.get_window(window_id) {
                            let Some(tab) = window.get_active() else {
                                return;
                            };
                            let tmux_window_id = match tmux_domain
                                .inner
                                .gui_tabs
                                .lock()
                                .iter()
                                .find(|(_, t)| t.tab_id == tab.tab_id())
                            {
                                Some((_, t)) => Some(t.tmux_window_id),
                                None => None,
                            };
                            if let Some(window_id) = tmux_window_id {
                                tmux_domain.inner.cmd_queue.lock().push_back(Box::new(
                                    SelectWindow {
                                        window_id: window_id,
                                    },
                                ));
                                TmuxDomainState::schedule_send_next_command(domain_id);
                            }
                        }
                    }
                    _ => {}
                }
            })
            .detach();
            true
        });
    }
}
