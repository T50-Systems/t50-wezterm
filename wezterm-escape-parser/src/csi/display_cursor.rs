impl Display for Cursor {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Cursor::BackwardTabulation(n) => n.write_csi(f, "Z")?,
            Cursor::CharacterAbsolute(col) => col.write_csi(f, "G")?,
            Cursor::ForwardTabulation(n) => n.write_csi(f, "I")?,
            Cursor::NextLine(n) => n.write_csi(f, "E")?,
            Cursor::PrecedingLine(n) => n.write_csi(f, "F")?,
            Cursor::ActivePositionReport { line, col } => write!(f, "{};{}R", line, col)?,
            Cursor::Left(n) => n.write_csi(f, "D")?,
            Cursor::Down(n) => n.write_csi(f, "B")?,
            Cursor::Right(n) => n.write_csi(f, "C")?,
            Cursor::Up(n) => n.write_csi(f, "A")?,
            Cursor::Position { line, col } => write!(f, "{};{}H", line, col)?,
            Cursor::LineTabulation(n) => n.write_csi(f, "Y")?,
            Cursor::TabulationControl(n) => n.write_csi(f, "W")?,
            Cursor::TabulationClear(n) => n.write_csi(f, "g")?,
            Cursor::CharacterPositionAbsolute(n) => n.write_csi(f, "`")?,
            Cursor::CharacterPositionBackward(n) => n.write_csi(f, "j")?,
            Cursor::CharacterPositionForward(n) => n.write_csi(f, "a")?,
            Cursor::CharacterAndLinePosition { line, col } => write!(f, "{};{}f", line, col)?,
            Cursor::LinePositionAbsolute(n) => n.write_csi(f, "d")?,
            Cursor::LinePositionBackward(n) => n.write_csi(f, "k")?,
            Cursor::LinePositionForward(n) => n.write_csi(f, "e")?,
            Cursor::SetTopAndBottomMargins { top, bottom } => {
                if top.as_one_based() == 1 && bottom.as_one_based() == u32::max_value() {
                    write!(f, "r")?;
                } else {
                    write!(f, "{};{}r", top, bottom)?;
                }
            }
            Cursor::SetLeftAndRightMargins { left, right } => {
                if left.as_one_based() == 1 && right.as_one_based() == u32::max_value() {
                    write!(f, "s")?;
                } else {
                    write!(f, "{};{}s", left, right)?;
                }
            }
            Cursor::RequestActivePositionReport => write!(f, "6n")?,
            Cursor::SaveCursor => write!(f, "s")?,
            Cursor::RestoreCursor => write!(f, "u")?,
            Cursor::CursorStyle(style) => write!(f, "{} q", *style as u8)?,
        }
        Ok(())
    }
}

/// This trait aids in parsing escape sequences.
/// In many cases we simply want to collect integral values >= 1,
/// but in some we build out an enum.  The trait helps to generalize
/// the parser code while keeping it relatively terse.
trait ParseParams: Sized {
    fn parse_params(params: &[CsiParam]) -> Result<Self, ()>;
}

/// Parse an input parameter into a 1-based unsigned value
impl ParseParams for u32 {
    fn parse_params(params: &[CsiParam]) -> Result<u32, ()> {
        match params {
            [] => Ok(1),
            [p] => to_1b_u32(p),
            _ => Err(()),
        }
    }
}

/// Parse an input parameter into a 1-based unsigned value
impl ParseParams for OneBased {
    fn parse_params(params: &[CsiParam]) -> Result<OneBased, ()> {
        match params {
            [] => Ok(OneBased::new(1)),
            [p] => OneBased::from_esc_param(p),
            _ => Err(()),
        }
    }
}

/// Parse a pair of 1-based unsigned values into a tuple.
/// This is typically used to build a struct comprised of
/// the pair of values.
impl ParseParams for (OneBased, OneBased) {
    fn parse_params(params: &[CsiParam]) -> Result<(OneBased, OneBased), ()> {
        match params {
            [] | [CsiParam::P(b';')] => Ok((OneBased::new(1), OneBased::new(1))),
            [a] | [a, CsiParam::P(b';')] => Ok((OneBased::from_esc_param(a)?, OneBased::new(1))),
            [a, CsiParam::P(b';'), b] => {
                Ok((OneBased::from_esc_param(a)?, OneBased::from_esc_param(b)?))
            }
            [CsiParam::P(b';'), b] => Ok((OneBased::new(1), OneBased::from_esc_param(b)?)),
            _ => Err(()),
        }
    }
}

