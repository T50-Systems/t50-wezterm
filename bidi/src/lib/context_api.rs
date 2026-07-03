impl BidiContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn base_level(&self) -> Level {
        self.base_level
    }

    /// When `reorder` is set to true, reordering will apply rule L3 to
    /// non-spacing marks.  This is likely more desirable for terminal
    /// based applications than it is for more modern GUI applications
    /// that feed into eg: harfbuzz.
    pub fn set_reorder_non_spacing_marks(&mut self, reorder: bool) {
        self.reorder_nsm = reorder;
    }

    /// Produces a sequence of `BidiRun` structs that represent runs of
    /// text and their direction (and level) across the entire paragraph.
    pub fn runs<'a>(&'a self) -> impl Iterator<Item = BidiRun> + 'a {
        RunIter {
            pos: 0,
            levels: Cow::Borrowed(&self.levels),
            line_range: 0..self.levels.len(),
        }
    }

    /// Given a line_range (a subslice of the current paragraph that represents
    /// a single wrapped line), this method resets whitespace levels for the line
    /// boundaries, and then returns the set of runs for that line.
    pub fn line_runs(&self, line_range: Range<usize>) -> impl Iterator<Item = BidiRun> {
        let levels = self.reset_whitespace_levels(line_range.clone());

        RunIter {
            pos: 0,
            levels: levels.into(),
            line_range,
        }
    }

    pub fn reordered_runs(&self, line_range: Range<usize>) -> Vec<ReorderedRun> {
        // reorder_line's `level` result includes entries that were
        // removed_by_x9() but `reordered` does NOT (for compatibility with
        // the UCD test suite).
        // We need to account for that when we reorder the levels here!
        let (levels, reordered) = self.reorder_line(line_range);
        let mut reordered_levels = vec![Level(NO_LEVEL); reordered.len()];

        for (vis_idx, &log_idx) in reordered.iter().enumerate() {
            reordered_levels[vis_idx] = levels[log_idx];
        }

        reordered_levels.retain(|l| !l.removed_by_x9());

        let mut runs = vec![];

        let mut idx = 0;
        while idx < reordered_levels.len() {
            let len = span_len(idx, &reordered_levels);
            let level = reordered_levels[idx];
            if !level.removed_by_x9() {
                let idx_range = idx..idx + len;
                let start = reordered[idx_range.clone()].iter().min().unwrap();
                let end = reordered[idx_range.clone()].iter().max().unwrap();
                runs.push(ReorderedRun {
                    direction: level.direction(),
                    level,
                    range: *start..*end + 1,
                    indices: reordered[idx_range].to_vec(),
                });
            }
            idx += len;
        }

        runs
    }

    /// `line_range` indicates a contiguous range of character indices
    /// in the paragraph set via `resolve_paragraph`.
    /// This method returns the reordered set of indices for display
    /// purposes.
    pub fn reorder_line(&self, line_range: Range<usize>) -> (Vec<Level>, Vec<usize>) {
        self.dump_state("before L1");
        let mut levels = self.reset_whitespace_levels(line_range.clone());
        assert_eq!(levels.len(), line_range.end - line_range.start);
        let reordered = self.reverse_levels(line_range.start, &mut levels);

        (levels, reordered)
    }

    /// Performs Rule L3.
    /// This rule is optional and must be enabled by calling the
    /// set_reorder_non_spacing_marks method
    fn reorder_non_spacing_marks(&self, levels: &mut [Level], visual: &mut [usize]) {
        let mut idx = levels.len() - 1;
        loop {
            if idx > 0
                && !levels[idx].removed_by_x9()
                && levels[idx].direction() == Direction::RightToLeft
                && self.orig_char_types[visual[idx]] == BidiClass::NonspacingMark
            {
                // Keep scanning backwards within this level
                let level = levels[idx];
                let seq_end = idx;

                idx -= 1;
                while idx > 0 && levels[idx].removed_by_x9()
                    || (levels[idx] == level
                        && matches!(
                            self.orig_char_types[visual[idx]],
                            BidiClass::LeftToRightEmbedding
                                | BidiClass::RightToLeftEmbedding
                                | BidiClass::LeftToRightOverride
                                | BidiClass::RightToLeftOverride
                                | BidiClass::PopDirectionalFormat
                                | BidiClass::BoundaryNeutral
                                | BidiClass::NonspacingMark
                        ))
                {
                    idx -= 1;
                }

                if levels[idx] != level {
                    idx += 1;
                }

                if seq_end > idx {
                    visual[idx..=seq_end].reverse();
                    levels[idx..=seq_end].reverse();
                }
            }

            if idx == 0 {
                return;
            }
            idx -= 1;
        }
    }

    /// This function runs Rule L2.
    ///
    /// Find the highest level among the resolved levels.
    /// Then from that highest level down to the lowest odd
    /// level, reverse any contiguous runs at that level or higher.
    fn reverse_levels(&self, first_cidx: usize, levels: &mut [Level]) -> Vec<usize> {
        // Not typed as Level because the Step trait required by the loop
        // below is nightly only
        let mut highest_level = 0;
        let mut lowest_odd_level = MAX_DEPTH as i8 + 1;
        let mut no_levels = true;

        for &level in levels.iter() {
            if level.removed_by_x9() {
                continue;
            }

            // Found something other than NO_LEVEL
            no_levels = false;
            highest_level = highest_level.max(level.0);
            if level.0 % 2 == 1 && level.0 < lowest_odd_level {
                lowest_odd_level = level.0;
            }
        }

        if no_levels {
            return vec![];
        }

        // Initial visual order
        let mut visual = vec![];
        for i in 0..levels.len() {
            if levels[i].removed_by_x9() {
                visual.push(DELETED);
            } else {
                visual.push(i + first_cidx);
            }
        }

        // Apply L3. UAX9 has this occur after L2, but we do it
        // before that for consistency with FriBidi's implementation.
        if self.reorder_nsm {
            self.reorder_non_spacing_marks(levels, &mut visual);
        }

        // Apply L2.
        for level in (lowest_odd_level..=highest_level).rev() {
            let level = Level(level);
            let mut i = 0;
            let mut in_range = false;
            let mut significant_range = false;
            let mut first_pos = None;
            let mut last_pos = None;

            while i < levels.len() {
                if levels[i] >= level {
                    if !in_range {
                        in_range = true;
                        first_pos.replace(i);
                    } else {
                        // Hit a second explicit level
                        significant_range = true;
                        last_pos.replace(i);
                    }
                } else if levels[i].removed_by_x9() {
                    // Don't break ranges for deleted controls
                    if in_range {
                        last_pos.replace(i);
                    }
                } else {
                    // End of a range.  Reset the range flag
                    // and rever the range.
                    in_range = false;
                    match (last_pos, first_pos, significant_range) {
                        (Some(last_pos), Some(first_pos), true) if last_pos > first_pos => {
                            visual[first_pos..=last_pos].reverse();
                        }
                        _ => {}
                    }
                    first_pos = None;
                    last_pos = None;
                }
                i += 1;
            }

            if in_range && significant_range {
                match (last_pos, first_pos) {
                    (Some(last_pos), Some(first_pos)) if last_pos > first_pos => {
                        visual[first_pos..=last_pos].reverse();
                    }
                    _ => {}
                }
            }
        }

        visual.retain(|&i| i != DELETED);
        visual
    }
}
