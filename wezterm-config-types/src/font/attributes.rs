#[derive(Debug, Clone, PartialEq, Eq, Hash, FromDynamic, ToDynamic)]
pub struct FontAttributes {
    /// The font family name
    pub family: String,
    /// Whether the font should be a bold variant
    #[dynamic(default)]
    pub weight: FontWeight,
    #[dynamic(default)]
    pub stretch: FontStretch,
    /// Whether the font should be an italic variant
    #[dynamic(default)]
    pub style: FontStyle,
    pub is_fallback: bool,
    pub is_synthetic: bool,

    #[dynamic(default)]
    pub harfbuzz_features: Option<Vec<String>>,
    #[dynamic(default)]
    pub freetype_load_target: Option<FreeTypeLoadTarget>,
    #[dynamic(default)]
    pub freetype_render_target: Option<FreeTypeLoadTarget>,
    #[dynamic(default)]
    pub freetype_load_flags: Option<FreeTypeLoadFlags>,
    #[dynamic(default)]
    pub scale: Option<NotNan<f64>>,
    #[dynamic(default)]
    pub assume_emoji_presentation: Option<bool>,
}
impl_lua_conversion_dynamic!(FontAttributes);

impl std::fmt::Display for FontAttributes {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "wezterm.font('{}', {{weight={}, stretch='{}', style={}}})",
            self.family, self.weight, self.stretch, self.style
        )
    }
}

impl FontAttributes {
    pub fn new(family: &str) -> Self {
        Self {
            family: family.into(),
            weight: FontWeight::default(),
            stretch: FontStretch::default(),
            style: FontStyle::Normal,
            is_fallback: false,
            is_synthetic: false,
            harfbuzz_features: None,
            freetype_load_target: None,
            freetype_render_target: None,
            freetype_load_flags: None,
            scale: None,
            assume_emoji_presentation: None,
        }
    }

    pub fn new_fallback(family: &str) -> Self {
        Self {
            family: family.into(),
            weight: FontWeight::default(),
            stretch: FontStretch::default(),
            style: FontStyle::Normal,
            is_fallback: true,
            is_synthetic: false,
            harfbuzz_features: None,
            freetype_load_target: None,
            freetype_render_target: None,
            freetype_load_flags: None,
            scale: None,
            assume_emoji_presentation: None,
        }
    }
}

impl Default for FontAttributes {
    fn default() -> Self {
        Self {
            family: "JetBrains Mono".into(),
            weight: FontWeight::default(),
            stretch: FontStretch::default(),
            style: FontStyle::Normal,
            is_fallback: false,
            is_synthetic: false,
            harfbuzz_features: None,
            freetype_load_target: None,
            freetype_render_target: None,
            freetype_load_flags: None,
            scale: None,
            assume_emoji_presentation: None,
        }
    }
}
