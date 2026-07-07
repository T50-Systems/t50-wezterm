use super::*;

impl TerminalState {
    pub(super) fn perform_csi_cursor(&mut self, cursor: Cursor) {
        let seqno = self.seqno;
        match cursor {
            Cursor::SetTopAndBottomMargins { top, bottom } => {
                let rows = self.screen().physical_rows;
                let top = i64::from(top.as_zero_based()).min(rows as i64 - 1).max(0);
                let bottom = i64::from(bottom.as_zero_based())
                    .min(rows as i64 - 1)
                    .max(0);
                if top >= bottom {
                    return;
                }
                self.top_and_bottom_margins = top..bottom + 1;
                self.set_cursor_pos(&Position::Absolute(0), &Position::Absolute(0));
                log::debug!(
                    "SetTopAndBottomMargins {:?} (and move cursor to top left: {:?})",
                    self.top_and_bottom_margins,
                    self.cursor
                );
            }

            Cursor::SetLeftAndRightMargins { left, right } => {
                self.set_left_and_right_margins(left, right);
            }

            Cursor::ForwardTabulation(n) => {
                for _ in 0..n {
                    self.c0_horizontal_tab();
                }
            }
            Cursor::BackwardTabulation(n) => {
                for _ in 0..n {
                    let x = match self.tabs.find_prev_tab_stop(self.cursor.x) {
                        Some(x) => x,
                        None => 0,
                    };
                    self.set_cursor_pos(&Position::Absolute(x as i64), &Position::Relative(0));
                }
            }

            Cursor::TabulationClear(to_clear) => {
                self.tabs.clear(
                    to_clear,
                    self.cursor.x,
                    self.config.log_unknown_escape_sequences(),
                );
            }

            Cursor::TabulationControl(_) => {}
            Cursor::LineTabulation(_) => {}

            Cursor::Left(_n) => {
                // https://vt100.net/docs/vt510-rm/CUB.html
                unreachable!("Actually handled in Performer::csi_dispatch by rewriting as ControlCode::Backspace");
            }

            Cursor::Right(n) => {
                // https://vt100.net/docs/vt510-rm/CUF.html
                let cols = self.screen().physical_cols;
                let new_x = if self.cursor.x >= self.left_and_right_margins.end {
                    // outside the margin, so allow movement to screen edge
                    (self.cursor.x + n as usize).min(cols - 1)
                } else {
                    // Else constrain to margin
                    (self.cursor.x + n as usize).min(self.left_and_right_margins.end - 1)
                };

                self.cursor.x = new_x;
                self.cursor.seqno = seqno;
                self.wrap_next = false;
            }

            Cursor::Up(n) => {
                // https://vt100.net/docs/vt510-rm/CUU.html

                let candidate = self.cursor.y.saturating_sub(i64::from(n));
                let new_y = if self.cursor.y < self.top_and_bottom_margins.start {
                    // above the top margin, so allow movement to
                    // top of screen
                    candidate
                } else {
                    // Else constrain to top margin
                    if candidate < self.top_and_bottom_margins.start {
                        self.top_and_bottom_margins.start
                    } else {
                        candidate
                    }
                };

                let new_y = new_y.max(0);

                self.cursor.y = new_y;
                self.cursor.seqno = seqno;
                self.wrap_next = false;
            }
            Cursor::Down(n) => {
                // https://vt100.net/docs/vt510-rm/CUD.html
                let rows = self.screen().physical_rows;
                let new_y = if self.cursor.y >= self.top_and_bottom_margins.end {
                    // below the bottom margin, so allow movement to
                    // bottom of screen
                    (self.cursor.y + i64::from(n)).min(rows as i64 - 1)
                } else {
                    // Else constrain to bottom margin
                    (self.cursor.y + i64::from(n)).min(self.top_and_bottom_margins.end - 1)
                };

                self.cursor.y = new_y;
                self.cursor.seqno = seqno;
                self.wrap_next = false;
            }

            Cursor::CharacterAndLinePosition { line, col } | Cursor::Position { line, col } => self
                .set_cursor_pos(
                    &Position::Absolute(i64::from(col.as_zero_based())),
                    &Position::Absolute(i64::from(line.as_zero_based())),
                ),
            Cursor::CharacterAbsolute(col) => self.set_cursor_pos(
                &Position::Absolute(i64::from(col.as_zero_based())),
                &Position::Relative(0),
            ),

            Cursor::CharacterPositionAbsolute(col) => {
                let col = col.as_zero_based() as usize;
                let col = if self.dec_origin_mode {
                    col + self.left_and_right_margins.start
                } else {
                    col
                };
                self.cursor.x = col.min(self.screen().physical_cols - 1);
                self.cursor.seqno = seqno;
                self.wrap_next = false;
            }

            Cursor::CharacterPositionBackward(col) => self.set_cursor_pos(
                &Position::Relative(-(i64::from(col))),
                &Position::Relative(0),
            ),
            Cursor::CharacterPositionForward(col) => {
                self.set_cursor_pos(&Position::Relative(i64::from(col)), &Position::Relative(0))
            }
            Cursor::LinePositionAbsolute(line) => self.set_cursor_pos(
                &Position::Relative(0),
                &Position::Absolute((i64::from(line)).saturating_sub(1)),
            ),
            Cursor::LinePositionBackward(line) => self.set_cursor_pos(
                &Position::Relative(0),
                &Position::Relative(-(i64::from(line))),
            ),
            Cursor::LinePositionForward(line) => {
                self.set_cursor_pos(&Position::Relative(0), &Position::Relative(i64::from(line)))
            }
            Cursor::NextLine(n) => {
                // https://vt100.net/docs/vt510-rm/CNL.html
                let rows = self.screen().physical_rows;
                let new_y = if self.cursor.y >= self.top_and_bottom_margins.end {
                    // below the bottom margin, so allow movement to
                    // bottom of screen
                    (self.cursor.y + i64::from(n)).min(rows as i64 - 1)
                } else {
                    // Else constrain to bottom margin
                    (self.cursor.y + i64::from(n)).min(self.top_and_bottom_margins.end - 1)
                };

                self.cursor.y = new_y;
                self.cursor.x = self.left_and_right_margins.start;
                self.cursor.seqno = seqno;
                self.wrap_next = false;
            }
            Cursor::PrecedingLine(n) => {
                // https://vt100.net/docs/vt510-rm/CPL.html
                let candidate = self.cursor.y.saturating_sub(i64::from(n));
                let new_y = if self.cursor.y < self.top_and_bottom_margins.start {
                    // above the top margin, so allow movement to
                    // top of screen
                    candidate
                } else {
                    // Else constrain to top margin
                    if candidate < self.top_and_bottom_margins.start {
                        self.top_and_bottom_margins.start
                    } else {
                        candidate
                    }
                };

                let new_y = new_y.max(0);

                self.cursor.y = new_y;
                self.cursor.x = self.left_and_right_margins.start;
                self.cursor.seqno = seqno;
                self.wrap_next = false;
            }

            Cursor::ActivePositionReport { .. } => {
                // This is really a response from the terminal, and
                // we don't need to process it as a terminal command
            }
            Cursor::RequestActivePositionReport => {
                let line = OneBased::from_zero_based(
                    (self.cursor.y.saturating_sub(if self.dec_origin_mode {
                        self.top_and_bottom_margins.start
                    } else {
                        0
                    })) as u32,
                );
                let col = OneBased::from_zero_based(
                    (self
                        .cursor
                        .x
                        .min(self.screen().physical_cols - 1)
                        .saturating_sub(if self.dec_origin_mode {
                            self.left_and_right_margins.start
                        } else {
                            0
                        })) as u32,
                );
                let report = CSI::Cursor(Cursor::ActivePositionReport { line, col });
                write!(self.writer, "{}", report).ok();
                self.writer.flush().ok();
            }
            Cursor::SaveCursor => {
                // The `CSI s` SaveCursor sequence is ambiguous with DECSLRM
                // with default parameters.  To resolve the ambiguity, DECSLRM
                // is recognized if DECLRMM mode is active which we do here
                // where we have the context!
                if self.left_and_right_margin_mode {
                    // https://vt100.net/docs/vt510-rm/DECSLRM.html
                    self.set_left_and_right_margins(
                        OneBased::new(1),
                        OneBased::new(self.screen().physical_cols as u32 + 1),
                    );
                } else {
                    self.dec_save_cursor();
                }
            }
            Cursor::RestoreCursor => self.dec_restore_cursor(),
            Cursor::CursorStyle(style) => {
                self.cursor.shape = match style {
                    CursorStyle::Default => CursorShape::Default,
                    CursorStyle::BlinkingBlock => CursorShape::BlinkingBlock,
                    CursorStyle::SteadyBlock => CursorShape::SteadyBlock,
                    CursorStyle::BlinkingUnderline => CursorShape::BlinkingUnderline,
                    CursorStyle::SteadyUnderline => CursorShape::SteadyUnderline,
                    CursorStyle::BlinkingBar => CursorShape::BlinkingBar,
                    CursorStyle::SteadyBar => CursorShape::SteadyBar,
                };
                log::debug!("Cursor shape is now {:?}", self.cursor.shape);
            }
        }
    }

