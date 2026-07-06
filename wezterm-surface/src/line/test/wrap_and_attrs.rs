#[test]
fn cluster_wrap_last() {
    let mut line: Line = "hello".into();
    line.compress_for_scrollback();
    line.set_last_cell_was_wrapped(true, 1);
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
                    cell_width: 4,
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
                        attributes: 2048,
                        intensity: Normal,
                        underline: None,
                        blink: None,
                        italic: false,
                        reverse: false,
                        strikethrough: false,
                        invisible: false,
                        wrapped: true,
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
    seqno: 1,
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

fn bold() -> CellAttributes {
    use wezterm_cell::Intensity;
    let mut attr = CellAttributes::default();
    attr.set_intensity(Intensity::Bold);
    attr
}

#[test]
fn cluster_representation_attributes() {
    let line = Line::from_cells(
        vec![
            Cell::new_grapheme("a", CellAttributes::default(), None),
            Cell::new_grapheme("b", bold(), None),
            Cell::new_grapheme("c", CellAttributes::default(), None),
            Cell::new_grapheme("d", bold(), None),
        ],
        SEQ_ZERO,
    );

    let mut compressed = line.clone();
    compressed.compress_for_scrollback();
    k9::snapshot!(
        &compressed.cells,
        r#"
C(
    ClusteredLine {
        text: "abcd",
        is_double_wide: None,
        clusters: [
            Cluster {
                cell_width: 1,
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
                cell_width: 1,
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
        ],
        len: 4,
        last_cell_width: Some(
            1,
        ),
    },
)
"#
    );
    compressed.coerce_vec_storage();
    assert_eq!(line, compressed);
}
