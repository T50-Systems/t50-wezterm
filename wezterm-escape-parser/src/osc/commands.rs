#[derive(Debug, Clone, PartialEq)]
pub enum ColorOrQuery {
    Color(SrgbaTuple),
    Query,
}

impl Display for ColorOrQuery {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            ColorOrQuery::Query => write!(f, "?"),
            ColorOrQuery::Color(c) => write!(f, "{}", c.to_x11_16bit_rgb_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperatingSystemCommand {
    SetIconNameAndWindowTitle(String),
    SetWindowTitle(String),
    SetWindowTitleSun(String),
    SetIconName(String),
    SetIconNameSun(String),
    SetHyperlink(Option<Hyperlink>),
    ClearSelection(Selection),
    QuerySelection(Selection),
    SetSelection(Selection, String),
    SystemNotification(String),
    ITermProprietary(ITermProprietary),
    FinalTermSemanticPrompt(FinalTermSemanticPrompt),
    ChangeColorNumber(Vec<ChangeColorPair>),
    ChangeDynamicColors(DynamicColorNumber, Vec<ColorOrQuery>),
    ResetDynamicColor(DynamicColorNumber),
    CurrentWorkingDirectory(String),
    ResetColors(Vec<u8>),
    RxvtExtension(Vec<String>),
    ConEmuProgress(Progress),

    Unspecified(Vec<Vec<u8>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[repr(u8)]
pub enum DynamicColorNumber {
    TextForegroundColor = 10,
    TextBackgroundColor = 11,
    TextCursorColor = 12,
    MouseForegroundColor = 13,
    MouseBackgroundColor = 14,
    TektronixForegroundColor = 15,
    TektronixBackgroundColor = 16,
    HighlightBackgroundColor = 17,
    TektronixCursorColor = 18,
    HighlightForegroundColor = 19,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChangeColorPair {
    pub palette_index: u8,
    pub color: ColorOrQuery,
}

bitflags! {
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection :u16{
    const NONE = 0;
    const CLIPBOARD = 1<<1;
    const PRIMARY=1<<2;
    const SELECT=1<<3;
    const CUT0=1<<4;
    const CUT1=1<<5;
    const CUT2=1<<6;
    const CUT3=1<<7;
    const CUT4=1<<8;
    const CUT5=1<<9;
    const CUT6=1<<10;
    const CUT7=1<<11;
    const CUT8=1<<12;
    const CUT9=1<<13;
}
}

impl Selection {
    fn try_parse(buf: &[u8]) -> Result<Selection> {
        if buf == b"" {
            Ok(Selection::SELECT | Selection::CUT0)
        } else {
            let mut s = Selection::NONE;
            for c in buf {
                s |= match c {
                    b'c' => Selection::CLIPBOARD,
                    b'p' => Selection::PRIMARY,
                    b's' => Selection::SELECT,
                    b'0' => Selection::CUT0,
                    b'1' => Selection::CUT1,
                    b'2' => Selection::CUT2,
                    b'3' => Selection::CUT3,
                    b'4' => Selection::CUT4,
                    b'5' => Selection::CUT5,
                    b'6' => Selection::CUT6,
                    b'7' => Selection::CUT7,
                    b'8' => Selection::CUT8,
                    b'9' => Selection::CUT9,
                    _ => bail!("invalid selection {:?}", buf),
                }
            }
            Ok(s)
        }
    }
}

impl Display for Selection {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        macro_rules! item {
            ($variant:ident, $s:expr) => {
                if (*self & Selection::$variant) != Selection::NONE {
                    write!(f, $s)?;
                }
            };
        }

        item!(CLIPBOARD, "c");
        item!(PRIMARY, "p");
        item!(SELECT, "s");
        item!(CUT0, "0");
        item!(CUT1, "1");
        item!(CUT2, "2");
        item!(CUT3, "3");
        item!(CUT4, "4");
        item!(CUT5, "5");
        item!(CUT6, "6");
        item!(CUT7, "7");
        item!(CUT8, "8");
        item!(CUT9, "9");
        Ok(())
    }
}

impl OperatingSystemCommand {
    pub fn parse(osc: &[&[u8]]) -> Self {
        Self::internal_parse(osc).unwrap_or_else(|err| {
            let mut vec = Vec::new();
            for slice in osc {
                vec.push(slice.to_vec());
            }
            log::trace!(
                "OSC internal parse err: {}, track as Unspecified {:?}",
                err,
                vec
            );
            OperatingSystemCommand::Unspecified(vec)
        })
    }

    fn parse_selection(osc: &[&[u8]]) -> Result<Self> {
        if osc.len() == 2 {
            Selection::try_parse(osc[1]).map(OperatingSystemCommand::ClearSelection)
        } else if osc.len() == 3 && osc[2] == b"?" {
            Selection::try_parse(osc[1]).map(OperatingSystemCommand::QuerySelection)
        } else if osc.len() == 3 {
            let sel = Selection::try_parse(osc[1])?;
            let bytes = base64_decode(osc[2])?;
            let s = String::from_utf8(bytes)?;
            Ok(OperatingSystemCommand::SetSelection(sel, s))
        } else {
            bail!("unhandled OSC 52: {:?}", osc);
        }
    }

    fn parse_reset_colors(osc: &[&[u8]]) -> Result<Self> {
        let mut colors = vec![];
        let mut iter = osc.iter();
        iter.next(); // skip the command word that we already know is present

        while let Some(index) = iter.next() {
            if index.is_empty() {
                continue;
            }
            let index: u8 = str::from_utf8(index)?.parse()?;
            colors.push(index);
        }

        Ok(OperatingSystemCommand::ResetColors(colors))
    }

    fn parse_change_color_number(osc: &[&[u8]]) -> Result<Self> {
        let mut pairs = vec![];
        let mut iter = osc.iter();
        iter.next(); // skip the command word that we already know is present

        while let (Some(index), Some(spec)) = (iter.next(), iter.next()) {
            let index: u8 = str::from_utf8(index)?.parse()?;
            let spec = str::from_utf8(spec)?;
            let spec = if spec == "?" {
                ColorOrQuery::Query
            } else {
                ColorOrQuery::Color(
                    SrgbaTuple::from_str(spec)
                        .map_err(|()| format!("invalid color spec {:?}", spec))?,
                )
            };

            pairs.push(ChangeColorPair {
                palette_index: index,
                color: spec,
            });
        }

        Ok(OperatingSystemCommand::ChangeColorNumber(pairs))
    }

    fn parse_reset_dynamic_color_number(idx: u8) -> Result<Self> {
        let which_color: DynamicColorNumber = FromPrimitive::from_u8(idx)
            .ok_or_else(|| format!("osc code is not a valid DynamicColorNumber!?"))?;

        Ok(OperatingSystemCommand::ResetDynamicColor(which_color))
    }

    fn parse_change_dynamic_color_number(idx: u8, osc: &[&[u8]]) -> Result<Self> {
        let which_color: DynamicColorNumber = FromPrimitive::from_u8(idx)
            .ok_or_else(|| format!("osc code is not a valid DynamicColorNumber!?"))?;
        let mut colors = vec![];
        for spec in osc.iter().skip(1) {
            if spec == b"?" {
                colors.push(ColorOrQuery::Query);
            } else {
                let spec = str::from_utf8(spec)?;
                colors.push(ColorOrQuery::Color(
                    SrgbaTuple::from_str(spec)
                        .map_err(|()| format!("invalid color spec {:?}", spec))?,
                ));
            }
        }

        Ok(OperatingSystemCommand::ChangeDynamicColors(
            which_color,
            colors,
        ))
    }

    fn internal_parse(osc: &[&[u8]]) -> Result<Self> {
        ensure!(!osc.is_empty(), "no params");
        let p1str = String::from_utf8_lossy(osc[0]);

        if p1str.is_empty() {
            bail!("zero length osc");
        }

        // Ugh, this is to handle "OSC ltitle" which is a legacyish
        // OSC for encoding a window title change request.  These days
        // OSC 2 is preferred for this purpose, but we need to support
        // generating and parsing the legacy form because it is the
        // response for the CSI ReportWindowTitle.
        // So, for non-numeric OSCs, we look up the prefix and use that.
        // This only works if the non-numeric OSC code has length == 1.
        let osc_code = if !p1str.chars().nth(0).unwrap().is_ascii_digit() && osc.len() == 1 {
            let mut p1 = String::new();
            p1.push(p1str.chars().nth(0).unwrap());
            OperatingSystemCommandCode::from_code(&p1)
        } else {
            OperatingSystemCommandCode::from_code(&p1str)
        }
        .ok_or_else(|| format!("unknown code"))?;

        macro_rules! single_string {
            ($variant:ident) => {{
                if osc.len() != 2 {
                    bail!("wrong param count");
                }
                let s = String::from_utf8(osc[1].to_vec())?;
                Ok(OperatingSystemCommand::$variant(s))
            }};
        }

        macro_rules! single_title_string {
            ($variant:ident) => {{
                if osc.len() < 2 {
                    bail!("wrong param count");
                }
                let mut s = String::from_utf8(osc[1].to_vec())?;
                for i in 2..osc.len() {
                    s = [s, String::from_utf8(osc[i].to_vec())?].join(";");
                }

                Ok(OperatingSystemCommand::$variant(s))
            }};
        }

        use self::OperatingSystemCommandCode::*;
        match osc_code {
            SetIconNameAndWindowTitle => single_title_string!(SetIconNameAndWindowTitle),
            SetWindowTitle => single_title_string!(SetWindowTitle),
            SetWindowTitleSun => Ok(OperatingSystemCommand::SetWindowTitleSun(
                p1str[1..].to_owned(),
            )),

            SetIconName => single_title_string!(SetIconName),
            SetIconNameSun => Ok(OperatingSystemCommand::SetIconNameSun(
                p1str[1..].to_owned(),
            )),
            SetHyperlink => Ok(OperatingSystemCommand::SetHyperlink(Hyperlink::parse(osc)?)),
            ManipulateSelectionData => Self::parse_selection(osc),
            SystemNotification => {
                if osc.len() >= 3 && osc[1] == b"4" {
                    fn get_pct(v: &&[u8]) -> u8 {
                        let number = str::from_utf8(v).unwrap_or("0");
                        number.parse::<u8>().unwrap_or(0).max(0).min(100)
                    }
                    match osc[2] {
                        b"0" => return Ok(OperatingSystemCommand::ConEmuProgress(Progress::None)),
                        b"1" => {
                            let pct = osc.get(3).map(get_pct).unwrap_or(0);
                            return Ok(OperatingSystemCommand::ConEmuProgress(
                                Progress::SetPercentage(pct),
                            ));
                        }
                        b"2" => {
                            let pct = osc.get(3).map(get_pct).unwrap_or(0);
                            return Ok(OperatingSystemCommand::ConEmuProgress(Progress::SetError(
                                pct,
                            )));
                        }
                        b"3" => {
                            return Ok(OperatingSystemCommand::ConEmuProgress(
                                Progress::SetIndeterminate,
                            ));
                        }
                        b"4" => {
                            return Ok(OperatingSystemCommand::ConEmuProgress(Progress::Paused));
                        }
                        _ => {}
                    }
                }
                single_string!(SystemNotification)
            }
            SetCurrentWorkingDirectory => single_string!(CurrentWorkingDirectory),
            ITermProprietary => {
                self::ITermProprietary::parse(osc).map(OperatingSystemCommand::ITermProprietary)
            }
            RxvtProprietary => {
                let mut vec = vec![];
                for slice in osc.iter().skip(1) {
                    vec.push(String::from_utf8_lossy(slice).to_string());
                }
                Ok(OperatingSystemCommand::RxvtExtension(vec))
            }
            FinalTermSemanticPrompt => self::FinalTermSemanticPrompt::parse(osc)
                .map(OperatingSystemCommand::FinalTermSemanticPrompt),
            ChangeColorNumber => Self::parse_change_color_number(osc),
            ResetColors => Self::parse_reset_colors(osc),

            ResetSpecialColor
            | ResetTextForegroundColor
            | ResetTextBackgroundColor
            | ResetTextCursorColor
            | ResetMouseForegroundColor
            | ResetMouseBackgroundColor
            | ResetTektronixForegroundColor
            | ResetTektronixBackgroundColor
            | ResetHighlightColor
            | ResetTektronixCursorColor
            | ResetHighlightForegroundColor => Self::parse_reset_dynamic_color_number(
                p1str.parse::<u8>().unwrap().saturating_sub(100),
            ),

            SetTextForegroundColor
            | SetTextBackgroundColor
            | SetTextCursorColor
            | SetMouseForegroundColor
            | SetMouseBackgroundColor
            | SetTektronixForegroundColor
            | SetTektronixBackgroundColor
            | SetHighlightBackgroundColor
            | SetTektronixCursorColor
            | SetHighlightForegroundColor => {
                Self::parse_change_dynamic_color_number(p1str.parse::<u8>().unwrap(), osc)
            }

            osc_code => bail!("{:?} not impl", osc_code),
        }
    }
}
