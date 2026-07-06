impl GuiFrontEnd {
    pub fn run_forever(&self) -> anyhow::Result<()> {
        self.connection
            .run_message_loop()
            .context("running message loop")
    }

    pub fn gui_windows(&self) -> Vec<GuiWin> {
        let windows = self.known_windows.borrow();
        let mut windows: Vec<GuiWin> = windows
            .iter()
            .map(|(window, &mux_window_id)| GuiWin {
                mux_window_id,
                window: window.clone(),
            })
            .collect();
        windows.sort_by(|a, b| a.window.cmp(&b.window));
        windows
    }

    pub fn reconcile_workspace(&self) -> Future<()> {
        let mut promise = Promise::new();
        let mux = Mux::get();
        let workspace = mux.active_workspace_for_client(&self.client_id);

        if mux.is_workspace_empty(&workspace) {
            // We don't want to silently kill off things that might
            // be running in other workspaces, so let's pick one
            // and activate it
            if self.is_switching_workspace() {
                promise.ok(());
                return promise.get_future().unwrap();
            }
            for workspace in mux.iter_workspaces() {
                if !mux.is_workspace_empty(&workspace) {
                    mux.set_active_workspace_for_client(&self.client_id, &workspace);
                    log::debug!("using {} instead, as it is not empty", workspace);
                    break;
                }
            }
        }

        let workspace = mux.active_workspace_for_client(&self.client_id);
        log::debug!("workspace is {}, fixup windows", workspace);

        let mut mux_windows = mux.iter_windows_in_workspace(&workspace);

        // First, repurpose existing windows.
        // Note that both iter_windows_in_workspace and self.known_windows have a
        // deterministic iteration order, so switching back and forth should result
        // in a consistent mux <-> gui window mapping.
        let known_windows = std::mem::take(&mut *self.known_windows.borrow_mut());
        let mut windows = BTreeMap::new();
        let mut unused = BTreeMap::new();

        for (window, window_id) in known_windows.into_iter() {
            if let Some(idx) = mux_windows.iter().position(|&id| id == window_id) {
                // it already points to the desired mux window
                windows.insert(window, window_id);
                mux_windows.remove(idx);
            } else {
                unused.insert(window, window_id);
            }
        }

        let mut mux_windows = mux_windows.into_iter();

        for (window, old_id) in unused.into_iter() {
            if let Some(mux_window_id) = mux_windows.next() {
                window.notify(TermWindowNotif::SwitchToMuxWindow(mux_window_id));
                windows.insert(window, mux_window_id);
            } else {
                // We have more windows than are in the new workspace;
                // we no longer need this one!
                window.close();
                front_end().spawned_mux_window.borrow_mut().remove(&old_id);
            }
        }

        log::trace!("reconcile: windows -> {:?}", windows);
        *self.known_windows.borrow_mut() = windows;

        let future = promise.get_future().unwrap();

        // then spawn any new windows that are needed
        promise::spawn::spawn(async move {
            while let Some(mux_window_id) = mux_windows.next() {
                if front_end().has_mux_window(mux_window_id)
                    || front_end()
                        .spawned_mux_window
                        .borrow()
                        .contains(&mux_window_id)
                {
                    continue;
                }
                front_end()
                    .spawned_mux_window
                    .borrow_mut()
                    .insert(mux_window_id);
                log::trace!("Creating TermWindow for mux_window_id={}", mux_window_id);
                if let Err(err) = TermWindow::new_window(mux_window_id).await {
                    log::error!("Failed to create window: {:#}", err);
                    let mux = Mux::get();
                    mux.kill_window(mux_window_id);
                    front_end()
                        .spawned_mux_window
                        .borrow_mut()
                        .remove(&mux_window_id);
                }
            }
            *front_end().switching_workspaces.borrow_mut() = false;
            promise.ok(());
        })
        .detach();
        future
    }

    fn has_mux_window(&self, mux_window_id: MuxWindowId) -> bool {
        for &mux_id in self.known_windows.borrow().values() {
            if mux_id == mux_window_id {
                return true;
            }
        }
        false
    }

    pub fn switch_workspace(&self, workspace: &str) {
        let mux = Mux::get();
        mux.set_active_workspace_for_client(&self.client_id, workspace);
        *self.switching_workspaces.borrow_mut() = false;
        self.reconcile_workspace();
    }

    pub fn record_known_window(&self, window: Window, mux_window_id: MuxWindowId) {
        self.known_windows
            .borrow_mut()
            .insert(window, mux_window_id);
        if !self.is_switching_workspace() {
            self.reconcile_workspace();
        }
    }

    pub fn forget_known_window(&self, window: &Window) {
        self.known_windows.borrow_mut().remove(window);
        if !self.is_switching_workspace() {
            self.reconcile_workspace();
        }
    }

    pub fn is_switching_workspace(&self) -> bool {
        *self.switching_workspaces.borrow()
    }

    pub fn gui_window_for_mux_window(&self, mux_window_id: MuxWindowId) -> Option<GuiWin> {
        let windows = self.known_windows.borrow();
        for (window, v) in windows.iter() {
            if *v == mux_window_id {
                return Some(GuiWin {
                    mux_window_id,
                    window: window.clone(),
                });
            }
        }
        None
    }
}
