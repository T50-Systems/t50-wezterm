struct FakePane {
    lines: Mutex<Vec<Line>>,
}

impl Pane for FakePane {
    fn pane_id(&self) -> PaneId {
        unimplemented!()
    }
    fn get_cursor_position(&self) -> StableCursorPosition {
        unimplemented!()
    }

    fn get_current_seqno(&self) -> SequenceNo {
        unimplemented!()
    }

    fn get_changed_since(
        &self,
        _: Range<StableRowIndex>,
        _: SequenceNo,
    ) -> RangeSet<StableRowIndex> {
        unimplemented!()
    }

    fn with_lines_mut(
        &self,
        stable_range: Range<StableRowIndex>,
        with_lines: &mut dyn WithPaneLines,
    ) {
        let mut line_refs = vec![];
        let mut lines = self.lines.lock();
        for line in lines
            .iter_mut()
            .skip(stable_range.start as usize)
            .take((stable_range.end - stable_range.start) as usize)
        {
            line_refs.push(line);
        }
        with_lines.with_lines_mut(stable_range.start, &mut line_refs);
    }

    fn for_each_logical_line_in_stable_range_mut(
        &self,
        lines: Range<StableRowIndex>,
        for_line: &mut dyn ForEachPaneLogicalLine,
    ) {
        crate::pane::impl_for_each_logical_line_via_get_logical_lines(self, lines, for_line)
    }

    fn get_logical_lines(&self, lines: Range<StableRowIndex>) -> Vec<LogicalLine> {
        crate::pane::impl_get_logical_lines_via_get_lines(self, lines)
    }

    fn get_lines(&self, lines: Range<StableRowIndex>) -> (StableRowIndex, Vec<Line>) {
        let first = lines.start;
        (
            first,
            self.lines
                .lock()
                .iter()
                .skip(lines.start as usize)
                .take((lines.end - lines.start) as usize)
                .cloned()
                .collect(),
        )
    }
    fn get_dimensions(&self) -> RenderableDimensions {
        unimplemented!()
    }

    fn get_title(&self) -> String {
        unimplemented!()
    }
    fn send_paste(&self, _: &str) -> anyhow::Result<()> {
        unimplemented!()
    }
    fn reader(&self) -> anyhow::Result<Option<Box<dyn std::io::Read + Send>>> {
        Ok(None)
    }
    fn writer(&self) -> MappedMutexGuard<'_, dyn std::io::Write> {
        unimplemented!()
    }
    fn resize(&self, _: TerminalSize) -> anyhow::Result<()> {
        unimplemented!()
    }

    fn mouse_event(&self, _: MouseEvent) -> anyhow::Result<()> {
        unimplemented!()
    }
    fn is_dead(&self) -> bool {
        unimplemented!()
    }
    fn palette(&self) -> ColorPalette {
        unimplemented!()
    }
    fn domain_id(&self) -> DomainId {
        unimplemented!()
    }

    fn is_mouse_grabbed(&self) -> bool {
        false
    }
    fn is_alt_screen_active(&self) -> bool {
        false
    }
    fn get_current_working_dir(&self, _policy: CachePolicy) -> Option<Url> {
        None
    }
    fn key_down(&self, _: KeyCode, _: KeyModifiers) -> anyhow::Result<()> {
        unimplemented!()
    }
    fn key_up(&self, _: KeyCode, _: KeyModifiers) -> anyhow::Result<()> {
        unimplemented!()
    }
}

fn physical_lines_from_text(text: &str, width: usize) -> Vec<Line> {
    let mut physical_lines = vec![];
    for logical in text.split('\n') {
        let chunks = logical
            .chars()
            .collect::<Vec<char>>()
            .chunks(width)
            .map(|c| c.into_iter().collect::<String>())
            .collect::<Vec<String>>();
        let n_chunks = chunks.len();
        for (idx, chunk) in chunks.into_iter().enumerate() {
            let mut line = Line::from_text(&chunk, &Default::default(), 1, None);
            if idx < n_chunks - 1 {
                line.set_last_cell_was_wrapped(true, 1);
            }
            physical_lines.push(line);
        }
    }
    physical_lines
}

fn summarize_logical_lines(lines: &[LogicalLine]) -> Vec<(StableRowIndex, Cow<'_, str>)> {
    lines
        .iter()
        .map(|l| (l.first_row, l.logical.as_str()))
        .collect::<Vec<_>>()
}
