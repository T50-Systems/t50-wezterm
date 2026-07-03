impl InputParser {
    pub fn new() -> Self {
        Self {
            key_map: Self::build_basic_key_map(),
            buf: ReadBuffer::new(),
            state: InputState::Normal,
        }
    }

    fn build_basic_key_map() -> KeyMap<InputEvent> {
        let mut map = KeyMap::new();

        let modifier_combos = &[
            ("", Modifiers::NONE),
            (";1", Modifiers::NONE),
            (";2", Modifiers::SHIFT),
            (";3", Modifiers::ALT),
            (";4", Modifiers::ALT | Modifiers::SHIFT),
            (";5", Modifiers::CTRL),
            (";6", Modifiers::CTRL | Modifiers::SHIFT),
            (";7", Modifiers::CTRL | Modifiers::ALT),
            (";8", Modifiers::CTRL | Modifiers::ALT | Modifiers::SHIFT),
        ];
        // Meta is theoretically a distinct modifier of its own, but modern systems don't
        // have a dedicated Meta key and use the Alt/Option key instead.  The mapping
        // below is reproduced from the xterm documentation from a time where it was
        // possible to hold both Alt and Meta down as modifiers.  Since we define meta to
        // ALT, the use of `meta | ALT` in the table below appears to be redundant,
        // but makes it easier to see that the mapping matches xterm when viewing
        // its documentation.
        let meta = Modifiers::ALT;
        let meta_modifier_combos = &[
            (";9", meta),
            (";10", meta | Modifiers::SHIFT),
            (";11", meta | Modifiers::ALT),
            (";12", meta | Modifiers::ALT | Modifiers::SHIFT),
            (";13", meta | Modifiers::CTRL),
            (";14", meta | Modifiers::CTRL | Modifiers::SHIFT),
            (";15", meta | Modifiers::CTRL | Modifiers::ALT),
            (
                ";16",
                meta | Modifiers::CTRL | Modifiers::ALT | Modifiers::SHIFT,
            ),
        ];

        let modifier_combos_including_meta =
            || modifier_combos.iter().chain(meta_modifier_combos.iter());

        for alpha in b'A'..=b'Z' {
            // Ctrl-[A..=Z] are sent as 1..=26
            let ctrl = [alpha & 0x1f];
            map.insert(
                &ctrl,
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Char((alpha as char).to_ascii_lowercase()),
                    modifiers: Modifiers::CTRL,
                }),
            );

            // ALT A-Z is often sent with a leading ESC
            let alt = [0x1b, alpha];
            map.insert(
                &alt,
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Char(alpha as char),
                    modifiers: Modifiers::ALT,
                }),
            );
        }

        for c in 0..=0x7fu8 {
            for (suffix, modifiers) in modifier_combos {
                // `CSI u` encodings for the ascii range;
                // see http://www.leonerd.org.uk/hacks/fixterms/
                let key = format!("\x1b[{}{}u", c, suffix);
                map.insert(
                    key,
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::Char(c as char),
                        modifiers: *modifiers,
                    }),
                );

                if !suffix.is_empty() {
                    // xterm modifyOtherKeys sequences
                    let key = format!("\x1b[27{};{}~", suffix, c);
                    map.insert(
                        key,
                        InputEvent::Key(KeyEvent {
                            key: match c {
                                8 | 0x7f => KeyCode::Backspace,
                                0x1b => KeyCode::Escape,
                                9 => KeyCode::Tab,
                                10 | 13 => KeyCode::Enter,
                                _ => KeyCode::Char(c as char),
                            },
                            modifiers: *modifiers,
                        }),
                    );
                }
            }
        }

        // Common arrow keys
        for (keycode, dir) in &[
            (KeyCode::UpArrow, b'A'),
            (KeyCode::DownArrow, b'B'),
            (KeyCode::RightArrow, b'C'),
            (KeyCode::LeftArrow, b'D'),
            (KeyCode::Home, b'H'),
            (KeyCode::End, b'F'),
        ] {
            // Arrow keys in normal mode encoded using CSI
            let arrow = [0x1b, b'[', *dir];
            map.insert(
                &arrow,
                InputEvent::Key(KeyEvent {
                    key: *keycode,
                    modifiers: Modifiers::NONE,
                }),
            );
            for (suffix, modifiers) in modifier_combos_including_meta() {
                let key = format!("\x1b[1{}{}", suffix, *dir as char);
                map.insert(
                    key,
                    InputEvent::Key(KeyEvent {
                        key: *keycode,
                        modifiers: *modifiers,
                    }),
                );
            }
        }
        for &(keycode, dir) in &[
            (KeyCode::UpArrow, b'a'),
            (KeyCode::DownArrow, b'b'),
            (KeyCode::RightArrow, b'c'),
            (KeyCode::LeftArrow, b'd'),
        ] {
            // rxvt-specific modified arrows.
            for &(seq, mods) in &[
                ([0x1b, b'[', dir], Modifiers::SHIFT),
                ([0x1b, b'O', dir], Modifiers::CTRL),
            ] {
                map.insert(
                    &seq,
                    InputEvent::Key(KeyEvent {
                        key: keycode,
                        modifiers: mods,
                    }),
                );
            }
        }

        for (keycode, dir) in &[
            (KeyCode::ApplicationUpArrow, b'A'),
            (KeyCode::ApplicationDownArrow, b'B'),
            (KeyCode::ApplicationRightArrow, b'C'),
            (KeyCode::ApplicationLeftArrow, b'D'),
        ] {
            // Arrow keys in application cursor mode encoded using SS3
            let app = [0x1b, b'O', *dir];
            map.insert(
                &app,
                InputEvent::Key(KeyEvent {
                    key: *keycode,
                    modifiers: Modifiers::NONE,
                }),
            );
            for (suffix, modifiers) in modifier_combos {
                let key = format!("\x1bO1{}{}", suffix, *dir as char);
                map.insert(
                    key,
                    InputEvent::Key(KeyEvent {
                        key: *keycode,
                        modifiers: *modifiers,
                    }),
                );
            }
        }

        // Function keys 1-4 with no modifiers encoded using SS3
        for (keycode, c) in &[
            (KeyCode::Function(1), b'P'),
            (KeyCode::Function(2), b'Q'),
            (KeyCode::Function(3), b'R'),
            (KeyCode::Function(4), b'S'),
        ] {
            let key = [0x1b, b'O', *c];
            map.insert(
                &key,
                InputEvent::Key(KeyEvent {
                    key: *keycode,
                    modifiers: Modifiers::NONE,
                }),
            );
        }

        // Function keys 1-4 with modifiers
        for (keycode, c) in &[
            (KeyCode::Function(1), b'P'),
            (KeyCode::Function(2), b'Q'),
            (KeyCode::Function(3), b'R'),
            (KeyCode::Function(4), b'S'),
        ] {
            for (suffix, modifiers) in modifier_combos_including_meta() {
                let key = format!("\x1b[1{suffix}{code}", code = *c as char, suffix = suffix);
                map.insert(
                    key,
                    InputEvent::Key(KeyEvent {
                        key: *keycode,
                        modifiers: *modifiers,
                    }),
                );
            }
        }

        // Function keys with modifiers encoded using CSI.
        // http://aperiodic.net/phil/archives/Geekery/term-function-keys.html
        for (range, offset) in &[
            // F1-F5 encoded as 11-15
            (1..=5, 10),
            // F6-F10 encoded as 17-21
            (6..=10, 11),
            // F11-F14 encoded as 23-26
            (11..=14, 12),
            // F15-F16 encoded as 28-29
            (15..=16, 13),
            // F17-F20 encoded as 31-34
            (17..=20, 14),
        ] {
            for n in range.clone() {
                for (suffix, modifiers) in modifier_combos_including_meta() {
                    let key = format!("\x1b[{code}{suffix}~", code = n + offset, suffix = suffix);
                    map.insert(
                        key,
                        InputEvent::Key(KeyEvent {
                            key: KeyCode::Function(n),
                            modifiers: *modifiers,
                        }),
                    );
                }
            }
        }

        for (keycode, c) in &[
            (KeyCode::Insert, b'2'),
            (KeyCode::Delete, b'3'),
            (KeyCode::Home, b'1'),
            (KeyCode::End, b'4'),
            (KeyCode::PageUp, b'5'),
            (KeyCode::PageDown, b'6'),
            // rxvt
            (KeyCode::Home, b'7'),
            (KeyCode::End, b'8'),
        ] {
            for (suffix, modifiers) in &[
                (b'~', Modifiers::NONE),
                (b'$', Modifiers::SHIFT),
                (b'^', Modifiers::CTRL),
                (b'@', Modifiers::SHIFT | Modifiers::CTRL),
            ] {
                let key = [0x1b, b'[', *c, *suffix];
                map.insert(
                    key,
                    InputEvent::Key(KeyEvent {
                        key: *keycode,
                        modifiers: *modifiers,
                    }),
                );
            }
        }

        map.insert(
            &[0x7f],
            InputEvent::Key(KeyEvent {
                key: KeyCode::Backspace,
                modifiers: Modifiers::NONE,
            }),
        );

        map.insert(
            &[0x8],
            InputEvent::Key(KeyEvent {
                key: KeyCode::Backspace,
                modifiers: Modifiers::NONE,
            }),
        );

        map.insert(
            &[0x1b],
            InputEvent::Key(KeyEvent {
                key: KeyCode::Escape,
                modifiers: Modifiers::NONE,
            }),
        );

        map.insert(
            &[b'\t'],
            InputEvent::Key(KeyEvent {
                key: KeyCode::Tab,
                modifiers: Modifiers::NONE,
            }),
        );
        map.insert(
            b"\x1b[Z",
            InputEvent::Key(KeyEvent {
                key: KeyCode::Tab,
                modifiers: Modifiers::SHIFT,
            }),
        );

        map.insert(
            &[b'\r'],
            InputEvent::Key(KeyEvent {
                key: KeyCode::Enter,
                modifiers: Modifiers::NONE,
            }),
        );
        map.insert(
            &[b'\n'],
            InputEvent::Key(KeyEvent {
                key: KeyCode::Enter,
                modifiers: Modifiers::NONE,
            }),
        );

        map.insert(
            b"\x1b[200~",
            InputEvent::Key(KeyEvent {
                key: KeyCode::InternalPasteStart,
                modifiers: Modifiers::NONE,
            }),
        );
        map.insert(
            b"\x1b[201~",
            InputEvent::Key(KeyEvent {
                key: KeyCode::InternalPasteEnd,
                modifiers: Modifiers::NONE,
            }),
        );
        map.insert(
            b"\x1b[",
            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('['),
                modifiers: Modifiers::ALT,
            }),
        );

        map
    }
}
