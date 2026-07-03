#[derive(Debug, Copy, Clone, FromDynamic, ToDynamic)]
pub struct HsbTransform {
    #[dynamic(default = "default_one_point_oh")]
    pub hue: f32,
    #[dynamic(default = "default_one_point_oh")]
    pub saturation: f32,
    #[dynamic(default = "default_one_point_oh")]
    pub brightness: f32,
}

impl Default for HsbTransform {
    fn default() -> Self {
        Self {
            hue: 1.,
            saturation: 1.,
            brightness: 1.,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, FromDynamic, ToDynamic)]
#[dynamic(try_from = "String", into = "String")]
pub struct RgbaColor {
    #[dynamic(flatten)]
    color: SrgbaTuple,
}

impl From<RgbColor> for RgbaColor {
    fn from(color: RgbColor) -> Self {
        Self {
            color: color.into(),
        }
    }
}

impl From<SrgbaTuple> for RgbaColor {
    fn from(color: SrgbaTuple) -> Self {
        Self { color }
    }
}

impl From<(u8, u8, u8)> for RgbaColor {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        let color: SrgbaTuple = (r, g, b).into();
        Self { color }
    }
}

impl std::ops::Deref for RgbaColor {
    type Target = SrgbaTuple;
    fn deref(&self) -> &SrgbaTuple {
        &self.color
    }
}

impl From<&RgbaColor> for String {
    fn from(val: &RgbaColor) -> Self {
        val.color.to_string()
    }
}

impl From<RgbaColor> for String {
    fn from(val: RgbaColor) -> Self {
        val.color.to_string()
    }
}

impl From<RgbaColor> for SrgbaTuple {
    fn from(val: RgbaColor) -> Self {
        val.color
    }
}

impl TryFrom<String> for RgbaColor {
    type Error = anyhow::Error;
    fn try_from(s: String) -> anyhow::Result<RgbaColor> {
        Ok(RgbaColor {
            color: SrgbaTuple::from_str(&s)
                .map_err(|_| anyhow::anyhow!("failed to parse {} as RgbaColor", &s))?,
        })
    }
}

#[derive(Debug, FromDynamic, ToDynamic, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpec {
    AnsiColor(AnsiColor),
    Color(RgbaColor),
    Default,
}

impl From<AnsiColor> for ColorSpec {
    fn from(color: AnsiColor) -> ColorSpec {
        Self::AnsiColor(color)
    }
}

impl From<ColorSpec> for ColorAttribute {
    fn from(val: ColorSpec) -> Self {
        match val {
            ColorSpec::AnsiColor(c) => ColorAttribute::PaletteIndex(c.into()),
            ColorSpec::Color(RgbaColor { color }) => {
                ColorAttribute::TrueColorWithDefaultFallback(color)
            }
            ColorSpec::Default => ColorAttribute::Default,
        }
    }
}

impl From<ColorSpec> for TWColorSpec {
    fn from(val: ColorSpec) -> Self {
        match val {
            ColorSpec::AnsiColor(c) => c.into(),
            ColorSpec::Color(RgbaColor { color }) => TWColorSpec::TrueColor(color),
            ColorSpec::Default => TWColorSpec::Default,
        }
    }
}
