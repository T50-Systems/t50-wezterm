#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ITermDimension {
    Automatic,
    Cells(i64),
    Pixels(i64),
    Percent(i64),
}

impl Default for ITermDimension {
    fn default() -> Self {
        Self::Automatic
    }
}

impl Display for ITermDimension {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use self::ITermDimension::*;
        match self {
            Automatic => write!(f, "auto"),
            Cells(n) => write!(f, "{}", n),
            Pixels(n) => write!(f, "{}px", n),
            Percent(n) => write!(f, "{}%", n),
        }
    }
}

impl core::str::FromStr for ITermDimension {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self> {
        ITermDimension::parse(s)
    }
}

impl ITermDimension {
    fn parse(s: &str) -> Result<Self> {
        if s == "auto" {
            Ok(ITermDimension::Automatic)
        } else if s.ends_with("px") {
            let s = &s[..s.len() - 2];
            let num = s.parse()?;
            Ok(ITermDimension::Pixels(num))
        } else if s.ends_with('%') {
            let s = &s[..s.len() - 1];
            let num = s.parse()?;
            Ok(ITermDimension::Percent(num))
        } else {
            let num = s.parse()?;
            Ok(ITermDimension::Cells(num))
        }
    }

    /// Convert the dimension into a number of pixels based on the provided
    /// size of a cell and number of cells in that dimension.
    /// Returns None for the Automatic variant.
    pub fn to_pixels(&self, cell_size: usize, num_cells: usize) -> Option<usize> {
        match self {
            ITermDimension::Automatic => None,
            ITermDimension::Cells(n) => Some((*n).max(0) as usize * cell_size),
            ITermDimension::Pixels(n) => Some((*n).max(0) as usize),
            ITermDimension::Percent(n) => Some(
                (((*n).max(0).min(100) as f32 / 100.0) * num_cells as f32 * cell_size as f32)
                    as usize,
            ),
        }
    }
}

impl ITermProprietary {
    #[allow(clippy::cyclomatic_complexity, clippy::cognitive_complexity)]
    fn parse(osc: &[&[u8]]) -> Result<Self> {
        // iTerm has a number of different styles of OSC parameter
        // encodings, which makes this section of code a bit gnarly.
        ensure!(osc.len() > 1, "not enough args");

        let param = String::from_utf8_lossy(osc[1]);

        let mut iter = param.splitn(2, '=');
        let keyword = iter.next().ok_or_else(|| format!("bad params"))?;
        let p1 = iter.next();

        macro_rules! single {
            ($variant:ident, $text:expr) => {
                if osc.len() == 2 && keyword == $text && p1.is_none() {
                    return Ok(ITermProprietary::$variant);
                }
            };
        }

        macro_rules! one_str {
            ($variant:ident, $text:expr) => {
                if osc.len() == 2 && keyword == $text {
                    if let Some(p1) = p1 {
                        return Ok(ITermProprietary::$variant(p1.into()));
                    }
                }
            };
        }
        macro_rules! const_arg {
            ($variant:ident, $text:expr, $value:expr, $res:expr) => {
                if osc.len() == 2 && keyword == $text {
                    if let Some(p1) = p1 {
                        if p1 == $value {
                            return Ok(ITermProprietary::$variant($res));
                        }
                    }
                }
            };
        }

        single!(SetMark, "SetMark");
        single!(StealFocus, "StealFocus");
        single!(ClearScrollback, "ClearScrollback");
        single!(EndCopy, "EndCopy");
        single!(RequestCellSize, "ReportCellSize");
        const_arg!(HighlightCursorLine, "HighlightCursorLine", "yes", true);
        const_arg!(HighlightCursorLine, "HighlightCursorLine", "no", false);
        one_str!(CurrentDir, "CurrentDir");
        one_str!(SetProfile, "SetProfile");
        one_str!(CopyToClipboard, "CopyToClipboard");

        let p1_empty = match p1 {
            Some(p1) if p1 == "" => true,
            None => true,
            _ => false,
        };

        if osc.len() == 3 && keyword == "Copy" && p1_empty {
            return Ok(ITermProprietary::Copy(String::from_utf8(base64_decode(
                osc[2],
            )?)?));
        }
        if osc.len() == 3 && keyword == "SetBadgeFormat" && p1_empty {
            return Ok(ITermProprietary::SetBadgeFormat(String::from_utf8(
                base64_decode(osc[2])?,
            )?));
        }

        if osc.len() == 3 && keyword == "ReportCellSize" && p1.is_some() {
            if let Some(p1) = p1 {
                return Ok(ITermProprietary::ReportCellSize {
                    height_pixels: NotNan::new(p1.parse()?).map_err(not_nan_err)?,
                    width_pixels: NotNan::new(String::from_utf8_lossy(osc[2]).parse()?)
                        .map_err(not_nan_err)?,
                    scale: None,
                });
            }
        }
        if osc.len() == 4 && keyword == "ReportCellSize" && p1.is_some() {
            if let Some(p1) = p1 {
                return Ok(ITermProprietary::ReportCellSize {
                    height_pixels: NotNan::new(p1.parse()?).map_err(not_nan_err)?,
                    width_pixels: NotNan::new(String::from_utf8_lossy(osc[2]).parse()?)
                        .map_err(not_nan_err)?,
                    scale: Some(
                        NotNan::new(String::from_utf8_lossy(osc[3]).parse()?)
                            .map_err(not_nan_err)?,
                    ),
                });
            }
        }

        if osc.len() == 2 && keyword == "SetUserVar" {
            if let Some(p1) = p1 {
                let mut iter = p1.splitn(2, '=');
                let p1 = iter.next();
                let p2 = iter.next();

                if let (Some(k), Some(v)) = (p1, p2) {
                    return Ok(ITermProprietary::SetUserVar {
                        name: k.to_string(),
                        value: String::from_utf8(base64_decode(v)?)?,
                    });
                }
            }
        }

        if osc.len() == 2 && keyword == "UnicodeVersion" {
            if let Some(p1) = p1 {
                let mut iter = p1.splitn(2, ' ');
                let keyword = iter.next();
                let label = iter.next();

                if let Some("push") = keyword {
                    return Ok(ITermProprietary::UnicodeVersion(
                        ITermUnicodeVersionOp::Push(label.map(|s| s.to_string())),
                    ));
                }
                if let Some("pop") = keyword {
                    return Ok(ITermProprietary::UnicodeVersion(
                        ITermUnicodeVersionOp::Pop(label.map(|s| s.to_string())),
                    ));
                }

                if let Ok(n) = p1.parse::<u8>() {
                    return Ok(ITermProprietary::UnicodeVersion(
                        ITermUnicodeVersionOp::Set(n),
                    ));
                }
            }
        }

        if keyword == "File" {
            return Ok(ITermProprietary::File(Box::new(ITermFileData::parse(osc)?)));
        }

        bail!("ITermProprietary {:?}", osc);
    }
}

