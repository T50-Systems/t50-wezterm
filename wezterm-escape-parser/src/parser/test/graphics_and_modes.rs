/* Withdrawn because xterm introduced a conflict:
 * <https://github.com/mintty/mintty/issues/1171#issuecomment-1336174469>
 * <https://github.com/mintty/mintty/issues/1189>
#[test]
fn dec_private_sgr() {
    use crate::cell::{VerticalAlign};
    assert_eq!(
        parse_as("\x1b[?0m", "\x1b[0m"),
        vec![Action::CSI(CSI::Sgr(Sgr::Reset))]
    );
    assert_eq!(
        parse_as("\x1b[?4m", "\x1b[73m"),
        vec![Action::CSI(CSI::Sgr(Sgr::VerticalAlign(
            VerticalAlign::SuperScript
        )))]
    );
    assert_eq!(
        parse_as("\x1b[?5m", "\x1b[74m"),
        vec![Action::CSI(CSI::Sgr(Sgr::VerticalAlign(
            VerticalAlign::SubScript
        )))]
    );
    assert_eq!(
        parse_as("\x1b[?24m", "\x1b[75m"),
        vec![Action::CSI(CSI::Sgr(Sgr::VerticalAlign(
            VerticalAlign::BaseLine
        )))]
    );
    assert_eq!(
        parse_as("\x1b[?6m", "\x1b[53m"),
        vec![Action::CSI(CSI::Sgr(Sgr::Overline(true)))]
    );
    assert_eq!(
        parse_as("\x1b[?26m", "\x1b[55m"),
        vec![Action::CSI(CSI::Sgr(Sgr::Overline(false)))]
    );
}
*/

#[test]
fn decset() {
    assert_eq!(
        round_trip_parse("\x1b[?23434h"),
        vec![Action::CSI(CSI::Mode(Mode::SetDecPrivateMode(
            DecPrivateMode::Unspecified(23434),
        )))]
    );

    /*
    {
        let res = CSI::parse(&[CsiParam::Integer(2026)], &[b'?', b'$'], false, 'p').collect();
        assert_eq!(encode(&res), "\x1b[?2026$p");
    }
    */

    assert_eq!(
        round_trip_parse("\x1b[?1l"),
        vec![Action::CSI(CSI::Mode(Mode::ResetDecPrivateMode(
            DecPrivateMode::Code(DecPrivateModeCode::ApplicationCursorKeys,)
        )))]
    );

    assert_eq!(
        round_trip_parse("\x1b[?25s"),
        vec![Action::CSI(CSI::Mode(Mode::SaveDecPrivateMode(
            DecPrivateMode::Code(DecPrivateModeCode::ShowCursor,)
        )))]
    );
    assert_eq!(
        round_trip_parse("\x1b[?2004r"),
        vec![Action::CSI(CSI::Mode(Mode::RestoreDecPrivateMode(
            DecPrivateMode::Code(DecPrivateModeCode::BracketedPaste),
        )))]
    );
    assert_eq!(
        round_trip_parse("\x1b[?12h\x1b[?25h"),
        vec![
            Action::CSI(CSI::Mode(Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::StartBlinkingCursor,
            )))),
            Action::CSI(CSI::Mode(Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::ShowCursor,
            )))),
        ]
    );

    assert_eq!(
        round_trip_parse("\x1b[?1002h\x1b[?1003h\x1b[?1005h\x1b[?1006h"),
        vec![
            Action::CSI(CSI::Mode(Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::ButtonEventMouse,
            )))),
            Action::CSI(CSI::Mode(Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::AnyEventMouse,
            )))),
            Action::CSI(CSI::Mode(Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::Utf8Mouse
            )))),
            Action::CSI(CSI::Mode(Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::SGRMouse,
            )))),
        ]
    );
}

#[test]
fn issue_1291() {
    use crate::osc::{ITermDimension, ITermFileData, ITermProprietary};

    let mut p = Parser::new();
    // Note the empty k=v pair immediately following `File=`
    let actions = p.parse_as_vec(b"\x1b]1337;File=;size=234:aGVsbG8=\x07");
    assert_eq!(
        vec![Action::OperatingSystemCommand(Box::new(
            OperatingSystemCommand::ITermProprietary(ITermProprietary::File(Box::new(
                ITermFileData {
                    name: None,
                    size: Some(234),
                    width: ITermDimension::Automatic,
                    height: ITermDimension::Automatic,
                    preserve_aspect_ratio: true,
                    inline: false,
                    do_not_move_cursor: false,
                    data: b"hello".to_vec(),
                }
            )))
        ))],
        actions
    );
}

#[test]
fn itermfiledata_oob() {
    let mut p = Parser::new();
    p.parse_as_vec(b"\x9d1337\xff;File\x1b");
}
