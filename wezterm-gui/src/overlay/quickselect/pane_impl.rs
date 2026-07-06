impl Pane for QuickSelectOverlay {
    fn pane_id(&self) -> PaneId {
        self.delegate.pane_id()
    }

    fn get_title(&self) -> String {
        self.delegate.get_title()
    }

    fn send_paste(&self, _text: &str) -> anyhow::Result<()> {
        // Ignore
        Ok(())
    }

    fn reader(&self) -> anyhow::Result<Option<Box<dyn std::io::Read + Send>>> {
        Ok(None)
    }

    fn writer(&self) -> MappedMutexGuard<'_, dyn std::io::Write> {
        self.delegate.writer()
    }

    fn resize(&self, size: TerminalSize) -> anyhow::Result<()> {
        self.delegate.resize(size)
    }

    fn key_up(&self, _key: KeyCode, _mods: KeyModifiers) -> anyhow::Result<()> {
        Ok(())
    }

    fn key_down(&self, key: KeyCode, mods: KeyModifiers) -> anyhow::Result<()> {
        let mods = mods.remove_positional_mods();
        match (key, mods) {
            (KeyCode::Escape, KeyModifiers::NONE) => self.renderer.lock().close(),
            (KeyCode::UpArrow, KeyModifiers::NONE)
            | (KeyCode::Enter, KeyModifiers::NONE)
            | (KeyCode::Char('p'), KeyModifiers::CTRL) => {
                // Move to prior match
                let mut r = self.renderer.lock();
                if let Some(cur) = r.result_pos.as_ref() {
                    let prior = if *cur > 0 {
                        cur - 1
                    } else {
                        r.results.len() - 1
                    };
                    r.activate_match_number(prior);
                }
            }
            (KeyCode::PageUp, KeyModifiers::NONE) => {
                // Skip this page of matches and move up to the first match from
                // the prior page.
                let dims = self.delegate.get_dimensions();
                let mut r = self.renderer.lock();
                if let Some(cur) = r.result_pos {
                    let top = r.viewport.unwrap_or(dims.physical_top);
                    let prior = top - dims.viewport_rows as isize;
                    if let Some(pos) = r
                        .results
                        .iter()
                        .position(|res| res.start_y > prior && res.start_y < top)
                    {
                        r.activate_match_number(pos);
                    } else {
                        r.activate_match_number(cur.saturating_sub(1));
                    }
                }
            }
            (KeyCode::PageDown, KeyModifiers::NONE) => {
                // Skip this page of matches and move down to the first match from
                // the next page.
                let dims = self.delegate.get_dimensions();
                let mut r = self.renderer.lock();
                if let Some(cur) = r.result_pos {
                    let top = r.viewport.unwrap_or(dims.physical_top);
                    let bottom = top + dims.viewport_rows as isize;
                    if let Some(pos) = r.results.iter().position(|res| res.start_y >= bottom) {
                        r.activate_match_number(pos);
                    } else {
                        let len = r.results.len().saturating_sub(1);
                        r.activate_match_number(cur.min(len));
                    }
                }
            }
            (KeyCode::DownArrow, KeyModifiers::NONE) | (KeyCode::Char('n'), KeyModifiers::CTRL) => {
                // Move to next match
                let mut r = self.renderer.lock();
                if let Some(cur) = r.result_pos.as_ref() {
                    let next = if *cur + 1 >= r.results.len() {
                        0
                    } else {
                        *cur + 1
                    };
                    r.activate_match_number(next);
                }
            }
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                // Type to add to the selection
                let mut r = self.renderer.lock();
                r.selection.push(c);
                let lowered = r.selection.to_lowercase();
                let paste = lowered != r.selection;
                if let Some(result_index) = r.by_label.get(&lowered).cloned() {
                    r.select_and_copy_match_number(result_index, paste);
                    r.close();
                } else {
                    r.recompute_results();
                }
            }
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                // Backspace to edit the selection
                let mut r = self.renderer.lock();
                r.selection.pop();
                r.recompute_results();
            }
            (KeyCode::Char('u'), KeyModifiers::CTRL) => {
                // CTRL-u to clear the selection
                let mut r = self.renderer.lock();
                r.selection.clear();
                r.recompute_results();
            }
            _ => {}
        }
        Ok(())
    }

    fn mouse_event(&self, event: MouseEvent) -> anyhow::Result<()> {
        self.delegate.mouse_event(event)
    }

    fn perform_actions(&self, actions: Vec<termwiz::escape::Action>) {
        self.delegate.perform_actions(actions)
    }

    fn is_dead(&self) -> bool {
        self.delegate.is_dead()
    }

    fn palette(&self) -> ColorPalette {
        self.delegate.palette()
    }
    fn domain_id(&self) -> DomainId {
        self.delegate.domain_id()
    }

    fn erase_scrollback(&self, erase_mode: ScrollbackEraseMode) {
        self.delegate.erase_scrollback(erase_mode)
    }

    fn is_mouse_grabbed(&self) -> bool {
        // Force grabbing off while we're searching
        false
    }

    fn is_alt_screen_active(&self) -> bool {
        false
    }

    fn set_clipboard(&self, clipboard: &Arc<dyn Clipboard>) {
        self.delegate.set_clipboard(clipboard)
    }

    fn get_current_working_dir(&self, policy: CachePolicy) -> Option<Url> {
        self.delegate.get_current_working_dir(policy)
    }

    fn get_cursor_position(&self) -> StableCursorPosition {
        // move to the search box
        let renderer = self.renderer.lock();
        StableCursorPosition {
            x: 8 + wezterm_term::unicode_column_width(&renderer.selection, None),
            y: renderer.compute_search_row(),
            shape: termwiz::surface::CursorShape::SteadyBlock,
            visibility: termwiz::surface::CursorVisibility::Visible,
        }
    }

    fn get_current_seqno(&self) -> SequenceNo {
        self.delegate.get_current_seqno()
    }

    fn get_changed_since(
        &self,
        lines: Range<StableRowIndex>,
        seqno: SequenceNo,
    ) -> RangeSet<StableRowIndex> {
        let mut dirty = self.delegate.get_changed_since(lines.clone(), seqno);
        dirty.add_set(&self.renderer.lock().dirty_results);
        dirty.intersection_with_range(lines)
    }

    fn for_each_logical_line_in_stable_range_mut(
        &self,
        lines: Range<StableRowIndex>,
        for_line: &mut dyn ForEachPaneLogicalLine,
    ) {
        self.delegate
            .for_each_logical_line_in_stable_range_mut(lines, for_line);
    }

    fn get_logical_lines(&self, lines: Range<StableRowIndex>) -> Vec<LogicalLine> {
        self.delegate.get_logical_lines(lines)
    }

    fn with_lines_mut(&self, lines: Range<StableRowIndex>, with_lines: &mut dyn WithPaneLines) {
        let mut renderer = self.renderer.lock();
        // Take care to access self.delegate methods here before we get into
        // calling into its own with_lines_mut to avoid a runtime
        // borrow erro!
        renderer.check_for_resize();
        let dims = self.get_dimensions();
        let search_row = renderer.compute_search_row();

        struct OverlayLines<'a> {
            with_lines: &'a mut dyn WithPaneLines,
            dims: RenderableDimensions,
            search_row: StableRowIndex,
            renderer: &'a mut QuickSelectRenderable,
        }

        self.delegate.with_lines_mut(
            lines,
            &mut OverlayLines {
                with_lines,
                dims,
                search_row,
                renderer: &mut *renderer,
            },
        );

        impl<'a> WithPaneLines for OverlayLines<'a> {
            fn with_lines_mut(&mut self, first_row: StableRowIndex, lines: &mut [&mut Line]) {
                let mut overlay_lines = vec![];

                let config = &self.renderer.config;
                let colors = config.resolved_palette.clone();
                let disable_attr = config.quick_select_remove_styling;

                // Process the lines; for the search row we want to render instead
                // the search UI.
                // For rows with search results, we want to highlight the matching ranges

                let lowered_prefix = self.renderer.selection.to_lowercase();
                for (idx, line) in lines.iter_mut().enumerate() {
                    let mut line: Line = line.clone();
                    if disable_attr {
                        line.cells_mut_for_attr_changes_only()
                            .iter_mut()
                            .for_each(|cell| cell.attrs_mut().clear());
                        line.clear_appdata();
                    }
                    let stable_idx = idx as StableRowIndex + first_row;
                    self.renderer.dirty_results.remove(stable_idx);
                    if stable_idx == self.search_row {
                        // Replace with search UI
                        let rev = CellAttributes::default().set_reverse(true).clone();
                        line.fill_range(0..self.dims.cols, &Cell::new(' ', rev.clone()), SEQ_ZERO);
                        line.overlay_text_with_attribute(
                            0,
                            &format!(
                                "Select: {}  (type highlighted prefix to {}, uppercase pastes, ESC to cancel)",
                                self.renderer.selection,
                                if self.renderer.args.label.is_empty() {
                                    "copy"
                                } else {
                                    &self.renderer.args.label
                                },
                            ),
                            rev,
                            SEQ_ZERO,
                        );
                        self.renderer.last_bar_pos = Some(self.search_row);
                        line.clear_appdata();
                    } else if let Some(matches) = self.renderer.by_line.get(&stable_idx) {
                        for m in matches {
                            if !label_matches_selection(&m.label, &lowered_prefix) {
                                // Skip displaying this label, it doesn't match the current filter.
                                continue;
                            }
                            // highlight
                            for cell_idx in m.range.clone() {
                                if let Some(cell) =
                                    line.cells_mut_for_attr_changes_only().get_mut(cell_idx)
                                {
                                    cell.attrs_mut()
                                        .set_background(
                                            colors
                                                .quick_select_match_bg
                                                .unwrap_or(AnsiColor::Black.into()),
                                        )
                                        .set_foreground(
                                            colors
                                                .quick_select_match_fg
                                                .unwrap_or(AnsiColor::Green.into()),
                                        )
                                        .set_reverse(false)
                                        .set_intensity(Intensity::Bold);
                                }
                            }
                            for (idx, c) in m.label.chars().enumerate() {
                                let mut attr = line
                                    .get_cell(idx)
                                    .map(|cell| cell.attrs().clone())
                                    .unwrap_or_else(|| CellAttributes::default());
                                attr.set_background(
                                    colors
                                        .quick_select_label_bg
                                        .unwrap_or(AnsiColor::Black.into()),
                                )
                                .set_foreground(
                                    colors
                                        .quick_select_label_fg
                                        .unwrap_or(AnsiColor::Olive.into()),
                                )
                                .set_reverse(false)
                                .set_intensity(Intensity::Bold);
                                line.set_cell(m.range.start + idx, Cell::new(c, attr), SEQ_ZERO);
                            }
                        }
                        line.clear_appdata();
                    }
                    overlay_lines.push(line);
                }

                let mut overlay_refs: Vec<&mut Line> = overlay_lines.iter_mut().collect();
                self.with_lines.with_lines_mut(first_row, &mut overlay_refs);
            }
        }
    }

    fn get_lines(&self, lines: Range<StableRowIndex>) -> (StableRowIndex, Vec<Line>) {
        let mut renderer = self.renderer.lock();
        renderer.check_for_resize();
        let dims = self.get_dimensions();

        let (top, mut lines) = self.delegate.get_lines(lines);
        let colors = renderer.config.resolved_palette.clone();
        let disable_attr = renderer.config.quick_select_remove_styling;

        // Process the lines; for the search row we want to render instead
        // the search UI.
        // For rows with search results, we want to highlight the matching ranges
        let search_row = renderer.compute_search_row();
        for (idx, line) in lines.iter_mut().enumerate() {
            if disable_attr {
                line.cells_mut_for_attr_changes_only()
                    .iter_mut()
                    .for_each(|cell| cell.attrs_mut().clear());
            }
            let stable_idx = idx as StableRowIndex + top;
            renderer.dirty_results.remove(stable_idx);
            if stable_idx == search_row {
                // Replace with search UI
                let rev = CellAttributes::default().set_reverse(true).clone();
                line.fill_range(0..dims.cols, &Cell::new(' ', rev.clone()), SEQ_ZERO);
                line.overlay_text_with_attribute(
                    0,
                    &format!(
                        "Select: {}  (type highlighted prefix to {}, uppercase pastes, ESC to cancel)",
                        renderer.selection,
                        if renderer.args.label.is_empty() {
                            "copy"
                        } else {
                            &renderer.args.label
                        },
                    ),
                    rev,
                    SEQ_ZERO,
                );
                renderer.last_bar_pos = Some(search_row);
            } else if let Some(matches) = renderer.by_line.get(&stable_idx) {
                for m in matches {
                    // highlight
                    for cell_idx in m.range.clone() {
                        if let Some(cell) = line.cells_mut_for_attr_changes_only().get_mut(cell_idx)
                        {
                            cell.attrs_mut()
                                .set_background(
                                    colors
                                        .quick_select_match_bg
                                        .unwrap_or(AnsiColor::Black.into()),
                                )
                                .set_foreground(
                                    colors
                                        .quick_select_match_fg
                                        .unwrap_or(AnsiColor::Green.into()),
                                )
                                .set_reverse(false)
                                .set_intensity(Intensity::Bold);
                        }
                    }
                    for (idx, c) in m.label.chars().enumerate() {
                        let mut attr = line
                            .get_cell(idx)
                            .map(|cell| cell.attrs().clone())
                            .unwrap_or_else(|| CellAttributes::default());
                        attr.set_background(
                            colors
                                .quick_select_label_bg
                                .unwrap_or(AnsiColor::Black.into()),
                        )
                        .set_foreground(
                            colors
                                .quick_select_label_fg
                                .unwrap_or(AnsiColor::Olive.into()),
                        )
                        .set_reverse(false)
                        .set_intensity(Intensity::Bold);
                        line.set_cell(m.range.start + idx, Cell::new(c, attr), SEQ_ZERO);
                    }
                }
            }
        }

        (top, lines)
    }

    fn get_dimensions(&self) -> RenderableDimensions {
        self.delegate.get_dimensions()
    }
}
