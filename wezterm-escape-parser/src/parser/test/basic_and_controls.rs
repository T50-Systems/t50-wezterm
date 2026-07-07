#[test]
fn basic_parse() {
    let mut p = Parser::new();
    let actions = p.parse_as_vec(b"hello");
    assert_eq!(
        vec![
            Action::Print('h'),
            Action::Print('e'),
            Action::Print('l'),
            Action::Print('l'),
            Action::Print('o'),
        ],
        actions
    );
    assert_eq!(encode(&actions), "hello");
}

#[test]
fn basic_bold() {
    let mut p = Parser::new();
    let actions = p.parse_as_vec(b"\x1b[1mb");
    assert_eq!(
        vec![
            Action::CSI(CSI::Sgr(Sgr::Intensity(Intensity::Bold))),
            Action::Print('b'),
        ],
        actions
    );
    assert_eq!(encode(&actions), "\x1b[1mb");
}

#[test]
fn basic_bold_italic() {
    let mut p = Parser::new();
    let actions = p.parse_as_vec(b"\x1b[1;3mb");
    assert_eq!(
        vec![
            Action::CSI(CSI::Sgr(Sgr::Intensity(Intensity::Bold))),
            Action::CSI(CSI::Sgr(Sgr::Italic(true))),
            Action::Print('b'),
        ],
        actions
    );

    assert_eq!(encode(&actions), "\x1b[1m\x1b[3mb");
}

#[test]
fn fancy_underline() {
    let mut p = Parser::new();

    let actions = p.parse_as_vec(b"\x1b[4:0;4:1;4:2;4:3;4:4;4:5mb");
    assert_eq!(
        vec![
            Action::CSI(CSI::Sgr(Sgr::Underline(Underline::None))),
            Action::CSI(CSI::Sgr(Sgr::Underline(Underline::Single))),
            Action::CSI(CSI::Sgr(Sgr::Underline(Underline::Double))),
            Action::CSI(CSI::Sgr(Sgr::Underline(Underline::Curly))),
            Action::CSI(CSI::Sgr(Sgr::Underline(Underline::Dotted))),
            Action::CSI(CSI::Sgr(Sgr::Underline(Underline::Dashed))),
            Action::Print('b'),
        ],
        actions
    );

    assert_eq!(
        encode(&actions),
        "\x1b[24m\x1b[4m\x1b[21m\x1b[4:3m\x1b[4:4m\x1b[4:5mb"
    );
}

#[test]
fn true_color() {
    let mut p = Parser::new();

    let actions = p.parse_as_vec(b"\x1b[38:2::128:64:192mw");
    assert_eq!(
        vec![
            Action::CSI(CSI::Sgr(Sgr::Foreground(ColorSpec::TrueColor(
                (128, 64, 192).into()
            )))),
            Action::Print('w'),
        ],
        actions
    );

    assert_eq!(encode(&actions), "\u{1b}[38:2::128:64:192mw");

    let actions = p.parse_as_vec(b"\x1b[38:2:0:255:0mw");
    assert_eq!(
        vec![
            Action::CSI(CSI::Sgr(Sgr::Foreground(ColorSpec::TrueColor(
                (0, 255, 0).into()
            )))),
            Action::Print('w'),
        ],
        actions
    );

    let actions = p.parse_as_vec(b"\x1b[38:6:0:255:0:127mw");
    assert_eq!(
        vec![
            Action::CSI(CSI::Sgr(Sgr::Foreground(ColorSpec::TrueColor(
                (0, 255, 0, 127).into()
            )))),
            Action::Print('w'),
        ],
        actions
    );
}

#[test]
fn basic_osc() {
    let mut p = Parser::new();
    let actions = p.parse_as_vec(b"\x1b]0;hello\x07");
    assert_eq!(
        vec![Action::OperatingSystemCommand(Box::new(
            OperatingSystemCommand::SetIconNameAndWindowTitle("hello".to_owned()),
        ))],
        actions
    );
    assert_eq!(encode(&actions), "\x1b]0;hello\x1b\\");

    let actions = p.parse_as_vec(b"\x1b]532534523;hello\x07");
    assert_eq!(
        vec![Action::OperatingSystemCommand(Box::new(
            OperatingSystemCommand::Unspecified(vec![b"532534523".to_vec(), b"hello".to_vec()]),
        ))],
        actions
    );
    assert_eq!(encode(&actions), "\x1b]532534523;hello\x1b\\");
}

