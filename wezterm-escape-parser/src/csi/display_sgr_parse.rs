impl Display for Sgr {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        macro_rules! code {
            ($t:ident) => {
                write!(f, "{}m", SgrCode::$t as i64)?
            };
        }

        macro_rules! ansi_color {
            ($idx:expr, $eightbit:ident, $( ($Ansi:ident, $code:ident) ),*) => {
                if let Some(ansi) = FromPrimitive::from_u8($idx) {
                    match ansi {
                        $(AnsiColor::$Ansi => code!($code) ,)*
                    }
                } else {
                    write!(f, "{}:5:{}m", SgrCode::$eightbit as i64, $idx)?
                }
            }
        }

        match self {
            Sgr::Reset => code!(Reset),
            Sgr::Intensity(Intensity::Bold) => code!(IntensityBold),
            Sgr::Intensity(Intensity::Half) => code!(IntensityDim),
            Sgr::Intensity(Intensity::Normal) => code!(NormalIntensity),
            Sgr::Underline(Underline::Single) => code!(UnderlineOn),
            Sgr::Underline(Underline::Double) => code!(UnderlineDouble),
            Sgr::Underline(Underline::Curly) => write!(f, "4:3m")?,
            Sgr::Underline(Underline::Dotted) => write!(f, "4:4m")?,
            Sgr::Underline(Underline::Dashed) => write!(f, "4:5m")?,
            Sgr::Underline(Underline::None) => code!(UnderlineOff),
            Sgr::Blink(Blink::Slow) => code!(BlinkOn),
            Sgr::Blink(Blink::Rapid) => code!(RapidBlinkOn),
            Sgr::Blink(Blink::None) => code!(BlinkOff),
            Sgr::Italic(true) => code!(ItalicOn),
            Sgr::Italic(false) => code!(ItalicOff),
            Sgr::Inverse(true) => code!(InverseOn),
            Sgr::Inverse(false) => code!(InverseOff),
            Sgr::Invisible(true) => code!(InvisibleOn),
            Sgr::Invisible(false) => code!(InvisibleOff),
            Sgr::StrikeThrough(true) => code!(StrikeThroughOn),
            Sgr::StrikeThrough(false) => code!(StrikeThroughOff),
            Sgr::Overline(true) => code!(OverlineOn),
            Sgr::Overline(false) => code!(OverlineOff),
            Sgr::VerticalAlign(VerticalAlign::BaseLine) => code!(VerticalAlignBaseLine),
            Sgr::VerticalAlign(VerticalAlign::SuperScript) => code!(VerticalAlignSuperScript),
            Sgr::VerticalAlign(VerticalAlign::SubScript) => code!(VerticalAlignSubScript),
            Sgr::Font(Font::Default) => code!(DefaultFont),
            Sgr::Font(Font::Alternate(1)) => code!(AltFont1),
            Sgr::Font(Font::Alternate(2)) => code!(AltFont2),
            Sgr::Font(Font::Alternate(3)) => code!(AltFont3),
            Sgr::Font(Font::Alternate(4)) => code!(AltFont4),
            Sgr::Font(Font::Alternate(5)) => code!(AltFont5),
            Sgr::Font(Font::Alternate(6)) => code!(AltFont6),
            Sgr::Font(Font::Alternate(7)) => code!(AltFont7),
            Sgr::Font(Font::Alternate(8)) => code!(AltFont8),
            Sgr::Font(Font::Alternate(9)) => code!(AltFont9),
            Sgr::Font(_) => { /* there are no other possible font values */ }
            Sgr::Foreground(ColorSpec::Default) => code!(ForegroundDefault),
            Sgr::Background(ColorSpec::Default) => code!(BackgroundDefault),
            Sgr::Foreground(ColorSpec::PaletteIndex(idx)) => ansi_color!(
                *idx,
                ForegroundColor,
                (Black, ForegroundBlack),
                (Maroon, ForegroundRed),
                (Green, ForegroundGreen),
                (Olive, ForegroundYellow),
                (Navy, ForegroundBlue),
                (Purple, ForegroundMagenta),
                (Teal, ForegroundCyan),
                (Silver, ForegroundWhite),
                // Note: these brights are emitted using codes in the 100 range.
                // I don't know how portable this is vs. the 256 color sequences,
                // so we may need to make an adjustment here later.
                (Grey, ForegroundBrightBlack),
                (Red, ForegroundBrightRed),
                (Lime, ForegroundBrightGreen),
                (Yellow, ForegroundBrightYellow),
                (Blue, ForegroundBrightBlue),
                (Fuchsia, ForegroundBrightMagenta),
                (Aqua, ForegroundBrightCyan),
                (White, ForegroundBrightWhite)
            ),
            Sgr::Foreground(ColorSpec::TrueColor(c)) => {
                let (red, green, blue, alpha) = c.to_srgb_u8();
                if alpha == 255 {
                    write!(
                        f,
                        "{}:2::{}:{}:{}m",
                        SgrCode::ForegroundColor as i64,
                        red,
                        green,
                        blue
                    )?
                } else {
                    write!(
                        f,
                        "{}:6::{}:{}:{}:{}m",
                        SgrCode::ForegroundColor as i64,
                        red,
                        green,
                        blue,
                        alpha
                    )?
                }
            }
            Sgr::Background(ColorSpec::PaletteIndex(idx)) => ansi_color!(
                *idx,
                BackgroundColor,
                (Black, BackgroundBlack),
                (Maroon, BackgroundRed),
                (Green, BackgroundGreen),
                (Olive, BackgroundYellow),
                (Navy, BackgroundBlue),
                (Purple, BackgroundMagenta),
                (Teal, BackgroundCyan),
                (Silver, BackgroundWhite),
                // Note: these brights are emitted using codes in the 100 range.
                // I don't know how portable this is vs. the 256 color sequences,
                // so we may need to make an adjustment here later.
                (Grey, BackgroundBrightBlack),
                (Red, BackgroundBrightRed),
                (Lime, BackgroundBrightGreen),
                (Yellow, BackgroundBrightYellow),
                (Blue, BackgroundBrightBlue),
                (Fuchsia, BackgroundBrightMagenta),
                (Aqua, BackgroundBrightCyan),
                (White, BackgroundBrightWhite)
            ),
            Sgr::Background(ColorSpec::TrueColor(c)) => {
                let (red, green, blue, alpha) = c.to_srgb_u8();
                if alpha == 255 {
                    write!(
                        f,
                        "{}:2::{}:{}:{}m",
                        SgrCode::BackgroundColor as i64,
                        red,
                        green,
                        blue
                    )?
                } else {
                    write!(
                        f,
                        "{}:6::{}:{}:{}:{}m",
                        SgrCode::BackgroundColor as i64,
                        red,
                        green,
                        blue,
                        alpha
                    )?
                }
            }
            Sgr::UnderlineColor(ColorSpec::Default) => code!(ResetUnderlineColor),
            Sgr::UnderlineColor(ColorSpec::TrueColor(c)) => {
                let (red, green, blue, alpha) = c.to_srgb_u8();
                if alpha == 255 {
                    write!(
                        f,
                        "{}:2::{}:{}:{}m",
                        SgrCode::UnderlineColor as i64,
                        red,
                        green,
                        blue
                    )?
                } else {
                    write!(
                        f,
                        "{}:6::{}:{}:{}:{}m",
                        SgrCode::UnderlineColor as i64,
                        red,
                        green,
                        blue,
                        alpha
                    )?
                }
            }
            Sgr::UnderlineColor(ColorSpec::PaletteIndex(idx)) => {
                write!(f, "{}:5:{}m", SgrCode::UnderlineColor as i64, *idx)?
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Font {
    Default,
    Alternate(u8),
}

/// Constrol Sequence Initiator (CSI) Parser.
/// Since many sequences allow for composition of actions by separating
/// `;` character, we need to be able to iterate over
/// the set of parsed actions from a given CSI sequence.
/// `CSIParser` implements an Iterator that yields `CSI` instances as
/// it parses them out from the input sequence.
struct CSIParser<'a> {
    /// this flag is set when more than two intermediates
    /// arrived and subsequent characters were ignored.
    parameters_truncated: bool,
    control: char,
    /// While params is_some we have more data to consume.  The advance_by
    /// method updates the slice as we consume data.
    /// In a number of cases an empty params list is used to indicate
    /// default values, especially for SGR, so we need to be careful not
    /// to update params to an empty slice.
    params: Option<&'a [CsiParam]>,
    orig_params: &'a [CsiParam],
}

impl CSI {
    /// Parse a CSI sequence.
    /// Returns an iterator that yields individual CSI actions.
    /// Why not a single?  Because sequences like `CSI [ 1 ; 3 m`
    /// embed two separate actions but are sent as a single unit.
    /// If no semantic meaning is known for a subsequence, the remainder
    /// of the sequence is returned wrapped in a `CSI::Unspecified` container.
    pub fn parse<'a>(
        params: &'a [CsiParam],
        parameters_truncated: bool,
        control: char,
    ) -> impl Iterator<Item = CSI> + 'a {
        CSIParser {
            parameters_truncated,
            control,
            params: Some(params),
            orig_params: params,
        }
    }
}

