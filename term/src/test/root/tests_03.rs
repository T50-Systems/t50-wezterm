#[test]
fn test_ri() {
    let mut term = TestTerm::new(3, 1, 10);
    term.print("1\n\u{8d}\n");
    assert_all_contents(&term, file!(), line!(), &["1", "", ""]);
}

#[test]
fn test_scroll_margins() {
    let mut term = TestTerm::new(3, 1, 10);
    term.print("1\n2\n3\n4\n");
    assert_all_contents(&term, file!(), line!(), &["1", "2", "3", "4", ""]);

    let margins = CSI::Cursor(wezterm_escape_parser::csi::Cursor::SetTopAndBottomMargins {
        top: OneBased::new(1),
        bottom: OneBased::new(2),
    });
    term.print(format!("{}", margins));

    term.print("z\n");
    assert_all_contents(&term, file!(), line!(), &["1", "2", "z", "4", ""]);

    term.print("a\n");
    assert_all_contents(&term, file!(), line!(), &["1", "2", "z", "a", "", ""]);

    term.cup(0, 1);
    term.print("W\n");
    assert_all_contents(&term, file!(), line!(), &["1", "2", "z", "a", "W", "", ""]);
}

#[test]
fn test_emoji_with_modifier() {
    let waving_hand = "\u{1f44b}";
    let waving_hand_dark_tone = "\u{1f44b}\u{1f3ff}";

    let mut term = TestTerm::new(3, 5, 0);
    term.print(waving_hand);
    term.print("\r\n");
    term.print(waving_hand_dark_tone);

    assert_all_contents(
        &term,
        file!(),
        line!(),
        &[waving_hand, waving_hand_dark_tone, ""],
    );
}

#[test]
fn test_1573() {
    let sequence = "\u{1112}\u{1161}\u{11ab}";

    let mut term = TestTerm::new(2, 5, 0);
    term.print(sequence);
    term.print("\r\n");

    assert_all_contents(&term, file!(), line!(), &[sequence, ""]);

    use unicode_normalization::UnicodeNormalization;
    let recomposed: String = sequence.nfc().collect();
    assert_eq!(recomposed, "\u{d55c}");

    use finl_unicode::grapheme_clusters::Graphemes;
    let graphemes: Vec<_> = Graphemes::new(sequence).collect();
    assert_eq!(graphemes, vec![sequence]);
}

#[test]
fn test_region_scroll() {
    let mut term = TestTerm::new(5, 1, 10);
    term.print("1\n2\n3\n4\n5");

    // Test scroll region that doesn't start on first row, scrollback not used
    term.set_scroll_region(1, 2);
    term.cup(0, 2);
    let seqno = term.current_seqno();
    term.print("\na");
    assert_all_contents(&term, file!(), line!(), &["1", "3", "a", "4", "5"]);
    term.assert_dirty_lines(seqno, &[1, 2], None);
    assert_eq!(term.screen().visible_row_to_stable_row(0), 0);
    assert_eq!(term.screen().visible_row_to_stable_row(4), 4);

    // Scroll region starting on first row, but is smaller than screen (see #6099)
    //  Scrollback will be used, which means lines below the scroll region
    //  have their stable index invalidated, and so need to be marked dirty
    term.set_scroll_region(0, 1);
    term.cup(0, 1);
    let seqno = term.current_seqno();
    term.print("\nb");
    assert_all_contents(&term, file!(), line!(), &["1", "3", "b", "a", "4", "5"]);
    term.assert_dirty_lines(seqno, &[2, 3, 4, 5], None);
    assert_eq!(term.screen().visible_row_to_stable_row(0), 1);
    assert_eq!(term.screen().visible_row_to_stable_row(4), 5);

    // Test deletion of more lines than exist in scroll region
    term.cup(0, 1);
    let seqno = term.current_seqno();
    term.delete_lines(3);
    assert_all_contents(&term, file!(), line!(), &["1", "3", "", "a", "4", "5"]);
    term.assert_dirty_lines(seqno, &[2], None);
    assert_eq!(term.screen().visible_row_to_stable_row(0), 1);
    assert_eq!(term.screen().visible_row_to_stable_row(4), 5);

    // Return to normal, entire-screen scrolling, optimal number of lines marked dirty
    term.set_scroll_region(0, 4);
    term.cup(0, 4);
    let seqno = term.current_seqno();
    term.print("\nX");
    assert_all_contents(&term, file!(), line!(), &["1", "3", "", "a", "4", "5", "X"]);
    term.assert_dirty_lines(seqno, &[6], None);
    assert_eq!(term.screen().visible_row_to_stable_row(4), 6);
}

