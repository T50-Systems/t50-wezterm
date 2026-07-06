impl Line {
    /// Returns true if the line contains a hyperlink
    #[inline]
    pub fn has_hyperlink(&self) -> bool {
        (self.bits & (LineBits::HAS_HYPERLINK | LineBits::HAS_IMPLICIT_HYPERLINKS))
            != LineBits::NONE
    }

    /// Recompose line into the corresponding utf8 string.
    pub fn as_str(&self) -> Cow<'_, str> {
        match &self.cells {
            CellStorage::V(_) => {
                let mut s = String::new();
                for cell in self.visible_cells() {
                    s.push_str(cell.str());
                }
                Cow::Owned(s)
            }
            CellStorage::C(cl) => Cow::Borrowed(&cl.text),
        }
    }

    pub fn split_off(&mut self, idx: usize, seqno: SequenceNo) -> Self {
        let my_cells = self.coerce_vec_storage();
        // Clamp to avoid out of bounds panic if the line is shorter
        // than the requested split point
        // <https://github.com/wezterm/wezterm/issues/2355>
        let idx = idx.min(my_cells.len());
        let cells = my_cells.split_off(idx);
        Self {
            bits: self.bits,
            cells: CellStorage::V(VecStorage::new(cells)),
            seqno,
            zones: vec![],
            #[cfg(feature = "appdata")]
            appdata: Mutex::new(None),
        }
    }

    pub fn compute_double_click_range<F: Fn(&str) -> bool>(
        &self,
        click_col: usize,
        is_word: F,
    ) -> DoubleClickRange {
        let len = self.len();

        if click_col >= len {
            return DoubleClickRange::Range(click_col..click_col);
        }

        let mut lower = click_col;
        let mut upper = click_col;

        // TODO: look back and look ahead for cells that are hidden by
        // a preceding multi-wide cell
        let cells = self.visible_cells().collect::<Vec<_>>();
        for cell in &cells {
            if cell.cell_index() < click_col {
                continue;
            }
            if !is_word(cell.str()) {
                break;
            }
            upper = cell.cell_index() + 1;
        }
        for cell in cells.iter().rev() {
            if cell.cell_index() > click_col {
                continue;
            }
            if !is_word(cell.str()) {
                break;
            }
            lower = cell.cell_index();
        }

        if upper > lower
            && upper >= len
            && cells
                .last()
                .map(|cell| cell.attrs().wrapped())
                .unwrap_or(false)
        {
            DoubleClickRange::RangeWithWrap(lower..upper)
        } else {
            DoubleClickRange::Range(lower..upper)
        }
    }

    /// Returns a substring from the line.
    pub fn columns_as_str(&self, range: Range<usize>) -> String {
        let mut s = String::new();
        for c in self.visible_cells() {
            if c.cell_index() < range.start {
                continue;
            }
            if c.cell_index() >= range.end {
                break;
            }
            s.push_str(c.str());
        }
        s
    }

    pub fn columns_as_line(&self, range: Range<usize>) -> Self {
        let mut cells = vec![];
        for c in self.visible_cells() {
            if c.cell_index() < range.start {
                continue;
            }
            if c.cell_index() >= range.end {
                break;
            }
            cells.push(c.as_cell());
        }
        Self {
            bits: LineBits::NONE,
            cells: CellStorage::V(VecStorage::new(cells)),
            seqno: self.current_seqno(),
            zones: vec![],
            #[cfg(feature = "appdata")]
            appdata: Mutex::new(None),
        }
    }

    /// If we're about to modify a cell obscured by a double-width
    /// character ahead of that cell, we need to nerf that sequence
    /// of cells to avoid partial rendering concerns.
    /// Similarly, when we assign a cell, we need to blank out those
    /// occluded successor cells.
    pub fn set_cell(&mut self, idx: usize, cell: Cell, seqno: SequenceNo) {
        self.set_cell_impl(idx, cell, false, seqno);
    }

    /// Assign a cell using grapheme text with a known width and attributes.
    /// This is a micro-optimization over first constructing a Cell from
    /// the grapheme info. If assigning this particular cell can be optimized
    /// to an append to the interal clustered storage then the cost of
    /// constructing and dropping the Cell can be avoided.
    pub fn set_cell_grapheme(
        &mut self,
        idx: usize,
        text: &str,
        width: usize,
        attr: CellAttributes,
        seqno: SequenceNo,
    ) {
        if attr.hyperlink().is_some() {
            self.bits |= LineBits::HAS_HYPERLINK;
        }

        if let CellStorage::C(cl) = &mut self.cells {
            if idx > cl.len() && text == " " && attr == CellAttributes::blank() {
                // Appending blank beyond end of line; is already
                // implicitly blank
                return;
            }
            while cl.len() < idx {
                // Fill out any implied blanks until we can append
                // their intended cell content
                cl.append_grapheme(" ", 1, CellAttributes::blank());
            }
            if idx == cl.len() {
                cl.append_grapheme(text, width, attr);
                self.invalidate_implicit_hyperlinks(seqno);
                self.invalidate_zones();
                self.update_last_change_seqno(seqno);
                return;
            }
        }

        self.set_cell(idx, Cell::new_grapheme_with_width(text, width, attr), seqno);
    }

    pub fn set_cell_clearing_image_placements(
        &mut self,
        idx: usize,
        cell: Cell,
        seqno: SequenceNo,
    ) {
        self.set_cell_impl(idx, cell, true, seqno)
    }

    fn raw_set_cell(&mut self, idx: usize, cell: Cell, clear: bool) {
        let cells = self.coerce_vec_storage();
        cells.set_cell(idx, cell, clear);
    }

    fn set_cell_impl(&mut self, idx: usize, cell: Cell, clear: bool, seqno: SequenceNo) {
        // The .max(1) stuff is here in case we get called with a
        // zero-width cell.  That shouldn't happen: those sequences
        // should get filtered out in the terminal parsing layer,
        // but in case one does sneak through, we need to ensure that
        // we grow the cells array to hold this bogus entry.
        // https://github.com/wezterm/wezterm/issues/768
        let width = cell.width().max(1);

        self.invalidate_implicit_hyperlinks(seqno);
        self.invalidate_zones();
        self.update_last_change_seqno(seqno);
        if cell.attrs().hyperlink().is_some() {
            self.bits |= LineBits::HAS_HYPERLINK;
        }

        if let CellStorage::C(cl) = &mut self.cells {
            if idx > cl.len() && cell == Cell::blank() {
                // Appending blank beyond end of line; is already
                // implicitly blank
                return;
            }
            while cl.len() < idx {
                // Fill out any implied blanks until we can append
                // their intended cell content
                cl.append_grapheme(" ", 1, CellAttributes::blank());
            }
            if idx == cl.len() {
                cl.append(cell);
                return;
            }
            /*
            log::info!(
                "cannot append {cell:?} to {:?} as idx={idx} and cl.len is {}",
                cl,
                cl.len
            );
            */
        }

        // if the line isn't wide enough, pad it out with the default attributes.
        {
            let cells = self.coerce_vec_storage();
            if idx + width > cells.len() {
                cells.resize_with(idx + width, Cell::blank);
            }
        }

        self.invalidate_grapheme_at_or_before(idx);

        // For double-wide or wider chars, ensure that the cells that
        // are overlapped by this one are blanked out.
        for i in 1..=width.saturating_sub(1) {
            self.raw_set_cell(idx + i, Cell::blank_with_attrs(cell.attrs().clone()), clear);
        }

        self.raw_set_cell(idx, cell, clear);
    }

    /// Place text starting at the specified column index.
    /// Each grapheme of the text run has the same attributes.
    pub fn overlay_text_with_attribute(
        &mut self,
        mut start_idx: usize,
        text: &str,
        attr: CellAttributes,
        seqno: SequenceNo,
    ) {
        for (i, c) in Graphemes::new(text).enumerate() {
            let cell = Cell::new_grapheme(c, attr.clone(), None);
            let width = cell.width();
            self.set_cell(i + start_idx, cell, seqno);

            // Compensate for required spacing/placement of
            // double width characters
            start_idx += width.saturating_sub(1);
        }
    }

    fn invalidate_grapheme_at_or_before(&mut self, idx: usize) {
        // Assumption: that the width of a grapheme is never > 2.
        // This constrains the amount of look-back that we need to do here.
        if idx > 0 {
            let prior = idx - 1;
            let cells = self.coerce_vec_storage();
            let width = cells[prior].width();
            if width > 1 {
                let attrs = cells[prior].attrs().clone();
                for nerf in prior..prior + width {
                    cells[nerf] = Cell::blank_with_attrs(attrs.clone());
                }
            }
        }
    }
}
