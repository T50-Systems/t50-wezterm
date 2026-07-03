#[test]
fn red_bold_text_no_terminfo() {
    let mut out = FakeTerm::new(no_terminfo_all_enabled());
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

#[test]
fn color_after_attribute_change() {
    let mut out = FakeTerm::new(xterm_terminfo());
    out.render(&[
        Change::Attribute(AttributeChange::Foreground(AnsiColor::Maroon.into())),
        Change::Attribute(AttributeChange::Intensity(Intensity::Bold)),
        Change::Text("red".into()),
        Change::Attribute(AttributeChange::Intensity(Intensity::Normal)),
        Change::Text("2".into()),
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
            // Turning off bold is translated into reset and set red again
            Action::Esc(Esc::Code(EscCode::AsciiCharacterSetG0)),
            Action::CSI(CSI::Sgr(Sgr::Reset)),
            Action::CSI(CSI::Sgr(Sgr::Foreground(AnsiColor::Maroon.into()))),
            Action::Print('2'),
        ]
    );

    assert_eq!(
        out.renderer.current_attr,
        CellAttributes::default()
            .set_foreground(AnsiColor::Maroon)
            .clone()
    );
}

#[test]
fn truecolor() {
    let mut out = FakeTerm::new(xterm_terminfo());
    out.render(&[
        Change::Attribute(AttributeChange::Foreground(
            ColorSpec::TrueColor((255, 128, 64).into()).into(),
        )),
        Change::Text("A".into()),
    ])
    .unwrap();

    let result = out.parse();
    assert_eq!(
        result,
        vec![
            Action::CSI(CSI::Sgr(Sgr::Foreground(
                ColorSpec::TrueColor((255, 128, 64).into()).into(),
            ))),
            Action::Print('A'),
        ]
    );
}

#[test]
fn truecolor_no_terminfo() {
    let mut out = FakeTerm::new(no_terminfo_all_enabled());
    out.render(&[
        Change::Attribute(AttributeChange::Foreground(
            ColorSpec::TrueColor((255, 128, 64).into()).into(),
        )),
        Change::Text("A".into()),
    ])
    .unwrap();

    let result = out.parse();
    assert_eq!(
        result,
        vec![
            Action::CSI(CSI::Sgr(Sgr::Foreground(
                ColorSpec::TrueColor((255, 128, 64).into()).into(),
            ))),
            Action::Print('A'),
        ]
    );
}
