impl<'a> Performer<'a> {
    pub fn new(state: &'a mut TerminalState) -> Self {
        Self {
            state,
            print: String::new(),
        }
    }

    /// Apply character set related remapping to the input glyph if required
    fn remap_grapheme<'b>(&self, g: &'b str) -> &'b str {
        if (self.shift_out && self.g1_charset == CharSet::DecLineDrawing)
            || (!self.shift_out && self.g0_charset == CharSet::DecLineDrawing)
        {
            match g {
                "`" => "◆",
                "a" => "▒",
                "b" => "␉",
                "c" => "␌",
                "d" => "␍",
                "e" => "␊",
                "f" => "°",
                "g" => "±",
                "h" => "␤",
                "i" => "␋",
                "j" => "┘",
                "k" => "┐",
                "l" => "┌",
                "m" => "└",
                "n" => "┼",
                "o" => "⎺",
                "p" => "⎻",
                "q" => "─",
                "r" => "⎼",
                "s" => "⎽",
                "t" => "├",
                "u" => "┤",
                "v" => "┴",
                "w" => "┬",
                "x" => "│",
                "y" => "≤",
                "z" => "≥",
                "{" => "π",
                "|" => "≠",
                "}" => "£",
                "~" => "·",
                _ => g,
            }
        } else if (self.shift_out && self.g1_charset == CharSet::Uk)
            || (!self.shift_out && self.g0_charset == CharSet::Uk)
        {
            match g {
                "#" => "£",
                _ => g,
            }
        } else {
            g
        }
    }

    fn flush_print(&mut self) {
        if self.print.is_empty() {
            return;
        }

        let seqno = self.seqno;
        let mut p = std::mem::take(&mut self.print);
        let normalized: String;
        let text = if self.config.normalize_output_to_unicode_nfc()
            && is_nfc_quick(p.chars()) != IsNormalized::Yes
        {
            normalized = p.as_str().nfc().collect();
            normalized.as_str()
        } else {
            p.as_str()
        };

        for g in Graphemes::new(text) {
            let g = self.remap_grapheme(g);

            let mut print_width = grapheme_column_width(g, Some(&self.unicode_version));
            if print_width == 0 {
                // We got a zero-width grapheme.

                // Relevant reading:
                // <https://github.com/wezterm/wezterm/issues/1422>
                // <https://github.com/wezterm/wezterm/issues/6637>
                // <https://github.com/harfbuzz/harfbuzz/issues/4279>
                // <https://www.unicode.org/faq/unsup_char.html#2>
                //
                // For White_Space we want to ensure that we display as a space.
                // Other non-printing, zero-width characters can be elided
                // to avoid presentation problems, but may introduce potential
                // weirdness elsewhere. For example, U+2068 is a BIDI control
                // character and will be elided by this logic. A consequence
                // of that is that when the user copies the surrounding text
                // from the terminal, that BIDI control will not be present.
                // We do not currently have a solution for that.
                if is_white_space_grapheme(g) {
                    // Ensure that White_Space shows as a space
                    print_width = 1;
                } else {
                    log::trace!("Eliding zero-width grapheme {:?}", g);
                    continue;
                }
            }

            if self.wrap_next {
                // Since we're implicitly moving the cursor to the next
                // line, we need to tag the current position as wrapped
                // so that we can correctly reflow it if the window is
                // resized.
                {
                    let y = self.cursor.y;
                    let is_conpty = self.state.enable_conpty_quirks;
                    let screen = self.screen_mut();
                    let y = screen.phys_row(y);

                    fn makes_sense_to_wrap(s: &str) -> bool {
                        let len = s.len();
                        match (len, s.chars().next()) {
                            (1, Some(c)) => c.is_alphanumeric() || c.is_ascii_punctuation(),
                            _ => true,
                        }
                    }

                    let should_mark_wrapped = !is_conpty
                        || screen
                            .line_mut(y)
                            .visible_cells()
                            .last()
                            .map(|cell| makes_sense_to_wrap(cell.str()))
                            .unwrap_or(false);
                    if should_mark_wrapped {
                        screen.line_mut(y).set_last_cell_was_wrapped(true, seqno);
                    }
                }
                self.new_line(true);
            }

            let x = self.cursor.x;
            let y = self.cursor.y;
            let width = self.left_and_right_margins.end;

            let pen = self.pen.clone();

            let wrappable = x + print_width >= width;

            if self.insert {
                let margin = self.left_and_right_margins.end;
                let screen = self.screen_mut();
                for _ in x..x + print_width as usize {
                    screen.insert_cell(x, y, margin, seqno);
                }
            }

            // Assign the cell
            log::trace!(
                "print x={} y={} print_width={} width={} cell={} {:?}",
                x,
                y,
                print_width,
                width,
                g,
                self.pen
            );
            self.screen_mut()
                .set_cell_grapheme(x, y, g, print_width, pen, seqno);

            if !wrappable {
                self.cursor.x += print_width;
                self.wrap_next = false;
            } else {
                self.wrap_next = self.dec_auto_wrap;
            }
        }

        std::mem::swap(&mut self.print, &mut p);
        self.print.clear();
    }

    /// ConPTY, at the time of writing, does something horrible to rewrite
    /// `ESC k TITLE ST` into something completely different and out-of-order,
    /// and critically, removes the ST.
    /// The result is that our hack to accumulate the tmux title gets stuck
    /// in a mode where all printable output is accumulated for the title.
    /// To combat this, we pop_tmux_title_state when we're obviously moving
    /// to different escape sequence parsing states.
    /// <https://github.com/wezterm/wezterm/issues/2442>
    fn pop_tmux_title_state(&mut self) {
        if let Some(title) = self.accumulating_title.take() {
            log::debug!("ST never received for pending tmux title escape sequence: {title:?}");
        }
    }

    pub fn perform(&mut self, action: Action) {
        debug!("perform {:?}", action);
        if self.suppress_initial_title_change {
            match &action {
                Action::OperatingSystemCommand(osc) => match **osc {
                    OperatingSystemCommand::SetIconNameAndWindowTitle(_) => {
                        debug!("suppressed {:?}", osc);
                        self.suppress_initial_title_change = false;
                        return;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        match action {
            Action::Print(c) => self.print(c),
            Action::PrintString(s) => {
                for c in s.chars() {
                    self.print(c)
                }
            }
            Action::Control(code) => self.control(code),
            Action::DeviceControl(ctrl) => self.device_control(ctrl),
            Action::OperatingSystemCommand(osc) => self.osc_dispatch(*osc),
            Action::Esc(esc) => self.esc_dispatch(esc),
            Action::CSI(csi) => self.csi_dispatch(csi),
            Action::Sixel(sixel) => self.sixel(sixel),
            Action::XtGetTcap(names) => self.xt_get_tcap(names),
            Action::KittyImage(img) => {
                self.flush_print();
                if let Err(err) = self.kitty_img(*img) {
                    log::error!("kitty_img: {:#}", err);
                }
            }
        }
    }
}
