impl CopyRenderable {
    fn compute_search_row(&self) -> StableRowIndex {
        let dims = self.delegate.get_dimensions();
        let top = self.viewport.unwrap_or_else(|| dims.physical_top);
        let bottom = (top + dims.viewport_rows as StableRowIndex).saturating_sub(1);
        bottom
    }

    fn check_for_resize(&mut self) {
        let dims = self.delegate.get_dimensions();
        if dims.cols == self.width && dims.viewport_rows == self.height {
            return;
        }

        self.width = dims.cols;
        self.height = dims.viewport_rows;

        let pos = self.result_pos;
        self.update_search();
        self.result_pos = pos;
    }

    fn incrementally_recompute_results(&mut self, mut results: Vec<SearchResult>) {
        results.sort();
        results.reverse();
        for (result_index, res) in results.iter().enumerate() {
            let result_index = self.results.len() + result_index;
            for idx in res.start_y..=res.end_y {
                let range = if idx == res.start_y && idx == res.end_y {
                    // Range on same line
                    res.start_x..res.end_x
                } else if idx == res.end_y {
                    // final line of multi-line
                    0..res.end_x
                } else if idx == res.start_y {
                    // first line of multi-line
                    res.start_x..self.width
                } else {
                    // a middle line
                    0..self.width
                };

                let result = MatchResult {
                    range,
                    result_index,
                };

                let matches = self.by_line.entry(idx).or_insert_with(|| vec![]);
                matches.push(result);

                self.dirty_results.add(idx);
            }
        }
        self.results.append(&mut results);
    }

    fn schedule_update_search(&mut self) {
        self.typing_cookie += 1;
        let cookie = self.typing_cookie;

        let window = self.window.clone();
        let pane_id = self.delegate.pane_id();

        promise::spawn::spawn(async move {
            smol::Timer::after(Duration::from_millis(350)).await;
            window.notify(TermWindowNotif::Apply(Box::new(move |term_window| {
                let state = term_window.pane_state(pane_id);
                if let Some(overlay) = state.overlay.as_ref() {
                    if let Some(copy_overlay) = overlay.pane.downcast_ref::<CopyOverlay>() {
                        let mut r = copy_overlay.render.lock();
                        if cookie == r.typing_cookie {
                            r.update_search();
                        }
                    }
                }
            })));
            anyhow::Result::<()>::Ok(())
        })
        .detach();
    }

    fn update_search(&mut self) {
        for idx in self.by_line.keys() {
            self.dirty_results.add(*idx);
        }
        if let Some(idx) = self.last_bar_pos.as_ref() {
            self.dirty_results.add(*idx);
        }

        self.results.clear();
        self.by_line.clear();
        self.result_pos.take();

        SAVED_PATTERN.lock().insert(self.tab_id, self.get_pattern());

        let bar_pos = self.compute_search_row();
        self.dirty_results.add(bar_pos);
        self.last_result_seqno = self.delegate.get_current_seqno();

        let pattern = self.get_pattern();
        if !pattern.is_empty() {
            let pane: Arc<dyn Pane> = self.delegate.clone();
            let window = self.window.clone();
            let dims = pane.get_dimensions();

            let end = dims.scrollback_top + dims.scrollback_rows as StableRowIndex;
            let range = end
                .saturating_sub(SEARCH_CHUNK_SIZE)
                .max(dims.scrollback_top)..end;

            self.searching.replace(Searching {
                remain: range.start - dims.scrollback_top,
            });

            promise::spawn::spawn(async move {
                let limit = None;
                log::trace!("Searching for {pattern:?} in {range:?}");
                let results = pane.search(pattern.clone(), range.clone(), limit).await?;

                let pane_id = pane.pane_id();
                let mut results = Some(results);
                window.notify(TermWindowNotif::Apply(Box::new(move |term_window| {
                    let state = term_window.pane_state(pane_id);
                    if let Some(overlay) = state.overlay.as_ref() {
                        if let Some(copy_overlay) = overlay.pane.downcast_ref::<CopyOverlay>() {
                            let mut r = copy_overlay.render.lock();
                            r.processed_search_chunk(pattern, results.take().unwrap(), range);
                        }
                    }
                })));

                anyhow::Result::<()>::Ok(())
            })
            .detach();
        } else {
            self.searching.take();
            self.clear_selection();
        }
        self.window.invalidate();
    }

