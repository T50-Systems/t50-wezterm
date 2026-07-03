fn empty_render() {
    let mut out = FakeTerm::new(xterm_terminfo());
    out.render(&[]).unwrap();
    assert_eq!("", String::from_utf8(out.write.buf).unwrap());
    assert_eq!(out.renderer.current_attr, CellAttributes::default());
}

#[test]
fn basic_text() {
    let mut out = FakeTerm::new(xterm_terminfo());
    out.render(&[Change::Text("foo".into())]).unwrap();
    assert_eq!("foo", String::from_utf8(out.write.buf).unwrap());
    assert_eq!(out.renderer.current_attr, CellAttributes::default());
}

#[test]
fn bold_text() {
    let mut out = FakeTerm::new(xterm_terminfo());
    out.render(&[
        Change::Text("not ".into()),
        Change::Attribute(AttributeChange::Intensity(Intensity::Bold)),
        Change::Text("foo".into()),
    ])
    .unwrap();

    let result = out.parse();
    assert_eq!(
        result,
        vec![
            Action::Print('n'),
            Action::Print('o'),
            Action::Print('t'),
            Action::Print(' '),
            Action::Esc(Esc::Code(EscCode::AsciiCharacterSetG0)),
            Action::CSI(CSI::Sgr(Sgr::Reset)),
            Action::CSI(CSI::Sgr(Sgr::Intensity(Intensity::Bold))),
            Action::Print('f'),
            Action::Print('o'),
            Action::Print('o'),
        ]
    );

    assert_eq!(
        out.renderer.current_attr,
        CellAttributes::default()
            .set_intensity(Intensity::Bold)
            .clone()
    );
}

#[test]
// Sanity that force_terminfo_render_to_use_ansi_sgr does something.
fn bold_text_force_ansi_sgr() {
    let mut out = FakeTerm::new(xterm_terminfo_with_hints(
        ProbeHints::default().force_terminfo_render_to_use_ansi_sgr(Some(true)),
    ));
    out.render(&[
        Change::Text("not ".into()),
        AttributeChange::Intensity(Intensity::Bold).into(),
        Change::Text("foo".into()),
    ])
    .unwrap();

    let result = out.parse();
    assert_eq!(
        result,
        vec![
            // Same as bold_text() above, but without the "(B" from srg/sgr0.
            Action::Print('n'),
            Action::Print('o'),
            Action::Print('t'),
            Action::Print(' '),
            Action::CSI(CSI::Sgr(Sgr::Reset)),
            Action::CSI(CSI::Sgr(Sgr::Intensity(Intensity::Bold))),
            Action::Print('f'),
            Action::Print('o'),
            Action::Print('o'),
        ],
    );
}

#[test]
fn clear_screen() {
    let mut out = FakeTerm::new_with_size(xterm_terminfo(), 4, 3);
    out.render(&[Change::ClearScreen(ColorAttribute::default())])
        .unwrap();

    let result = out.parse();
    assert_eq!(
        result,
        vec![
            Action::CSI(CSI::Cursor(Cursor::Position {
                line: OneBased::new(1),
                col: OneBased::new(1)
            })),
            Action::CSI(CSI::Edit(Edit::EraseInDisplay(
                EraseInDisplay::EraseDisplay,
            ))),
        ]
    );

    assert_eq!(out.renderer.current_attr, CellAttributes::default());
}

#[test]
fn clear_screen_bce() {
    let mut out = FakeTerm::new_with_size(xterm_terminfo(), 4, 3);
    out.render(&[Change::ClearScreen(AnsiColor::Maroon.into())])
        .unwrap();

    let result = out.parse();
    assert_eq!(
        result,
        vec![
            Action::CSI(CSI::Sgr(Sgr::Background(AnsiColor::Maroon.into()))),
            Action::CSI(CSI::Cursor(Cursor::Position {
                line: OneBased::new(1),
                col: OneBased::new(1)
            })),
            Action::CSI(CSI::Edit(Edit::EraseInDisplay(
                EraseInDisplay::EraseDisplay,
            ))),
        ]
    );

    assert_eq!(
        out.renderer.current_attr,
        CellAttributes::default()
            .set_background(AnsiColor::Maroon)
            .clone()
    );
}

