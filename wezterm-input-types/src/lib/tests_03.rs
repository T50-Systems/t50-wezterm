#[test]
fn encode_issue_3484() {
    let flags = KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES
        | KittyKeyboardFlags::REPORT_EVENT_TYPES
        | KittyKeyboardFlags::REPORT_ALTERNATE_KEYS
        | KittyKeyboardFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
        | KittyKeyboardFlags::REPORT_ASSOCIATED_TEXT;

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
        "\x1b[1092::97;5;1092u".to_string()
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
        "\x1b[1092:1060:97;6;1060u".to_string()
    );
}

#[test]
fn encode_issue_3526() {
    let flags = KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES;

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char(' '),
            modifiers: Modifiers::NONE,
            leds: KeyboardLedStatus::NUM_LOCK,
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        " ".to_string()
    );

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char(' '),
            modifiers: Modifiers::NONE,
            leds: KeyboardLedStatus::CAPS_LOCK,
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        " ".to_string()
    );

    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::NumLock,
                modifiers: Modifiers::NONE,
                leds: KeyboardLedStatus::empty(),
                repeat_count: 1,
                key_is_down: true,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            Some(PhysKeyCode::NumLock)
        )
        .encode_kitty(flags),
        "".to_string()
    );

    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::CapsLock,
                modifiers: Modifiers::NONE,
                leds: KeyboardLedStatus::empty(),
                repeat_count: 1,
                key_is_down: true,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            Some(PhysKeyCode::CapsLock)
        )
        .encode_kitty(flags),
        "".to_string()
    );
}

#[test]
fn encode_issue_4436() {
    let flags = KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES;

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('q'),
            modifiers: Modifiers::NONE,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "q".to_string()
    );

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('f'),
            modifiers: Modifiers::SUPER,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\u{1b}[102;9u".to_string()
    );

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('f'),
            modifiers: Modifiers::SUPER | Modifiers::SHIFT,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\u{1b}[102;10u".to_string()
    );

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('f'),
            modifiers: Modifiers::SUPER | Modifiers::SHIFT | Modifiers::CTRL,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\u{1b}[102;14u".to_string()
    );
}

/// ESC with DISAMBIGUATE_ESCAPE_CODES must produce \x1b[27;1u, not a raw \x1b.
/// https://sw.kovidgoyal.net/kitty/keyboard-protocol/#disambiguate
#[test]
fn encode_escape_disambiguate() {
    // Flag 1 only: ESC on key-down → \x1b[27;1u
    let flags = KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES;
    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('\x1b'),
            modifiers: Modifiers::NONE,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\x1b[27;1u".to_string()
    );

    // No flags at all: ESC on key-down must still be sent as a raw \x1b
    // (legacy behaviour is unchanged).
    let flags = KittyKeyboardFlags::NONE;
    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('\x1b'),
            modifiers: Modifiers::NONE,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\x1b".to_string()
    );

    // DISAMBIGUATE + REPORT_EVENT_TYPES: key-up must produce \x1b[27;1:3u
    let flags =
        KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES | KittyKeyboardFlags::REPORT_EVENT_TYPES;
    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('\x1b'),
            modifiers: Modifiers::NONE,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: false,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\x1b[27;1:3u".to_string()
    );
}