#[test]
fn test_alt_screen_region_scroll() {
    // Test that scrollback is never used, and lines below the scroll region
    //  aren't made dirty or invalid. Only the scroll region is marked dirty.
    let mut term = TestTerm::new(5, 1, 10);
    term.print("M\no\nn\nk\ne\ny");

    // Enter alternate-screen mode, saving current state
    term.set_mode("?1049", true);
    term.print("1\n2\n3\n4\n5");

    // Test scroll region that doesn't start on first row
    term.set_scroll_region(1, 2);
    term.cup(0, 2);
    let seqno = term.current_seqno();
    term.print("\na");
    assert_all_contents(&term, file!(), line!(), &["1", "3", "a", "4", "5"]);
    term.assert_dirty_lines(seqno, &[1, 2], None);
    assert_eq!(term.screen().visible_row_to_stable_row(4), 4);

    // Test scroll region that starts on first row, still no scrollback
    term.set_scroll_region(0, 1);
    term.cup(0, 1);
    let seqno = term.current_seqno();
    term.print("\nb");
    assert_all_contents(&term, file!(), line!(), &["3", "b", "a", "4", "5"]);
    term.assert_dirty_lines(seqno, &[0, 1], None);
    assert_eq!(term.screen().visible_row_to_stable_row(4), 4);

    // Return to normal, entire-screen scrolling
    //  Not optimal, the entire screen is marked dirty for every line scrolled
    term.set_scroll_region(0, 4);
    term.cup(0, 4);
    let seqno = term.current_seqno();
    term.print("\nX");
    assert_all_contents(&term, file!(), line!(), &["b", "a", "4", "5", "X"]);
    term.assert_dirty_lines(seqno, &[0, 1, 2, 3, 4], None);
    assert_eq!(term.screen().visible_row_to_stable_row(4), 4);

    // Leave alternate-mode and ensure screen is restored, with all lines marked dirty
    let seqno = term.current_seqno();
    term.set_mode("?1049", false);
    assert_all_contents(&term, file!(), line!(), &["M", "o", "n", "k", "e", "y"]);
    term.assert_dirty_lines(seqno, &[0, 1, 2, 3, 4], None);
    assert_eq!(term.screen().visible_row_to_stable_row(0), 1);
}

#[test]
fn test_region_scrollback_limit() {
    // Ensure scrollback is truncated properly, when it reaches the line limit
    let mut term = TestTerm::new(4, 1, 2);
    term.print("1\n2\n3\n4");
    term.set_scroll_region(0, 1);
    term.cup(0, 1);

    let seqno = term.current_seqno();
    term.print("A\nB\nC\nD");
    assert_all_contents(&term, file!(), line!(), &["A", "B", "C", "D", "3", "4"]);
    term.assert_dirty_lines(seqno, &[0, 1, 2, 3, 4, 5], None);
    assert_eq!(term.screen().visible_row_to_stable_row(4), 7);
}

#[test]
fn test_hyperlinks() {
    let mut term = TestTerm::new(3, 5, 0);
    let link = Arc::new(Hyperlink::new("http://example.com"));
    term.hyperlink(&link);
    term.print("hello");
    term.hyperlink_off();

    let mut linked = CellAttributes::default();
    linked.set_hyperlink(Some(Arc::clone(&link)));

    assert_lines_equal(
        file!(),
        line!(),
        &term.screen().visible_lines(),
        &[
            Line::from_text("hello", &linked, SEQ_ZERO, None),
            "".into(),
            "".into(),
        ],
        Compare::TEXT | Compare::ATTRS,
    );

    term.hyperlink(&link);
    term.print("he");
    // Resetting pen should not reset the link
    term.print("\x1b[m");
    term.print("y!!");

    assert_lines_equal(
        file!(),
        line!(),
        &term.screen().visible_lines(),
        &[
            Line::from_text_with_wrapped_last_col("hello", &linked, SEQ_ZERO),
            Line::from_text("hey!!", &linked, SEQ_ZERO, None),
            "".into(),
        ],
        Compare::TEXT | Compare::ATTRS,
    );

    let otherlink = Arc::new(Hyperlink::new_with_id("http://example.com/other", "w00t"));

    // Switching link and turning it off
    term.hyperlink(&otherlink);
    term.print("wo");
    // soft reset also disables hyperlink attribute
    term.soft_reset();
    term.print("00t");

    let mut partial_line = Line::from_text("wo00t", &CellAttributes::default(), SEQ_ZERO, None);
    partial_line.set_cell(
        0,
        Cell::new(
            'w',
            CellAttributes::default()
                .set_hyperlink(Some(Arc::clone(&otherlink)))
                .clone(),
        ),
        SEQ_ZERO,
    );
    partial_line.set_cell(
        1,
        Cell::new(
            'o',
            CellAttributes::default()
                .set_hyperlink(Some(Arc::clone(&otherlink)))
                .clone(),
        ),
        SEQ_ZERO,
    );

    assert_lines_equal(
        file!(),
        line!(),
        &term.screen().visible_lines(),
        &[
            Line::from_text_with_wrapped_last_col("hello", &linked, SEQ_ZERO),
            Line::from_text_with_wrapped_last_col("hey!!", &linked, SEQ_ZERO),
            partial_line,
        ],
        Compare::TEXT | Compare::ATTRS,
    );
}
