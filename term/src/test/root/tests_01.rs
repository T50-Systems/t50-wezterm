#[test]
fn test_semantic_1539() {
    use wezterm_escape_parser::osc::FinalTermSemanticPrompt;
    let mut term = TestTerm::new(5, 10, 0);
    term.print(format!(
        "{}prompt\r\nwoot",
        OperatingSystemCommand::FinalTermSemanticPrompt(
            FinalTermSemanticPrompt::MarkEndOfPromptAndStartOfInputUntilEndOfLine
        )
    ));

    assert_visible_contents(&term, file!(), line!(), &["prompt", "woot", "", "", ""]);

    k9::snapshot!(
        term.get_semantic_zones().unwrap(),
        "
[
    SemanticZone {
        start_y: 0,
        start_x: 0,
        end_y: 0,
        end_x: 5,
        semantic_type: Input,
    },
    SemanticZone {
        start_y: 1,
        start_x: 0,
        end_y: 1,
        end_x: 3,
        semantic_type: Output,
    },
]
"
    );
}

#[test]
fn test_semantic() {
    use wezterm_escape_parser::osc::FinalTermSemanticPrompt;
    let mut term = TestTerm::new(5, 10, 0);
    term.print("hello");
    term.print(format!(
        "{}",
        OperatingSystemCommand::FinalTermSemanticPrompt(FinalTermSemanticPrompt::FreshLine)
    ));
    term.print("there");

    assert_visible_contents(&term, file!(), line!(), &["hello", "there", "", "", ""]);

    term.cup(0, 2);
    term.print(format!(
        "{}",
        OperatingSystemCommand::FinalTermSemanticPrompt(FinalTermSemanticPrompt::FreshLine)
    ));
    term.print("three");
    assert_visible_contents(
        &term,
        file!(),
        line!(),
        &["hello", "there", "three", "", ""],
    );

    k9::snapshot!(
        term.get_semantic_zones().unwrap(),
        "
[
    SemanticZone {
        start_y: 0,
        start_x: 0,
        end_y: 2,
        end_x: 4,
        semantic_type: Output,
    },
]
"
    );

    term.print(format!(
        "{}",
        OperatingSystemCommand::FinalTermSemanticPrompt(
            FinalTermSemanticPrompt::FreshLineAndStartPrompt {
                aid: None,
                cl: None
            }
        )
    ));
    term.print("> ");
    term.print(format!(
        "{}",
        OperatingSystemCommand::FinalTermSemanticPrompt(
            FinalTermSemanticPrompt::MarkEndOfPromptAndStartOfInputUntilNextMarker
        )
    ));
    term.print("ls -l\r\n");
    term.print(format!(
        "{}",
        OperatingSystemCommand::FinalTermSemanticPrompt(
            FinalTermSemanticPrompt::MarkEndOfInputAndStartOfOutput { aid: None }
        )
    ));
    term.print("some file");

    let output = CellAttributes::default();
    let mut input = CellAttributes::default();
    input.set_semantic_type(SemanticType::Input);

    let mut prompt_line = Line::from_text("> ls -l", &output, SEQ_ZERO, None);
    for i in 0..2 {
        prompt_line.cells_mut()[i]
            .attrs_mut()
            .set_semantic_type(SemanticType::Prompt);
    }
    for i in 2..7 {
        prompt_line.cells_mut()[i]
            .attrs_mut()
            .set_semantic_type(SemanticType::Input);
    }

    k9::snapshot!(
        term.get_semantic_zones().unwrap(),
        "
[
    SemanticZone {
        start_y: 0,
        start_x: 0,
        end_y: 2,
        end_x: 4,
        semantic_type: Output,
    },
    SemanticZone {
        start_y: 3,
        start_x: 0,
        end_y: 3,
        end_x: 1,
        semantic_type: Prompt,
    },
    SemanticZone {
        start_y: 3,
        start_x: 2,
        end_y: 3,
        end_x: 6,
        semantic_type: Input,
    },
    SemanticZone {
        start_y: 4,
        start_x: 0,
        end_y: 4,
        end_x: 8,
        semantic_type: Output,
    },
]
"
    );

    assert_lines_equal(
        file!(),
        line!(),
        &term.screen().visible_lines(),
        &[
            Line::from_text("hello", &output, SEQ_ZERO, None),
            Line::from_text("there", &output, SEQ_ZERO, None),
            Line::from_text("three", &output, SEQ_ZERO, None),
            prompt_line,
            Line::from_text("some file", &output, SEQ_ZERO, None),
        ],
        Compare::TEXT | Compare::ATTRS,
    );
}

