// The \x20's look a little awkward, but we can't use a plain
// space in the first chararcter of a multi-line continuation;
// it gets eaten up and ignored.

#[test]
fn basic_print() {
    let mut s = Surface::new(4, 3);
    assert_eq!(
        s.screen_chars_to_string(),
        "\x20\x20\x20\x20\n\
             \x20\x20\x20\x20\n\
             \x20\x20\x20\x20\n"
    );

    s.add_change("w00t");
    assert_eq!(
        s.screen_chars_to_string(),
        "w00t\n\
             \x20\x20\x20\x20\n\
             \x20\x20\x20\x20\n"
    );

    s.add_change("foo");
    assert_eq!(
        s.screen_chars_to_string(),
        "w00t\n\
             foo\x20\n\
             \x20\x20\x20\x20\n"
    );

    s.add_change("baar");
    assert_eq!(
        s.screen_chars_to_string(),
        "w00t\n\
             foob\n\
             aar\x20\n"
    );

    s.add_change("baz");
    assert_eq!(
        s.screen_chars_to_string(),
        "foob\n\
             aarb\n\
             az\x20\x20\n"
    );
}

#[test]
fn newline() {
    let mut s = Surface::new(4, 4);
    s.add_change("bloo\rwat\n hey\r\nho");
    assert_eq!(
        s.screen_chars_to_string(),
        "wato\n\
             \x20\x20\x20\x20\n\
             hey \n\
             ho  \n"
    );
}

#[test]
fn clear_screen() {
    let mut s = Surface::new(2, 2);
    s.add_change("hello");
    assert_eq!(s.xpos, 1);
    assert_eq!(s.ypos, 1);
    s.add_change(Change::ClearScreen(Default::default()));
    assert_eq!(s.xpos, 0);
    assert_eq!(s.ypos, 0);
    assert_eq!(s.screen_chars_to_string(), "  \n  \n");
}

#[test]
fn clear_eol() {
    let mut s = Surface::new(3, 3);
    s.add_change("helwowfoo");
    s.add_change(Change::ClearToEndOfLine(Default::default()));
    assert_eq!(s.screen_chars_to_string(), "hel\nwow\nfoo\n");
    s.add_change(Change::CursorPosition {
        x: Position::Absolute(0),
        y: Position::Absolute(0),
    });
    s.add_change(Change::ClearToEndOfLine(Default::default()));
    assert_eq!(s.screen_chars_to_string(), "   \nwow\nfoo\n");
    s.add_change(Change::CursorPosition {
        x: Position::Absolute(1),
        y: Position::Absolute(1),
    });
    s.add_change(Change::ClearToEndOfLine(Default::default()));
    assert_eq!(s.screen_chars_to_string(), "   \nw\nfoo\n");
}

#[test]
fn clear_eos() {
    let mut s = Surface::new(3, 3);
    s.add_change("helwowfoo");
    s.add_change(Change::ClearToEndOfScreen(Default::default()));
    assert_eq!(s.screen_chars_to_string(), "hel\nwow\nfoo\n");
    s.add_change(Change::CursorPosition {
        x: Position::Absolute(1),
        y: Position::Absolute(1),
    });
    s.add_change(Change::ClearToEndOfScreen(Default::default()));
    assert_eq!(s.screen_chars_to_string(), "hel\nw\n   \n");

    let (_seq, changes) = s.get_changes(0);
    assert_eq!(
        &[
            Change::CursorVisibility(CursorVisibility::Hidden),
            Change::ClearScreen(Default::default()),
            Change::Text("hel".into()),
            Change::CursorPosition {
                x: Position::Absolute(0),
                y: Position::Relative(1),
            },
            Change::Text("w".into()),
            Change::CursorPosition {
                x: Position::Absolute(1),
                y: Position::Absolute(1),
            },
            Change::CursorVisibility(CursorVisibility::Visible),
        ],
        &*changes
    );
}

#[test]
fn clear_eos_back_color() {
    let mut s = Surface::new(3, 3);
    s.add_change(Change::ClearScreen(AnsiColor::Red.into()));
    s.add_change("helwowfoo");
    assert_eq!(s.screen_chars_to_string(), "hel\nwow\nfoo\n");
    s.add_change(Change::CursorPosition {
        x: Position::Absolute(1),
        y: Position::Absolute(1),
    });
    s.add_change(Change::ClearToEndOfScreen(AnsiColor::Red.into()));
    assert_eq!(s.screen_chars_to_string(), "hel\nw  \n   \n");

    let (_seq, changes) = s.get_changes(0);
    assert_eq!(
        &[
            Change::CursorVisibility(CursorVisibility::Hidden),
            Change::ClearScreen(Default::default()),
            Change::AllAttributes(
                CellAttributes::default()
                    .set_background(AnsiColor::Red)
                    .clone()
            ),
            Change::Text("hel".into()),
            Change::CursorPosition {
                x: Position::Absolute(0),
                y: Position::Relative(1),
            },
            Change::Text("w".into()),
            Change::ClearToEndOfScreen(AnsiColor::Red.into()),
            Change::CursorPosition {
                x: Position::Absolute(1),
                y: Position::Absolute(1),
            },
            Change::CursorVisibility(CursorVisibility::Visible),
        ],
        &*changes
    );
}

