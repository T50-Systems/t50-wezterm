use super::*;

impl TerminalState {
    /// Informs the terminal that the viewport of the window has resized to the
    /// specified dimensions.
    /// We need to resize both the primary and alt screens, adjusting
    /// the cursor positions of both accordingly.
    pub fn resize(&mut self, size: TerminalSize) {
        self.increment_seqno();
        let (cursor_main, cursor_alt) = if self.screen.alt_screen_is_active {
            (
                self.screen
                    .screen
                    .saved_cursor
                    .as_ref()
                    .map(|s| s.position)
                    .unwrap_or_else(CursorPosition::default),
                self.cursor,
            )
        } else {
            (
                self.cursor,
                self.screen
                    .alt_screen
                    .saved_cursor
                    .as_ref()
                    .map(|s| s.position)
                    .unwrap_or_else(CursorPosition::default),
            )
        };

        let (adjusted_cursor_main, adjusted_cursor_alt) = self.screen.resize(
            size,
            cursor_main,
            cursor_alt,
            self.seqno,
            self.enable_conpty_quirks,
        );
        self.top_and_bottom_margins = 0..size.rows as i64;
        self.left_and_right_margins = 0..size.cols;
        self.pixel_height = size.pixel_height;
        self.pixel_width = size.pixel_width;
        self.dpi = size.dpi;
        self.tabs.resize(size.cols);

        if self.screen.alt_screen_is_active {
            self.set_cursor_pos(
                &Position::Absolute(adjusted_cursor_alt.x as i64),
                &Position::Absolute(adjusted_cursor_alt.y),
            );

            if let Some(saved) = self.screen.screen.saved_cursor.as_mut() {
                saved.position.x = adjusted_cursor_main.x;
                saved.position.y = adjusted_cursor_main.y;
                saved.position.seqno = self.seqno;
                saved.wrap_next = false;
            }
        } else {
            self.set_cursor_pos(
                &Position::Absolute(adjusted_cursor_main.x as i64),
                &Position::Absolute(adjusted_cursor_main.y),
            );
            if let Some(saved) = self.screen.alt_screen.saved_cursor.as_mut() {
                saved.position.x = adjusted_cursor_alt.x;
                saved.position.y = adjusted_cursor_alt.y;
                saved.position.seqno = self.seqno;
                saved.wrap_next = false;
            }
        }
    }

    pub fn get_size(&self) -> TerminalSize {
        let screen = self.screen();
        TerminalSize {
            dpi: self.dpi,
            pixel_width: self.pixel_width,
            pixel_height: self.pixel_height,
            rows: screen.physical_rows,
            cols: screen.physical_cols,
        }
    }

    pub(super) fn palette_did_change(&mut self) {
        self.make_all_lines_dirty();
        if let Some(handler) = self.alert_handler.as_mut() {
            handler.alert(Alert::PaletteChanged);
        }
    }

    /// When dealing with selection, mark a range of lines as dirty
    pub fn make_all_lines_dirty(&mut self) {
        let seqno = self.seqno;
        let screen = self.screen_mut();
        screen.for_each_phys_line_mut(|_, line| {
            line.update_last_change_seqno(seqno);
        });
    }

    /// Returns the 0-based cursor position relative to the top left of
    /// the visible screen
    pub fn cursor_pos(&self) -> CursorPosition {
        CursorPosition {
            x: self.cursor.x,
            y: self.cursor.y,
            shape: self.cursor.shape,
            visibility: if self.cursor_visible {
                CursorVisibility::Visible
            } else {
                CursorVisibility::Hidden
            },
            seqno: self.cursor.seqno,
        }
    }

    /// Returns the current cell attributes of the screen
    pub fn pen(&self) -> CellAttributes {
        self.pen.clone()
    }

    pub fn user_vars(&self) -> &HashMap<String, String> {
        &self.user_vars
    }

    pub(super) fn clear_semantic_attribute_due_to_movement(&mut self) {
        if self.clear_semantic_attribute_on_newline {
            self.clear_semantic_attribute_on_newline = false;
            self.pen.set_semantic_type(SemanticType::default());
        }
    }

    /// Sets the cursor position to precisely the x and values provided
    pub(super) fn set_cursor_position_absolute(&mut self, x: usize, y: VisibleRowIndex) {
        if self.cursor.y != y {
            self.clear_semantic_attribute_due_to_movement();
        }
        self.cursor.y = y;
        self.cursor.x = x;
        self.cursor.seqno = self.seqno;
        self.wrap_next = false;
    }

    /// Sets the cursor position. x and y are 0-based and relative to the
    /// top left of the visible screen.
    pub(super) fn set_cursor_pos(&mut self, x: &Position, y: &Position) {
        let x = match *x {
            Position::Relative(x) => (self.cursor.x as i64 + x)
                .min(
                    if self.dec_origin_mode {
                        self.left_and_right_margins.end
                    } else {
                        self.screen().physical_cols
                    } as i64
                        - 1,
                )
                .max(0),
            Position::Absolute(x) => (x + if self.dec_origin_mode {
                self.left_and_right_margins.start
            } else {
                0
            } as i64)
                .min(
                    if self.dec_origin_mode {
                        self.left_and_right_margins.end
                    } else {
                        // We allow 1 extra for the cursor x position
                        // to account for some resize/rewrap scenarios
                        // where we don't want to forget that the
                        // cursor belongs to a wrapped line
                        self.screen().physical_cols + 1
                    } as i64
                        - 1,
                )
                .max(0),
        };

        let y = match *y {
            Position::Relative(y) => (self.cursor.y + y)
                .min(
                    if self.dec_origin_mode {
                        self.top_and_bottom_margins.end
                    } else {
                        self.screen().physical_rows as i64
                    } - 1,
                )
                .max(0),
            Position::Absolute(y) => (y + if self.dec_origin_mode {
                self.top_and_bottom_margins.start
            } else {
                0
            })
            .min(
                if self.dec_origin_mode {
                    self.top_and_bottom_margins.end
                } else {
                    self.screen().physical_rows as i64
                } - 1,
            )
            .max(0),
        };

        self.set_cursor_position_absolute(x as usize, y);
    }

