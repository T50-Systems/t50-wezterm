impl InputParser {
    /// Returns the first char from a str and the length of that char
    /// in *bytes*.
    fn first_char_and_len(s: &str) -> (char, usize) {
        let mut iter = s.chars();
        let c = iter.next().unwrap();
        (c, c.len_utf8())
    }

    /// This is a horrible function to pull off the first unicode character
    /// from the sequence of bytes and return it and the remaining slice.
    fn decode_one_char(bytes: &[u8]) -> Option<(char, usize)> {
        // This has the potential to be an ugly hotspot since the complexity
        // is a function of the length of the entire buffer rather than the length
        // of the first char component.  A simple mitigation might be to slice off
        // the first 4 bytes.  We pick 4 bytes because the docs for str::len_utf8()
        // state that the maximum expansion for a `char` is 4 bytes.
        let bytes = &bytes[..bytes.len().min(4)];
        match std::str::from_utf8(bytes) {
            Ok(s) => {
                let (c, len) = Self::first_char_and_len(s);
                Some((c, len))
            }
            Err(err) => {
                let (valid, _after_valid) = bytes.split_at(err.valid_up_to());
                if !valid.is_empty() {
                    let s = unsafe { std::str::from_utf8_unchecked(valid) };
                    let (c, len) = Self::first_char_and_len(s);
                    Some((c, len))
                } else {
                    None
                }
            }
        }
    }

    fn dispatch_callback<F: FnMut(InputEvent)>(&mut self, mut callback: F, event: InputEvent) {
        match (self.state, event) {
            (
                InputState::Normal,
                InputEvent::Key(KeyEvent {
                    key: KeyCode::InternalPasteStart,
                    ..
                }),
            ) => {
                self.state = InputState::Pasting(0);
            }
            (
                InputState::EscapeMaybeAlt,
                InputEvent::Key(KeyEvent {
                    key: KeyCode::InternalPasteStart,
                    ..
                }),
            ) => {
                // The prior ESC was not part of an ALT sequence, so emit
                // it before we start collecting for paste.
                callback(InputEvent::Key(KeyEvent {
                    key: KeyCode::Escape,
                    modifiers: Modifiers::NONE,
                }));
                self.state = InputState::Pasting(0);
            }
            (InputState::EscapeMaybeAlt, InputEvent::Key(KeyEvent { key, modifiers })) => {
                // Treat this as ALT-key
                self.state = InputState::Normal;
                callback(InputEvent::Key(KeyEvent {
                    key,
                    modifiers: modifiers | Modifiers::ALT,
                }));
            }
            (InputState::EscapeMaybeAlt, event) => {
                // The prior ESC was not part of an ALT sequence, so emit
                // both it and the current event
                callback(InputEvent::Key(KeyEvent {
                    key: KeyCode::Escape,
                    modifiers: Modifiers::NONE,
                }));
                callback(event);
            }
            (_, event) => callback(event),
        }
    }

    fn process_bytes<F: FnMut(InputEvent)>(&mut self, mut callback: F, maybe_more: bool) {
        while !self.buf.is_empty() {
            match self.state {
                InputState::Pasting(offset) => {
                    let end_paste = b"\x1b[201~";
                    if let Some(idx) = self.buf.find_subsequence(offset, end_paste) {
                        let pasted =
                            String::from_utf8_lossy(&self.buf.as_slice()[0..idx]).to_string();
                        self.buf.advance(pasted.len() + end_paste.len());
                        callback(InputEvent::Paste(pasted));
                        self.state = InputState::Normal;
                    } else {
                        // Advance our offset so that in the case where we receive a paste that
                        // is spread across N reads of size 8K, we don't need to search for the
                        // end marker in 8K, 16K, 24K etc. of text until the final buffer is received.
                        // Ensure that we use saturating math here for the case where the amount
                        // of buffered data after the begin paste is smaller than the end paste marker
                        // <https://github.com/wezterm/wezterm/pull/1832>
                        self.state =
                            InputState::Pasting(self.buf.len().saturating_sub(end_paste.len()));
                        return;
                    }
                }
                InputState::EscapeMaybeAlt | InputState::Normal => {
                    if self.state == InputState::Normal && self.buf.as_slice()[0] == b'\x1b' {
                        // This feels a bit gross because we have two different parsers at play
                        // here.  We want to re-use the escape sequence parser to crack the
                        // parameters out from things like mouse reports.  The keymap tree doesn't
                        // know how to grok this.
                        let mut parser = Parser::new();
                        if let Some((Action::CSI(CSI::Mouse(mouse)), len)) =
                            parser.parse_first(self.buf.as_slice())
                        {
                            self.buf.advance(len);

                            match mouse {
                                MouseReport::SGR1006 {
                                    x,
                                    y,
                                    button,
                                    modifiers,
                                } => {
                                    callback(InputEvent::Mouse(MouseEvent {
                                        x,
                                        y,
                                        mouse_buttons: button.into(),
                                        modifiers,
                                    }));
                                }
                                MouseReport::SGR1016 {
                                    x_pixels,
                                    y_pixels,
                                    button,
                                    modifiers,
                                } => {
                                    callback(InputEvent::PixelMouse(PixelMouseEvent {
                                        x_pixels: x_pixels,
                                        y_pixels: y_pixels,
                                        mouse_buttons: button.into(),
                                        modifiers,
                                    }));
                                }
                            }
                            continue;
                        }

                        // If we have a potential SGR mouse sequence prefix and more
                        // data might come, wait for additional input before falling
                        // through to keymap lookup
                        if maybe_more && self.buf.as_slice().starts_with(b"\x1b[<") {
                            return;
                        }
                    }

                    match (
                        self.key_map.lookup(self.buf.as_slice(), maybe_more),
                        maybe_more,
                    ) {
                        // If we got an unambiguous ESC and we have more data to
                        // follow, then this is likely the Meta version of the
                        // following keypress.  Buffer up the escape key and
                        // consume it from the input.  dispatch_callback() will
                        // emit either the ESC or the ALT modified following key.
                        (
                            Found::Exact(
                                len,
                                InputEvent::Key(KeyEvent {
                                    key: KeyCode::Escape,
                                    modifiers: Modifiers::NONE,
                                }),
                            ),
                            _,
                        ) if self.state == InputState::Normal && self.buf.len() > len => {
                            self.state = InputState::EscapeMaybeAlt;
                            self.buf.advance(len);
                        }
                        (Found::Exact(len, event), _) | (Found::Ambiguous(len, event), false) => {
                            self.dispatch_callback(&mut callback, event.clone());
                            self.buf.advance(len);
                        }
                        (Found::Ambiguous(_, _), true) | (Found::NeedData, true) => {
                            return;
                        }
                        (Found::None, _) | (Found::NeedData, false) => {
                            // No pre-defined key, so pull out a unicode character
                            if let Some((c, len)) = Self::decode_one_char(self.buf.as_slice()) {
                                self.buf.advance(len);
                                self.dispatch_callback(
                                    &mut callback,
                                    InputEvent::Key(KeyEvent {
                                        key: KeyCode::Char(c),
                                        modifiers: Modifiers::NONE,
                                    }),
                                );
                            } else {
                                // We need more data to recognize the input, so
                                // yield the remainder of the slice
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Push a sequence of bytes into the parser.
    /// Each time input is recognized, the provided `callback` will be passed
    /// the decoded `InputEvent`.
    /// If not enough data are available to fully decode a sequence, the
    /// remaining data will be buffered until the next call.
    /// The `maybe_more` flag controls how ambiguous partial sequences are
    /// handled. The intent is that `maybe_more` should be set to true if
    /// you believe that you will be able to provide more data momentarily.
    /// This will cause the parser to defer judgement on partial prefix
    /// matches. You should attempt to read and pass the new data in
    /// immediately afterwards. If you have attempted a read and no data is
    /// immediately available, you should follow up with a call to parse
    /// with an empty slice and `maybe_more=false` to allow the partial
    /// data to be recognized and processed.
    pub fn parse<F: FnMut(InputEvent)>(&mut self, bytes: &[u8], callback: F, maybe_more: bool) {
        self.buf.extend_with(bytes);
        self.process_bytes(callback, maybe_more);
    }

    pub fn parse_as_vec(&mut self, bytes: &[u8], maybe_more: bool) -> Vec<InputEvent> {
        let mut result = Vec::new();
        self.parse(bytes, |event| result.push(event), maybe_more);
        result
    }

    #[cfg(windows)]
    pub fn decode_input_records_as_vec(&mut self, records: &[INPUT_RECORD]) -> Vec<InputEvent> {
        let mut result = Vec::new();
        self.decode_input_records(records, &mut |event| result.push(event));
        result
    }
}