#[test]
fn issue_1161() {
    let mut term = TestTerm::new(1, 5, 0);
    term.print("x\u{3000}x");
    assert_visible_contents(
        &term,
        file!(),
        line!(),
        &[
            // U+3000 is ideographic space, a double-width space
            "x\u{3000}x",
        ],
    );
}

#[test]
fn basic_output() {
    let mut term = TestTerm::new(5, 10, 0);

    term.cup(1, 1);

    term.set_auto_wrap(false);
    term.print("hello, world!");
    assert_visible_contents(&term, file!(), line!(), &["", " hello, w!", "", "", ""]);

    term.set_auto_wrap(true);
    term.erase_in_display(EraseInDisplay::EraseToStartOfDisplay);
    term.cup(1, 1);
    term.print("hello, world!");
    assert_visible_contents(&term, file!(), line!(), &["", " hello, wo", "rld!", "", ""]);

    term.erase_in_display(EraseInDisplay::EraseToStartOfDisplay);
    assert_visible_contents(
        &term,
        file!(),
        line!(),
        &["", "          ", "     ", "", ""],
    );

    term.cup(0, 2);
    term.print("woot");
    term.cup(2, 2);
    term.erase_in_line(EraseInLine::EraseToEndOfLine);
    assert_visible_contents(&term, file!(), line!(), &["", "          ", "wo", "", ""]);

    term.erase_in_line(EraseInLine::EraseToStartOfLine);
    assert_visible_contents(&term, file!(), line!(), &["", "          ", "   ", "", ""]);
}

/// Ensure that we dirty lines as the cursor is moved around, otherwise
/// the renderer won't draw the cursor in the right place
#[test]
fn cursor_movement_damage() {
    let mut term = TestTerm::new(2, 3, 0);

    let seqno = term.current_seqno();
    term.print("fooo.");
    assert_visible_contents(&term, file!(), line!(), &["foo", "o."]);
    term.assert_cursor_pos(2, 1, None, None);
    term.assert_dirty_lines(seqno, &[0, 1], None);

    term.cup(0, 1);

    let seqno = term.current_seqno();
    term.print("\x08");
    term.assert_cursor_pos(0, 1, Some("BS doesn't change the line"), Some(seqno));
    // Since we didn't move, the line isn't dirty
    term.assert_dirty_lines(seqno, &[], None);

    let seqno = term.current_seqno();
    term.cup(0, 0);
    term.assert_dirty_lines(
        seqno,
        &[],
        Some("cursor movement no longer dirties old and new lines"),
    );
    term.assert_cursor_pos(0, 0, None, None);
}
const NUM_COLS: usize = 3;