#[test]
fn clear_screen_no_terminfo() {
    let mut out = FakeTerm::new_with_size(no_terminfo_all_enabled(), 4, 3);
    out.render(&[Change::ClearScreen(ColorAttribute::default())])
        .unwrap();

    let result = out.parse();
    assert_eq!(
        result,
        vec![
            Action::CSI(CSI::Cursor(Cursor::Position {
                line: OneBased::new(1),
                col: OneBased::new(1)
            })),
            Action::CSI(CSI::Edit(Edit::EraseInDisplay(
                EraseInDisplay::EraseDisplay,
            ))),
        ]
    );

    assert_eq!(out.renderer.current_attr, CellAttributes::default());
}

#[test]
fn clear_screen_bce_no_terminfo() {
    let mut out = FakeTerm::new_with_size(no_terminfo_all_enabled(), 4, 3);
    out.render(&[Change::ClearScreen(AnsiColor::Maroon.into())])
        .unwrap();

    let result = out.parse();
    assert_eq!(
        result,
        vec![
            Action::CSI(CSI::Sgr(Sgr::Background(AnsiColor::Maroon.into()))),
            Action::CSI(CSI::Cursor(Cursor::Position {
                line: OneBased::new(1),
                col: OneBased::new(1)
            })),
            // bce is not known to be available, so we emit a bunch of spaces.
            // TODO: could we use ECMA-48 REP for this?
            Action::Print(' '),
            Action::Print(' '),
            Action::Print(' '),
            Action::Print(' '),
            Action::Print(' '),
            Action::Print(' '),
            Action::Print(' '),
            Action::Print(' '),
            Action::Print(' '),
            Action::Print(' '),
            Action::Print(' '),
            Action::Print(' '),
        ]
    );

    assert_eq!(
        out.renderer.current_attr,
        CellAttributes::default()
            .set_background(AnsiColor::Maroon)
            .clone()
    );
}

#[test]
fn bold_text_no_terminfo() {
    let mut out = FakeTerm::new(no_terminfo_all_enabled());
    out.render(&[
        Change::Text("not ".into()),
        Change::Attribute(AttributeChange::Intensity(Intensity::Bold)),
        Change::Text("foo".into()),
    ])
    .unwrap();

    let result = out.parse();
    assert_eq!(
        result,
        vec![
            Action::Print('n'),
            Action::Print('o'),
            Action::Print('t'),
            Action::Print(' '),
            Action::CSI(CSI::Sgr(Sgr::Reset)),
            Action::CSI(CSI::Sgr(Sgr::Intensity(Intensity::Bold))),
            Action::Print('f'),
            Action::Print('o'),
            Action::Print('o'),
        ]
    );

    assert_eq!(
        out.renderer.current_attr,
        CellAttributes::default()
            .set_intensity(Intensity::Bold)
            .clone()
    );
}

#[test]
fn red_bold_text() {
    let mut out = FakeTerm::new(xterm_terminfo());
    out.render(&[
        Change::Attribute(AttributeChange::Foreground(AnsiColor::Maroon.into())),
        Change::Attribute(AttributeChange::Intensity(Intensity::Bold)),
        Change::Text("red".into()),
        Change::Attribute(AttributeChange::Foreground(AnsiColor::Red.into())),
    ])
    .unwrap();

    let result = out.parse();
    assert_eq!(
        result,
        vec![
            Action::Esc(Esc::Code(EscCode::AsciiCharacterSetG0)),
            Action::CSI(CSI::Sgr(Sgr::Reset)),
            // Note that the render code rearranges (red,bold) to (bold,red)
            Action::CSI(CSI::Sgr(Sgr::Intensity(Intensity::Bold))),
            Action::CSI(CSI::Sgr(Sgr::Foreground(AnsiColor::Maroon.into()))),
            Action::Print('r'),
            Action::Print('e'),
            Action::Print('d'),
            Action::CSI(CSI::Sgr(Sgr::Foreground(AnsiColor::Red.into()))),
        ]
    );

    assert_eq!(
        out.renderer.current_attr,
        CellAttributes::default()
            .set_intensity(Intensity::Bold)
            .set_foreground(AnsiColor::Red)
            .clone()
    );
}
