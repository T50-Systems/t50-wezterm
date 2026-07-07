use super::*;

impl TerminalState {
    pub(super) fn checksum_rectangle(
        &mut self,
        left: u32,
        top: u32,
        right: u32,
        bottom: u32,
    ) -> u16 {
        let y_origin = if self.dec_origin_mode {
            self.top_and_bottom_margins.start
        } else {
            0
        } as u32;
        let x_origin = if self.dec_origin_mode {
            self.left_and_right_margins.start
        } else {
            0
        };
        let screen = self.screen_mut();
        let mut checksum = 0;
        /*
        debug!(
            "checksum left={} top={} right={} bottom={}",
            left as usize + x_origin,
            top + y_origin,
            right as usize + x_origin,
            bottom + y_origin
        );
        */

        for y in top..=bottom {
            let line_idx = screen.phys_row(VisibleRowIndex::from(y_origin + y));
            let line = screen.line_mut(line_idx);
            for cell in line.visible_cells().skip(x_origin + left as usize) {
                if cell.cell_index() > x_origin + right as usize {
                    break;
                }

                let ch = cell.str().chars().next().unwrap() as u32;
                // debug!("y={} col={} ch={:x} cell={:?}", y + y_origin, col, ch, cell);

                checksum += u16::from(ch as u8);
            }
        }

        // Treat uninitialized cells as spaces.
        // The concept of uninitialized cells in wezterm is not the same as that on VT520 or that
        // on xterm, so, to prevent a lot of noise in esctest, treat them as spaces, at least when
        // asking for the checksum of a single cell (which is what esctest does).
        // See: https://github.com/wezterm/wezterm/pull/4565
        if checksum == 0 {
            32u16
        } else {
            checksum
        }
    }

    pub(super) fn perform_csi_window(&mut self, window: Window) {
        match window {
            Window::ReportTextAreaSizeCells => {
                let screen = self.screen();
                let height = Some(screen.physical_rows as i64);
                let width = Some(screen.physical_cols as i64);

                let response = Box::new(Window::ResizeWindowCells { width, height });
                write!(self.writer, "{}", CSI::Window(response)).ok();
                self.writer.flush().ok();
            }

            Window::ReportCellSizePixels => {
                let screen = self.screen();
                let height = screen.physical_rows;
                let width = screen.physical_cols;
                let response = Box::new(Window::ReportCellSizePixelsResponse {
                    width: Some((self.pixel_width / width) as i64),
                    height: Some((self.pixel_height / height) as i64),
                });
                write!(self.writer, "{}", CSI::Window(response)).ok();
                self.writer.flush().ok();
            }

            Window::ReportTextAreaSizePixels => {
                let response = Box::new(Window::ResizeWindowPixels {
                    width: Some(self.pixel_width as i64),
                    height: Some(self.pixel_height as i64),
                });
                write!(self.writer, "{}", CSI::Window(response)).ok();
                self.writer.flush().ok();
            }

            Window::ReportWindowTitle => {
                if self.config.enable_title_reporting() {
                    write!(
                        self.writer,
                        "{}",
                        OperatingSystemCommand::SetWindowTitleSun(self.title.clone())
                    )
                    .ok();
                    self.writer.flush().ok();
                }
            }

            Window::ChecksumRectangularArea {
                request_id,
                top,
                left,
                bottom,
                right,
                ..
            } => {
                if self.config.enable_checksum_rectangular_area() {
                    let checksum = self.checksum_rectangle(
                        left.as_zero_based(),
                        top.as_zero_based(),
                        right.as_zero_based(),
                        bottom.as_zero_based(),
                    );
                    write!(self.writer, "\x1bP{}!~{:04x}\x1b\\", request_id, checksum).ok();
                    self.writer.flush().ok();
                }
            }
            Window::ResizeWindowCells { .. } => {
                // We don't allow the application to change the window size; that's
                // up to the user!
            }
            Window::Iconify | Window::DeIconify => {}
            Window::PopIconAndWindowTitle
            | Window::PopWindowTitle
            | Window::PopIconTitle
            | Window::PushIconAndWindowTitle
            | Window::PushIconTitle
            | Window::PushWindowTitle => {}

            _ => {
                if self.config.log_unknown_escape_sequences() {
                    log::warn!("unhandled Window CSI {:?}", window);
                }
            }
        }
    }

