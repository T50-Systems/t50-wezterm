fn diff_line(
    diff_state: &mut DiffState,
    line: &Line,
    row_num: usize,
    other_line: &Line,
    x: usize,
    width: usize,
    other_x: usize,
) {
    let mut cells = line
        .visible_cells()
        .skip_while(|cell| cell.cell_index() < x)
        .take_while(|cell| cell.cell_index() < x + width)
        .peekable();
    let other_cells = other_line
        .visible_cells()
        .skip_while(|cell| cell.cell_index() < other_x)
        .take_while(|cell| cell.cell_index() < other_x + width);

    for other_cell in other_cells {
        let rel_x = other_cell.cell_index() - other_x;
        let mut comparison_cell = None;

        // Advance the `cells` iterator to try to find the visible cell in `line` in the equivalent
        // position to `other_cell`. If there is no visible cell in equivalent position, advance
        // one past and wait for next iteration.
        while let Some(cell) = cells.peek() {
            let cell_rel_x = cell.cell_index() - x;

            if cell_rel_x == rel_x {
                comparison_cell = Some(*cell);
                break;
            } else if cell_rel_x > rel_x {
                break;
            }

            cells.next();
        }

        // If we find a cell in the equivalent position, diff against it. If not, we know
        // there is a multi-cell grapheme in `line` that partially overlaps `other_cell`,
        // so we have to overwrite anyway.
        if let Some(comparison_cell) = comparison_cell {
            diff_state.diff_cells(x + rel_x, row_num, comparison_cell, other_cell);
        } else {
            diff_state.set_cell(x + rel_x, row_num, other_cell);
        }
    }
}

/// Applies a Position update to either the x or y position.
/// The value is clamped to be in the range: 0..limit
fn compute_position_change(current: usize, pos: &Position, limit: usize) -> usize {
    use self::Position::*;
    match pos {
        Relative(delta) => {
            if *delta >= 0 {
                min(
                    current.saturating_add(*delta as usize),
                    limit.saturating_sub(1),
                )
            } else {
                current.saturating_sub((*delta).abs() as usize)
            }
        }
        Absolute(abs) => min(*abs, limit.saturating_sub(1)),
        EndRelative(delta) => limit.saturating_sub(*delta),
    }
}