#[test]
fn clear_eol_opt() {
    let mut s = Surface::new(3, 3);
    s.add_change(Change::Attribute(AttributeChange::Background(
        AnsiColor::Red.into(),
    )));
    s.add_change("111   333");
    let (_seq, changes) = s.get_changes(0);
    assert_eq!(
        &[
            Change::CursorVisibility(CursorVisibility::Hidden),
            Change::ClearScreen(Default::default()),
            Change::AllAttributes(
                CellAttributes::default()
                    .set_background(AnsiColor::Red)
                    .clone()
            ),
            Change::Text("111".into()),
            Change::CursorPosition {
                x: Position::Absolute(0),
                y: Position::Relative(1),
            },
            Change::ClearToEndOfLine(AnsiColor::Red.into()),
            Change::CursorPosition {
                x: Position::Absolute(0),
                y: Position::Relative(1),
            },
            Change::Text("333".into()),
            Change::CursorPosition {
                x: Position::Absolute(3),
                y: Position::Absolute(2),
            },
            Change::CursorVisibility(CursorVisibility::Visible),
        ],
        &*changes
    );
}

#[test]
fn clear_and_move_cursor() {
    let mut s = Surface::new(4, 3);
    s.add_change(Change::CursorPosition {
        x: Position::Absolute(3),
        y: Position::Absolute(2),
    });
    let (_seq, changes) = s.get_changes(0);
    assert_eq!(
        &[
            Change::CursorVisibility(CursorVisibility::Hidden),
            Change::ClearScreen(Default::default()),
            Change::CursorPosition {
                x: Position::Absolute(3),
                y: Position::Absolute(2),
            },
            Change::CursorVisibility(CursorVisibility::Visible),
        ],
        &*changes
    );
}

#[test]
fn cursor_movement() {
    let mut s = Surface::new(4, 3);
    s.add_change(Change::CursorPosition {
        x: Position::Absolute(3),
        y: Position::Absolute(2),
    });
    s.add_change("X");
    assert_eq!(
        s.screen_chars_to_string(),
        "\x20\x20\x20\x20\n\
             \x20\x20\x20\x20\n\
             \x20\x20\x20X\n"
    );

    s.add_change(Change::CursorPosition {
        x: Position::Relative(-2),
        y: Position::Relative(-1),
    });
    s.add_change("-");
    assert_eq!(
        s.screen_chars_to_string(),
        "\x20\x20\x20\x20\n\
             \x20\x20-\x20\n\
             \x20\x20\x20X\n"
    );

    s.add_change(Change::CursorPosition {
        x: Position::Relative(1),
        y: Position::Relative(-1),
    });
    s.add_change("-");
    assert_eq!(
        s.screen_chars_to_string(),
        "\x20\x20\x20-\n\
             \x20\x20-\x20\n\
             \x20\x20\x20X\n"
    );
}

#[test]
fn attribute_setting() {
    use wezterm_cell::Intensity;

    let mut s = Surface::new(3, 1);
    s.add_change("n");
    s.add_change(AttributeChange::Intensity(Intensity::Bold));
    s.add_change("b");

    let mut bold = CellAttributes::default();
    bold.set_intensity(Intensity::Bold);

    assert_eq!(
        s.screen_cells(),
        [[
            Cell::new('n', CellAttributes::default()),
            Cell::new('b', bold),
            Cell::default(),
        ]]
    );
}

#[test]
fn empty_changes() {
    let s = Surface::new(4, 3);

    let empty = &[
        Change::CursorVisibility(CursorVisibility::Hidden),
        Change::ClearScreen(Default::default()),
        Change::CursorVisibility(CursorVisibility::Visible),
    ];

    let (seq, changes) = s.get_changes(0);
    assert_eq!(seq, 0);
    assert_eq!(empty, &*changes);

    // Using an invalid sequence number should get us the full
    // repaint also.
    let (seq, changes) = s.get_changes(1);
    assert_eq!(seq, 0);
    assert_eq!(empty, &*changes);
}

