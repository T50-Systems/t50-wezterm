
#[test]
fn encode_issue_3220() {
    let flags =
        KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES | KittyKeyboardFlags::REPORT_EVENT_TYPES;

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('o'),
            modifiers: Modifiers::NONE,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "o".to_string()
    );
    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('o'),
            modifiers: Modifiers::NONE,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: false,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\x1b[111;1:3u".to_string()
    );
}

#[test]
fn encode_issue_3473() {
    let flags = KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES
        | KittyKeyboardFlags::REPORT_EVENT_TYPES
        | KittyKeyboardFlags::REPORT_ALTERNATE_KEYS
        | KittyKeyboardFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES;

    assert_eq!(
        KeyEvent {
            key: KeyCode::Function(1),
            modifiers: Modifiers::NONE,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\x1b[11;1~".to_string()
    );
    assert_eq!(
        KeyEvent {
            key: KeyCode::Function(1),
            modifiers: Modifiers::NONE,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: false,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\x1b[11;1:3~".to_string()
    );
}

#[test]
fn encode_issue_2546() {
    let flags = KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES;

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('i'),
            modifiers: Modifiers::ALT | Modifiers::SHIFT,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\x1b[105;4u".to_string()
    );
    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('I'),
            modifiers: Modifiers::ALT | Modifiers::SHIFT,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\x1b[105;4u".to_string()
    );

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('1'),
            modifiers: Modifiers::ALT | Modifiers::SHIFT,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\x1b[49;4u".to_string()
    );

    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::Char('!'),
                modifiers: Modifiers::ALT | Modifiers::SHIFT,
                leds: KeyboardLedStatus::empty(),
                repeat_count: 1,
                key_is_down: true,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            Some(PhysKeyCode::K1)
        )
        .encode_kitty(flags),
        "\x1b[49;4u".to_string()
    );

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('i'),
            modifiers: Modifiers::SHIFT | Modifiers::CTRL,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\x1b[105;6u".to_string()
    );
    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('I'),
            modifiers: Modifiers::SHIFT | Modifiers::CTRL,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\x1b[105;6u".to_string()
    );

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('I'),
            modifiers: Modifiers::CTRL,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: Some(RawKeyEvent {
                key: KeyCode::Char('I'),
                modifiers: Modifiers::SHIFT | Modifiers::CTRL,
                handled: Handled::new(),
                key_is_down: true,
                raw_code: 0,
                leds: KeyboardLedStatus::empty(),
                phys_code: Some(PhysKeyCode::I),
                #[cfg(windows)]
                scan_code: 0,
                repeat_count: 1,
            }),
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\x1b[105;6u".to_string()
    );

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('i'),
            modifiers: Modifiers::ALT | Modifiers::SHIFT | Modifiers::CTRL,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\x1b[105;8u".to_string()
    );
    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('I'),
            modifiers: Modifiers::ALT | Modifiers::SHIFT | Modifiers::CTRL,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\x1b[105;8u".to_string()
    );

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('\x08'),
            modifiers: Modifiers::NONE,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\x7f".to_string()
    );

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('\x08'),
            modifiers: Modifiers::CTRL,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\x1b[127;5u".to_string()
    );
}

#[test]
fn encode_issue_3474() {
    let flags = KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES
        | KittyKeyboardFlags::REPORT_EVENT_TYPES
        | KittyKeyboardFlags::REPORT_ALTERNATE_KEYS
        | KittyKeyboardFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES;

    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('A'),
            modifiers: Modifiers::NONE,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: true,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\u{1b}[97:65;1u".to_string()
    );
    assert_eq!(
        KeyEvent {
            key: KeyCode::Char('A'),
            modifiers: Modifiers::NONE,
            leds: KeyboardLedStatus::empty(),
            repeat_count: 1,
            key_is_down: false,
            raw: None,
            #[cfg(windows)]
            win32_uni_char: None,
        }
        .encode_kitty(flags),
        "\u{1b}[97:65;1:3u".to_string()
    );
}

fn make_event_with_raw(mut event: KeyEvent, phys: Option<PhysKeyCode>) -> KeyEvent {
    let phys = match phys {
        Some(phys) => Some(phys),
        None => event.key.to_phys(),
    };

    event.raw = Some(RawKeyEvent {
        key: event.key.clone(),
        modifiers: event.modifiers,
        leds: KeyboardLedStatus::empty(),
        phys_code: phys,
        raw_code: 0,
        #[cfg(windows)]
        scan_code: 0,
        repeat_count: 1,
        key_is_down: event.key_is_down,
        handled: Handled::new(),
    });

    event
}

#[test]
fn encode_issue_3476() {
    let flags = KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES
        | KittyKeyboardFlags::REPORT_EVENT_TYPES
        | KittyKeyboardFlags::REPORT_ALTERNATE_KEYS
        | KittyKeyboardFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES;

    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::LeftShift,
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
        "\u{1b}[57441;1u".to_string()
    );
    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::LeftShift,
                modifiers: Modifiers::NONE,
                leds: KeyboardLedStatus::empty(),
                repeat_count: 1,
                key_is_down: false,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            None
        )
        .encode_kitty(flags),
        "\u{1b}[57441;1:3u".to_string()
    );
    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::LeftControl,
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
        "\u{1b}[57442;1u".to_string()
    );
    assert_eq!(
        make_event_with_raw(
            KeyEvent {
                key: KeyCode::LeftControl,
                modifiers: Modifiers::NONE,
                leds: KeyboardLedStatus::empty(),
                repeat_count: 1,
                key_is_down: false,
                raw: None,
                #[cfg(windows)]
                win32_uni_char: None,
            },
            None
        )
        .encode_kitty(flags),
        "\u{1b}[57442;1:3u".to_string()
    );
}
