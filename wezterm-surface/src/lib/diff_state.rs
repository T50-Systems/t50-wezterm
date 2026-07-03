impl DiffState {
    #[inline]
    fn diff_cells(&mut self, col_num: usize, row_num: usize, cell: CellRef, other_cell: CellRef) {
        if cell.same_contents(&other_cell) {
            return;
        }

        self.set_cell(col_num, row_num, other_cell);
    }

    #[inline]
    fn set_cell(&mut self, col_num: usize, row_num: usize, other_cell: CellRef) {
        self.cursor = match self.cursor.take() {
            Some((cursor_row, cursor_col)) if cursor_row == row_num && cursor_col == col_num => {
                // It is on the current column, so we don't need
                // to explicitly move it.  Move the cursor by the
                // width of the text we're about to add.
                Some((row_num, col_num + other_cell.width()))
            }
            _ => {
                // Need to explicitly move the cursor
                self.changes.push(Change::CursorPosition {
                    y: Position::Absolute(row_num),
                    x: Position::Absolute(col_num),
                });
                // and update the position for next time
                Some((row_num, col_num + other_cell.width()))
            }
        };

        // we could get fancy and try to minimize the update traffic
        // by computing a series of AttributeChange values here.
        // For now, let's just record the new value
        self.attr = match self.attr.take() {
            Some(ref attr) if attr == other_cell.attrs() => {
                // Active attributes match, so we don't need
                // to emit a change for them
                Some(attr.clone())
            }
            _ => {
                // Attributes are different
                self.changes
                    .push(Change::AllAttributes(other_cell.attrs().clone()));
                Some(other_cell.attrs().clone())
            }
        };
        // A little bit of bloat in the code to avoid runs of single
        // character Text entries; just append to the string.
        let result_len = self.changes.len();
        if result_len > 0 && self.changes[result_len - 1].is_text() {
            if let Some(Change::Text(ref mut prefix)) = self.changes.get_mut(result_len - 1) {
                prefix.push_str(other_cell.str());
            }
        } else {
            self.changes
                .push(Change::Text(other_cell.str().to_string()));
        }
    }
}
