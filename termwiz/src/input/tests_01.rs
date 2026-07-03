#[test]
fn simple() {
    let mut p = InputParser::new();
    let inputs = p.parse_as_vec(b"hello", NO_MORE);
    assert_eq!(
        vec![
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::Char('h'),
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::Char('e'),
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::Char('l'),
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::Char('l'),
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::Char('o'),
            }),
        ],
        inputs
    );
}

#[test]
fn control_characters() {
    let mut p = InputParser::new();
    let inputs = p.parse_as_vec(b"\x03\x1bJ\x7f", NO_MORE);
    assert_eq!(
        vec![
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::CTRL,
                key: KeyCode::Char('c'),
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::ALT,
                key: KeyCode::Char('J'),
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::Backspace,
            }),
        ],
        inputs
    );
}

#[test]
fn arrow_keys() {
    let mut p = InputParser::new();
    let inputs = p.parse_as_vec(b"\x1bOA\x1bOB\x1bOC\x1bOD", NO_MORE);
    assert_eq!(
        vec![
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::ApplicationUpArrow,
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::ApplicationDownArrow,
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::ApplicationRightArrow,
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::ApplicationLeftArrow,
            }),
        ],
        inputs
    );
}

#[test]
fn partial() {
    let mut p = InputParser::new();
    let mut inputs = Vec::new();
    // Fragment this F-key sequence across two different pushes
    p.parse(b"\x1b[11", |evt| inputs.push(evt), true);
    p.parse(b"~", |evt| inputs.push(evt), true);
    // make sure we recognize it as just the F-key
    assert_eq!(
        vec![InputEvent::Key(KeyEvent {
            modifiers: Modifiers::NONE,
            key: KeyCode::Function(1),
        })],
        inputs
    );
}

#[test]
fn partial_ambig() {
    let mut p = InputParser::new();

    assert_eq!(
        vec![InputEvent::Key(KeyEvent {
            key: KeyCode::Escape,
            modifiers: Modifiers::NONE,
        })],
        p.parse_as_vec(b"\x1b", false)
    );

    let mut inputs = Vec::new();
    // An incomplete F-key sequence fragmented across two different pushes
    p.parse(b"\x1b[11", |evt| inputs.push(evt), MAYBE_MORE);
    p.parse(b"", |evt| inputs.push(evt), NO_MORE);
    // since we finish with maybe_more false (NO_MORE), the results should be the longest matching
    // parts of said f-key sequence
    assert_eq!(
        vec![
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::ALT,
                key: KeyCode::Char('['),
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::Char('1'),
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::Char('1'),
            }),
        ],
        inputs
    );
}

#[test]
fn partial_mouse() {
    let mut p = InputParser::new();
    let mut inputs = Vec::new();
    // Fragment this mouse sequence across two different pushes
    p.parse(b"\x1b[<0;0;0", |evt| inputs.push(evt), true);
    p.parse(b"M", |evt| inputs.push(evt), true);
    // make sure we recognize it as just the mouse event
    assert_eq!(
        vec![InputEvent::Mouse(MouseEvent {
            x: 0,
            y: 0,
            mouse_buttons: MouseButtons::LEFT,
            modifiers: Modifiers::NONE,
        })],
        inputs
    );
}

#[test]
fn partial_mouse_ambig() {
    let mut p = InputParser::new();
    let mut inputs = Vec::new();
    // Fragment this mouse sequence across two different pushes
    p.parse(b"\x1b[<", |evt| inputs.push(evt), MAYBE_MORE);
    p.parse(b"0;0;0", |evt| inputs.push(evt), NO_MORE);
    // since we finish with maybe_more false (NO_MORE), the results should be the longest matching
    // parts of said mouse sequence
    assert_eq!(
        vec![
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::ALT,
                key: KeyCode::Char('['),
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::Char('<'),
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::Char('0'),
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::Char(';'),
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::Char('0'),
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::Char(';'),
            }),
            InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::Char('0'),
            }),
        ],
        inputs
    );
}

#[test]
fn alt_left_bracket() {
    // tests that `Alt` + `[` is recognized as a single
    // event rather than two events (one `Esc` the second `Char('[')`)
    let mut p = InputParser::new();

    let mut inputs = Vec::new();
    p.parse(b"\x1b[", |evt| inputs.push(evt), false);

    assert_eq!(
        vec![InputEvent::Key(KeyEvent {
            modifiers: Modifiers::ALT,
            key: KeyCode::Char('['),
        }),],
        inputs
    );
}

