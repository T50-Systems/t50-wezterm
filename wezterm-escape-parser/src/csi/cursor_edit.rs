#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecPrivateMode {
    Code(DecPrivateModeCode),
    Unspecified(u16),
}

#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum DecPrivateModeCode {
    /// https://vt100.net/docs/vt510-rm/DECCKM.html
    /// This mode is only effective when the terminal is in keypad application mode (see DECKPAM)
    /// and the ANSI/VT52 mode (DECANM) is set (see DECANM). Under these conditions, if the cursor
    /// key mode is reset, the four cursor function keys will send ANSI cursor control commands. If
    /// cursor key mode is set, the four cursor function keys will send application functions.
    ApplicationCursorKeys = 1,

    /// https://vt100.net/docs/vt510-rm/DECANM.html
    /// Behave like a vt52
    DecAnsiMode = 2,

    /// https://vt100.net/docs/vt510-rm/DECCOLM.html
    Select132Columns = 3,
    /// https://vt100.net/docs/vt510-rm/DECSCLM.html
    SmoothScroll = 4,
    /// https://vt100.net/docs/vt510-rm/DECSCNM.html
    ReverseVideo = 5,
    /// https://vt100.net/docs/vt510-rm/DECOM.html
    /// When OriginMode is enabled, cursor is constrained to the
    /// scroll region and its position is relative to the scroll
    /// region.
    OriginMode = 6,
    /// https://vt100.net/docs/vt510-rm/DECAWM.html
    /// When enabled, wrap to next line, Otherwise replace the last
    /// character
    AutoWrap = 7,
    /// https://vt100.net/docs/vt510-rm/DECARM.html
    AutoRepeat = 8,
    StartBlinkingCursor = 12,
    ShowCursor = 25,

    ReverseWraparound = 45,

    /// https://vt100.net/docs/vt510-rm/DECLRMM.html
    LeftRightMarginMode = 69,

    /// DECSDM - https://vt100.net/dec/ek-vt38t-ug-001.pdf#page=132
    SixelDisplayMode = 80,
    /// Enable mouse button press/release reporting
    MouseTracking = 1000,
    /// Warning: this requires a cooperative and timely response from
    /// the application otherwise the terminal can hang
    HighlightMouseTracking = 1001,
    /// Enable mouse button press/release and drag reporting
    ButtonEventMouse = 1002,
    /// Enable mouse motion, button press/release and drag reporting
    AnyEventMouse = 1003,
    /// Enable FocusIn/FocusOut events
    FocusTracking = 1004,
    Utf8Mouse = 1005,
    /// Use extended coordinate system in mouse reporting.  Does not
    /// enable mouse reporting itself, it just controls how reports
    /// will be encoded.
    SGRMouse = 1006,
    /// Use pixels rather than text cells in mouse reporting.  Does
    /// not enable mouse reporting itself, it just controls how
    /// reports will be encoded.
    SGRPixelsMouse = 1016,

    XTermMetaSendsEscape = 1036,
    XTermAltSendsEscape = 1039,

    /// Save cursor as in DECSC
    SaveCursor = 1048,
    ClearAndEnableAlternateScreen = 1049,
    EnableAlternateScreen = 47,
    OptEnableAlternateScreen = 1047,
    BracketedPaste = 2004,

    /// <https://github.com/contour-terminal/terminal-unicode-core/>
    /// Grapheme clustering mode
    GraphemeClustering = 2027,

    /// Applies to sixel and regis modes
    UsePrivateColorRegistersForEachGraphic = 1070,

    /// <https://gist.github.com/christianparpart/d8a62cc1ab659194337d73e399004036>
    SynchronizedOutput = 2026,

    MinTTYApplicationEscapeKeyMode = 7727,

    /// xterm: adjust cursor positioning after emitting sixel
    SixelScrollsRight = 8452,

    /// Windows Terminal: win32-input-mode
    /// <https://github.com/microsoft/terminal/blob/main/doc/specs/%234999%20-%20Improved%20keyboard%20handling%20in%20Conpty.md>
    Win32InputMode = 9001,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalMode {
    Code(TerminalModeCode),
    Unspecified(u16),
}