#[test]
fn test_emoji_title_osc() {
    let input = "\x1b]0;\u{1f915}\x07";
    let mut p = Parser::new();
    let actions = p.parse_as_vec(input.as_bytes());
    assert_eq!(
        vec![Action::OperatingSystemCommand(Box::new(
            OperatingSystemCommand::SetIconNameAndWindowTitle("\u{1f915}".to_owned()),
        ))],
        actions
    );
    assert_eq!(encode(&actions), "\x1b]0;\u{1f915}\x1b\\");
}

#[test]
fn basic_esc() {
    let mut p = Parser::new();
    let actions = p.parse_as_vec(b"\x1bH");
    assert_eq!(
        vec![Action::Esc(Esc::Code(EscCode::HorizontalTabSet))],
        actions
    );
    assert_eq!(encode(&actions), "\x1bH");

    let actions = p.parse_as_vec(b"\x1b%H");
    assert_eq!(
        vec![Action::Esc(Esc::Unspecified {
            intermediate: Some(b'%'),
            control: b'H',
        })],
        actions
    );
    assert_eq!(encode(&actions), "\x1b%H");
}

#[test]
fn soft_reset() {
    let mut p = Parser::new();
    let actions = p.parse_as_vec(b"\x1b[!p");
    assert_eq!(
        vec![Action::CSI(CSI::Device(Box::new(
            crate::csi::Device::SoftReset
        )))],
        actions
    );
    assert_eq!(encode(&actions), "\x1b[!p");
}

#[test]
fn tmux_title_escape() {
    let mut p = Parser::new();
    let actions = p.parse_as_vec(b"\x1bktitle\x1b\\");
    assert_eq!(
        vec![
            Action::Esc(Esc::Code(EscCode::TmuxTitle)),
            Action::Print('t'),
            Action::Print('i'),
            Action::Print('t'),
            Action::Print('l'),
            Action::Print('e'),
            Action::Esc(Esc::Code(EscCode::StringTerminator)),
        ],
        actions
    );
}

fn round_trip_parse(s: &str) -> Vec<Action> {
    let mut p = Parser::new();
    let actions = p.parse_as_vec(s.as_bytes());
    assert_eq!(s, encode(&actions), "actions: {actions:?}");
    actions
}

fn parse_as(s: &str, expected: &str) -> Vec<Action> {
    let mut p = Parser::new();
    let actions = p.parse_as_vec(s.as_bytes());
    assert_eq!(expected, encode(&actions), "actions: {actions:?}");
    actions
}

#[test]
fn xtgettcap() {
    assert_eq!(
        round_trip_parse("\x1bP+q544e\x1b\\"),
        vec![
            Action::XtGetTcap(vec!["TN".to_string()]),
            Action::Esc(Esc::Code(EscCode::StringTerminator)),
        ]
    );
}

#[test]
fn bidi_modes() {
    assert_eq!(
        round_trip_parse("\x1b[1 k"),
        vec![Action::CSI(CSI::SelectCharacterPath(
            CharacterPath::LeftToRightOrTopToBottom,
            0
        ))]
    );
    assert_eq!(
        round_trip_parse("\x1b[2;1 k"),
        vec![Action::CSI(CSI::SelectCharacterPath(
            CharacterPath::RightToLeftOrBottomToTop,
            1
        ))]
    );
}

#[test]
fn xterm_key() {
    assert_eq!(
        round_trip_parse("\x1b[>4;2m"),
        vec![Action::CSI(CSI::Mode(Mode::XtermKeyMode {
            resource: XtermKeyModifierResource::OtherKeys,
            value: Some(2),
        }))]
    );
    assert_eq!(
        round_trip_parse("\x1b[>4;m"),
        vec![Action::CSI(CSI::Mode(Mode::XtermKeyMode {
            resource: XtermKeyModifierResource::OtherKeys,
            value: None,
        }))]
    );
}

#[test]
fn window() {
    assert_eq!(
        round_trip_parse("\x1b[22;2t"),
        vec![Action::CSI(CSI::Window(Box::new(Window::PushWindowTitle)))]
    );
}

