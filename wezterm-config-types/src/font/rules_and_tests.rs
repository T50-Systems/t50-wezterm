/// Defines a rule that can be used to select a `TextStyle` given
/// an input `CellAttributes` value.  The logic that applies the
/// matching can be found in src/font/mod.rs.  The concept is that
/// the user can specify something like this:
///
/// ```toml
/// [[font_rules]]
/// italic = true
/// font = { font = [{family = "Operator Mono SSm Lig", italic=true}]}
/// ```
///
/// The above is translated as: "if the `CellAttributes` have the italic bit
/// set, then use the italic style of font rather than the default", and
/// stop processing further font rules.
#[derive(Debug, Default, Clone, FromDynamic, ToDynamic)]
pub struct StyleRule {
    /// If present, this rule matches when CellAttributes::intensity holds
    /// a value that matches this rule.  Valid values are "Bold", "Normal",
    /// "Half".
    pub intensity: Option<wezterm_term_api::Intensity>,
    /// If present, this rule matches when CellAttributes::underline holds
    /// a value that matches this rule.  Valid values are "None", "Single",
    /// "Double".
    pub underline: Option<wezterm_term_api::Underline>,
    /// If present, this rule matches when CellAttributes::italic holds
    /// a value that matches this rule.
    pub italic: Option<bool>,
    /// If present, this rule matches when CellAttributes::blink holds
    /// a value that matches this rule.
    pub blink: Option<wezterm_term_api::Blink>,
    /// If present, this rule matches when CellAttributes::reverse holds
    /// a value that matches this rule.
    pub reverse: Option<bool>,
    /// If present, this rule matches when CellAttributes::strikethrough holds
    /// a value that matches this rule.
    pub strikethrough: Option<bool>,
    /// If present, this rule matches when CellAttributes::invisible holds
    /// a value that matches this rule.
    pub invisible: Option<bool>,

    /// When this rule matches, `font` specifies the styling to be used.
    pub font: TextStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromDynamic, ToDynamic)]
pub enum AllowSquareGlyphOverflow {
    Never,
    Always,
    WhenFollowedBySpace,
}

impl Default for AllowSquareGlyphOverflow {
    fn default() -> Self {
        Self::WhenFollowedBySpace
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromDynamic, ToDynamic)]
pub enum FontLocatorSelection {
    /// Use fontconfig APIs to resolve fonts (!macos, posix systems)
    FontConfig,
    /// Use GDI on win32 systems
    Gdi,
    /// Use CoreText on macOS
    CoreText,
    /// Use only the font_dirs configuration to locate fonts
    ConfigDirsOnly,
}

impl Default for FontLocatorSelection {
    fn default() -> Self {
        if cfg!(windows) {
            FontLocatorSelection::Gdi
        } else if cfg!(target_os = "macos") {
            FontLocatorSelection::CoreText
        } else {
            FontLocatorSelection::FontConfig
        }
    }
}

#[derive(Debug, Clone, Copy, FromDynamic, ToDynamic, Default)]
pub enum FontRasterizerSelection {
    #[default]
    FreeType,
    Harfbuzz,
}

#[derive(Debug, Clone, Copy, FromDynamic, ToDynamic, Default)]
pub enum FontShaperSelection {
    Allsorts,
    #[default]
    Harfbuzz,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_reduce() {
        for family in &[
            "Inconsolata SemiCondensed ExtraBold",
            "Inconsolata SemiCondensed Regular",
            "Inconsolata SemiCondensed Medium",
            "Inconsolata SemiCondensed SemiBold",
        ] {
            let style = TextStyle {
                font: vec![FontAttributes::new(family)],
                foreground: None,
            };
            let style = style.reduce_first_font_to_family();
            assert_eq!(style.font[0].family, "Inconsolata");
        }
    }
}