#[test]
fn modify_other_keys_parse() {
    let mut p = InputParser::new();
    let inputs = p.parse_as_vec(
        b"\x1b[27;5;13~\x1b[27;5;9~\x1b[27;6;8~\x1b[27;2;127~\x1b[27;6;27~",
        NO_MORE,
    );
    assert_eq!(
        vec![
            InputEvent::Key(KeyEvent {
                key: KeyCode::Enter,
                modifiers: Modifiers::CTRL,
            }),
            InputEvent::Key(KeyEvent {
                key: KeyCode::Tab,
                modifiers: Modifiers::CTRL,
            }),
            InputEvent::Key(KeyEvent {
                key: KeyCode::Backspace,
                modifiers: Modifiers::CTRL | Modifiers::SHIFT,
            }),
            InputEvent::Key(KeyEvent {
                key: KeyCode::Backspace,
                modifiers: Modifiers::SHIFT,
            }),
            InputEvent::Key(KeyEvent {
                key: KeyCode::Escape,
                modifiers: Modifiers::CTRL | Modifiers::SHIFT,
            }),
        ],
        inputs
    );
}

#[test]
fn modify_other_keys_encode() {
    let mode = KeyCodeEncodeModes {
        encoding: KeyboardEncoding::Xterm,
        newline_mode: false,
        application_cursor_keys: false,
        modify_other_keys: None,
    };
    let mode_1 = KeyCodeEncodeModes {
        encoding: KeyboardEncoding::Xterm,
        newline_mode: false,
        application_cursor_keys: false,
        modify_other_keys: Some(1),
    };
    let mode_2 = KeyCodeEncodeModes {
        encoding: KeyboardEncoding::Xterm,
        newline_mode: false,
        application_cursor_keys: false,
        modify_other_keys: Some(2),
    };

    assert_eq!(
        KeyCode::Enter.encode(Modifiers::CTRL, mode, true).unwrap(),
        "\r".to_string()
    );
    assert_eq!(
        KeyCode::Enter
            .encode(Modifiers::CTRL, mode_1, true)
            .unwrap(),
        "\x1b[27;5;13~".to_string()
    );
    assert_eq!(
        KeyCode::Enter
            .encode(Modifiers::CTRL | Modifiers::SHIFT, mode_1, true)
            .unwrap(),
        "\x1b[27;6;13~".to_string()
    );

    // This case is not conformant with xterm!
    // xterm just returns tab for CTRL-Tab when modify_other_keys
    // is not set.
    assert_eq!(
        KeyCode::Tab.encode(Modifiers::CTRL, mode, true).unwrap(),
        "\x1b[9;5u".to_string()
    );
    assert_eq!(
        KeyCode::Tab.encode(Modifiers::CTRL, mode_1, true).unwrap(),
        "\x1b[27;5;9~".to_string()
    );
    assert_eq!(
        KeyCode::Tab
            .encode(Modifiers::CTRL | Modifiers::SHIFT, mode_1, true)
            .unwrap(),
        "\x1b[27;6;9~".to_string()
    );

    assert_eq!(
        KeyCode::Char('c')
            .encode(Modifiers::CTRL, mode, true)
            .unwrap(),
        "\x03".to_string()
    );
    assert_eq!(
        KeyCode::Char('c')
            .encode(Modifiers::CTRL, mode_1, true)
            .unwrap(),
        "\x03".to_string()
    );
    assert_eq!(
        KeyCode::Char('c')
            .encode(Modifiers::CTRL, mode_2, true)
            .unwrap(),
        "\x1b[27;5;99~".to_string()
    );

    assert_eq!(
        KeyCode::Char('1')
            .encode(Modifiers::CTRL, mode, true)
            .unwrap(),
        "1".to_string()
    );
    assert_eq!(
        KeyCode::Char('1')
            .encode(Modifiers::CTRL, mode_2, true)
            .unwrap(),
        "\x1b[27;5;49~".to_string()
    );

    assert_eq!(
        KeyCode::Char(',')
            .encode(Modifiers::CTRL, mode, true)
            .unwrap(),
        ",".to_string()
    );
    assert_eq!(
        KeyCode::Char(',')
            .encode(Modifiers::CTRL, mode_2, true)
            .unwrap(),
        "\x1b[27;5;44~".to_string()
    );
}
