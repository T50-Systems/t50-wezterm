#[test]
fn rxvt() {
    assert_eq!(
        parse(
            &["777", "notify", "alert user", "the tea is ready"],
            "\x1b]777;notify;alert user;the tea is ready\x1b\\"
        ),
        OperatingSystemCommand::RxvtExtension(vec![
            "notify".into(),
            "alert user".into(),
            "the tea is ready".into()
        ]),
    )
}

#[test]
fn conemu() {
    assert_eq!(
        parse(&["9", "4", "1", "42"], "\x1b]9;4;1;42\x1b\\"),
        OperatingSystemCommand::ConEmuProgress(Progress::SetPercentage(42))
    );
    assert_eq!(
        parse(&["9", "4", "2", "64"], "\x1b]9;4;2;64\x1b\\"),
        OperatingSystemCommand::ConEmuProgress(Progress::SetError(64))
    );
    assert_eq!(
        parse(&["9", "4", "3"], "\x1b]9;4;3\x1b\\"),
        OperatingSystemCommand::ConEmuProgress(Progress::SetIndeterminate)
    );
    assert_eq!(
        parse(&["9", "4", "4"], "\x1b]9;4;4\x1b\\"),
        OperatingSystemCommand::ConEmuProgress(Progress::Paused)
    );
}