    pub(super) fn erase_in_display(&mut self, erase: EraseInDisplay) {
        let seqno = self.seqno;
        let cy = self.cursor.y;
        let pen = self.pen.clone_sgr_only();
        let rows = self.screen().physical_rows as VisibleRowIndex;
        let col_range = 0..self.screen().physical_cols;
        let row_range = match erase {
            EraseInDisplay::EraseToEndOfDisplay => {
                self.perform_csi_edit(Edit::EraseInLine(EraseInLine::EraseToEndOfLine));
                cy + 1..rows
            }
            EraseInDisplay::EraseToStartOfDisplay => {
                self.perform_csi_edit(Edit::EraseInLine(EraseInLine::EraseToStartOfLine));
                0..cy
            }
            EraseInDisplay::EraseDisplay => 0..rows,
            EraseInDisplay::EraseScrollback => {
                self.screen_mut().erase_scrollback();
                return;
            }
        };

        {
            let bidi_mode = self.get_bidi_mode();
            let screen = self.screen_mut();
            for y in row_range {
                screen.clear_line(y, col_range.clone(), &pen, seqno, bidi_mode);
                let line_idx = screen.phys_row(y);
                screen.line_mut(line_idx).set_single_width(seqno);
            }
        }
    }

    pub(super) fn get_bidi_mode(&self) -> BidiMode {
        let mut mode = self.config.bidi_mode();
        if let Some(enabled) = &self.bidi_enabled {
            mode.enabled = *enabled;
        }
        if let Some(hint) = &self.bidi_hint {
            mode.hint = *hint;
        }
        mode
    }