#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum TerminalModeCode {
    /// https://vt100.net/docs/vt510-rm/KAM.html
    KeyboardAction = 2,
    /// https://vt100.net/docs/vt510-rm/IRM.html
    Insert = 4,
    /// <https://terminal-wg.pages.freedesktop.org/bidi/recommendation/escape-sequences.html>
    BiDirectionalSupportMode = 8,
    /// https://vt100.net/docs/vt510-rm/SRM.html
    /// But in the MS terminal this is cursor blinking.
    SendReceive = 12,
    /// https://vt100.net/docs/vt510-rm/LNM.html
    AutomaticNewline = 20,
    /// MS terminal cursor visibility
    ShowCursor = 25,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cursor {
    /// CBT Moves cursor to the Ps tabs backward. The default value of Ps is 1.
    BackwardTabulation(u32),

    /// TBC - TABULATION CLEAR
    TabulationClear(TabulationClear),

    /// CHA: Moves cursor to the Ps-th column of the active line. The default
    /// value of Ps is 1.
    CharacterAbsolute(OneBased),

    /// HPA CHARACTER POSITION ABSOLUTE
    /// HPA Moves cursor to the Ps-th column of the active line. The default
    /// value of Ps is 1.
    CharacterPositionAbsolute(OneBased),

    /// HPB - CHARACTER POSITION BACKWARD
    /// HPB Moves cursor to the left Ps columns. The default value of Ps is 1.
    CharacterPositionBackward(u32),

    /// HPR - CHARACTER POSITION FORWARD
    /// HPR Moves cursor to the right Ps columns. The default value of Ps is 1.
    CharacterPositionForward(u32),

    /// HVP - CHARACTER AND LINE POSITION
    /// HVP Moves cursor to the Ps1-th line and to the Ps2-th column. The
    /// default value of Ps1 and Ps2 is 1.
    CharacterAndLinePosition {
        line: OneBased,
        col: OneBased,
    },

    /// VPA - LINE POSITION ABSOLUTE
    /// Move to the corresponding vertical position (line Ps) of the current
    /// column. The default value of Ps is 1.
    LinePositionAbsolute(u32),

    /// VPB - LINE POSITION BACKWARD
    /// Moves cursor up Ps lines in the same column. The default value of Ps is
    /// 1.
    LinePositionBackward(u32),

    /// VPR - LINE POSITION FORWARD
    /// Moves cursor down Ps lines in the same column. The default value of Ps
    /// is 1.
    LinePositionForward(u32),

    /// CHT
    /// Moves cursor to the Ps tabs forward. The default value of Ps is 1.
    ForwardTabulation(u32),

    /// CNL Moves cursor to the first column of Ps-th following line. The
    /// default value of Ps is 1.
    NextLine(u32),

    /// CPL Moves cursor to the first column of Ps-th preceding line. The
    /// default value of Ps is 1.
    PrecedingLine(u32),

    /// CPR - ACTIVE POSITION REPORT
    /// If the DEVICE COMPONENT SELECT MODE (DCSM)
    /// is set to PRESENTATION, CPR is used to report the active presentation
    /// position of the sending device as residing in the presentation
    /// component at the n-th line position according to the line progression
    /// and at the m-th character position according to the character path,
    /// where n equals the value of Pn1 and m equal s the value of Pn2.
    /// If the DEVICE COMPONENT SELECT MODE (DCSM) is set to DATA, CPR is used
    /// to report the active data position of the sending device as
    /// residing in the data component at the n-th line position according
    /// to the line progression and at the m-th character position
    /// according to the character progression, where n equals the value of
    /// Pn1 and m equals the value of Pn2. CPR may be solicited by a DEVICE
    /// STATUS REPORT (DSR) or be sent unsolicited .
    ActivePositionReport {
        line: OneBased,
        col: OneBased,
    },

    /// CPR: this is the request from the client.
    /// The terminal will respond with ActivePositionReport.
    RequestActivePositionReport,

    /// SCP - Save Cursor Position.
    /// Only works when DECLRMM is disabled
    SaveCursor,
    RestoreCursor,

    /// CTC - CURSOR TABULATION CONTROL
    /// CTC causes one or more tabulation stops to be set or cleared in the
    /// presentation component, depending on the parameter values.
    /// In the case of parameter values 0, 2 or 4 the number of lines affected
    /// depends on the setting of the TABULATION STOP MODE (TSM).
    TabulationControl(CursorTabulationControl),

    /// CUB - Cursor Left
    /// Moves cursor to the left Ps columns. The default value of Ps is 1.
    Left(u32),

    /// CUD - Cursor Down
    Down(u32),

    /// CUF - Cursor Right
    Right(u32),

    /// CUP - Cursor Position
    /// Moves cursor to the Ps1-th line and to the Ps2-th column. The default
    /// value of Ps1 and Ps2 is 1.
    Position {
        line: OneBased,
        col: OneBased,
    },

    /// CUU - Cursor Up
    Up(u32),

    /// CVT - Cursor Line Tabulation
    /// CVT causes the active presentation position to be moved to the
    /// corresponding character position of the line corresponding to the n-th
    /// following line tabulation stop in the presentation component, where n
    /// equals the value of Pn.
    LineTabulation(u32),

    /// DECSTBM - Set top and bottom margins.
    SetTopAndBottomMargins {
        top: OneBased,
        bottom: OneBased,
    },

    /// https://vt100.net/docs/vt510-rm/DECSLRM.html
    SetLeftAndRightMargins {
        left: OneBased,
        right: OneBased,
    },

    CursorStyle(CursorStyle),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    /// DCH - DELETE CHARACTER
    /// Deletes Ps characters from the cursor position to the right. The
    /// default value of Ps is 1. If the DEVICE COMPONENT SELECT MODE
    /// (DCSM) is set to PRESENTATION, DCH causes the contents of the
    /// active presentation position and, depending on the setting of the
    /// CHARACTER EDITING MODE (HEM), the contents of the n-1 preceding or
    /// following character positions to be removed from the presentation
    /// component, where n equals the value of Pn. The resulting gap is
    /// closed by shifting the contents of the adjacent character positions
    /// towards the active presentation position. At the other end of the
    /// shifted part, n character positions are put into the erased state.
    DeleteCharacter(u32),

    /// DL - DELETE LINE
    /// If the DEVICE COMPONENT SELECT MODE (DCSM) is set to PRESENTATION, DL
    /// causes the contents of the active line (the line that contains the
    /// active presentation position) and, depending on the setting of the
    /// LINE EDITING MODE (VEM), the contents of the n-1 preceding or
    /// following lines to be removed from the presentation component, where n
    /// equals the value of Pn. The resulting gap is closed by shifting the
    /// contents of a number of adjacent lines towards the active line. At
    /// the other end of the shifted part, n lines are put into the
    /// erased state.  The active presentation position is moved to the line
    /// home position in the active line. The line home position is
    /// established by the parameter value of SET LINE HOME (SLH). If the
    /// TABULATION STOP MODE (TSM) is set to SINGLE, character tabulation stops
    /// are cleared in the lines that are put into the erased state.  The
    /// extent of the shifted part is established by SELECT EDITING EXTENT
    /// (SEE).  Any occurrences of the start or end of a selected area, the
    /// start or end of a qualified area, or a tabulation stop in the shifted
    /// part, are also shifted.
    DeleteLine(u32),

    /// ECH - ERASE CHARACTER
    /// If the DEVICE COMPONENT SELECT MODE (DCSM) is set to PRESENTATION, ECH
    /// causes the active presentation position and the n-1 following
    /// character positions in the presentation component to be put into
    /// the erased state, where n equals the value of Pn.
    EraseCharacter(u32),

    /// EL - ERASE IN LINE
    /// If the DEVICE COMPONENT SELECT MODE (DCSM) is set to PRESENTATION, EL
    /// causes some or all character positions of the active line (the line
    /// which contains the active presentation position in the presentation
    /// component) to be put into the erased state, depending on the
    /// parameter values
    EraseInLine(EraseInLine),

    /// ICH - INSERT CHARACTER
    /// If the DEVICE COMPONENT SELECT MODE (DCSM) is set to PRESENTATION, ICH
    /// is used to prepare the insertion of n characters, by putting into the
    /// erased state the active presentation position and, depending on the
    /// setting of the CHARACTER EDITING MODE (HEM), the n-1 preceding or
    /// following character positions in the presentation component, where n
    /// equals the value of Pn. The previous contents of the active
    /// presentation position and an adjacent string of character positions are
    /// shifted away from the active presentation position. The contents of n
    /// character positions at the other end of the shifted part are removed.
    /// The active presentation position is moved to the line home position in
    /// the active line. The line home position is established by the parameter
    /// value of SET LINE HOME (SLH).
    InsertCharacter(u32),

    /// IL - INSERT LINE
    /// If the DEVICE COMPONENT SELECT MODE (DCSM) is set to PRESENTATION, IL
    /// is used to prepare the insertion of n lines, by putting into the
    /// erased state in the presentation component the active line (the
    /// line that contains the active presentation position) and, depending on
    /// the setting of the LINE EDITING MODE (VEM), the n-1 preceding or
    /// following lines, where n equals the value of Pn. The previous
    /// contents of the active line and of adjacent lines are shifted away
    /// from the active line. The contents of n lines at the other end of the
    /// shifted part are removed. The active presentation position is moved
    /// to the line home position in the active line. The line home
    /// position is established by the parameter value of SET LINE
    /// HOME (SLH).
    InsertLine(u32),

    /// SD - SCROLL DOWN
    /// SD causes the data in the presentation component to be moved by n line
    /// positions if the line orientation is horizontal, or by n character
    /// positions if the line orientation is vertical, such that the data
    /// appear to move down; where n equals the value of Pn. The active
    /// presentation position is not affected by this control function.
    ///
    /// Also known as Pan Up in DEC:
    /// https://vt100.net/docs/vt510-rm/SD.html
    ScrollDown(u32),

    /// SU - SCROLL UP
    /// SU causes the data in the presentation component to be moved by n line
    /// positions if the line orientation is horizontal, or by n character
    /// positions if the line orientation is vertical, such that the data
    /// appear to move up; where n equals the value of Pn. The active
    /// presentation position is not affected by this control function.
    ScrollUp(u32),

    /// ED - ERASE IN PAGE (XTerm calls this Erase in Display)
    EraseInDisplay(EraseInDisplay),

    /// REP - Repeat the preceding character n times
    Repeat(u32),
}