    fn processed_search_chunk(
        &mut self,
        pattern: Pattern,
        results: Vec<SearchResult>,
        range: Range<StableRowIndex>,
    ) {
        self.window.invalidate();
        if pattern != self.get_pattern() {
            return;
        }
        let is_first = self.results.is_empty();
        self.incrementally_recompute_results(results);

        if is_first {
            if !self.results.is_empty() {
                self.activate_match_number(0);
            } else {
                self.set_viewport(None);
                self.clear_selection();
            }
        }

        let dims = self.delegate.get_dimensions();
        if range.start == dims.scrollback_top {
            self.searching.take();
            return;
        }

        // Search next chunk
        let pane: Arc<dyn Pane> = self.delegate.clone();
        let window = self.window.clone();
        let end = range.start;
        let range = end
            .saturating_sub(SEARCH_CHUNK_SIZE)
            .max(dims.scrollback_top)..end;

        self.searching.replace(Searching {
            remain: range.start - dims.scrollback_top,
        });

        promise::spawn::spawn(async move {
            let limit = None;
            log::trace!("Searching for {pattern:?} in {range:?}");
            let results = pane.search(pattern.clone(), range.clone(), limit).await?;

            let pane_id = pane.pane_id();
            let mut results = Some(results);
            window.notify(TermWindowNotif::Apply(Box::new(move |term_window| {
                let state = term_window.pane_state(pane_id);
                if let Some(overlay) = state.overlay.as_ref() {
                    if let Some(copy_overlay) = overlay.pane.downcast_ref::<CopyOverlay>() {
                        let mut r = copy_overlay.render.lock();
                        r.processed_search_chunk(pattern, results.take().unwrap(), range);
                    }
                }
            })));

            anyhow::Result::<()>::Ok(())
        })
        .detach();
    }

    fn clear_selection(&mut self) {
        let pane_id = self.delegate.pane_id();
        self.window
            .notify(TermWindowNotif::Apply(Box::new(move |term_window| {
                let mut selection = term_window.selection(pane_id);
                selection.origin.take();
                selection.range.take();
            })));
    }

    fn activate_match_number(&mut self, n: usize) {
        self.result_pos.replace(n);
        let result = self.results[n].clone();
        self.cursor.y = result.end_y;
        self.cursor.x = result.end_x.saturating_sub(1);

        let start = SelectionCoordinate::x_y(result.start_x, result.start_y);
        let end = SelectionCoordinate::x_y(result.end_x.saturating_sub(1), result.end_y);
        self.start.replace(start);
        self.adjust_selection(start, SelectionRange { start, end });
    }

    fn clamp_cursor_to_scrollback(&mut self) {
        let dims = self.delegate.get_dimensions();
        if self.cursor.x >= dims.cols {
            self.cursor.x = dims.cols - 1;
        }
        if self.cursor.y < dims.scrollback_top {
            self.cursor.y = dims.scrollback_top;
        }

        let max_row = dims.scrollback_top + dims.scrollback_rows as isize;
        if self.cursor.y >= max_row {
            self.cursor.y = max_row - 1;
        }
    }

    fn select_to_cursor_pos(&mut self) {
        self.clamp_cursor_to_scrollback();
        if let Some(sel_start) = self.start {
            let cursor = SelectionCoordinate::x_y(self.cursor.x, self.cursor.y);

            let (start, end) = match self.selection_mode {
                SelectionMode::Line => {
                    let cursor_is_above_start = self.cursor.y < sel_start.y;

                    let start = SelectionCoordinate::x_y(
                        if cursor_is_above_start {
                            usize::max_value()
                        } else {
                            0
                        },
                        sel_start.y,
                    );
                    let end = SelectionCoordinate::x_y(
                        if cursor_is_above_start {
                            0
                        } else {
                            usize::max_value()
                        },
                        self.cursor.y,
                    );
                    (start, end)
                }
                SelectionMode::SemanticZone => {
                    let zone_range = SelectionRange::zone_around(cursor, &*self.delegate);
                    let start_zone = SelectionRange::zone_around(sel_start, &*self.delegate);

                    let range = zone_range.extend_with(start_zone);

                    (range.start, range.end)
                }
                _ => {
                    let start = SelectionCoordinate {
                        x: sel_start.x,
                        y: sel_start.y,
                    };
                    let end = cursor;
                    (start, end)
                }
            };

            self.adjust_selection(start, SelectionRange { start, end });
        } else {
            self.adjust_viewport_for_cursor_position();
            self.window.invalidate();
        }
    }

    fn adjust_selection(&self, start: SelectionCoordinate, range: SelectionRange) {
        let pane_id = self.delegate.pane_id();
        let window = self.window.clone();
        let mode = self.selection_mode;
        self.window
            .notify(TermWindowNotif::Apply(Box::new(move |term_window| {
                let mut selection = term_window.selection(pane_id);
                selection.origin = Some(start);
                selection.range = Some(range);
                selection.rectangular = mode == SelectionMode::Block;
                window.invalidate();
            })));
        self.adjust_viewport_for_cursor_position();
    }

    fn dimensions(&self) -> Dimensions {
        const VERTICAL_GAP: isize = 5;
        let dims = self.delegate.get_dimensions();
        let vertical_gap = if dims.physical_top <= VERTICAL_GAP {
            1
        } else {
            VERTICAL_GAP
        };
        let top = self.viewport.unwrap_or_else(|| dims.physical_top);
        Dimensions {
            vertical_gap,
            top,
            dims,
        }
    }

    fn adjust_viewport_for_cursor_position(&self) {
        let dims = self.dimensions();

        if dims.top > self.cursor.y {
            // Cursor is off the top of the viewport; adjust
            self.set_viewport(Some(self.cursor.y.saturating_sub(dims.vertical_gap)));
            return;
        }

        let top_gap = self.cursor.y - dims.top;
        if top_gap < dims.vertical_gap {
            // Increase the gap so we can "look ahead"
            self.set_viewport(Some(self.cursor.y.saturating_sub(dims.vertical_gap)));
            return;
        }

        let bottom_gap = (dims.dims.viewport_rows as isize).saturating_sub(top_gap);
        if bottom_gap < dims.vertical_gap {
            self.set_viewport(Some(dims.top + dims.vertical_gap - bottom_gap));
        }
    }