#[test]
fn add_changes_empty() {
    let mut s = Surface::new(2, 2);
    let last_seq = s.add_change("foo");
    assert_eq!(0, last_seq);
    assert_eq!(last_seq, s.add_changes(vec![]));
    assert_eq!(last_seq + 1, s.add_changes(vec![Change::Text("a".into())]));
}

#[test]
fn resize_delta_flush() {
    let mut s = Surface::new(4, 3);
    s.add_change("a");
    let (seq, _) = s.get_changes(0);
    s.resize(2, 2);

    let full = &[
        Change::CursorVisibility(CursorVisibility::Hidden),
        Change::ClearScreen(Default::default()),
        Change::Text("a".to_string()),
        Change::CursorPosition {
            x: Position::Absolute(1),
            y: Position::Absolute(0),
        },
        Change::CursorVisibility(CursorVisibility::Visible),
    ];

    let (_seq, changes) = s.get_changes(seq);
    // The resize causes get_changes to return a full repaint
    assert_eq!(full, &*changes);
}

#[test]
fn dont_lose_first_char_on_attr_change() {
    let mut s = Surface::new(2, 2);
    s.add_change(Change::Attribute(AttributeChange::Foreground(
        AnsiColor::Maroon.into(),
    )));
    s.add_change("ab");
    let (_seq, changes) = s.get_changes(0);
    assert_eq!(
        &[
            Change::CursorVisibility(CursorVisibility::Hidden),
            Change::ClearScreen(Default::default()),
            Change::AllAttributes(
                CellAttributes::default()
                    .set_foreground(AnsiColor::Maroon)
                    .clone()
            ),
            Change::Text("ab".into()),
            Change::CursorPosition {
                x: Position::Absolute(2),
                y: Position::Absolute(0),
            },
            Change::CursorVisibility(CursorVisibility::Visible),
        ],
        &*changes
    );
}

#[test]
fn resize_cursor_position() {
    let mut s = Surface::new(4, 4);

    s.add_change(" a");
    s.add_change(Change::CursorPosition {
        x: Position::Absolute(3),
        y: Position::Absolute(3),
    });

    assert_eq!(s.xpos, 3);
    assert_eq!(s.ypos, 3);
    s.resize(2, 2);
    assert_eq!(s.xpos, 1);
    assert_eq!(s.ypos, 1);

    let full = &[
        Change::CursorVisibility(CursorVisibility::Hidden),
        Change::ClearScreen(Default::default()),
        Change::Text(" a".to_string()),
        Change::CursorPosition {
            x: Position::Absolute(1),
            y: Position::Absolute(1),
        },
        Change::CursorVisibility(CursorVisibility::Visible),
    ];

    let (_seq, changes) = s.get_changes(0);
    assert_eq!(full, &*changes);
}

#[test]
fn delta_change() {
    let mut s = Surface::new(4, 3);
    // flushing nothing should be a NOP
    s.flush_changes_older_than(0);

    // check that using an invalid index doesn't panic
    s.flush_changes_older_than(1);

    let initial = &[
        Change::CursorVisibility(CursorVisibility::Hidden),
        Change::ClearScreen(Default::default()),
        Change::Text("a".to_string()),
        Change::CursorPosition {
            x: Position::Absolute(1),
            y: Position::Absolute(0),
        },
        Change::CursorVisibility(CursorVisibility::Visible),
    ];

    let seq_pos = {
        let next_seq = s.add_change("a");
        let (seq, changes) = s.get_changes(0);
        assert_eq!(seq, next_seq + 1);
        assert_eq!(initial, &*changes);
        seq
    };

    let seq_pos = {
        let next_seq = s.add_change("b");
        let (seq, changes) = s.get_changes(seq_pos);
        assert_eq!(seq, next_seq + 1);
        assert_eq!(&[Change::Text("b".to_string())], &*changes);
        seq
    };

    // prep some deltas for the loop to test below
    {
        s.add_change(Change::Attribute(AttributeChange::Intensity(
            Intensity::Bold,
        )));
        s.add_change("c");
        s.add_change(Change::Attribute(AttributeChange::Intensity(
            Intensity::Normal,
        )));
        s.add_change("d");
    }

    // Do this three times to ennsure that the behavior is consistent
    // across multiple flush calls
    for _ in 0..3 {
        {
            let (_seq, changes) = s.get_changes(seq_pos);

            assert_eq!(
                &[
                    Change::Attribute(AttributeChange::Intensity(Intensity::Bold)),
                    Change::Text("c".to_string()),
                    Change::Attribute(AttributeChange::Intensity(Intensity::Normal)),
                    Change::Text("d".to_string()),
                ],
                &*changes
            );
        }

        // Flush the changes so that the next iteration is run on a pruned
        // set of changes.  It should not change the outcome of the body
        // of the loop.
        s.flush_changes_older_than(seq_pos);
    }
}
