impl Line {
    pub fn insert_cell(&mut self, x: usize, cell: Cell, right_margin: usize, seqno: SequenceNo) {
        self.invalidate_implicit_hyperlinks(seqno);

        let cells = self.coerce_vec_storage();
        if right_margin <= cells.len() {
            cells.remove(right_margin - 1);
        }

        if x >= cells.len() {
            cells.resize_with(x, Cell::blank);
        }

        // If we're inserting a wide cell, we should also insert the overlapped cells.
        // We insert them first so that the grapheme winds up left-most.
        let width = cell.width();
        for _ in 1..=width.saturating_sub(1) {
            cells.insert(x, Cell::blank_with_attrs(cell.attrs().clone()));
        }

        cells.insert(x, cell);
        self.update_last_change_seqno(seqno);
        self.invalidate_zones();
    }

    pub fn erase_cell(&mut self, x: usize, seqno: SequenceNo) {
        if x >= self.len() {
            // Already implicitly erased
            return;
        }
        self.invalidate_implicit_hyperlinks(seqno);
        self.invalidate_grapheme_at_or_before(x);
        {
            let cells = self.coerce_vec_storage();
            cells.remove(x);
            cells.push(Cell::default());
        }
        self.update_last_change_seqno(seqno);
        self.invalidate_zones();
    }

    pub fn remove_cell(&mut self, x: usize, seqno: SequenceNo) {
        if x >= self.len() {
            // Already implicitly removed
            return;
        }
        self.invalidate_implicit_hyperlinks(seqno);
        self.invalidate_grapheme_at_or_before(x);
        self.coerce_vec_storage().remove(x);
        self.update_last_change_seqno(seqno);
        self.invalidate_zones();
    }

    pub fn erase_cell_with_margin(
        &mut self,
        x: usize,
        right_margin: usize,
        seqno: SequenceNo,
        blank_attr: CellAttributes,
    ) {
        self.invalidate_implicit_hyperlinks(seqno);
        if x < self.len() {
            self.invalidate_grapheme_at_or_before(x);
            self.coerce_vec_storage().remove(x);
        }
        if right_margin <= self.len() + 1
        /* we just removed one */
        {
            self.coerce_vec_storage()
                .insert(right_margin - 1, Cell::blank_with_attrs(blank_attr));
        }
        self.update_last_change_seqno(seqno);
        self.invalidate_zones();
    }

    pub fn prune_trailing_blanks(&mut self, seqno: SequenceNo) {
        if let CellStorage::C(cl) = &mut self.cells {
            if cl.prune_trailing_blanks() {
                self.update_last_change_seqno(seqno);
                self.invalidate_zones();
            }
            return;
        }

        let def_attr = CellAttributes::blank();
        let cells = self.coerce_vec_storage();
        if let Some(end_idx) = cells
            .iter()
            .rposition(|c| c.str() != " " || c.attrs() != &def_attr)
        {
            cells.resize_with(end_idx + 1, Cell::blank);
            self.update_last_change_seqno(seqno);
            self.invalidate_zones();
        }
    }

    pub fn fill_range(&mut self, cols: Range<usize>, cell: &Cell, seqno: SequenceNo) {
        if self.len() == 0 && *cell == Cell::blank() {
            // We would be filling it with blanks only to prune
            // them all away again before we return; NOP
            return;
        }
        for x in cols {
            // FIXME: we can skip the look-back for second and subsequent iterations
            self.set_cell_impl(x, cell.clone(), true, seqno);
        }
        self.prune_trailing_blanks(seqno);
    }

    pub fn len(&self) -> usize {
        match &self.cells {
            CellStorage::V(cells) => cells.len(),
            CellStorage::C(cl) => cl.len(),
        }
    }