#[test]
fn iterm() {
    assert_eq!(
        parse(&["1337", "SetMark"], "\x1b]1337;SetMark\x1b\\"),
        OperatingSystemCommand::ITermProprietary(ITermProprietary::SetMark)
    );

    assert_eq!(
        parse(
            &["1337", "CurrentDir=woot"],
            "\x1b]1337;CurrentDir=woot\x1b\\"
        ),
        OperatingSystemCommand::ITermProprietary(ITermProprietary::CurrentDir("woot".into()))
    );

    assert_eq!(
        parse(
            &["1337", "HighlightCursorLine=yes"],
            "\x1b]1337;HighlightCursorLine=yes\x1b\\"
        ),
        OperatingSystemCommand::ITermProprietary(ITermProprietary::HighlightCursorLine(true))
    );

    assert_eq!(
        parse(
            &["1337", "Copy=", "aGVsbG8="],
            "\x1b]1337;Copy=;aGVsbG8=\x1b\\"
        ),
        OperatingSystemCommand::ITermProprietary(ITermProprietary::Copy("hello".into()))
    );

    assert_eq!(
        parse(
            &["1337", "SetUserVar=foo=aGVsbG8="],
            "\x1b]1337;SetUserVar=foo=aGVsbG8=\x1b\\"
        ),
        OperatingSystemCommand::ITermProprietary(ITermProprietary::SetUserVar {
            name: "foo".into(),
            value: "hello".into()
        })
    );

    assert_eq!(
        parse(
            &["1337", "SetBadgeFormat=", "aGVsbG8="],
            "\x1b]1337;SetBadgeFormat=aGVsbG8=\x1b\\"
        ),
        OperatingSystemCommand::ITermProprietary(ITermProprietary::SetBadgeFormat("hello".into()))
    );

    assert_eq!(
        parse(
            &["1337", "ReportCellSize=12.0", "15.5"],
            "\x1b]1337;ReportCellSize=12.0;15.5\x1b\\"
        ),
        OperatingSystemCommand::ITermProprietary(ITermProprietary::ReportCellSize {
            height_pixels: NotNan::new(12.0).unwrap(),
            width_pixels: NotNan::new(15.5).unwrap(),
            scale: None,
        })
    );

    assert_eq!(
        parse(
            &["1337", "ReportCellSize=12.0", "15.5", "2.0"],
            "\x1b]1337;ReportCellSize=12.0;15.5;2.0\x1b\\"
        ),
        OperatingSystemCommand::ITermProprietary(ITermProprietary::ReportCellSize {
            height_pixels: NotNan::new(12.0).unwrap(),
            width_pixels: NotNan::new(15.5).unwrap(),
            scale: Some(NotNan::new(2.0).unwrap()),
        })
    );

    assert_eq!(
        parse(
            &["1337", "File=:aGVsbG8="],
            "\x1b]1337;File=:aGVsbG8=\x1b\\"
        ),
        OperatingSystemCommand::ITermProprietary(ITermProprietary::File(Box::new(ITermFileData {
            name: None,
            size: None,
            width: ITermDimension::Automatic,
            height: ITermDimension::Automatic,
            preserve_aspect_ratio: true,
            inline: false,
            do_not_move_cursor: false,
            data: b"hello".to_vec(),
        })))
    );

    assert_eq!(
        parse(
            &["1337", "File=name=bXluYW1l:aGVsbG8="],
            "\x1b]1337;File=name=bXluYW1l:aGVsbG8=\x1b\\"
        ),
        OperatingSystemCommand::ITermProprietary(ITermProprietary::File(Box::new(ITermFileData {
            name: Some("myname".into()),
            size: None,
            width: ITermDimension::Automatic,
            height: ITermDimension::Automatic,
            preserve_aspect_ratio: true,
            inline: false,
            do_not_move_cursor: false,
            data: b"hello".to_vec(),
        })))
    );

    assert_eq!(
        parse(
            &["1337", "File=size=123", "name=bXluYW1l:aGVsbG8="],
            "\x1b]1337;File=size=123;name=bXluYW1l:aGVsbG8=\x1b\\"
        ),
        OperatingSystemCommand::ITermProprietary(ITermProprietary::File(Box::new(ITermFileData {
            name: Some("myname".into()),
            size: Some(123),
            width: ITermDimension::Automatic,
            height: ITermDimension::Automatic,
            preserve_aspect_ratio: true,
            inline: false,
            do_not_move_cursor: false,
            data: b"hello".to_vec(),
        })))
    );

    assert_eq!(
        parse(
            &["1337", "File=name=bXluYW1l", "size=234:aGVsbG8="],
            "\x1b]1337;File=size=234;name=bXluYW1l:aGVsbG8=\x1b\\"
        ),
        OperatingSystemCommand::ITermProprietary(ITermProprietary::File(Box::new(ITermFileData {
            name: Some("myname".into()),
            size: Some(234),
            width: ITermDimension::Automatic,
            height: ITermDimension::Automatic,
            preserve_aspect_ratio: true,
            inline: false,
            do_not_move_cursor: false,
            data: b"hello".to_vec(),
        })))
    );

    assert_eq!(
        parse(
            &[
                "1337",
                "File=name=bXluYW1l",
                "width=auto",
                "size=234:aGVsbG8="
            ],
            "\x1b]1337;File=size=234;name=bXluYW1l:aGVsbG8=\x1b\\"
        ),
        OperatingSystemCommand::ITermProprietary(ITermProprietary::File(Box::new(ITermFileData {
            name: Some("myname".into()),
            size: Some(234),
            width: ITermDimension::Automatic,
            height: ITermDimension::Automatic,
            preserve_aspect_ratio: true,
            inline: false,
            do_not_move_cursor: false,
            data: b"hello".to_vec(),
        })))
    );

    assert_eq!(
        parse(
            &["1337", "File=name=bXluYW1l", "width=5", "size=234:aGVsbG8="],
            "\x1b]1337;File=size=234;name=bXluYW1l;width=5:aGVsbG8=\x1b\\"
        ),
        OperatingSystemCommand::ITermProprietary(ITermProprietary::File(Box::new(ITermFileData {
            name: Some("myname".into()),
            size: Some(234),
            width: ITermDimension::Cells(5),
            height: ITermDimension::Automatic,
            preserve_aspect_ratio: true,
            inline: false,
            do_not_move_cursor: false,
            data: b"hello".to_vec(),
        })))
    );

    assert_eq!(
        parse(
            &[
                "1337",
                "File=name=bXluYW1l",
                "width=5",
                "height=10%",
                "size=234:aGVsbG8="
            ],
            "\x1b]1337;File=size=234;name=bXluYW1l;width=5;height=10%:aGVsbG8=\x1b\\"
        ),
        OperatingSystemCommand::ITermProprietary(ITermProprietary::File(Box::new(ITermFileData {
            name: Some("myname".into()),
            size: Some(234),
            width: ITermDimension::Cells(5),
            height: ITermDimension::Percent(10),
            preserve_aspect_ratio: true,
            inline: false,
            do_not_move_cursor: false,
            data: b"hello".to_vec(),
        })))
    );

    assert_eq!(
            parse(
                &[
                    "1337",
                    "File=name=bXluYW1l",
                    "preserveAspectRatio=0",
                    "width=5",
                    "inline=1",
                    "height=10px",
                    "size=234:aGVsbG8="
                ],
                "\x1b]1337;File=size=234;name=bXluYW1l;width=5;height=10px;preserveAspectRatio=0;inline=1:aGVsbG8=\x1b\\"
            ),
            OperatingSystemCommand::ITermProprietary(ITermProprietary::File(Box::new(
                ITermFileData {
                    name: Some("myname".into()),
                    size: Some(234),
                    width: ITermDimension::Cells(5),
                    height: ITermDimension::Pixels(10),
                    preserve_aspect_ratio: false,
                    inline: true,
                    do_not_move_cursor: false,
                    data: b"hello".to_vec(),
                }
            )))
        );

    assert_eq!(
            parse(
                &[
                    "1337",
                    "File=name=bXluYW1l",
                    "preserveAspectRatio=0",
                    "width=5",
                    "inline=1",
                    "doNotMoveCursor=1",
                    "height=10px",
                    "size=234:aGVsbG8="
                ],
                "\x1b]1337;File=size=234;name=bXluYW1l;width=5;height=10px;preserveAspectRatio=0;inline=1;doNotMoveCursor=1:aGVsbG8=\x1b\\"
            ),
            OperatingSystemCommand::ITermProprietary(ITermProprietary::File(Box::new(
                ITermFileData {
                    name: Some("myname".into()),
                    size: Some(234),
                    width: ITermDimension::Cells(5),
                    height: ITermDimension::Pixels(10),
                    preserve_aspect_ratio: false,
                    inline: true,
                    do_not_move_cursor: true,
                    data: b"hello".to_vec(),
                }
            )))
        );
}
