impl<'a> Performer<'a> {
    fn device_control(&mut self, ctrl: DeviceControlMode) {
        self.pop_tmux_title_state();
        match &ctrl {
            DeviceControlMode::ShortDeviceControl(s) => {
                match (s.byte, s.intermediates.as_slice()) {
                    (b'q', &[b'$']) => {
                        // DECRQSS - Request Status String
                        // https://vt100.net/docs/vt510-rm/DECRQSS.html
                        // The response is described here:
                        // https://vt100.net/docs/vt510-rm/DECRPSS.html
                        // but note that *that* text has the validity value
                        // inverted; there's a note about this in the xterm
                        // ctlseqs docs.
                        match s.data.as_slice() {
                            &[b'"', b'p'] => {
                                // DECSCL - select conformance level
                                write!(self.writer, "{}1$r65;1\"p{}", DCS, ST).ok();
                                self.writer.flush().ok();
                            }
                            &[b'r'] => {
                                // DECSTBM - top and bottom margins
                                let margins = self.top_and_bottom_margins.clone();
                                write!(
                                    self.writer,
                                    "{}1$r{};{}r{}",
                                    DCS,
                                    margins.start + 1,
                                    margins.end,
                                    ST
                                )
                                .ok();
                                self.writer.flush().ok();
                            }
                            &[b's'] => {
                                // DECSLRM - left and right margins
                                let margins = self.left_and_right_margins.clone();
                                write!(
                                    self.writer,
                                    "{}1$r{};{}s{}",
                                    DCS,
                                    margins.start + 1,
                                    margins.end,
                                    ST
                                )
                                .ok();
                                self.writer.flush().ok();
                            }
                            _ => {
                                if self.config.log_unknown_escape_sequences() {
                                    log::warn!("unhandled DECRQSS {:?}", s);
                                }
                                // Reply that the request is invalid
                                write!(self.writer, "{}0$r{}", DCS, ST).ok();
                                self.writer.flush().ok();
                            }
                        }
                    }
                    _ => {
                        if self.config.log_unknown_escape_sequences() {
                            log::warn!("unhandled {:?}", s);
                        }
                    }
                }
            }
            _ => match self.device_control_handler.as_mut() {
                Some(handler) => handler.handle_device_control(ctrl),
                None => {
                    if self.config.log_unknown_escape_sequences() {
                        log::warn!("unhandled {:?}", ctrl);
                    }
                }
            },
        }
    }

    /// Draw a character to the screen
    fn print(&mut self, c: char) {
        // We buffer up the chars to increase the chances of correctly grouping graphemes into cells
        if let Some(title) = self.accumulating_title.as_mut() {
            title.push(c);
        } else {
            self.print.push(c);
        }
    }

    fn control(&mut self, control: ControlCode) {
        let seqno = self.seqno;
        self.pop_tmux_title_state();
        self.flush_print();
        match control {
            ControlCode::LineFeed | ControlCode::VerticalTab | ControlCode::FormFeed => {
                if self.left_and_right_margins.contains(&self.cursor.x) {
                    self.new_line(false);
                } else {
                    // Do move down, but don't trigger a scroll when we're
                    // outside of the left/right margins
                    let old_y = self.cursor.y;
                    let y = if old_y == self.top_and_bottom_margins.end - 1 {
                        old_y
                    } else {
                        (old_y + 1).min(self.screen().physical_rows as i64 - 1)
                    };
                    self.screen_mut().dirty_line(old_y, seqno);
                    self.screen_mut().dirty_line(y, seqno);
                    self.cursor.y = y;
                    self.wrap_next = false;
                }
                if self.newline_mode {
                    self.cursor.x = 0;
                    self.clear_semantic_attribute_due_to_movement();
                }
            }
            ControlCode::CarriageReturn => {
                if self.cursor.x >= self.left_and_right_margins.start {
                    self.cursor.x = self.left_and_right_margins.start;
                } else {
                    self.cursor.x = 0;
                }
                let y = self.cursor.y;
                self.wrap_next = false;
                self.clear_semantic_attribute_due_to_movement();
                self.screen_mut().dirty_line(y, seqno);
            }

            ControlCode::Backspace => {
                if self.reverse_wraparound_mode
                    && self.dec_auto_wrap
                    && self.cursor.x == self.left_and_right_margins.start
                    && self.cursor.y == self.top_and_bottom_margins.start
                {
                    // Backspace off the top-left wraps around to the bottom right
                    let x_pos = Position::Absolute(self.left_and_right_margins.end as i64 - 1);
                    let y_pos = Position::Absolute(self.top_and_bottom_margins.end - 1);
                    self.set_cursor_pos(&x_pos, &y_pos);
                } else if self.reverse_wraparound_mode
                    && self.dec_auto_wrap
                    && self.cursor.x <= self.left_and_right_margins.start
                {
                    // Backspace off the left wraps around to the prior line on the right
                    let x_pos = Position::Absolute(self.left_and_right_margins.end as i64 - 1);
                    let y_pos = Position::Relative(-1);
                    self.set_cursor_pos(&x_pos, &y_pos);
                } else if self.reverse_wraparound_mode
                    && self.dec_auto_wrap
                    && self.cursor.x == self.left_and_right_margins.end - 1
                    && self.wrap_next
                {
                    // If the cursor is in the last column and a character was
                    // just output and reverse-wraparound is on then backspace
                    // by 1 cancels the pending wrap.
                    self.wrap_next = false;
                } else if self.cursor.x == self.left_and_right_margins.start {
                    // Respect the left margin and don't BS outside it
                } else {
                    self.set_cursor_pos(&Position::Relative(-1), &Position::Relative(0));
                }
            }
            ControlCode::HorizontalTab => self.c0_horizontal_tab(),
            ControlCode::HTS => self.c1_hts(),
            ControlCode::IND => self.c1_index(),
            ControlCode::NEL => self.c1_nel(),
            ControlCode::Bell => {
                if let Some(handler) = self.alert_handler.as_mut() {
                    handler.alert(Alert::Bell);
                } else {
                    log::info!("Ding! (this is the bell)");
                }
            }
            ControlCode::RI => self.c1_reverse_index(),

            // wezterm only supports UTF-8, so does not support the
            // DEC National Replacement Character Sets.  However, it does
            // support the DEC Special Graphics character set used by
            // numerous ncurses applications.  DEC Special Graphics can be
            // selected by ASCII Shift Out (0x0E, ^N) or by setting G0
            // via ESC ( 0 .
            ControlCode::ShiftIn => {
                self.shift_out = false;
            }
            ControlCode::ShiftOut => {
                self.shift_out = true;
            }

            ControlCode::Enquiry => {
                let response = self.config.enq_answerback();
                if response.len() > 0 {
                    write!(self.writer, "{}", response).ok();
                    self.writer.flush().ok();
                }
            }

            ControlCode::Null => {}

            _ => {
                if self.config.log_unknown_escape_sequences() {
                    log::warn!("unhandled ControlCode {:?}", control);
                }
            }
        }
    }