/// A little helper to convert i64 -> u8 if safe
fn to_u8(v: &CsiParam) -> Result<u8, ()> {
    match v {
        CsiParam::P(_) => Err(()),
        CsiParam::Integer(v) => {
            if *v <= i64::from(u8::max_value()) {
                Ok(*v as u8)
            } else {
                Err(())
            }
        }
    }
}

/// Convert the input value to 1-based u32.
/// The intent is to protect consumers from out of range values
/// when operating on the data, while balancing strictness with
/// practical implementation bugs.  For example, it is common
/// to see 0 values being emitted from existing libraries, and
/// we desire to see the intended output.
/// Ensures that the value is in the range 1..=max_value.
/// If the input is 0 it is treated as 1.  If the value is
/// otherwise outside that range, an error is propagated and
/// that will typically case the sequence to be reported via
/// the Unspecified placeholder.
fn to_1b_u32(v: &CsiParam) -> Result<u32, ()> {
    match v {
        CsiParam::Integer(v) if *v == 0 => Ok(1),
        CsiParam::Integer(v) if *v > 0 && *v <= i64::from(u32::max_value()) => Ok(*v as u32),
        _ => Err(()),
    }
}

struct Cracked {
    params: Vec<Option<CsiParam>>,
}

impl Cracked {
    pub fn parse(params: &[CsiParam]) -> Result<Self, ()> {
        let mut res = vec![];
        let mut iter = params.iter().peekable();
        while let Some(p) = iter.next() {
            match p {
                CsiParam::P(b';') => {
                    res.push(None);
                }
                CsiParam::Integer(_) => {
                    res.push(Some(p.clone()));
                    if let Some(CsiParam::P(b';')) = iter.peek() {
                        iter.next();
                    }
                }
                _ => return Err(()),
            }
        }
        Ok(Self { params: res })
    }

    pub fn get(&self, idx: usize) -> Option<&CsiParam> {
        self.params.get(idx)?.as_ref()
    }

    pub fn opt_int(&self, idx: usize) -> Option<i64> {
        self.get(idx).and_then(CsiParam::as_integer)
    }

    pub fn int(&self, idx: usize) -> Result<i64, ()> {
        self.get(idx).and_then(CsiParam::as_integer).ok_or(())
    }

    pub fn len(&self) -> usize {
        self.params.len()
    }
}
