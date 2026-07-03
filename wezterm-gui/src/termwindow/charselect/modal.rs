impl Modal for CharSelector {
    fn perform_assignment(
        &self,
        _assignment: &KeyAssignment,
        _term_window: &mut TermWindow,
    ) -> bool {
        false
    }

    fn mouse_event(&self, _event: MouseEvent, _term_window: &mut TermWindow) -> anyhow::Result<()> {
        Ok(())
    }

    fn key_down(
        &self,
        key: KeyCode,
        mods: KeyModifiers,
        term_window: &mut TermWindow,
    ) -> anyhow::Result<bool> {
        const CTRL_AND_SHIFT: Modifiers = KeyModifiers::CTRL.union(KeyModifiers::SHIFT);

        match (key, mods) {
            (KeyCode::Escape, KeyModifiers::NONE) | (KeyCode::Char('g'), KeyModifiers::CTRL) => {
                term_window.cancel_modal();
            }
            (KeyCode::Char('r'), KeyModifiers::CTRL) => {
                // Cycle the selected group
                let mut group = self.group.borrow_mut();
                *group = group.next();
                self.selection.borrow_mut().clear();
                self.updated_input();
            }
            (KeyCode::Char('R'), KeyModifiers::CTRL) | (KeyCode::Char('r'), CTRL_AND_SHIFT) => {
                // Cycle the selected group in reverse direction
                let mut group = self.group.borrow_mut();
                *group = group.previous();
                self.selection.borrow_mut().clear();
                self.updated_input();
            }
            (KeyCode::PageUp, KeyModifiers::NONE) => {
                self.do_move(Move::PageUp);
            }
            (KeyCode::PageDown, KeyModifiers::NONE) => {
                self.do_move(Move::PageDown);
            }
            (KeyCode::UpArrow, KeyModifiers::NONE) => {
                self.do_move(Move::Up(1));
            }
            (KeyCode::DownArrow, KeyModifiers::NONE) => {
                self.do_move(Move::Down(1));
            }
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                // Type to add to the selection
                let mut selection = self.selection.borrow_mut();
                selection.push(c);
                self.updated_input();
            }
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                // Backspace to edit the selection
                let mut selection = self.selection.borrow_mut();
                selection.pop();
                self.updated_input();
            }
            (KeyCode::Char('u'), KeyModifiers::CTRL) => {
                // CTRL-u to clear the selection
                let mut selection = self.selection.borrow_mut();
                selection.clear();
                self.updated_input();
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                // Enter the selected character to the current pane
                let selected_idx = *self.selected_row.borrow();
                let alias_idx = match self.matches.borrow().as_ref() {
                    None => return Ok(true),
                    Some(results) => match results.matches.get(selected_idx) {
                        Some(i) => *i,
                        None => return Ok(true),
                    },
                };
                let item = &self.aliases[alias_idx];
                if let Err(err) = save_recent(item) {
                    log::error!("Error while saving recents: {err:#}");
                }
                let glyph = item.glyph();
                log::trace!(
                    "selected: {glyph}. copy_on_select={} -> {:?}",
                    self.copy_on_select,
                    self.copy_to
                );

                if self.copy_on_select {
                    term_window.copy_to_clipboard(self.copy_to, glyph.clone());
                }
                if let Some(pane) = term_window.get_active_pane_or_overlay() {
                    pane.writer().write_all(glyph.as_bytes()).ok();
                }
                term_window.cancel_modal();
                return Ok(true);
            }
            _ => return Ok(false),
        }
        term_window.invalidate_modal();
        Ok(true)
    }

    fn computed_element(
        &self,
        term_window: &mut TermWindow,
    ) -> anyhow::Result<Ref<'_, [ComputedElement]>> {
        let selection = self.selection.borrow();
        let selection = selection.as_str();

        let group = *self.group.borrow();

        let mut results = self.matches.borrow_mut();

        let font = term_window
            .fonts
            .char_select_font()
            .expect("to resolve char selection font");
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());

        let max_rows_on_screen = ((term_window.dimensions.pixel_height * 8 / 10)
            / metrics.cell_size.height as usize)
            - 2;
        *self.max_rows_on_screen.borrow_mut() = max_rows_on_screen;

        let rebuild_matches = results
            .as_ref()
            .map(|m| m.selection != selection || m.group != group)
            .unwrap_or(true);
        if rebuild_matches {
            results.replace(MatchResults {
                selection: selection.to_string(),
                matches: compute_matches(selection, &self.aliases, group),
                group,
            });
        };
        let matches = results.as_ref().unwrap();

        if self.element.borrow().is_none() {
            let element = Self::compute(
                term_window,
                selection,
                group,
                &self.aliases,
                matches,
                max_rows_on_screen,
                *self.selected_row.borrow(),
                *self.top_row.borrow(),
            )?;
            self.element.borrow_mut().replace(element);
        }
        Ok(Ref::map(self.element.borrow(), |v| {
            v.as_ref().unwrap().as_slice()
        }))
    }

    fn reconfigure(&self, _term_window: &mut TermWindow) {
        self.element.borrow_mut().take();
    }
}
