fn is_double_click_word(s: &str) -> bool {
    match s.chars().count() {
        1 => !" \t\n{[}]()\"'`".contains(s),
        0 => false,
        _ => true,
    }
}

#[test]
fn double_click() {
    let attr = Default::default();
    let logical = LogicalLine {
        physical_lines: vec![
            Line::from_text("hello", &attr, SEQ_ZERO, None),
            Line::from_text("yo", &attr, SEQ_ZERO, None),
        ],
        logical: Line::from_text("helloyo", &attr, SEQ_ZERO, None),
        first_row: 0,
    };

    assert_eq!(logical.xy_to_logical_x(2, -1), 0);
    assert_eq!(logical.xy_to_logical_x(20, 1), 25);

    let start_idx = logical.xy_to_logical_x(2, 1);

    use termwiz::surface::line::DoubleClickRange;

    assert_eq!(start_idx, 7);
    match logical
        .logical
        .compute_double_click_range(start_idx, is_double_click_word)
    {
        DoubleClickRange::Range(click_range) => {
            assert_eq!(click_range, 7..7);
        }
        _ => unreachable!(),
    }
}