    fn set_viewport(&self, row: Option<StableRowIndex>) {
        let dims = self.delegate.get_dimensions();
        let pane_id = self.delegate.pane_id();
        self.window
            .notify(TermWindowNotif::Apply(Box::new(move |term_window| {
                term_window.set_viewport(pane_id, row, dims);
            })));
    }

    fn close(&self) {
        TermWindow::schedule_cancel_overlay_for_pane(self.window.clone(), self.delegate.pane_id());
    }

    fn move_by_page(&mut self, amount: f64) {
        let dims = self.dimensions();
        let rows = (dims.dims.viewport_rows as f64 * amount) as isize;
        self.cursor.y += rows;
        self.select_to_cursor_pos();
    }

    /// Move to next match
    fn next_match(&mut self) {
        if let Some(cur) = self.result_pos.as_ref() {
            let prior = if *cur > 0 {
                cur - 1
            } else {
                self.results.len() - 1
            };
            self.activate_match_number(prior);
        }
    }

    /// Move to prior match
    fn prior_match(&mut self) {
        if let Some(cur) = self.result_pos.as_ref() {
            let next = if *cur + 1 >= self.results.len() {
                0
            } else {
                *cur + 1
            };
            self.activate_match_number(next);
        }
    }

    /// Skip this page of matches and move down to the first match from
    /// the next page.
    fn next_match_page(&mut self) {
        let dims = self.delegate.get_dimensions();
        if let Some(cur) = self.result_pos {
            let top = self.viewport.unwrap_or(dims.physical_top);
            let prior = top - dims.viewport_rows as isize;
            if let Some(pos) = self
                .results
                .iter()
                .position(|res| res.start_y > prior && res.start_y < top)
            {
                self.activate_match_number(pos);
            } else {
                self.activate_match_number(cur.saturating_sub(1));
            }
        }
    }

    /// Skip this page of matches and move up to the first match from
    /// the prior page.
    fn prior_match_page(&mut self) {
        let dims = self.delegate.get_dimensions();
        if let Some(cur) = self.result_pos {
            let top = self.viewport.unwrap_or(dims.physical_top);
            let bottom = top + dims.viewport_rows as isize;
            if let Some(pos) = self.results.iter().position(|res| res.start_y >= bottom) {
                self.activate_match_number(pos);
            } else {
                let len = self.results.len().saturating_sub(1);
                self.activate_match_number(cur.min(len));
            }
        }
    }

    fn get_pattern(&self) -> Pattern {
        let pattern = self.search_line.get_line().to_string();
        match self.pattern_type {
            PatternType::CaseSensitiveString => Pattern::CaseSensitiveString(pattern),
            PatternType::CaseInSensitiveString => Pattern::CaseInSensitiveString(pattern),
            PatternType::Regex => Pattern::Regex(pattern),
        }
    }

    fn clear_pattern(&mut self) {
        self.search_line.clear();
        self.update_search();
    }

    fn edit_pattern(&mut self) {
        self.editing_search = true;
        self.update_key_table();
    }

    fn accept_pattern(&mut self) {
        self.editing_search = false;
        self.update_key_table();
    }

    fn update_key_table(&mut self) {
        let window = self.window.clone();
        let pane_id = self.delegate.pane_id();

        window.notify(TermWindowNotif::Apply(Box::new(move |term_window| {
            let mut state = term_window.pane_state(pane_id);
            if let Some(overlay) = state.overlay.as_mut() {
                if let Some(copy_overlay) = overlay.pane.downcast_ref::<CopyOverlay>() {
                    let editing_search = copy_overlay.render.lock().editing_search;

                    overlay.key_table_state.activate(KeyTableArgs {
                        name: if editing_search {
                            "search_mode"
                        } else {
                            "copy_mode"
                        },
                        timeout_milliseconds: None,
                        replace_current: true,
                        one_shot: false,
                        until_unknown: false,
                        prevent_fallback: false,
                    });
                }
            }
        })));
    }

    fn cycle_match_type(&mut self) {
        let pattern_type = match &self.pattern_type {
            PatternType::CaseSensitiveString => PatternType::CaseInSensitiveString,
            PatternType::CaseInSensitiveString => PatternType::Regex,
            PatternType::Regex => PatternType::CaseSensitiveString,
        };
        self.pattern_type = pattern_type;
        self.schedule_update_search();
    }

    fn move_to_viewport_middle(&mut self) {
        let dims = self.dimensions();
        self.cursor.y = dims.top + (dims.dims.viewport_rows as isize) / 2;
        self.select_to_cursor_pos();
    }

    fn move_to_viewport_top(&mut self) {
        let dims = self.dimensions();
        self.cursor.y = dims.top + dims.vertical_gap;
        self.select_to_cursor_pos();
    }
}
