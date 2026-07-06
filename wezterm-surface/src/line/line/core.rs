impl Line {
    pub fn with_width_and_cell(width: usize, cell: Cell, seqno: SequenceNo) -> Self {
        let mut cells = Vec::with_capacity(width);
        cells.resize(width, cell.clone());
        let bits = LineBits::NONE;
        Self {
            bits,
            cells: CellStorage::V(VecStorage::new(cells)),
            seqno,
            zones: vec![],
            #[cfg(feature = "appdata")]
            appdata: Mutex::new(None),
        }
    }

    pub fn from_cells(cells: Vec<Cell>, seqno: SequenceNo) -> Self {
        let bits = LineBits::NONE;
        Self {
            bits,
            cells: CellStorage::V(VecStorage::new(cells)),
            seqno,
            zones: vec![],
            #[cfg(feature = "appdata")]
            appdata: Mutex::new(None),
        }
    }

    /// Create a new line using cluster storage, optimized for appending
    /// and lower memory utilization.
    /// The line will automatically switch to cell storage when necessary
    /// to apply edits.
    pub fn new(seqno: SequenceNo) -> Self {
        Self {
            bits: LineBits::NONE,
            cells: CellStorage::C(ClusteredLine::new()),
            seqno,
            zones: vec![],
            #[cfg(feature = "appdata")]
            appdata: Mutex::new(None),
        }
    }

    /// Computes a hash over the line that will change if the way that
    /// the line contents are shaped would change.
    /// This is independent of the seqno and is based purely on the
    /// content of the line.
    ///
    /// Line doesn't implement Hash in terms of this function as compute_shape_hash
    /// doesn't every possible bit of internal state, and we don't want to
    /// encourage using Line directly as a hash key.
    pub fn compute_shape_hash(&self) -> [u8; 16] {
        let mut hasher = SipHasher::new();
        self.bits.bits().hash(&mut hasher);
        for cell in self.visible_cells() {
            cell.compute_shape_hash(&mut hasher);
        }
        hasher.finish128().as_bytes()
    }

    pub fn with_width(width: usize, seqno: SequenceNo) -> Self {
        let mut cells = Vec::with_capacity(width);
        cells.resize_with(width, Cell::blank);
        let bits = LineBits::NONE;
        Self {
            bits,
            cells: CellStorage::V(VecStorage::new(cells)),
            seqno,
            zones: vec![],
            #[cfg(feature = "appdata")]
            appdata: Mutex::new(None),
        }
    }

    pub fn from_text(
        s: &str,
        attrs: &CellAttributes,
        seqno: SequenceNo,
        unicode_version: Option<&UnicodeVersion>,
    ) -> Line {
        let mut cells = Vec::new();

        for sub in Graphemes::new(s) {
            let cell = Cell::new_grapheme(sub, attrs.clone(), unicode_version);
            let width = cell.width();
            cells.push(cell);
            for _ in 1..width {
                cells.push(Cell::new(' ', attrs.clone()));
            }
        }

        Line {
            cells: CellStorage::V(VecStorage::new(cells)),
            bits: LineBits::NONE,
            seqno,
            zones: vec![],
            #[cfg(feature = "appdata")]
            appdata: Mutex::new(None),
        }
    }

    pub fn from_text_with_wrapped_last_col(
        s: &str,
        attrs: &CellAttributes,
        seqno: SequenceNo,
    ) -> Line {
        let mut line = Self::from_text(s, attrs, seqno, None);
        line.cells_mut()
            .last_mut()
            .map(|cell| cell.attrs_mut().set_wrapped(true));
        line
    }

    pub fn resize_and_clear(
        &mut self,
        width: usize,
        seqno: SequenceNo,
        blank_attr: CellAttributes,
    ) {
        {
            let cells = self.coerce_vec_storage();
            for c in cells.iter_mut() {
                *c = Cell::blank_with_attrs(blank_attr.clone());
            }
            cells.resize_with(width, || Cell::blank_with_attrs(blank_attr.clone()));
            cells.shrink_to_fit();
        }
        self.update_last_change_seqno(seqno);
        self.invalidate_zones();
        self.bits = LineBits::NONE;
    }

    pub fn resize(&mut self, width: usize, seqno: SequenceNo) {
        self.coerce_vec_storage().resize_with(width, Cell::blank);
        self.update_last_change_seqno(seqno);
        self.invalidate_zones();
    }

    /// Wrap the line so that it fits within the provided width.
    /// Returns the list of resultant line(s)
    pub fn wrap(self, width: usize, seqno: SequenceNo) -> Vec<Self> {
        let mut cells: Vec<CellRef> = self.visible_cells().collect();
        if let Some(end_idx) = cells.iter().rposition(|c| c.str() != " ") {
            cells.truncate(end_idx + 1);

            let mut lines: Vec<Self> = vec![];
            let mut delta = 0;
            for cell in cells {
                let need_new_line = lines
                    .last_mut()
                    .map(|line| line.len() + cell.width() > width)
                    .unwrap_or(true);
                if need_new_line {
                    lines
                        .last_mut()
                        .map(|line| line.set_last_cell_was_wrapped(true, seqno));
                    lines.push(Line::new(seqno));
                    delta = cell.cell_index();
                }
                let line = lines.last_mut().unwrap();
                line.set_cell_grapheme(
                    cell.cell_index() - delta,
                    cell.str(),
                    cell.width(),
                    (*cell.attrs()).clone(),
                    seqno,
                );
            }

            lines
        } else {
            vec![self]
        }
    }

    /// Set arbitrary application specific data for the line.
    /// Only one piece of appdata can be tracked per line,
    /// so this is only suitable for the overall application
    /// and not for use by "middleware" crates.
    /// A Weak reference is stored.
    /// `get_appdata` is used to retrieve a previously stored reference.
    #[cfg(feature = "appdata")]
    pub fn set_appdata<T: Any + Send + Sync>(&self, appdata: Arc<T>) {
        let appdata: Arc<dyn Any + Send + Sync> = appdata;
        self.appdata
            .lock()
            .unwrap()
            .replace(Arc::downgrade(&appdata));
    }

    #[cfg(feature = "appdata")]
    pub fn clear_appdata(&self) {
        self.appdata.lock().unwrap().take();
    }

    /// Retrieve the appdata for the line, if any.
    /// This may return None in the case where the underlying data has
    /// been released: Line only stores a Weak reference to it.
    #[cfg(feature = "appdata")]
    pub fn get_appdata(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.appdata
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|data| data.upgrade())
    }

    /// Returns true if the line's last changed seqno is more recent
    /// than the provided seqno parameter
    pub fn changed_since(&self, seqno: SequenceNo) -> bool {
        self.seqno == SEQ_ZERO || self.seqno > seqno
    }

    pub fn current_seqno(&self) -> SequenceNo {
        self.seqno
    }

    /// Annotate the line with the sequence number of a change.
    /// This can be used together with Line::changed_since to
    /// manage caching and rendering
    #[inline]
    pub fn update_last_change_seqno(&mut self, seqno: SequenceNo) {
        self.seqno = self.seqno.max(seqno);
    }

    /// Check whether the line is single-width.
    #[inline]
    pub fn is_single_width(&self) -> bool {
        (self.bits
            & (LineBits::DOUBLE_WIDTH
                | LineBits::DOUBLE_HEIGHT_TOP
                | LineBits::DOUBLE_HEIGHT_BOTTOM))
            == LineBits::NONE
    }

    /// Force single-width.  This also implicitly sets
    /// double-height-(top/bottom) and dirty.
    #[inline]
    pub fn set_single_width(&mut self, seqno: SequenceNo) {
        self.bits.remove(LineBits::DOUBLE_WIDTH_HEIGHT_MASK);
        self.update_last_change_seqno(seqno);
    }

    /// Check whether the line is double-width and not double-height.
    #[inline]
    pub fn is_double_width(&self) -> bool {
        (self.bits & LineBits::DOUBLE_WIDTH_HEIGHT_MASK) == LineBits::DOUBLE_WIDTH
    }

    /// Force double-width.  This also implicitly sets
    /// double-height-(top/bottom) and dirty.
    #[inline]
    pub fn set_double_width(&mut self, seqno: SequenceNo) {
        self.bits
            .remove(LineBits::DOUBLE_HEIGHT_TOP | LineBits::DOUBLE_HEIGHT_BOTTOM);
        self.bits.insert(LineBits::DOUBLE_WIDTH);
        self.update_last_change_seqno(seqno);
    }

    /// Check whether the line is double-height-top.
    #[inline]
    pub fn is_double_height_top(&self) -> bool {
        (self.bits & LineBits::DOUBLE_WIDTH_HEIGHT_MASK)
            == LineBits::DOUBLE_WIDTH | LineBits::DOUBLE_HEIGHT_TOP
    }

    /// Force double-height top-half.  This also implicitly sets
    /// double-width and dirty.
    #[inline]
    pub fn set_double_height_top(&mut self, seqno: SequenceNo) {
        self.bits.remove(LineBits::DOUBLE_HEIGHT_BOTTOM);
        self.bits
            .insert(LineBits::DOUBLE_WIDTH | LineBits::DOUBLE_HEIGHT_TOP);
        self.update_last_change_seqno(seqno);
    }

    /// Check whether the line is double-height-bottom.
    #[inline]
    pub fn is_double_height_bottom(&self) -> bool {
        (self.bits & LineBits::DOUBLE_WIDTH_HEIGHT_MASK)
            == LineBits::DOUBLE_WIDTH | LineBits::DOUBLE_HEIGHT_BOTTOM
    }

    /// Force double-height bottom-half.  This also implicitly sets
    /// double-width and dirty.
    #[inline]
    pub fn set_double_height_bottom(&mut self, seqno: SequenceNo) {
        self.bits.remove(LineBits::DOUBLE_HEIGHT_TOP);
        self.bits
            .insert(LineBits::DOUBLE_WIDTH | LineBits::DOUBLE_HEIGHT_BOTTOM);
        self.update_last_change_seqno(seqno);
    }

    /// Set a flag the indicate whether the line should have the bidi
    /// algorithm applied during rendering
    pub fn set_bidi_enabled(&mut self, enabled: bool, seqno: SequenceNo) {
        self.bits.set(LineBits::BIDI_ENABLED, enabled);
        self.update_last_change_seqno(seqno);
    }

    /// Set the bidi direction for the line.
    /// This affects both the bidi algorithm (if enabled via set_bidi_enabled)
    /// and the layout direction of the line.
    /// `auto_detect` specifies whether the direction should be auto-detected
    /// before falling back to the specified direction.
    pub fn set_direction(&mut self, direction: Direction, auto_detect: bool, seqno: SequenceNo) {
        self.bits
            .set(LineBits::RTL, direction == Direction::LeftToRight);
        self.bits.set(LineBits::AUTO_DETECT_DIRECTION, auto_detect);
        self.update_last_change_seqno(seqno);
    }

    pub fn set_bidi_info(
        &mut self,
        enabled: bool,
        direction: ParagraphDirectionHint,
        seqno: SequenceNo,
    ) {
        self.bits.set(LineBits::BIDI_ENABLED, enabled);
        let (auto, rtl) = match direction {
            ParagraphDirectionHint::AutoRightToLeft => (true, true),
            ParagraphDirectionHint::AutoLeftToRight => (true, false),
            ParagraphDirectionHint::LeftToRight => (false, false),
            ParagraphDirectionHint::RightToLeft => (false, true),
        };
        self.bits.set(LineBits::AUTO_DETECT_DIRECTION, auto);
        self.bits.set(LineBits::RTL, rtl);
        self.update_last_change_seqno(seqno);
    }

    /// Returns a tuple of (BIDI_ENABLED, Direction), indicating whether
    /// the line should have the bidi algorithm applied and its base
    /// direction, respectively.
    pub fn bidi_info(&self) -> (bool, ParagraphDirectionHint) {
        (
            self.bits.contains(LineBits::BIDI_ENABLED),
            match (
                self.bits.contains(LineBits::AUTO_DETECT_DIRECTION),
                self.bits.contains(LineBits::RTL),
            ) {
                (true, true) => ParagraphDirectionHint::AutoRightToLeft,
                (false, true) => ParagraphDirectionHint::RightToLeft,
                (true, false) => ParagraphDirectionHint::AutoLeftToRight,
                (false, false) => ParagraphDirectionHint::LeftToRight,
            },
        )
    }

    fn invalidate_zones(&mut self) {
        self.zones.clear();
    }
}