    pub(super) fn scroll_up(&mut self, num_rows: usize) {
        let seqno = self.seqno;
        let blank_attr = self.pen.clone_sgr_only();
        let top_and_bottom_margins = self.top_and_bottom_margins.clone();
        let left_and_right_margins = self.left_and_right_margins.clone();
        let bidi_mode = self.get_bidi_mode();
        self.screen_mut().scroll_up_within_margins(
            &top_and_bottom_margins,
            &left_and_right_margins,
            num_rows,
            seqno,
            blank_attr,
            bidi_mode,
        )
    }

    pub(super) fn scroll_down(&mut self, num_rows: usize) {
        let seqno = self.seqno;
        let blank_attr = self.pen.clone_sgr_only();
        let top_and_bottom_margins = self.top_and_bottom_margins.clone();
        let left_and_right_margins = self.left_and_right_margins.clone();
        let bidi_mode = self.get_bidi_mode();
        self.screen_mut().scroll_down_within_margins(
            &top_and_bottom_margins,
            &left_and_right_margins,
            num_rows,
            seqno,
            blank_attr,
            bidi_mode,
        )
    }

    /// Defined by FinalTermSemanticPrompt; a fresh-line is a NOP if the
    /// cursor is already at the left margin, otherwise it is the same as
    /// a new line.
    pub(super) fn fresh_line(&mut self) {
        if self.cursor.x == self.left_and_right_margins.start {
            return;
        }
        self.new_line(true);
    }

    pub(super) fn new_line(&mut self, move_to_first_column: bool) {
        let x = if move_to_first_column {
            self.left_and_right_margins.start
        } else {
            self.cursor.x
        };
        let y = self.cursor.y;
        let y = if y == self.top_and_bottom_margins.end - 1 {
            self.scroll_up(1);
            y
        } else {
            y + 1
        };
        self.set_cursor_pos(&Position::Absolute(x as i64), &Position::Absolute(y as i64));
    }

    /// Moves the cursor down one line in the same column.
    /// If the cursor is at the bottom margin, the page scrolls up.
    pub(super) fn c1_index(&mut self) {
        if self.left_and_right_margins.contains(&self.cursor.x) {
            if self.cursor.y == self.top_and_bottom_margins.end - 1 {
                self.scroll_up(1);
            } else {
                self.set_cursor_pos(&Position::Relative(0), &Position::Relative(1));
            }
        }
    }

    /// Moves the cursor to the first position on the next line.
    /// If the cursor is at the bottom margin, the page scrolls up.
    pub(super) fn c1_nel(&mut self) {
        let y_clamp = if self.top_and_bottom_margins.contains(&self.cursor.y) {
            self.top_and_bottom_margins.end - 1
        } else {
            self.screen().physical_rows as VisibleRowIndex - 1
        };

        if self.left_and_right_margins.contains(&self.cursor.x) {
            if self.cursor.y == self.top_and_bottom_margins.end - 1 {
                self.scroll_up(1);
                self.set_cursor_position_absolute(self.left_and_right_margins.start, self.cursor.y);
            } else {
                self.set_cursor_position_absolute(
                    self.left_and_right_margins.start,
                    (self.cursor.y + 1).min(y_clamp),
                );
            }
        } else {
            // When outside left/right margins, NEL moves but does not scroll
            self.set_cursor_position_absolute(
                if self.cursor.x < self.left_and_right_margins.start {
                    self.cursor.x
                } else {
                    self.left_and_right_margins.start
                },
                (self.cursor.y + 1).min(y_clamp),
            );
        }
    }

    /// Sets a horizontal tab stop at the column where the cursor is.
    pub(super) fn c1_hts(&mut self) {
        self.tabs.set_tab_stop(self.cursor.x);
    }

    /// Moves the cursor to the next tab stop. If there are no more tab stops,
    /// the cursor moves to the right margin. HT does not cause text to auto
    /// wrap.
    pub(super) fn c0_horizontal_tab(&mut self) {
        let seqno = self.seqno;
        let x = match self.tabs.find_next_tab_stop(self.cursor.x) {
            Some(x) => x,
            None => self.left_and_right_margins.end - 1,
        };
        self.cursor.x = x.min(self.left_and_right_margins.end - 1);
        self.cursor.seqno = seqno;
    }

    /// Move the cursor up 1 line.  If the position is at the top scroll margin,
    /// scroll the region down.
    pub(super) fn c1_reverse_index(&mut self) {
        if self.left_and_right_margins.contains(&self.cursor.x) {
            if self.cursor.y == self.top_and_bottom_margins.start {
                self.scroll_down(1);
            } else {
                self.set_cursor_pos(&Position::Relative(0), &Position::Relative(-1));
            }
        }
    }

    pub(super) fn set_hyperlink(&mut self, link: Option<Hyperlink>) {
        self.pen.set_hyperlink(match link {
            Some(hyperlink) => Some(Arc::new(hyperlink)),
            None => None,
        });
    }
}
