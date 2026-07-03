macro_rules! osc_entries {
($(
    $( #[doc=$doc:expr] )*
    $label:ident = $value:expr
),* $(,)?) => {

#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, Hash, Copy)]
pub enum OperatingSystemCommandCode {
    $(
        $( #[doc=$doc] )*
        $label,
    )*
}

impl OscMap {
#[cfg(feature = "std")]
    fn new() -> Self {
        let mut code_to_variant = HashMap::new();
        let mut variant_to_code = HashMap::new();

        use OperatingSystemCommandCode::*;

        $(
            code_to_variant.insert($value, $label);
            variant_to_code.insert($label, $value);
        )*

        Self {
            code_to_variant,
            variant_to_code,
        }
    }

#[cfg(not(feature = "std"))]
    fn linear_search_code(code: &str) -> Option<OperatingSystemCommandCode> {
        use OperatingSystemCommandCode::*;
        match code {
        $(
            $value => Some($label),
        )*
            _ => None,
        }
    }

#[cfg(not(feature = "std"))]
    fn linear_search_variant(v: &OperatingSystemCommandCode) -> &'static str {
        use OperatingSystemCommandCode::*;
        match *v {
        $(
            $label => $value,
        )*
        }
    }

}
    };
}

osc_entries!(
    SetIconNameAndWindowTitle = "0",
    SetIconName = "1",
    SetWindowTitle = "2",
    SetXWindowProperty = "3",
    ChangeColorNumber = "4",
    ChangeSpecialColorNumber = "5",
    /// iTerm2
    ChangeTitleTabColor = "6",
    SetCurrentWorkingDirectory = "7",
    /// See https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda
    SetHyperlink = "8",
    /// iTerm2
    SystemNotification = "9",
    SetTextForegroundColor = "10",
    SetTextBackgroundColor = "11",
    SetTextCursorColor = "12",
    SetMouseForegroundColor = "13",
    SetMouseBackgroundColor = "14",
    SetTektronixForegroundColor = "15",
    SetTektronixBackgroundColor = "16",
    SetHighlightBackgroundColor = "17",
    SetTektronixCursorColor = "18",
    SetHighlightForegroundColor = "19",
    SetLogFileName = "46",
    SetFont = "50",
    EmacsShell = "51",
    ManipulateSelectionData = "52",
    ResetColors = "104",
    ResetSpecialColor = "105",
    ResetTextForegroundColor = "110",
    ResetTextBackgroundColor = "111",
    ResetTextCursorColor = "112",
    ResetMouseForegroundColor = "113",
    ResetMouseBackgroundColor = "114",
    ResetTektronixForegroundColor = "115",
    ResetTektronixBackgroundColor = "116",
    ResetHighlightColor = "117",
    ResetTektronixCursorColor = "118",
    ResetHighlightForegroundColor = "119",
    RxvtProprietary = "777",
    FinalTermSemanticPrompt = "133",
    ITermProprietary = "1337",
    /// Here the "Sun" suffix comes from the table in
    /// <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Miscellaneous>
    /// that lays out various window related escape sequences.
    SetWindowTitleSun = "l",
    SetIconNameSun = "L",
);

struct OscMap {
    #[cfg(feature = "std")]
    code_to_variant: HashMap<&'static str, OperatingSystemCommandCode>,
    #[cfg(feature = "std")]
    variant_to_code: HashMap<OperatingSystemCommandCode, &'static str>,
}

#[cfg(feature = "std")]
static OSC_MAP: LazyLock<OscMap> = LazyLock::new(OscMap::new);

#[cfg(feature = "std")]
impl OperatingSystemCommandCode {
    fn from_code(code: &str) -> Option<Self> {
        OSC_MAP.code_to_variant.get(code).copied()
    }

    fn as_code(self) -> &'static str {
        OSC_MAP.variant_to_code.get(&self).unwrap()
    }
}

#[cfg(not(feature = "std"))]
impl OperatingSystemCommandCode {
    fn from_code(code: &str) -> Option<Self> {
        OscMap::linear_search_code(code)
    }

    fn as_code(self) -> &'static str {
        OscMap::linear_search_variant(&self)
    }
}

impl Display for OperatingSystemCommand {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "\x1b]")?;

        macro_rules! single_string {
            ($variant:ident, $s:expr) => {{
                let code = OperatingSystemCommandCode::$variant.as_code();
                match OperatingSystemCommandCode::$variant {
                    OperatingSystemCommandCode::SetWindowTitleSun
                    | OperatingSystemCommandCode::SetIconNameSun => {
                        // For the legacy sun terminals, the `l` and `L` OSCs are
                        // not separated by `;`.
                        write!(f, "{}{}", code, $s)?;
                    }
                    _ => {
                        // In the common case, the OSC is numeric and is separated
                        // from the rest of the string
                        write!(f, "{};{}", code, $s)?;
                    }
                }
            }};
        }

        use self::OperatingSystemCommand::*;
        match self {
            SetIconNameAndWindowTitle(title) => single_string!(SetIconNameAndWindowTitle, title),
            SetWindowTitle(title) => single_string!(SetWindowTitle, title),
            SetWindowTitleSun(title) => single_string!(SetWindowTitleSun, title),
            SetIconName(title) => single_string!(SetIconName, title),
            SetIconNameSun(title) => single_string!(SetIconNameSun, title),
            SetHyperlink(Some(link)) => link.fmt(f)?,
            SetHyperlink(None) => write!(f, "8;;")?,
            RxvtExtension(params) => write!(f, "777;{}", params.join(";"))?,
            Unspecified(v) => {
                for (idx, item) in v.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ";")?;
                    }
                    f.write_str(&String::from_utf8_lossy(item))?;
                }
            }
            ClearSelection(s) => write!(f, "52;{}", s)?,
            QuerySelection(s) => write!(f, "52;{};?", s)?,
            SetSelection(s, val) => write!(f, "52;{};{}", s, base64_encode(val))?,
            SystemNotification(s) => write!(f, "9;{}", s)?,
            ITermProprietary(i) => i.fmt(f)?,
            FinalTermSemanticPrompt(i) => i.fmt(f)?,
            ResetColors(colors) => {
                write!(f, "104")?;
                for c in colors {
                    write!(f, ";{}", c)?;
                }
            }
            ChangeColorNumber(specs) => {
                write!(f, "4;")?;
                for pair in specs {
                    write!(f, "{};{}", pair.palette_index, pair.color)?
                }
            }
            ChangeDynamicColors(first_color, colors) => {
                write!(f, "{}", *first_color as u8)?;
                for color in colors {
                    write!(f, ";{}", color)?
                }
            }
            ResetDynamicColor(color) => {
                write!(f, "{}", 100 + *color as u8)?;
            }
            CurrentWorkingDirectory(s) => write!(f, "7;{}", s)?,
            ConEmuProgress(Progress::None) => write!(f, "9;4;0")?,
            ConEmuProgress(Progress::SetPercentage(pct)) => write!(f, "9;4;1;{pct}")?,
            ConEmuProgress(Progress::SetError(pct)) => write!(f, "9;4;2;{pct}")?,
            ConEmuProgress(Progress::SetIndeterminate) => write!(f, "9;4;3")?,
            ConEmuProgress(Progress::Paused) => write!(f, "9;4;4")?,
        };
        // Use the longer form ST as neovim doesn't like the BEL version
        write!(f, "\x1b\\")?;
        Ok(())
    }
}