/// This is ostensibly a marker trait that is used within this module
/// to denote an enum.  It does double duty as a stand-in for Default.
/// We need separate traits for this to disambiguate from a regular
/// primitive integer.
trait ParamEnum: FromPrimitive {
    fn default() -> Self;
}

/// implement ParseParams for the enums that also implement ParamEnum.
impl<T: ParamEnum> ParseParams for T {
    fn parse_params(params: &[CsiParam]) -> Result<Self, ()> {
        match params {
            [] => Ok(ParamEnum::default()),
            [CsiParam::Integer(i)] => FromPrimitive::from_i64(*i).ok_or(()),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, Copy, ToPrimitive)]
pub enum CursorTabulationControl {
    SetCharacterTabStopAtActivePosition = 0,
    SetLineTabStopAtActiveLine = 1,
    ClearCharacterTabStopAtActivePosition = 2,
    ClearLineTabstopAtActiveLine = 3,
    ClearAllCharacterTabStopsAtActiveLine = 4,
    ClearAllCharacterTabStops = 5,
    ClearAllLineTabStops = 6,
}

impl ParamEnum for CursorTabulationControl {
    fn default() -> Self {
        CursorTabulationControl::SetCharacterTabStopAtActivePosition
    }
}

#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, Copy, ToPrimitive)]
pub enum TabulationClear {
    ClearCharacterTabStopAtActivePosition = 0,
    ClearLineTabStopAtActiveLine = 1,
    ClearCharacterTabStopsAtActiveLine = 2,
    ClearAllCharacterTabStops = 3,
    ClearAllLineTabStops = 4,
    ClearAllTabStops = 5,
}

impl ParamEnum for TabulationClear {
    fn default() -> Self {
        TabulationClear::ClearCharacterTabStopAtActivePosition
    }
}

#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, Copy, ToPrimitive)]
pub enum EraseInLine {
    EraseToEndOfLine = 0,
    EraseToStartOfLine = 1,
    EraseLine = 2,
}

impl ParamEnum for EraseInLine {
    fn default() -> Self {
        EraseInLine::EraseToEndOfLine
    }
}

#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, Copy, ToPrimitive)]
pub enum EraseInDisplay {
    /// the active presentation position and the character positions up to the
    /// end of the page are put into the erased state
    EraseToEndOfDisplay = 0,
    /// the character positions from the beginning of the page up to and
    /// including the active presentation position are put into the erased
    /// state
    EraseToStartOfDisplay = 1,
    /// all character positions of the page are put into the erased state
    EraseDisplay = 2,
    /// Clears the scrollback.  This is an Xterm extension to ECMA-48.
    EraseScrollback = 3,
}

impl ParamEnum for EraseInDisplay {
    fn default() -> Self {
        EraseInDisplay::EraseToEndOfDisplay
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sgr {
    /// Resets rendition to defaults.  Typically switches off
    /// all other Sgr options, but may have greater or lesser impact.
    Reset,
    /// Set the intensity/bold level
    Intensity(Intensity),
    Underline(Underline),
    UnderlineColor(ColorSpec),
    Blink(Blink),
    Italic(bool),
    Inverse(bool),
    Invisible(bool),
    StrikeThrough(bool),
    Font(Font),
    Foreground(ColorSpec),
    Background(ColorSpec),
    Overline(bool),
    VerticalAlign(VerticalAlign),
}

#[cfg(all(test, target_pointer_width = "64"))]
#[test]
fn sgr_size() {
    assert_eq!(core::mem::size_of::<Intensity>(), 1);
    assert_eq!(core::mem::size_of::<Underline>(), 1);
    assert_eq!(core::mem::size_of::<ColorSpec>(), 20);
    assert_eq!(core::mem::size_of::<Blink>(), 1);
    assert_eq!(core::mem::size_of::<Font>(), 2);
}