trait EncodeCSIParam {
    fn write_csi(&self, f: &mut Formatter, control: &str) -> Result<(), FmtError>;
}

impl<T: ParamEnum + PartialEq + ToPrimitive> EncodeCSIParam for T {
    fn write_csi(&self, f: &mut Formatter, control: &str) -> Result<(), FmtError> {
        if *self == ParamEnum::default() {
            write!(f, "{}", control)
        } else {
            let value = self.to_i64().ok_or_else(|| FmtError)?;
            write!(f, "{}{}", value, control)
        }
    }
}

impl EncodeCSIParam for u32 {
    fn write_csi(&self, f: &mut Formatter, control: &str) -> Result<(), FmtError> {
        if *self == 1 {
            write!(f, "{}", control)
        } else {
            write!(f, "{}{}", *self, control)
        }
    }
}

impl EncodeCSIParam for OneBased {
    fn write_csi(&self, f: &mut Formatter, control: &str) -> Result<(), FmtError> {
        if self.as_one_based() == 1 {
            write!(f, "{}", control)
        } else {
            write!(f, "{}{}", *self, control)
        }
    }
}

impl Display for Edit {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Edit::DeleteCharacter(n) => n.write_csi(f, "P")?,
            Edit::DeleteLine(n) => n.write_csi(f, "M")?,
            Edit::EraseCharacter(n) => n.write_csi(f, "X")?,
            Edit::EraseInLine(n) => n.write_csi(f, "K")?,
            Edit::InsertCharacter(n) => n.write_csi(f, "@")?,
            Edit::InsertLine(n) => n.write_csi(f, "L")?,
            Edit::ScrollDown(n) => n.write_csi(f, "T")?,
            Edit::ScrollUp(n) => n.write_csi(f, "S")?,
            Edit::EraseInDisplay(n) => n.write_csi(f, "J")?,
            Edit::Repeat(n) => n.write_csi(f, "b")?,
        }
        Ok(())
    }
}
