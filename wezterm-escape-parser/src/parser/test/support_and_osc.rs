fn encode(seq: &Vec<Action>) -> String {
    let mut res = Vec::new();
    for s in seq {
        write!(res, "{}", s).unwrap();
    }
    String::from_utf8(res).unwrap()
}

// <https://github.com/markbt/streampager/issues/57>
#[test]
fn osc_bel_parse_first_as_vec() {
    let data = b"\x1b]8;;http://example.com\x07example\x1b]8;;\x07";
    let mut p = Parser::new();

    let mut offset = 0;
    let mut actions = vec![];
    while let Some((mut act, off)) = p.parse_first_as_vec(&data[offset..]) {
        actions.append(&mut act);
        offset += off;
    }

    k9::snapshot!(
        actions,
        r#"
[
    OperatingSystemCommand(
        SetHyperlink(
            Some(
                Hyperlink {
                    params: {},
                    uri: "http://example.com",
                    implicit: false,
                },
            ),
        ),
    ),
    Print(
        'e',
    ),
    Print(
        'x',
    ),
    Print(
        'a',
    ),
    Print(
        'm',
    ),
    Print(
        'p',
    ),
    Print(
        'l',
    ),
    Print(
        'e',
    ),
    OperatingSystemCommand(
        SetHyperlink(
            None,
        ),
    ),
]
"#
    );
}

// <https://github.com/markbt/streampager/issues/57>
#[test]
fn osc_st_parse_first_as_vec() {
    // This string includes an assitional trailing ST sequence which should
    // be parsed separately.
    let data = b"\x1b]8;;http://example.com\x1b\\example\x1b]8;;\x1b\\\x1b\\";
    let mut p = Parser::new();

    let mut offset = 0;
    let mut actions = vec![];
    let mut slices = vec![];
    while let Some((act, off)) = p.parse_first_as_vec(&data[offset..]) {
        // Store each vec of actions so we can confirm that the ST sequence is bundled with the
        // OSC SetHyperlink command.
        actions.push(act);
        // Additionally store all non-single-character slices so we can confirm these are split
        // correctly.
        if off > 1 {
            slices.push(&data[offset..offset + off]);
        }
        offset += off;
    }

    assert_eq!(
        slices,
        vec![
            b"\x1b]8;;http://example.com\x1b\\".as_slice(),
            b"\x1b]8;;\x1b\\".as_slice(),
            b"\x1b\\".as_slice()
        ]
    );

    k9::snapshot!(
        actions,
        r#"
[
    [
        OperatingSystemCommand(
            SetHyperlink(
                Some(
                    Hyperlink {
                        params: {},
                        uri: "http://example.com",
                        implicit: false,
                    },
                ),
            ),
        ),
        Esc(
            Code(
                StringTerminator,
            ),
        ),
    ],
    [
        Print(
            'e',
        ),
    ],
    [
        Print(
            'x',
        ),
    ],
    [
        Print(
            'a',
        ),
    ],
    [
        Print(
            'm',
        ),
    ],
    [
        Print(
            'p',
        ),
    ],
    [
        Print(
            'l',
        ),
    ],
    [
        Print(
            'e',
        ),
    ],
    [
        OperatingSystemCommand(
            SetHyperlink(
                None,
            ),
        ),
        Esc(
            Code(
                StringTerminator,
            ),
        ),
    ],
    [
        Esc(
            Code(
                StringTerminator,
            ),
        ),
    ],
]
"#
    );
}