    /// Iterates the visible cells, respecting the width of the cell.
    /// For instance, a double-width cell overlaps the following (blank)
    /// cell, so that blank cell is omitted from the iterator results.
    /// The iterator yields (column_index, Cell).  Column index is the
    /// index into Self::cells, and due to the possibility of skipping
    /// the characters that follow wide characters, the column index may
    /// skip some positions.  It is returned as a convenience to the consumer
    /// as using .enumerate() on this iterator wouldn't be as useful.
    pub fn visible_cells<'a>(&'a self) -> impl Iterator<Item = CellRef<'a>> {
        match &self.cells {
            CellStorage::V(cells) => VisibleCellIter::V(VecStorageIter {
                cells: cells.iter(),
                idx: 0,
                skip_width: 0,
            }),
            CellStorage::C(cl) => VisibleCellIter::C(cl.iter()),
        }
    }

    pub fn get_cell(&self, cell_index: usize) -> Option<CellRef<'_>> {
        self.visible_cells()
            .find(|cell| cell.cell_index() == cell_index)
    }

    pub fn cluster(&self, bidi_hint: Option<ParagraphDirectionHint>) -> Vec<CellCluster> {
        CellCluster::make_cluster(self.len(), self.visible_cells(), bidi_hint)
    }

    fn make_cells(&mut self) {
        let cells = match &self.cells {
            CellStorage::V(_) => return,
            CellStorage::C(cl) => cl.to_cell_vec(),
        };
        // log::info!("make_cells\n{:?}", backtrace::Backtrace::new());
        self.cells = CellStorage::V(VecStorage::new(cells));
    }

    pub(crate) fn coerce_vec_storage(&mut self) -> &mut VecStorage {
        self.make_cells();

        match &mut self.cells {
            CellStorage::V(c) => return c,
            CellStorage::C(_) => unreachable!(),
        }
    }

    /// Adjusts the internal storage so that it occupies less
    /// space. Subsequent mutations will incur some overhead to
    /// re-materialize the storage in a form that is suitable
    /// for mutation.
    pub fn compress_for_scrollback(&mut self) {
        let cv = match &self.cells {
            CellStorage::V(v) => ClusteredLine::from_cell_vec(v.len(), self.visible_cells()),
            CellStorage::C(_) => return,
        };
        self.cells = CellStorage::C(cv);
    }

    pub fn cells_mut(&mut self) -> &mut [Cell] {
        self.coerce_vec_storage().as_mut_slice()
    }

    /// Return true if the line consists solely of whitespace cells
    pub fn is_whitespace(&self) -> bool {
        self.visible_cells().all(|c| c.str() == " ")
    }

    /// Return true if the last cell in the line has the wrapped attribute,
    /// indicating that the following line is logically a part of this one.
    pub fn last_cell_was_wrapped(&self) -> bool {
        self.visible_cells()
            .last()
            .map(|c| c.attrs().wrapped())
            .unwrap_or(false)
    }

    /// Adjust the value of the wrapped attribute on the last cell of this
    /// line.
    pub fn set_last_cell_was_wrapped(&mut self, wrapped: bool, seqno: SequenceNo) {
        self.update_last_change_seqno(seqno);
        if let CellStorage::C(cl) = &mut self.cells {
            if cl.len() == 0 {
                // Need to mark that implicit space as wrapped, so
                // explicitly add it
                cl.append(Cell::blank());
            }
            cl.set_last_cell_was_wrapped(wrapped);
            return;
        }

        let cells = self.coerce_vec_storage();
        if let Some(cell) = cells.last_mut() {
            cell.attrs_mut().set_wrapped(wrapped);
        }
    }

    /// Concatenate the cells from other with this line, appending them
    /// to this line.
    /// This function is used by rewrapping logic when joining wrapped
    /// lines back together.
    pub fn append_line(&mut self, other: Line, seqno: SequenceNo) {
        match &mut self.cells {
            CellStorage::V(cells) => {
                for cell in other.visible_cells() {
                    cells.push(cell.as_cell());
                    for _ in 1..cell.width() {
                        cells.push(Cell::new(' ', cell.attrs().clone()));
                    }
                }
            }
            CellStorage::C(cl) => {
                for cell in other.visible_cells() {
                    cl.append(cell.as_cell());
                }
            }
        }
        self.update_last_change_seqno(seqno);
        self.invalidate_zones();
    }

    /// mutable access the cell data, but the caller must take care
    /// to only mutate attributes rather than the cell textual content.
    /// Use set_cell if you need to modify the textual content of the
    /// cell, so that important invariants are upheld.
    pub fn cells_mut_for_attr_changes_only(&mut self) -> &mut [Cell] {
        self.coerce_vec_storage().as_mut_slice()
    }

    /// Given a starting attribute value, produce a series of Change
    /// entries to recreate the current line
    pub fn changes(&self, start_attr: &CellAttributes) -> Vec<Change> {
        let mut result = Vec::new();
        let mut attr = start_attr.clone();
        let mut text_run = String::new();

        for cell in self.visible_cells() {
            if *cell.attrs() == attr {
                text_run.push_str(cell.str());
            } else {
                // flush out the current text run
                if !text_run.is_empty() {
                    result.push(Change::Text(text_run.clone()));
                    text_run.clear();
                }

                attr = cell.attrs().clone();
                result.push(Change::AllAttributes(attr.clone()));
                text_run.push_str(cell.str());
            }
        }

        // flush out any remaining text run
        if !text_run.is_empty() {
            // if this is just spaces then it is likely cheaper
            // to emit ClearToEndOfLine instead.
            if attr
                == CellAttributes::default()
                    .set_background(attr.background())
                    .clone()
            {
                let left = text_run.trim_end_matches(' ').to_string();
                let num_trailing_spaces = text_run.len() - left.len();

                if num_trailing_spaces > 0 {
                    if !left.is_empty() {
                        result.push(Change::Text(left));
                    } else if result.len() == 1 {
                        // if the only queued result prior to clearing
                        // to the end of the line is an attribute change,
                        // we can prune it out and return just the line
                        // clearing operation
                        if let Change::AllAttributes(_) = result[0] {
                            result.clear()
                        }
                    }

                    // Since this function is only called in the full repaint
                    // case, and we always emit a clear screen with the default
                    // background color, we don't need to emit an instruction
                    // to clear the remainder of the line unless it has a different
                    // background color.
                    if attr.background() != Default::default() {
                        result.push(Change::ClearToEndOfLine(attr.background()));
                    }
                } else {
                    result.push(Change::Text(text_run));
                }
            } else {
                result.push(Change::Text(text_run));
            }
        }

        result
    }
}