/// base64::encode is deprecated, so make a less frustrating helper
pub(crate) fn base64_encode<T: AsRef<[u8]>>(s: T) -> String {
    base64::engine::general_purpose::STANDARD.encode(s)
}

/// base64::decode is deprecated, so make a less frustrating helper
pub(crate) fn base64_decode<T: AsRef<[u8]>>(s: T) -> Result<Vec<u8>> {
    use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
    GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        GeneralPurposeConfig::new().with_decode_allow_trailing_bits(true),
    )
    .decode(s)
    .map_err(|err| crate::format_err!("base64_decode: {:#}", err))
}

impl Display for ITermProprietary {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "1337;")?;
        use self::ITermProprietary::*;
        match self {
            SetMark => write!(f, "SetMark")?,
            StealFocus => write!(f, "StealFocus")?,
            ClearScrollback => write!(f, "ClearScrollback")?,
            CurrentDir(s) => write!(f, "CurrentDir={}", s)?,
            SetProfile(s) => write!(f, "SetProfile={}", s)?,
            CopyToClipboard(s) => write!(f, "CopyToClipboard={}", s)?,
            EndCopy => write!(f, "EndCopy")?,
            HighlightCursorLine(yes) => {
                write!(f, "HighlightCursorLine={}", if *yes { "yes" } else { "no" })?
            }
            RequestCellSize => write!(f, "ReportCellSize")?,
            ReportCellSize {
                height_pixels,
                width_pixels,
                scale: None,
            } => write!(f, "ReportCellSize={height_pixels:.1};{width_pixels:.1}")?,
            ReportCellSize {
                height_pixels,
                width_pixels,
                scale: Some(scale),
            } => write!(
                f,
                "ReportCellSize={height_pixels:.1};{width_pixels:.1};{scale:.1}",
            )?,
            Copy(s) => write!(f, "Copy=;{}", base64_encode(s))?,
            ReportVariable(s) => write!(f, "ReportVariable={}", base64_encode(s))?,
            SetUserVar { name, value } => {
                write!(f, "SetUserVar={}={}", name, base64_encode(value))?
            }
            SetBadgeFormat(s) => write!(f, "SetBadgeFormat={}", base64_encode(s))?,
            File(file) => file.fmt(f)?,
            UnicodeVersion(ITermUnicodeVersionOp::Set(n)) => write!(f, "UnicodeVersion={}", n)?,
            UnicodeVersion(ITermUnicodeVersionOp::Push(Some(label))) => {
                write!(f, "UnicodeVersion=push {}", label)?
            }
            UnicodeVersion(ITermUnicodeVersionOp::Push(None)) => write!(f, "UnicodeVersion=push")?,
            UnicodeVersion(ITermUnicodeVersionOp::Pop(Some(label))) => {
                write!(f, "UnicodeVersion=pop {}", label)?
            }
            UnicodeVersion(ITermUnicodeVersionOp::Pop(None)) => write!(f, "UnicodeVersion=pop")?,
        }
        Ok(())
    }
}

fn not_nan_err(err: ordered_float::FloatIsNan) -> crate::Error {
    format_err!("{:#}", err)
}