#[test]
fn checksum_area() {
    assert_eq!(
        round_trip_parse("\x1b[1;2;3;4;5;6*y"),
        vec![Action::CSI(CSI::Window(Box::new(
            Window::ChecksumRectangularArea {
                request_id: 1,
                page_number: 2,
                top: OneBased::new(3),
                left: OneBased::new(4),
                bottom: OneBased::new(5),
                right: OneBased::new(6),
            }
        )))]
    );
}

#[test]
fn dec_private_modes() {
    assert_eq!(
        parse_as("\x1b[?1;1006h", "\x1b[?1h\x1b[?1006h"),
        vec![
            Action::CSI(CSI::Mode(Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::ApplicationCursorKeys
            ),))),
            Action::CSI(CSI::Mode(Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::SGRMouse
            ),))),
        ]
    );
}

#[test]
fn xtsmgraphics() {
    assert_eq!(
        round_trip_parse("\x1b[?1;3;256S"),
        vec![Action::CSI(CSI::Device(Box::new(Device::XtSmGraphics(
            XtSmGraphics {
                item: XtSmGraphicsItem::NumberOfColorRegisters,
                action_or_status: 3,
                value: vec![256]
            }
        ))))]
    );
}

#[test]
fn req_attr() {
    assert_eq!(
        round_trip_parse("\x1b[=c"),
        vec![Action::CSI(CSI::Device(Box::new(
            Device::RequestTertiaryDeviceAttributes
        )))]
    );
    assert_eq!(
        round_trip_parse("\x1b[>c"),
        vec![Action::CSI(CSI::Device(Box::new(
            Device::RequestSecondaryDeviceAttributes
        )))]
    );
}

#[test]
fn sgr() {
    assert_eq!(
        parse_as("\x1b[;4m", "\x1b[0m\x1b[4m"),
        vec![
            Action::CSI(CSI::Sgr(Sgr::Reset)),
            Action::CSI(CSI::Sgr(Sgr::Underline(Underline::Single))),
        ]
    );
}

#[test]
fn kitty_img() {
    use crate::apc::*;
    assert_eq!(
        round_trip_parse("\x1b_Gf=24,s=10,v=20;aGVsbG8=\x1b\\"),
        vec![
            Action::KittyImage(Box::new(KittyImage::TransmitData {
                transmit: KittyImageTransmit {
                    format: Some(KittyImageFormat::Rgb),
                    data: KittyImageData::Direct("aGVsbG8=".to_string()),
                    width: Some(10),
                    height: Some(20),
                    image_id: None,
                    image_number: None,
                    compression: KittyImageCompression::None,
                    more_data_follows: false,
                },
                verbosity: KittyImageVerbosity::Verbose,
            })),
            Action::Esc(Esc::Code(EscCode::StringTerminator)),
        ]
    );

    assert_eq!(
        parse_as(
            "\x1b_Ga=q,s=1,v=1,i=1;YWJjZA==\x1b\\",
            "\x1b_Ga=q,i=1,s=1,v=1;YWJjZA==\x1b\\"
        ),
        vec![
            Action::KittyImage(Box::new(KittyImage::Query {
                transmit: KittyImageTransmit {
                    format: None,
                    data: KittyImageData::Direct("YWJjZA==".to_string()),
                    width: Some(1),
                    height: Some(1),
                    image_id: Some(1),
                    image_number: None,
                    compression: KittyImageCompression::None,
                    more_data_follows: false,
                },
            })),
            Action::Esc(Esc::Code(EscCode::StringTerminator)),
        ]
    );
    assert_eq!(
        parse_as(
            "\x1b_Ga=q,t=f,s=1,v=1,i=2;L3Zhci90bXAvdG1wdGYxd3E4Ym4=\x1b\\",
            "\x1b_Ga=q,i=2,s=1,t=f,v=1;L3Zhci90bXAvdG1wdGYxd3E4Ym4=\x1b\\"
        ),
        vec![
            Action::KittyImage(Box::new(KittyImage::Query {
                transmit: KittyImageTransmit {
                    format: None,
                    data: KittyImageData::File {
                        path: "/var/tmp/tmptf1wq8bn".to_string(),
                        data_offset: None,
                        data_size: None,
                    },
                    width: Some(1),
                    height: Some(1),
                    image_id: Some(2),
                    image_number: None,
                    compression: KittyImageCompression::None,
                    more_data_follows: false,
                },
            })),
            Action::Esc(Esc::Code(EscCode::StringTerminator)),
        ]
    );
}