    /// https://vt100.net/docs/vt510-rm/DECSC.html
    pub(super) fn dec_save_cursor(&mut self) {
        let saved = SavedCursor {
            position: self.cursor,
            wrap_next: self.wrap_next,
            pen: self.pen.clone(),
            dec_origin_mode: self.dec_origin_mode,
            g0_charset: self.g0_charset,
            g1_charset: self.g1_charset,
        };
        debug!(
            "saving cursor {:?} is_alt={}",
            saved,
            self.screen.is_alt_screen_active()
        );
        *self.screen.saved_cursor() = Some(saved);
    }

    /// https://vt100.net/docs/vt510-rm/DECRC.html
    pub(super) fn dec_restore_cursor(&mut self) {
        let saved = self
            .screen
            .saved_cursor()
            .clone()
            .unwrap_or_else(|| SavedCursor {
                position: CursorPosition::default(),
                wrap_next: false,
                pen: Default::default(),
                dec_origin_mode: false,
                g0_charset: CharSet::Ascii,
                g1_charset: CharSet::Ascii,
            });
        debug!(
            "restore cursor {:?} is_alt={}",
            saved,
            self.screen.is_alt_screen_active()
        );
        let x = saved.position.x;
        let y = saved.position.y;
        // Disable origin mode so that we can set the cursor position directly
        self.dec_origin_mode = false;
        self.set_cursor_pos(&Position::Absolute(x as i64), &Position::Absolute(y));
        self.cursor.shape = saved.position.shape;
        self.wrap_next = saved.wrap_next;
        self.pen = saved.pen;
        self.dec_origin_mode = saved.dec_origin_mode;
        self.g0_charset = saved.g0_charset;
        self.g1_charset = saved.g1_charset;
        self.shift_out = false;
        self.newline_mode = false;
    }

