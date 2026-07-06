/// Represents textual styling.
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromDynamic, ToDynamic)]
pub struct TextStyle {
    #[dynamic(default)]
    pub font: Vec<FontAttributes>,

    /// If set, when rendering text that is set to the default
    /// foreground color, use this color instead.  This is most
    /// useful in a `[[font_rules]]` section to implement changing
    /// the text color for eg: bold text.
    pub foreground: Option<RgbaColor>,
}
impl_lua_conversion_dynamic!(TextStyle);

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            foreground: None,
            font: vec![FontAttributes::default()],
        }
    }
}

impl TextStyle {
    /// Make a version of this style where the first entry
    /// has any explicitly named bold/italic components
    /// removed.  The intent is to set it up for make_bold
    /// and make_italic below.
    ///
    /// This is done heuristically based on the family name
    /// string as we cannot depend on the font parser from
    /// this crate, and even if we did have a parser, that
    /// doesn't help us know anything about the name until
    /// we have a parsed font to compare with.
    ///
    /// <https://github.com/wezterm/wezterm/issues/456>
    pub fn reduce_first_font_to_family(&self) -> Self {
        fn reduce(mut family: &str) -> String {
            loop {
                let start = family;

                for s in &[
                    "Black",
                    "Bold",
                    "Book",
                    "Condensed",
                    "Demi",
                    "Expanded",
                    "Extra",
                    "Italic",
                    "Light",
                    "Medium",
                    "Regular",
                    "Semi",
                    "Thin",
                    "Ultra",
                ] {
                    family = family.trim().trim_end_matches(s);
                }

                if family == start {
                    break;
                }
            }

            family.trim().to_string()
        }
        Self {
            foreground: self.foreground,
            font: self
                .font
                .iter()
                .enumerate()
                .map(|(idx, orig_attr)| {
                    let mut attr = orig_attr.clone();
                    if idx == 0 {
                        attr.family = reduce(&attr.family);
                    }
                    attr
                })
                .collect(),
        }
    }

    /// Make a version of this style with bold enabled.
    pub fn make_bold(&self) -> Self {
        Self {
            foreground: self.foreground,
            font: self
                .font
                .iter()
                .map(|attr| {
                    let mut attr = attr.clone();
                    attr.weight = attr.weight.bolder();
                    attr.is_synthetic = true;
                    attr
                })
                .collect(),
        }
    }

    pub fn make_half_bright(&self) -> Self {
        Self {
            foreground: self.foreground,
            font: self
                .font
                .iter()
                .map(|attr| {
                    let mut attr = attr.clone();
                    attr.weight = attr.weight.lighter();
                    attr.is_synthetic = true;
                    attr
                })
                .collect(),
        }
    }

    /// Make a version of this style with italic enabled.
    pub fn make_italic(&self) -> Self {
        Self {
            foreground: self.foreground,
            font: self
                .font
                .iter()
                .map(|attr| {
                    let mut attr = attr.clone();
                    attr.style = FontStyle::Italic;
                    attr.is_synthetic = true;
                    attr
                })
                .collect(),
        }
    }

    #[allow(clippy::let_and_return)]
    pub fn font_with_fallback(&self) -> Vec<FontAttributes> {
        let mut font = self.font.clone();

        let mut default_font = FontAttributes::default();

        // Insert our bundled default JetBrainsMono as a fallback
        // in case their preference doesn't match anything.
        // But don't add it if it is already their preference.
        if !font.iter().any(|f| *f == default_font) {
            default_font.is_fallback = true;
            font.push(default_font);
        }

        // We bundle this emoji font as an in-memory fallback
        font.push(FontAttributes::new_fallback("Noto Color Emoji"));

        // Add symbols that many people end up using via patched fonts
        font.push(FontAttributes::new_fallback("Symbols Nerd Font Mono"));

        font
    }
}
