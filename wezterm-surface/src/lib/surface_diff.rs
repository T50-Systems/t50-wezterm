impl Surface {
    fn repaint_all(&self) -> Vec<Change> {
        let mut result = vec![
            // Home the cursor and clear the screen to defaults.  Hide the
            // cursor while we're repainting.
            Change::CursorVisibility(CursorVisibility::Hidden),
            Change::ClearScreen(Default::default()),
        ];

        if !self.title.is_empty() {
            result.push(Change::Title(self.title.to_owned()));
        }

        let mut attr = CellAttributes::default();

        let crlf = Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Relative(1),
        };

        // Walk backwards through the lines; the goal is to determine
        // if the screen ends with a number of clear lines that we
        // can coalesce together as a ClearToEndOfScreen op.
        // We track the index (from the end) of the last matching
        // run, together with the color of that run.
        let mut trailing_color = None;
        let mut trailing_idx = None;

        for (idx, line) in self.lines.iter().rev().enumerate() {
            let changes = line.changes(&attr);
            if changes.is_empty() {
                // The line recorded no changes; this means that the line
                // consists of spaces and the default background color
                match trailing_color {
                    Some(other) if other != Default::default() => {
                        // Color doesn't match up, so we have to stop
                        // looking for the ClearToEndOfScreen run here
                        break;
                    }
                    // Color does match
                    Some(_) => continue,
                    // we don't have a run, we should start one
                    None => {
                        trailing_color = Some(Default::default());
                        trailing_idx = Some(idx);
                        continue;
                    }
                }
            } else {
                let last_change = changes.len() - 1;
                match (&changes[last_change], trailing_color) {
                    (&Change::ClearToEndOfLine(ref color), None) => {
                        trailing_color = Some(*color);
                        trailing_idx = Some(idx);
                    }
                    (&Change::ClearToEndOfLine(ref color), Some(other)) => {
                        if other == *color {
                            trailing_idx = Some(idx);
                            continue;
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        }

        for (idx, line) in self.lines.iter().enumerate() {
            match trailing_idx {
                Some(t) if self.height - t == idx => {
                    let color =
                        trailing_color.expect("didn't set trailing_color along with trailing_idx");

                    // The first in the sequence of the ClearToEndOfLine may
                    // be batched up here; let's remove it if that is the case.
                    let last_result = result.len() - 1;
                    match result[last_result] {
                        Change::ClearToEndOfLine(col) if col == color => {
                            result.remove(last_result);
                        }
                        _ => {}
                    }

                    result.push(Change::ClearToEndOfScreen(color));
                    break;
                }
                _ => {}
            }

            let mut changes = line.changes(&attr);

            if idx != 0 {
                // We emit a relative move at the end of each
                // line with the theory that this will translate
                // to a short \r\n sequence rather than the longer
                // absolute cursor positioning sequence
                result.push(crlf.clone());
            }

            result.append(&mut changes);
            if let Some(c) = line.visible_cells().last() {
                attr = c.attrs().clone();
            }
        }

        // Remove any trailing sequence of cursor movements, as we're
        // going to just finish up with an absolute move anyway.
        loop {
            let result_len = result.len();
            if result_len == 0 {
                break;
            }
            match result[result_len - 1] {
                Change::CursorPosition { .. } => {
                    result.remove(result_len - 1);
                }
                _ => break,
            }
        }

        // Place the cursor at its intended position, but only if we moved the
        // cursor.  We don't explicitly track movement but can infer it from the
        // size of the results: results will have an initial ClearScreen entry
        // that homes the cursor and a CursorShape entry that hides the cursor.
        // If the screen is otherwise blank there will be no further entries
        // and we don't need to emit cursor movement.  However, in the
        // optimization passes above, we may have removed some number of
        // movement entries, so let's be sure to check the cursor position to
        // make sure that we don't fail to emit movement.

        let moved_cursor = result.len() != 2;
        if moved_cursor || self.xpos != 0 || self.ypos != 0 {
            result.push(Change::CursorPosition {
                x: Position::Absolute(self.xpos),
                y: Position::Absolute(self.ypos),
            });
        }

        // Set the intended cursor shape.  We hid the cursor at the start
        // of the repaint, so no need to hide it again.
        if self.cursor_visibility != CursorVisibility::Hidden {
            result.push(Change::CursorVisibility(CursorVisibility::Visible));
            if let Some(shape) = self.cursor_shape {
                result.push(Change::CursorShape(shape));
            }
        }

        result
    }

    /// Computes the change stream required to make the region within `self`
    /// at coordinates `x`, `y` and size `width`, `height` look like the
    /// same sized region within `other` at coordinates `other_x`, `other_y`.
    ///
    /// `other` and `self` may be the same, causing regions within the same
    /// `Surface` to be differenced; this is used by the `copy_region` method.
    ///
    /// The returned list of `Change`s can be passed to the `add_changes` method
    /// to make the region within self match the region within other.
    #[allow(clippy::too_many_arguments)]
    pub fn diff_region(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        other: &Surface,
        other_x: usize,
        other_y: usize,
    ) -> Vec<Change> {
        let mut diff_state = DiffState::default();

        for ((row_num, line), other_line) in self
            .lines
            .iter()
            .enumerate()
            .skip(y)
            .take_while(|(row_num, _)| *row_num < y + height)
            .zip(other.lines.iter().skip(other_y))
        {
            diff_line(
                &mut diff_state,
                line,
                row_num,
                other_line,
                x,
                width,
                other_x,
            );
        }

        diff_state.changes
    }

    pub fn diff_lines(&self, other_lines: Vec<&Line>) -> Vec<Change> {
        let mut diff_state = DiffState::default();
        for ((row_num, line), other_line) in self.lines.iter().enumerate().zip(other_lines.iter()) {
            diff_line(&mut diff_state, line, row_num, other_line, 0, line.len(), 0);
        }
        diff_state.changes
    }

    pub fn diff_against_numbered_line(&self, row_num: usize, other_line: &Line) -> Vec<Change> {
        let mut diff_state = DiffState::default();
        if let Some(line) = self.lines.get(row_num) {
            diff_line(&mut diff_state, line, row_num, other_line, 0, line.len(), 0);
        }
        diff_state.changes
    }

    /// Computes the change stream required to make `self` have the same
    /// screen contents as `other`.
    pub fn diff_screens(&self, other: &Surface) -> Vec<Change> {
        self.diff_region(0, 0, self.width, self.height, other, 0, 0)
    }

    /// Draw the contents of `other` into self at the specified coordinates.
    /// The required updates are recorded as Change entries as well as stored
    /// in the screen line/cell data.
    /// Saves the cursor position and attributes that were in effect prior to
    /// calling `draw_from_screen` and restores them after applying the changes
    /// from the other surface.
    pub fn draw_from_screen(&mut self, other: &Surface, x: usize, y: usize) -> SequenceNo {
        let attrs = self.attributes.clone();
        let cursor = (self.xpos, self.ypos);
        let changes = self.diff_region(x, y, other.width, other.height, other, 0, 0);
        let seq = self.add_changes(changes);
        self.xpos = cursor.0;
        self.ypos = cursor.1;
        self.attributes = attrs;
        seq
    }

    /// Copy the contents of the specified region to the same sized
    /// region elsewhere in the screen display.
    /// The regions may overlap.
    /// # Panics
    /// The destination region must be the same size as the source
    /// (which is implied by the function parameters) and must fit
    /// within the width and height of the Surface or this operation
    /// will panic.
    pub fn copy_region(
        &mut self,
        src_x: usize,
        src_y: usize,
        width: usize,
        height: usize,
        dest_x: usize,
        dest_y: usize,
    ) -> SequenceNo {
        let changes = self.diff_region(dest_x, dest_y, width, height, self, src_x, src_y);
        self.add_changes(changes)
    }
}
