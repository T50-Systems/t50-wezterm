pub fn impl_with_lines_via_get_lines<P: Pane + ?Sized>(
    pane: &P,
    lines: Range<StableRowIndex>,
    with_lines: &mut dyn WithPaneLines,
) {
    let (first, mut lines) = pane.get_lines(lines);
    let mut line_refs = vec![];
    for line in lines.iter_mut() {
        line_refs.push(line);
    }
    with_lines.with_lines_mut(first, &mut line_refs);
}

/// A helper that allows you to implement Pane::for_each_logical_line_in_stable_range_mut
/// in terms of your existing Pane::get_logical_lines method.
///
/// The mutability is really a lie: while `with_lines` is passed something
/// that is mutable, it is operating on a copy the lines that won't persist
/// beyond the call to Pane::with_lines_mut.
pub fn impl_for_each_logical_line_via_get_logical_lines<P: Pane + ?Sized>(
    pane: &P,
    lines: Range<StableRowIndex>,
    for_line: &mut dyn ForEachPaneLogicalLine,
) {
    let mut logical = pane.get_logical_lines(lines);

    for line in &mut logical {
        let num_lines = line.physical_lines.len() as StableRowIndex;
        let mut line_refs = vec![];
        for phys in line.physical_lines.iter_mut() {
            line_refs.push(phys);
        }
        let should_continue = for_line
            .with_logical_line_mut(line.first_row..line.first_row + num_lines, &mut line_refs);
        if !should_continue {
            break;
        }
    }
}

/// A helper that allows you to implement Pane::get_logical_lines in terms of
/// your Pane::get_lines method.
pub fn impl_get_logical_lines_via_get_lines<P: Pane + ?Sized>(
    pane: &P,
    lines: Range<StableRowIndex>,
) -> Vec<LogicalLine> {
    let (mut first, mut phys) = pane.get_lines(lines);

    // Avoid pathological cases where we have eg: a really long logical line
    // (such as 1.5MB of json) that we previously wrapped.  We don't want to
    // un-wrap, scan, and re-wrap that thing.
    // This is an imperfect length constraint to partially manage the cost.
    const MAX_LOGICAL_LINE_LEN: usize = 1024;
    let mut back_len = 0;

    // Look backwards to find the start of the first logical line
    while first > 0 {
        let (prior, back) = pane.get_lines(first - 1..first);
        if prior == first {
            break;
        }
        if !back[0].last_cell_was_wrapped() {
            break;
        }
        if back[0].len() + back_len > MAX_LOGICAL_LINE_LEN {
            break;
        }
        back_len += back[0].len();
        first = prior;
        for (idx, line) in back.into_iter().enumerate() {
            phys.insert(idx, line);
        }
    }

    // Look forwards to find the end of the last logical line
    while let Some(last) = phys.last() {
        if !last.last_cell_was_wrapped() {
            break;
        }
        if last.len() > MAX_LOGICAL_LINE_LEN {
            break;
        }

        let next_row = first + phys.len() as StableRowIndex;
        let (last_row, mut ahead) = pane.get_lines(next_row..next_row + 1);
        if last_row != next_row {
            break;
        }
        phys.append(&mut ahead);
    }

    // Now process this stuff into logical lines
    let mut lines = vec![];
    for (idx, line) in phys.into_iter().enumerate() {
        match lines.last_mut() {
            None => {
                let logical = line.clone();
                lines.push(LogicalLine {
                    physical_lines: vec![line],
                    logical,
                    first_row: first + idx as StableRowIndex,
                });
            }
            Some(prior) => {
                if prior.logical.last_cell_was_wrapped()
                    && prior.logical.len() <= MAX_LOGICAL_LINE_LEN
                {
                    let seqno = prior.logical.current_seqno().max(line.current_seqno());
                    prior.logical.set_last_cell_was_wrapped(false, seqno);
                    prior.logical.append_line(line.clone(), seqno);
                    prior.physical_lines.push(line);
                } else {
                    let logical = line.clone();
                    lines.push(LogicalLine {
                        physical_lines: vec![line],
                        logical,
                        first_row: first + idx as StableRowIndex,
                    });
                }
            }
        }
    }
    lines
}

/// A helper that allows you to implement Pane::get_lines in terms
/// of your Pane::with_lines_mut method.
pub fn impl_get_lines_via_with_lines<P: Pane + ?Sized>(
    pane: &P,
    lines: Range<StableRowIndex>,
) -> (StableRowIndex, Vec<Line>) {
    struct LineCollector {
        first: StableRowIndex,
        lines: Vec<Line>,
    }

    let mut collector = LineCollector {
        first: 0,
        lines: vec![],
    };

    impl WithPaneLines for LineCollector {
        fn with_lines_mut(&mut self, first_row: StableRowIndex, lines: &mut [&mut Line]) {
            self.first = first_row;
            for line in lines.iter_mut() {
                self.lines.push(line.clone());
            }
        }
    }

    pane.with_lines_mut(lines, &mut collector);
    (collector.first, collector.lines)
}
