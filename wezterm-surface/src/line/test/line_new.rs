#[test]
fn cluster_line_new() {
    let mut line = Line::new(1);
    line.set_cell(
        0,
        Cell::new_grapheme("h", CellAttributes::default(), None),
        1,
    );
    line.set_cell(
        1,
        Cell::new_grapheme("e", CellAttributes::default(), None),
        2,
    );
    line.set_cell(2, Cell::new_grapheme("l", bold(), None), 3);
    line.set_cell(
        3,
        Cell::new_grapheme("l", CellAttributes::default(), None),
        4,
    );
    line.set_cell(
        4,
        Cell::new_grapheme("o", CellAttributes::default(), None),
        5,
    );
    k9::snapshot!(
        line,
        r#"
Line {
    cells: C(
        ClusteredLine {
            text: "hello",
            is_double_wide: None,
            clusters: [
                Cluster {
                    cell_width: 2,
                    attrs: CellAttributes {
                        attributes: 0,
                        intensity: Normal,
                        underline: None,
                        blink: None,
                        italic: false,
                        reverse: false,
                        strikethrough: false,
                        invisible: false,
                        wrapped: false,
                        overline: false,
                        semantic_type: Output,
                        foreground: Default,
                        background: Default,
                        fat: None,
                    },
                },
                Cluster {
                    cell_width: 1,
                    attrs: CellAttributes {
                        attributes: 1,
                        intensity: Bold,
                        underline: None,
                        blink: None,
                        italic: false,
                        reverse: false,
                        strikethrough: false,
                        invisible: false,
                        wrapped: false,
                        overline: false,
                        semantic_type: Output,
                        foreground: Default,
                        background: Default,
                        fat: None,
                    },
                },
                Cluster {
                    cell_width: 2,
                    attrs: CellAttributes {
                        attributes: 0,
                        intensity: Normal,
                        underline: None,
                        blink: None,
                        italic: false,
                        reverse: false,
                        strikethrough: false,
                        invisible: false,
                        wrapped: false,
                        overline: false,
                        semantic_type: Output,
                        foreground: Default,
                        background: Default,
                        fat: None,
                    },
                },
            ],
            len: 5,
            last_cell_width: Some(
                1,
            ),
        },
    ),
    zones: [],
    seqno: 5,
    bits: LineBits(
        0x0,
    ),
    appdata: Mutex {
        data: None,
        poisoned: false,
        ..
    },
}
"#
    );
}