#[test]
fn scroll_up_within_left_and_right_margins() {
    let ones = "1".repeat(NUM_COLS);
    let twos = "2".repeat(NUM_COLS);
    let threes = "3".repeat(NUM_COLS);
    let fours = "4".repeat(NUM_COLS + 2);
    let fives = "5".repeat(NUM_COLS);

    let mut term = TestTerm::new(5, NUM_COLS + 2, 0);

    term.print(&ones);
    term.print("\r\n");
    term.print(&twos);
    term.print("\r\n");
    term.print(&threes);
    term.print("\r\n");
    term.print(&fours);
    term.print("\r\n");
    term.print(&fives);

    assert_visible_contents(
        &term,
        file!(),
        line!(),
        &["111", "222", "333", "44444", "555"],
    );

    term.set_mode("?69", true); // allow left/right margins to be set
    term.set_left_and_right_margins(1, NUM_COLS + 1);
    term.set_scroll_region(2, 4);
    term.cup(1, 4);
    term.print("\n");
    assert_visible_contents(
        &term,
        file!(),
        line!(),
        &[
            "111",
            "222",
            &format!("3{}", "4".repeat(NUM_COLS + 1)),
            &format!("4{}", "5".repeat(NUM_COLS - 1)),
            &format!("5{}", " ".repeat(NUM_COLS - 1)),
        ],
    );
}

#[test]
fn scroll_down_within_left_and_right_margins() {
    let ones = "1".repeat(NUM_COLS);
    let twos = "2".repeat(NUM_COLS);
    let threes = "3".repeat(NUM_COLS);
    let fours = "4".repeat(NUM_COLS + 2);
    let fives = "5".repeat(NUM_COLS);

    let mut term = TestTerm::new(5, NUM_COLS + 2, 0);

    term.print(&ones);
    term.print("\r\n");
    term.print(&twos);
    term.print("\r\n");
    term.print(&threes);
    term.print("\r\n");
    term.print(&fours);
    term.print("\r\n");
    term.print(&fives);

    assert_visible_contents(
        &term,
        file!(),
        line!(),
        &["111", "222", "333", "44444", "555"],
    );

    term.set_mode("?69", true); // allow left/right margins to be set
    term.set_left_and_right_margins(1, NUM_COLS + 1);
    term.set_scroll_region(2, 5);
    term.cup(1, 2);

    // IL: Insert Line
    term.print(CSI);
    term.print("L");

    assert_visible_contents(
        &term,
        file!(),
        line!(),
        &[
            "111",
            "222",
            &format!("3{}", " ".repeat(NUM_COLS - 1)),
            &format!("4{}", "3".repeat(NUM_COLS - 1)),
            &format!("5{}", "4".repeat(NUM_COLS + 1)),
        ],
    );
}

/// Replicates a bug I initially found via:
/// $ vim
/// :help
/// PageDown
#[test]
fn test_delete_lines() {
    let mut term = TestTerm::new(5, 3, 0);

    let seqno = term.current_seqno();
    term.print("111\r\n222\r\n333\r\n444\r\n555");
    assert_visible_contents(
        &term,
        file!(),
        line!(),
        &["111", "222", "333", "444", "555"],
    );
    term.assert_dirty_lines(seqno, &[0, 1, 2, 3, 4], None);
    term.cup(0, 1);

    let seqno = term.current_seqno();
    term.assert_dirty_lines(seqno, &[], None);
    term.delete_lines(2);
    assert_visible_contents(&term, file!(), line!(), &["111", "444", "555", "", ""]);
    term.assert_dirty_lines(seqno, &[1, 2, 3, 4], None);

    term.cup(0, 3);
    term.print("aaa\r\nbbb");
    term.cup(0, 1);

    let seqno = term.current_seqno();
    assert_visible_contents(
        &term,
        file!(),
        line!(),
        &["111", "444", "555", "aaa", "bbb"],
    );

    // test with a scroll region smaller than the screen
    term.set_scroll_region(1, 3);
    term.cup(0, 1);
    print_all_lines(&term);
    term.delete_lines(2);

    assert_visible_contents(&term, file!(), line!(), &["111", "aaa", "", "", "bbb"]);
    term.assert_dirty_lines(seqno, &[1, 2, 3], None);

    // expand the scroll region to fill the screen
    term.set_scroll_region(0, 4);

    let seqno = term.current_seqno();
    print_all_lines(&term);
    term.delete_lines(1);

    assert_visible_contents(&term, file!(), line!(), &["aaa", "", "", "bbb", ""]);
    term.assert_dirty_lines(seqno, &[4], None);
}