    pub(super) fn perform_csi_sgr(&mut self, sgr: Sgr) {
        debug!("{:?}", sgr);
        match sgr {
            Sgr::Reset => {
                let link = self.pen.hyperlink().map(Arc::clone);
                let semantic_type = self.pen.semantic_type();
                self.pen = CellAttributes::default();
                self.pen.set_hyperlink(link);
                self.pen.set_semantic_type(semantic_type);
            }
            Sgr::Intensity(intensity) => {
                self.pen.set_intensity(intensity);
            }
            Sgr::Underline(underline) => {
                self.pen.set_underline(underline);
            }
            Sgr::Overline(overline) => {
                self.pen.set_overline(overline);
            }
            Sgr::VerticalAlign(align) => {
                self.pen.set_vertical_align(align);
            }
            Sgr::Blink(blink) => {
                self.pen.set_blink(blink);
            }
            Sgr::Italic(italic) => {
                self.pen.set_italic(italic);
            }
            Sgr::Inverse(inverse) => {
                self.pen.set_reverse(inverse);
            }
            Sgr::Invisible(invis) => {
                self.pen.set_invisible(invis);
            }
            Sgr::StrikeThrough(strike) => {
                self.pen.set_strikethrough(strike);
            }
            Sgr::Foreground(col) => {
                self.pen.set_foreground(col);
            }
            Sgr::Background(col) => {
                self.pen.set_background(col);
            }
            Sgr::UnderlineColor(col) => {
                self.pen.set_underline_color(col);
            }
            Sgr::Font(_) => {}
        }
    }

    /// Computes the set of `SemanticZone`s for the current terminal screen.
    /// Semantic zones are contiguous runs of cells that have the same
    /// `SemanticType` (Prompt, Input, Output).
    /// Due to the way that the terminal clears the screen, the raw, literal
    /// set of zones is overly fragmented by blanks.  This method will ignore
    /// trailing Output regions when computing the SemanticZone bounds.
    ///
    /// By default, all screen data is of type Output.  The shell needs to
    /// employ OSC 133 escapes to markup its output.
    pub fn get_semantic_zones(&mut self) -> anyhow::Result<Vec<SemanticZone>> {
        let screen = self.screen_mut();

        let mut current_zone: Option<SemanticZone> = None;
        let mut zones = vec![];

        let first_stable_row = screen.phys_to_stable_row_index(0);
        screen.for_each_phys_line_mut(|idx, line| {
            let stable_row = first_stable_row + idx as StableRowIndex;

            for zone_range in line.semantic_zone_ranges() {
                let new_zone = match current_zone.as_ref() {
                    None => true,
                    Some(zone) => zone.semantic_type != zone_range.semantic_type,
                };

                if new_zone {
                    if let Some(zone) = current_zone.take() {
                        zones.push(zone);
                    }

                    current_zone.replace(SemanticZone {
                        start_x: zone_range.range.start as usize,
                        start_y: stable_row,
                        end_x: zone_range.range.end as usize,
                        end_y: stable_row,
                        semantic_type: zone_range.semantic_type,
                    });
                }

                if let Some(zone) = current_zone.as_mut() {
                    zone.end_x = zone_range.range.end as usize;
                    zone.end_y = stable_row;
                }
            }
        });
        if let Some(zone) = current_zone.take() {
            zones.push(zone);
        }

        Ok(zones)
    }

    #[inline]
    pub fn get_reverse_video(&self) -> bool {
        self.reverse_video_mode
    }

    pub fn get_keyboard_encoding(&self) -> KeyboardEncoding {
        self.screen()
            .keyboard_stack
            .last()
            .copied()
            .unwrap_or(self.keyboard_encoding)
    }
}
