impl QuickSelectRenderable {
    fn compute_search_row(&self) -> StableRowIndex {
        let dims = self.delegate.get_dimensions();
        let top = self.viewport.unwrap_or_else(|| dims.physical_top);
        let bottom = (top + dims.viewport_rows as StableRowIndex).saturating_sub(1);
        bottom
    }

    fn close(&self) {
        TermWindow::schedule_cancel_overlay_for_pane(self.window.clone(), self.delegate.pane_id());
    }

    fn set_viewport(&self, row: Option<StableRowIndex>) {
        let dims = self.delegate.get_dimensions();
        let pane_id = self.delegate.pane_id();
        self.window
            .notify(TermWindowNotif::Apply(Box::new(move |term_window| {
                term_window.set_viewport(pane_id, row, dims);
            })));
    }

    fn check_for_resize(&mut self) {
        let dims = self.delegate.get_dimensions();
        if dims.cols == self.width && dims.viewport_rows == self.height {
            return;
        }

        self.width = dims.cols;
        self.height = dims.viewport_rows;

        let pos = self.result_pos;
        self.update_search(false);
        self.result_pos = pos;
    }

    fn recompute_results(&mut self) {
        /// Produce the sorted seq of unique match_ids from the results
        fn compute_uniq_results(results: &[SearchResult]) -> Vec<usize> {
            let mut ids: Vec<usize> = results.iter().map(|sr| sr.match_id).collect();
            ids.sort();
            ids.dedup();
            ids
        }

        let uniq_results = compute_uniq_results(&self.results);

        // Label each unique result
        let labels = compute_labels_for_alphabet(
            if !self.args.alphabet.is_empty() {
                &self.args.alphabet
            } else {
                &self.config.quick_select_alphabet
            },
            uniq_results.len(),
        );
        self.by_label.clear();

        // Keep track of match_id -> label
        let mut assigned_labels: HashMap<usize, usize> = HashMap::new();

        // Work through the results in reverse order, so that we assign eg: `a` to the
        // bottom-right-most result first and so on
        for (result_index, res) in self.results.iter().enumerate().rev() {
            // Figure out which label to use based on the match_id
            let label_index = match assigned_labels.get(&res.match_id).copied() {
                Some(idx) => idx,
                None => {
                    let idx = assigned_labels.len();
                    assigned_labels.insert(res.match_id, idx);
                    idx
                }
            };
            let label = match labels.get(label_index) {
                Some(l) => l,
                None => {
                    // There are more result candidates than the alphabet
                    // can support, so we skip this one and keep looking:
                    // we may still have matches that have an assigned
                    // label, so we keep going rather than breaking
                    // out of the loop.
                    continue;
                }
            };

            self.by_label.entry(label.clone()).or_insert(result_index);
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
                    label: label.clone(),
                };

                let matches = self.by_line.entry(idx).or_insert_with(|| vec![]);
                matches.push(result);

                self.dirty_results.add(idx);
            }
        }
    }

    fn update_search(&mut self, is_initial_run: bool) {
        for idx in self.by_line.keys() {
            self.dirty_results.add(*idx);
        }
        if let Some(idx) = self.last_bar_pos.as_ref() {
            self.dirty_results.add(*idx);
        }

        self.results.clear();
        self.by_line.clear();
        self.result_pos.take();

        let bar_pos = self.compute_search_row();
        self.dirty_results.add(bar_pos);

        if !self.pattern.is_empty() {
            let pane: Arc<dyn Pane> = self.delegate.clone();
            let window = self.window.clone();
            let pattern = self.pattern.clone();
            let scope = self.args.scope_lines;
            let viewport = self.viewport;
            promise::spawn::spawn(async move {
                let dims = pane.get_dimensions();
                let scope = scope.unwrap_or(1000).max(dims.viewport_rows);
                let top = viewport.unwrap_or(dims.physical_top);
                let range = top.saturating_sub(scope as StableRowIndex)
                    ..top + (dims.viewport_rows + scope) as StableRowIndex;
                let limit = None;
                let mut results = pane.search(pattern, range, limit).await?;
                results.sort();

                let pane_id = pane.pane_id();
                let mut results = Some(results);
                window.notify(TermWindowNotif::Apply(Box::new(move |term_window| {
                    let state = term_window.pane_state(pane_id);
                    if let Some(overlay) = state.overlay.as_ref() {
                        if let Some(search_overlay) =
                            overlay.pane.downcast_ref::<QuickSelectOverlay>()
                        {
                            let mut r = search_overlay.renderer.lock();
                            r.results = results.take().unwrap();
                            r.recompute_results();
                            let num_results = r.results.len();

                            if !r.results.is_empty() {
                                match &r.viewport {
                                    Some(y) if is_initial_run => {
                                        r.result_pos = r
                                            .results
                                            .iter()
                                            .position(|result| result.start_y >= *y);
                                    }
                                    _ => {
                                        r.activate_match_number(num_results - 1);
                                    }
                                }
                            } else {
                                if !is_initial_run {
                                    r.set_viewport(None);
                                }
                                r.clear_selection();
                            }
                        }
                    }
                })));
                anyhow::Result::<()>::Ok(())
            })
            .detach();
        } else {
            if !is_initial_run {
                self.set_viewport(None);
            }
            self.clear_selection();
        }
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

    fn select_and_copy_match_number(&mut self, n: usize, paste: bool) {
        let result = self.results[n].clone();

        let pane_id = self.delegate.pane_id();
        let action = self.args.action.clone();
        let skip_action_on_paste = self.args.skip_action_on_paste;
        self.window
            .notify(TermWindowNotif::Apply(Box::new(move |term_window| {
                let mux = mux::Mux::get();
                if let Some(pane) = mux.get_pane(pane_id) {
                    {
                        let mut selection = term_window.selection(pane_id);
                        let start = SelectionCoordinate::x_y(result.start_x, result.start_y);
                        selection.origin = Some(start);
                        selection.range = Some(SelectionRange {
                            start,
                            // inclusive range for selection, but the result
                            // range is exclusive
                            end: SelectionCoordinate::x_y(
                                result.end_x.saturating_sub(1),
                                result.end_y,
                            ),
                        });
                        // Ensure that selection doesn't get invalidated when
                        // the overlay is closed
                        selection.seqno = pane.get_current_seqno();
                    }

                    let text = term_window.selection_text(&pane);
                    if !text.is_empty() {
                        if paste {
                            let _ = pane.send_paste(&text);
                        }
                        if let Some(action) = action {
                            if !paste || !skip_action_on_paste {
                                let _ = term_window.perform_key_assignment(&pane, &action);
                            }
                        } else {
                            term_window.copy_to_clipboard(
                                ClipboardCopyDestination::ClipboardAndPrimarySelection,
                                text,
                            );
                        }
                    }
                }
            })));
    }

    fn activate_match_number(&mut self, n: usize) {
        self.result_pos.replace(n);
        let result = self.results[n].clone();
        self.set_viewport(Some(result.start_y));
    }
}
