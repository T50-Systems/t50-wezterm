#[test]
fn encode_issue_892() {
    let mode = KeyCodeEncodeModes {
        encoding: KeyboardEncoding::Xterm,
        newline_mode: false,
        application_cursor_keys: false,
        modify_other_keys: None,
    };

    assert_eq!(
        KeyCode::LeftArrow
            .encode(Modifiers::NONE, mode, true)
            .unwrap(),
        "\x1b[D".to_string()
    );
    assert_eq!(
        KeyCode::LeftArrow
            .encode(Modifiers::ALT, mode, true)
            .unwrap(),
        "\x1b[1;3D".to_string()
    );
    assert_eq!(
        KeyCode::Home.encode(Modifiers::NONE, mode, true).unwrap(),
        "\x1b[H".to_string()
    );
    assert_eq!(
        KeyCode::Home.encode(Modifiers::ALT, mode, true).unwrap(),
        "\x1b[1;3H".to_string()
    );
    assert_eq!(
        KeyCode::End.encode(Modifiers::NONE, mode, true).unwrap(),
        "\x1b[F".to_string()
    );
    assert_eq!(
        KeyCode::End.encode(Modifiers::ALT, mode, true).unwrap(),
        "\x1b[1;3F".to_string()
    );
    assert_eq!(
        KeyCode::Tab.encode(Modifiers::ALT, mode, true).unwrap(),
        "\x1b\t".to_string()
    );
    assert_eq!(
        KeyCode::PageUp.encode(Modifiers::ALT, mode, true).unwrap(),
        "\x1b[5;3~".to_string()
    );
    assert_eq!(
        KeyCode::Function(1)
            .encode(Modifiers::NONE, mode, true)
            .unwrap(),
        "\x1bOP".to_string()
    );
}

#[test]
fn partial_bracketed_paste() {
    let mut p = InputParser::new();

    let input = b"\x1b[200~1234";
    let input2 = b"5678\x1b[201~";

    let mut inputs = vec![];

    p.parse(input, |e| inputs.push(e), false);
    p.parse(input2, |e| inputs.push(e), false);

    assert_eq!(vec![InputEvent::Paste("12345678".to_owned())], inputs)
}

#[test]
fn mouse_horizontal_scroll() {
    let mut p = InputParser::new();

    let input = b"\x1b[<66;42;12M\x1b[<67;42;12M";
    let res = p.parse_as_vec(input, MAYBE_MORE);

    assert_eq!(
        vec![
            InputEvent::Mouse(MouseEvent {
                x: 42,
                y: 12,
                mouse_buttons: MouseButtons::HORZ_WHEEL | MouseButtons::WHEEL_POSITIVE,
                modifiers: Modifiers::NONE,
            }),
            InputEvent::Mouse(MouseEvent {
                x: 42,
                y: 12,
                mouse_buttons: MouseButtons::HORZ_WHEEL,
                modifiers: Modifiers::NONE,
            })
        ],
        res
    );
}

#[test]
fn encode_issue_3478_xterm() {
    let mode = KeyCodeEncodeModes {
        encoding: KeyboardEncoding::Xterm,
        newline_mode: false,
        application_cursor_keys: false,
        modify_other_keys: None,
    };

    assert_eq!(
        KeyCode::Numpad0
            .encode(Modifiers::NONE, mode, true)
            .unwrap(),
        "\u{1b}[2~".to_string()
    );
    assert_eq!(
        KeyCode::Numpad0
            .encode(Modifiers::SHIFT, mode, true)
            .unwrap(),
        "\u{1b}[2;2~".to_string()
    );

    assert_eq!(
        KeyCode::Numpad1
            .encode(Modifiers::NONE, mode, true)
            .unwrap(),
        "\u{1b}[F".to_string()
    );
    assert_eq!(
        KeyCode::Numpad1
            .encode(Modifiers::NONE | Modifiers::SHIFT, mode, true)
            .unwrap(),
        "\u{1b}[1;2F".to_string()
    );
}

#[test]
fn encode_tab_with_modifiers() {
    let mode = KeyCodeEncodeModes {
        encoding: KeyboardEncoding::Xterm,
        newline_mode: false,
        application_cursor_keys: false,
        modify_other_keys: None,
    };

    let mods_to_result = [
        (Modifiers::SHIFT, "\u{1b}[Z"),
        (Modifiers::SHIFT | Modifiers::LEFT_SHIFT, "\u{1b}[Z"),
        (Modifiers::SHIFT | Modifiers::RIGHT_SHIFT, "\u{1b}[Z"),
        (Modifiers::CTRL, "\u{1b}[9;5u"),
        (Modifiers::CTRL | Modifiers::LEFT_CTRL, "\u{1b}[9;5u"),
        (Modifiers::CTRL | Modifiers::RIGHT_CTRL, "\u{1b}[9;5u"),
        (
            Modifiers::SHIFT | Modifiers::CTRL | Modifiers::LEFT_CTRL | Modifiers::LEFT_SHIFT,
            "\u{1b}[1;5Z",
        ),
    ];
    for (mods, result) in mods_to_result {
        assert_eq!(
            KeyCode::Tab.encode(mods, mode, true).unwrap(),
            result,
            "{:?}",
            mods
        );
    }
}