    pub(super) fn perform_csi_edit(&mut self, edit: Edit) {
        let seqno = self.seqno;
        match edit {
            Edit::DeleteCharacter(n) => {
                let y = self.cursor.y;
                let x = self.cursor.x;

                if x >= self.left_and_right_margins.start && x < self.left_and_right_margins.end {
                    let right_margin = self.left_and_right_margins.end;
                    let limit = (x + n as usize).min(right_margin);

                    let blank_attr = self.pen.clone_sgr_only();
                    let screen = self.screen_mut();
                    for _ in x..limit as usize {
                        screen.erase_cell(x, y, right_margin, seqno, blank_attr.clone());
                    }
                }
            }
            Edit::DeleteLine(n) => {
                if self.top_and_bottom_margins.contains(&self.cursor.y)
                    && self.left_and_right_margins.contains(&self.cursor.x)
                {
                    let top_and_bottom_margins = self.cursor.y..self.top_and_bottom_margins.end;
                    let left_and_right_margins = self.left_and_right_margins.clone();
                    let blank_attr = self.pen.clone_sgr_only();
                    let bidi_mode = self.get_bidi_mode();
                    self.screen_mut().scroll_up_within_margins(
                        &top_and_bottom_margins,
                        &left_and_right_margins,
                        n as usize,
                        seqno,
                        blank_attr,
                        bidi_mode,
                    );
                }
            }
            Edit::EraseCharacter(n) => {
                let y = self.cursor.y;
                let x = self.cursor.x;
                let limit = (x + n as usize).min(self.screen().physical_cols);
                {
                    let blank = Cell::blank_with_attrs(self.pen.clone_sgr_only());
                    let screen = self.screen_mut();
                    for x in x..limit as usize {
                        screen.set_cell(x, y, &blank, seqno);
                    }
                }
            }

            Edit::EraseInLine(erase) => {
                let cx = self.cursor.x;
                let cy = self.cursor.y;
                let pen = self.pen.clone_sgr_only();
                let cols = self.screen().physical_cols;
                let bidi_mode = self.get_bidi_mode();
                let range = match erase {
                    // If wrap_next is true, then cx is effectively 1 column to the right.
                    // It feels wrong to handle this here, but in trying to centralize
                    // the logic for updating the cursor position, it causes regressions
                    // in the test suite.
                    // So this is here for now until a better solution is found.
                    // <https://github.com/wezterm/wezterm/issues/3548>
                    EraseInLine::EraseToEndOfLine => cx + if self.wrap_next { 1 } else { 0 }..cols,
                    EraseInLine::EraseToStartOfLine => 0..cx + 1,
                    EraseInLine::EraseLine => 0..cols,
                };

                self.screen_mut()
                    .clear_line(cy, range, &pen, seqno, bidi_mode);
            }
            Edit::InsertCharacter(n) => {
                // https://vt100.net/docs/vt510-rm/ICH.html
                // The ICH sequence inserts Pn blank characters with the normal character
                // attribute. The cursor remains at the beginning of the blank characters. Text
                // between the cursor and right margin moves to the right. Characters scrolled past
                // the right margin are lost. ICH has no effect outside the scrolling margins.

                let y = self.cursor.y;
                let x = self.cursor.x;
                if self.top_and_bottom_margins.contains(&y)
                    && self.left_and_right_margins.contains(&x)
                {
                    let margin = self.left_and_right_margins.end;
                    let screen = self.screen_mut();
                    for _ in 0..n as usize {
                        screen.insert_cell(x, y, margin, seqno);
                    }
                }
            }
            Edit::InsertLine(n) => {
                if self.top_and_bottom_margins.contains(&self.cursor.y)
                    && self.left_and_right_margins.contains(&self.cursor.x)
                {
                    let bidi_mode = self.get_bidi_mode();
                    let top_and_bottom_margins = self.cursor.y..self.top_and_bottom_margins.end;
                    let left_and_right_margins = self.left_and_right_margins.clone();
                    let blank_attr = self.pen.clone_sgr_only();
                    self.screen_mut().scroll_down_within_margins(
                        &top_and_bottom_margins,
                        &left_and_right_margins,
                        n as usize,
                        seqno,
                        blank_attr,
                        bidi_mode,
                    );
                }
            }
            Edit::ScrollDown(n) => self.scroll_down(n as usize),
            Edit::ScrollUp(n) => self.scroll_up(n as usize),
            Edit::EraseInDisplay(erase) => self.erase_in_display(erase),
            Edit::Repeat(n) => {
                let mut y = self.cursor.y;
                let mut x = self.cursor.x;
                let left_and_right_margins = self.left_and_right_margins.clone();
                let top_and_bottom_margins = self.top_and_bottom_margins.clone();

                // Resolve the source cell.  It may be a double-wide character.
                let cell = {
                    let screen = self.screen_mut();
                    let to_copy = x.saturating_sub(1);
                    let line_idx = screen.phys_row(y);
                    let line = screen.line_mut(line_idx);

                    match line.cells_mut().get(to_copy).cloned() {
                        None => Cell::blank(),
                        Some(candidate) => {
                            if candidate.str() == " " && to_copy > 0 {
                                // It's a blank.  It may be the second part of
                                // a double-wide pair; look ahead of it.
                                let prior = &line.cells_mut()[to_copy - 1];
                                if prior.width() > 1 {
                                    prior.clone()
                                } else {
                                    candidate
                                }
                            } else {
                                candidate
                            }
                        }
                    }
                };

                for _ in 0..n {
                    {
                        let screen = self.screen_mut();
                        let line_idx = screen.phys_row(y);
                        let line = screen.line_mut(line_idx);

                        line.set_cell(x, cell.clone(), seqno);
                    }
                    x += 1;
                    if x > left_and_right_margins.end - 1 {
                        x = left_and_right_margins.start;
                        if y == top_and_bottom_margins.end - 1 {
                            self.scroll_up(1);
                        } else {
                            y += 1;
                            if y > top_and_bottom_margins.end - 1 {
                                y = top_and_bottom_margins.end;
                            }
                        }
                    }
                }
                self.cursor.x = x;
                self.cursor.y = y;
            }
        }
    }

    /// https://vt100.net/docs/vt510-rm/DECSLRM.html
    pub(super) fn set_left_and_right_margins(&mut self, left: OneBased, right: OneBased) {
        // The terminal only recognizes this control function if vertical split
        // screen mode (DECLRMM) is set.
        if self.left_and_right_margin_mode {
            let cols = self.screen().physical_cols as u32;
            let left = left.as_zero_based().min(cols - 1).max(0) as usize;
            let right = right.as_zero_based().min(cols - 1).max(0) as usize;

            // The value of the left margin (Pl) must be less than the right margin (Pr).
            if left >= right {
                return;
            }

            // The minimum size of the scrolling region is two columns per DEC,
            // but xterm allows 1.
            /*
            if right - left < 2 {
                return;
            }
            */

            self.left_and_right_margins = left..right + 1;

            // DECSLRM moves the cursor to column 1, line 1 of the page.
            self.set_cursor_pos(&Position::Absolute(0), &Position::Absolute(0));
        }
    }
}
