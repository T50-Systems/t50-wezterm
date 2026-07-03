#[test]
fn encode_issue_3478() {
    let flags = KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES
        | KittyKeyboardFlags::REPORT_EVENT_TYPES
        | KittyKeyboardFlags::REPORT_ALTERNATE_KEYS
        | KittyKeyboardFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES;

    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::Numpad(0),
                modifiers: Modifiers::NONE,
                leds: KeyboardLedStatus::empty(),
                repeat_count: 1,
                key_is_down: true,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            None
        )
        .encode_kitty(flags),
        "\u{1b}[57425;1u".to_string()
    );
    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::Numpad(0),
                modifiers: Modifiers::SHIFT,
                leds: KeyboardLedStatus::empty(),
                repeat_count: 1,
                key_is_down: true,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            None
        )
        .encode_kitty(flags),
        "\u{1b}[57425;2u".to_string()
    );

    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::Numpad(1),
                modifiers: Modifiers::NONE,
                leds: KeyboardLedStatus::empty(),
                repeat_count: 1,
                key_is_down: true,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            None
        )
        .encode_kitty(flags),
        "\u{1b}[57424;1u".to_string()
    );
    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::Numpad(1),
                modifiers: Modifiers::SHIFT,
                leds: KeyboardLedStatus::empty(),
                repeat_count: 1,
                key_is_down: true,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            None
        )
        .encode_kitty(flags),
        "\u{1b}[57424;2u".to_string()
    );

    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::Numpad(0),
                modifiers: Modifiers::NONE,
                leds: KeyboardLedStatus::NUM_LOCK,
                repeat_count: 1,
                key_is_down: true,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            Some(PhysKeyCode::Keypad0)
        )
        .encode_kitty(flags),
        "\u{1b}[57399;129u".to_string()
    );
    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::Numpad(0),
                modifiers: Modifiers::SHIFT,
                leds: KeyboardLedStatus::NUM_LOCK,
                repeat_count: 1,
                key_is_down: true,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            Some(PhysKeyCode::Keypad0)
        )
        .encode_kitty(flags),
        "\u{1b}[57399;130u".to_string()
    );

    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::Numpad(5),
                modifiers: Modifiers::NONE,
                leds: KeyboardLedStatus::NUM_LOCK,
                repeat_count: 1,
                key_is_down: true,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            Some(PhysKeyCode::Keypad5)
        )
        .encode_kitty(flags),
        "\u{1b}[57404;129u".to_string()
    );

    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::Numpad(5),
                modifiers: Modifiers::NONE,
                leds: KeyboardLedStatus::empty(),
                repeat_count: 1,
                key_is_down: true,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            Some(PhysKeyCode::Keypad5)
        )
        .encode_kitty(flags),
        "\u{1b}[E".to_string()
    );
}

#[test]
fn encode_issue_3478_extra() {
    let flags = KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES
        | KittyKeyboardFlags::REPORT_EVENT_TYPES
        | KittyKeyboardFlags::REPORT_ALTERNATE_KEYS
        | KittyKeyboardFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
        | KittyKeyboardFlags::REPORT_ASSOCIATED_TEXT;

    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::Numpad(5),
                modifiers: Modifiers::NONE,
                leds: KeyboardLedStatus::NUM_LOCK,
                repeat_count: 1,
                key_is_down: true,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            Some(PhysKeyCode::Keypad5)
        )
        .encode_kitty(flags),
        "\u{1b}[57404;129;53u".to_string()
    );
    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::Numpad(5),
                modifiers: Modifiers::NONE,
                leds: KeyboardLedStatus::NUM_LOCK,
                repeat_count: 1,
                key_is_down: false,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            Some(PhysKeyCode::Keypad5)
        )
        .encode_kitty(flags),
        "\u{1b}[57404;129:3u".to_string()
    );

    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::Numpad(5),
                modifiers: Modifiers::NONE,
                leds: KeyboardLedStatus::empty(),
                repeat_count: 1,
                key_is_down: true,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            Some(PhysKeyCode::Keypad5)
        )
        .encode_kitty(flags),
        "\u{1b}[E".to_string()
    );

    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::Numpad(5),
                modifiers: Modifiers::NONE,
                leds: KeyboardLedStatus::empty(),
                repeat_count: 1,
                key_is_down: false,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            Some(PhysKeyCode::Keypad5)
        )
        .encode_kitty(flags),
        "\u{1b}[1;1:3E".to_string()
    );
}

#[test]
fn encode_issue_3315() {
    let flags = KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES;

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('"'),
            modifiers: Modifiers::NONE,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\"".to_string()
    );

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('"'),
            modifiers: Modifiers::SHIFT,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\"".to_string()
    );

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('!'),
            modifiers: Modifiers::SHIFT,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "!".to_string()
    );

    assert_eq!(
        KeyEvent {
            key: KeyCode::LeftShift,
            modifiers: Modifiers::NONE,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "".to_string()
    );
}

#[test]
fn encode_issue_3479() {
    let flags = KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES
        | KittyKeyboardFlags::REPORT_EVENT_TYPES
        | KittyKeyboardFlags::REPORT_ALTERNATE_KEYS
        | KittyKeyboardFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES;

    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::Char('ф'),
                modifiers: Modifiers::CTRL,
                leds: KeyboardLedStatus::empty(),
                repeat_count: 1,
                key_is_down: true,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            Some(PhysKeyCode::A)
        )
        .encode_kitty(flags),
        "\x1b[1092::97;5u".to_string()
    );

    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::Char('Ф'),
                modifiers: Modifiers::CTRL | Modifiers::SHIFT,
                leds: KeyboardLedStatus::empty(),
                repeat_count: 1,
                key_is_down: true,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            Some(PhysKeyCode::A)
        )
        .encode_kitty(flags),
        "\x1b[1092:1060:97;6u".to_string()
    );
}
