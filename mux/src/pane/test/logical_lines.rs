#[test]
fn logical_lines() {
    let text = "Hello there this is a long line.\nlogical line two\nanother long line here\nlogical line four\nlogical line five\ncap it off with another long line";
    let width = 20;
    let physical_lines = physical_lines_from_text(text, width);

    fn text_from_lines(lines: &[Line]) -> Vec<Cow<'_, str>> {
        lines.iter().map(|l| l.as_str()).collect::<Vec<_>>()
    }

    let line_text = text_from_lines(&physical_lines);
    snapshot!(
        line_text,
        r#"
[
    "Hello there this is ",
    "a long line.",
    "logical line two",
    "another long line he",
    "re",
    "logical line four",
    "logical line five",
    "cap it off with anot",
    "her long line",
]
"#
    );

    let pane = FakePane {
        lines: Mutex::new(physical_lines),
    };

    let logical = pane.get_logical_lines(0..30);
    snapshot!(
        summarize_logical_lines(&logical),
        r#"
[
    (
        0,
        "Hello there this is a long line.",
    ),
    (
        2,
        "logical line two",
    ),
    (
        3,
        "another long line here",
    ),
    (
        5,
        "logical line four",
    ),
    (
        6,
        "logical line five",
    ),
    (
        7,
        "cap it off with another long line",
    ),
]
"#
    );

    // Now try with offset bounds
    let offset = pane.get_logical_lines(1..3);
    snapshot!(
        summarize_logical_lines(&offset),
        r#"
[
    (
        0,
        "Hello there this is a long line.",
    ),
    (
        2,
        "logical line two",
    ),
]
"#
    );

    let offset = pane.get_logical_lines(1..4);
    snapshot!(
        summarize_logical_lines(&offset),
        r#"
[
    (
        0,
        "Hello there this is a long line.",
    ),
    (
        2,
        "logical line two",
    ),
    (
        3,
        "another long line here",
    ),
]
"#
    );

    let offset = pane.get_logical_lines(1..5);
    snapshot!(
        summarize_logical_lines(&offset),
        r#"
[
    (
        0,
        "Hello there this is a long line.",
    ),
    (
        2,
        "logical line two",
    ),
    (
        3,
        "another long line here",
    ),
]
"#
    );

    let offset = pane.get_logical_lines(1..6);
    snapshot!(
        summarize_logical_lines(&offset),
        r#"
[
    (
        0,
        "Hello there this is a long line.",
    ),
    (
        2,
        "logical line two",
    ),
    (
        3,
        "another long line here",
    ),
    (
        5,
        "logical line four",
    ),
]
"#
    );

    let offset = pane.get_logical_lines(1..7);
    snapshot!(
        summarize_logical_lines(&offset),
        r#"
[
    (
        0,
        "Hello there this is a long line.",
    ),
    (
        2,
        "logical line two",
    ),
    (
        3,
        "another long line here",
    ),
    (
        5,
        "logical line four",
    ),
    (
        6,
        "logical line five",
    ),
]
"#
    );

    let offset = pane.get_logical_lines(1..8);
    snapshot!(
        summarize_logical_lines(&offset),
        r#"
[
    (
        0,
        "Hello there this is a long line.",
    ),
    (
        2,
        "logical line two",
    ),
    (
        3,
        "another long line here",
    ),
    (
        5,
        "logical line four",
    ),
    (
        6,
        "logical line five",
    ),
    (
        7,
        "cap it off with another long line",
    ),
]
"#
    );

    let line = &offset[0];
    let coords = (0..line.logical.len())
        .map(|idx| line.logical_x_to_physical_coord(idx))
        .collect::<Vec<_>>();
    snapshot!(
        coords,
        "
[
    (
        0,
        0,
    ),
    (
        0,
        1,
    ),
    (
        0,
        2,
    ),
    (
        0,
        3,
    ),
    (
        0,
        4,
    ),
    (
        0,
        5,
    ),
    (
        0,
        6,
    ),
    (
        0,
        7,
    ),
    (
        0,
        8,
    ),
    (
        0,
        9,
    ),
    (
        0,
        10,
    ),
    (
        0,
        11,
    ),
    (
        0,
        12,
    ),
    (
        0,
        13,
    ),
    (
        0,
        14,
    ),
    (
        0,
        15,
    ),
    (
        0,
        16,
    ),
    (
        0,
        17,
    ),
    (
        0,
        18,
    ),
    (
        0,
        19,
    ),
    (
        1,
        0,
    ),
    (
        1,
        1,
    ),
    (
        1,
        2,
    ),
    (
        1,
        3,
    ),
    (
        1,
        4,
    ),
    (
        1,
        5,
    ),
    (
        1,
        6,
    ),
    (
        1,
        7,
    ),
    (
        1,
        8,
    ),
    (
        1,
        9,
    ),
    (
        1,
        10,
    ),
    (
        1,
        11,
    ),
]
"
    );
}
