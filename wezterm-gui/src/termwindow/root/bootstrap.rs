impl TermWindow {
    fn load_os_parameters(&mut self) {
        if let Some(ref window) = self.window {
            self.os_parameters = match window.get_os_parameters(&self.config, self.window_state) {
                Ok(os_parameters) => os_parameters,
                Err(err) => {
                    log::warn!("Error while getting OS parameters: {:#}", err);
                    None
                }
            };
        }
    }

    fn close_requested(&mut self, window: &Window) {
        let mux = Mux::get();
        match self.config.window_close_confirmation {
            WindowCloseConfirmation::NeverPrompt => {
                // Immediately kill the tabs and allow the window to close
                mux.kill_window(self.mux_window_id);
                window.close();
                front_end().forget_known_window(window);
            }
            WindowCloseConfirmation::AlwaysPrompt => {
                let tab = match mux.get_active_tab_for_window(self.mux_window_id) {
                    Some(tab) => tab,
                    None => {
                        mux.kill_window(self.mux_window_id);
                        window.close();
                        front_end().forget_known_window(window);
                        return;
                    }
                };

                let mux_window_id = self.mux_window_id;

                let can_close = mux
                    .get_window(mux_window_id)
                    .map_or(false, |w| w.can_close_without_prompting());
                if can_close {
                    mux.kill_window(self.mux_window_id);
                    window.close();
                    front_end().forget_known_window(window);
                    return;
                }
                let window = self.window.clone().unwrap();
                let (overlay, future) = start_overlay(self, &tab, move |tab_id, term| {
                    confirm_close_window(term, mux_window_id, window, tab_id)
                });
                self.assign_overlay(tab.tab_id(), overlay);
                promise::spawn::spawn(future).detach();

                // Don't close right now; let the close happen from
                // the confirmation overlay
            }
        }
    }

    fn focus_changed(&mut self, focused: bool, window: &Window) {
        log::trace!("Setting focus to {:?}", focused);
        self.focused = if focused { Some(Instant::now()) } else { None };
        self.quad_generation += 1;
        self.load_os_parameters();

        if self.focused.is_none() {
            self.last_mouse_click = None;
            self.current_mouse_buttons.clear();
            self.current_mouse_capture = None;
            self.is_click_to_focus_window = false;

            for state in self.pane_state.borrow_mut().values_mut() {
                state.mouse_terminal_coords.take();
            }
        }

        // Reset the cursor blink phase
        self.prev_cursor.bump();

        // force cursor to be repainted
        window.invalidate();

        if let Some(pane) = self.get_active_pane_or_overlay() {
            pane.focus_changed(focused);
        }

        self.update_title();
        self.emit_window_event("window-focus-changed", None);
    }

    fn created(&mut self, ctx: RenderContext) -> anyhow::Result<()> {
        self.render_state = None;

        let render_info = ctx.renderer_info();
        self.opengl_info.replace(render_info.clone());

        match RenderState::new(ctx, &self.fonts, &self.render_metrics, ATLAS_SIZE) {
            Ok(render_state) => {
                log::debug!(
                    "OpenGL initialized! {} wezterm version: {}",
                    render_info,
                    config::wezterm_version(),
                );
                self.render_state.replace(render_state);
            }
            Err(err) => {
                log::error!("failed to create RenderState: {}", err);
            }
        }

        if self.render_state.is_none() {
            panic!("No OpenGL");
        }

        Ok(())
    }
}