    fn csi_dispatch(&mut self, csi: CSI) {
        self.pop_tmux_title_state();
        self.flush_print();
        match csi {
            CSI::Sgr(sgr) => self.state.perform_csi_sgr(sgr),
            CSI::Cursor(wezterm_escape_parser::csi::Cursor::Left(n)) => {
                // We treat CUB (Cursor::Left) the same as Backspace as
                // that is what xterm does.
                // <https://github.com/wezterm/wezterm/issues/1273>
                for _ in 0..n {
                    self.control(ControlCode::Backspace);
                }
            }
            CSI::Cursor(cursor) => self.state.perform_csi_cursor(cursor),
            CSI::Edit(edit) => self.state.perform_csi_edit(edit),
            CSI::Mode(mode) => self.state.perform_csi_mode(mode),
            CSI::Device(dev) => self.state.perform_device(*dev),
            CSI::Mouse(mouse) => error!("mouse report sent by app? {:?}", mouse),
            CSI::Window(window) => self.state.perform_csi_window(*window),
            CSI::SelectCharacterPath(CharacterPath::ImplementationDefault, _) => {
                self.state.bidi_hint.take();
            }
            CSI::SelectCharacterPath(CharacterPath::LeftToRightOrTopToBottom, _) => {
                self.state
                    .bidi_hint
                    .replace(ParagraphDirectionHint::LeftToRight);
            }
            CSI::SelectCharacterPath(CharacterPath::RightToLeftOrBottomToTop, _) => {
                self.state
                    .bidi_hint
                    .replace(ParagraphDirectionHint::RightToLeft);
            }
            CSI::Keyboard(Keyboard::SetKittyState { flags, mode }) => {
                if self.config.enable_kitty_keyboard() {
                    let current_flags = match self.screen().keyboard_stack.last() {
                        Some(KeyboardEncoding::Kitty(flags)) => *flags,
                        _ => KittyKeyboardFlags::NONE,
                    };
                    let flags = match mode {
                        KittyKeyboardMode::AssignAll => flags,
                        KittyKeyboardMode::SetSpecified => current_flags | flags,
                        KittyKeyboardMode::ClearSpecified => current_flags - flags,
                    };
                    self.screen_mut().keyboard_stack.pop();
                    self.screen_mut()
                        .keyboard_stack
                        .push(KeyboardEncoding::Kitty(flags));
                }
            }
            CSI::Keyboard(Keyboard::PushKittyState { flags, mode }) => {
                if self.config.enable_kitty_keyboard() {
                    let current_flags = match self.screen().keyboard_stack.last() {
                        Some(KeyboardEncoding::Kitty(flags)) => *flags,
                        _ => KittyKeyboardFlags::NONE,
                    };
                    let flags = match mode {
                        KittyKeyboardMode::AssignAll => flags,
                        KittyKeyboardMode::SetSpecified => current_flags | flags,
                        KittyKeyboardMode::ClearSpecified => current_flags - flags,
                    };
                    let screen = self.screen_mut();
                    screen.keyboard_stack.push(KeyboardEncoding::Kitty(flags));
                    if screen.keyboard_stack.len() > 128 {
                        screen.keyboard_stack.remove(0);
                    }
                }
            }
            CSI::Keyboard(Keyboard::PopKittyState(n)) => {
                for _ in 0..n {
                    self.screen_mut().keyboard_stack.pop();
                }
            }
            CSI::Keyboard(Keyboard::QueryKittySupport) => {
                if self.config.enable_kitty_keyboard() {
                    let flags = match self.screen().keyboard_stack.last() {
                        Some(KeyboardEncoding::Kitty(flags)) => *flags,
                        _ => KittyKeyboardFlags::NONE,
                    };
                    write!(self.writer, "\x1b[?{}u", flags.bits()).ok();
                    self.writer.flush().ok();
                }
            }
            CSI::Keyboard(Keyboard::ReportKittyState(_)) => {
                // This is a response to QueryKittySupport and it is invalid for us
                // to receive it. Just ignore it.
            }
            CSI::Unspecified(unspec) => {
                if self.config.log_unknown_escape_sequences() {
                    log::warn!("unknown unspecified CSI: {:?}", format!("{}", unspec));
                }
            }
        };
    }
}
