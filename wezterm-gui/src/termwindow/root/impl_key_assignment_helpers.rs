impl TermWindow {
    fn start_search_overlay(&mut self, pattern: &Pattern) -> anyhow::Result<()> {
        if let Some(pane) = self.get_active_pane_or_overlay() {
            let mut replace_current = false;
            if let Some(existing) = pane.downcast_ref::<CopyOverlay>() {
                let mut params = existing.get_params();
                params.editing_search = true;
                if !pattern.is_empty() {
                    params.pattern = self.resolve_search_pattern(pattern.clone(), &pane);
                }
                existing.apply_params(params);
                replace_current = true;
            } else {
                let search = CopyOverlay::with_pane(
                    self,
                    &pane,
                    CopyModeParams {
                        pattern: self.resolve_search_pattern(pattern.clone(), &pane),
                        editing_search: true,
                    },
                )?;
                self.assign_overlay_for_pane(pane.pane_id(), search);
            }
            self.activate_overlay_key_table(pane.pane_id(), "search_mode", replace_current);
        }
        Ok(())
    }

    fn activate_copy_mode_overlay(&mut self) -> anyhow::Result<()> {
        if let Some(pane) = self.get_active_pane_or_overlay() {
            let mut replace_current = false;
            if let Some(existing) = pane.downcast_ref::<CopyOverlay>() {
                let mut params = existing.get_params();
                params.editing_search = false;
                existing.apply_params(params);
                replace_current = true;
            } else {
                let copy = CopyOverlay::with_pane(
                    self,
                    &pane,
                    CopyModeParams {
                        pattern: MuxPattern::default(),
                        editing_search: false,
                    },
                )?;
                self.assign_overlay_for_pane(pane.pane_id(), copy);
            }
            self.activate_overlay_key_table(pane.pane_id(), "copy_mode", replace_current);
        }
        Ok(())
    }

    fn activate_overlay_key_table(
        &mut self,
        pane_id: PaneId,
        name: &'static str,
        replace_current: bool,
    ) {
        self.pane_state(pane_id).overlay.as_mut().map(|overlay| {
            overlay.key_table_state.activate(KeyTableArgs {
                name,
                timeout_milliseconds: None,
                replace_current,
                one_shot: false,
                until_unknown: false,
                prevent_fallback: false,
            });
        });
    }

    fn switch_workspace_relative(&self, delta: isize) {
        let mux = Mux::get();
        let workspace = mux.active_workspace();
        let workspaces = mux.iter_workspaces();
        let idx = workspaces.iter().position(|w| *w == workspace).unwrap_or(0);
        let new_idx = idx as isize + delta;
        let new_idx = if new_idx < 0 {
            workspaces.len() as isize + new_idx
        } else {
            new_idx
        };
        let new_idx = new_idx as usize % workspaces.len();
        if let Some(w) = workspaces.get(new_idx) {
            front_end().switch_workspace(w);
        }
    }

    fn switch_to_workspace(&self, name: &Option<String>, spawn: &Option<SpawnCommand>) {
        let activity = crate::Activity::new();
        let mux = Mux::get();
        let name = name
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or_else(|| mux.generate_workspace_name());
        let switcher = crate::frontend::WorkspaceSwitcher::new(&name);
        mux.set_active_workspace(&name);

        if mux.iter_windows_in_workspace(&name).is_empty() {
            let spawn = spawn.as_ref().map(|s| s.clone()).unwrap_or_default();
            let size = self.terminal_size;
            let term_config = Arc::new(TermConfig::with_config(self.config.clone()));
            let src_window_id = self.mux_window_id;

            promise::spawn::spawn(async move {
                if let Err(err) = crate::spawn::spawn_command_internal(
                    spawn,
                    SpawnWhere::NewWindow,
                    size,
                    Some(src_window_id),
                    term_config,
                )
                .await
                {
                    log::error!("Failed to spawn: {:#}", err);
                }
                switcher.do_switch();
                drop(activity);
            })
            .detach();
        } else {
            switcher.do_switch();
        }
    }
}
