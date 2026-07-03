/// A pixel holding SRGBA32 data in big endian format
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SrgbaPixel(u32);

impl SrgbaPixel {
    /// Create a pixel with the provided sRGBA values in u8 format
    pub fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        #[allow(clippy::cast_lossless)]
        let word = (blue as u32) << 24 | (green as u32) << 16 | (red as u32) << 8 | alpha as u32;
        Self(word.to_be())
    }

    /// Returns the unpacked sRGBA components as u8
    #[inline]
    pub fn as_rgba(self) -> (u8, u8, u8, u8) {
        let host = u32::from_be(self.0);
        (
            (host >> 8) as u8,
            (host >> 16) as u8,
            (host >> 24) as u8,
            (host & 0xff) as u8,
        )
    }

    /// Returns RGBA channels in linear f32 format
    pub fn to_linear(self) -> LinearRgba {
        let (r, g, b, a) = self.as_rgba();
        LinearRgba::with_srgba(r, g, b, a)
    }

    /// Create a pixel with the provided big-endian u32 SRGBA data
    pub fn with_srgba_u32(word: u32) -> Self {
        Self(word)
    }

    /// Returns the underlying big-endian u32 SRGBA data
    pub fn as_srgba32(self) -> u32 {
        self.0
    }

    pub fn as_srgba_tuple(self) -> (f32, f32, f32, f32) {
        let u8tuple = self.as_rgba();
        let SrgbaTuple(r, g, b, a) = u8tuple.into();
        (r, g, b, a)
    }
}

/// A pixel value encoded as SRGBA RGBA values in f32 format (range: 0.0-1.0)
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct SrgbaTuple(pub f32, pub f32, pub f32, pub f32);

impl SrgbaTuple {
    pub fn premultiply(self) -> Self {
        let SrgbaTuple(r, g, b, a) = self;
        Self(r * a, g * a, b * a, a)
    }

    pub fn demultiply(self) -> Self {
        let SrgbaTuple(r, g, b, a) = self;
        if a != 0. {
            Self(r / a, g / a, b / a, a)
        } else {
            self
        }
    }

    pub fn to_tuple_rgba(self) -> (f32, f32, f32, f32) {
        (self.0, self.1, self.2, self.3)
    }

    pub fn as_rgba_u8(self) -> (u8, u8, u8, u8) {
        let (r, g, b, a) = (self.0, self.1, self.2, self.3);
        (
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            (a * 255.0) as u8,
        )
    }

    pub fn interpolate(self, other: Self, k: f64) -> Self {
        let k = k as f32;

        let SrgbaTuple(r0, g0, b0, a0) = self.premultiply();
        let SrgbaTuple(r1, g1, b1, a1) = other.premultiply();

        let r = SrgbaTuple(
            r0 + k * (r1 - r0),
            g0 + k * (g1 - g0),
            b0 + k * (b1 - b0),
            a0 + k * (a1 - a0),
        );

        r.demultiply()
    }
}

impl ToDynamic for SrgbaTuple {
    fn to_dynamic(&self) -> Value {
        self.to_string().to_dynamic()
    }
}

impl FromDynamic for SrgbaTuple {
    fn from_dynamic(
        value: &Value,
        options: FromDynamicOptions,
    ) -> Result<Self, wezterm_dynamic::Error> {
        let s = String::from_dynamic(value, options)?;
        Ok(SrgbaTuple::from_str(&s).map_err(|()| format!("unknown color name: {}", s))?)
    }
}

impl From<SrgbaPixel> for SrgbaTuple {
    fn from(pixel: SrgbaPixel) -> SrgbaTuple {
        let (r, g, b, a) = pixel.as_srgba_tuple();
        SrgbaTuple(r, g, b, a)
    }
}

impl From<(f32, f32, f32, f32)> for SrgbaTuple {
    fn from((r, g, b, a): (f32, f32, f32, f32)) -> SrgbaTuple {
        SrgbaTuple(r, g, b, a)
    }
}

impl From<(u8, u8, u8, u8)> for SrgbaTuple {
    fn from((r, g, b, a): (u8, u8, u8, u8)) -> SrgbaTuple {
        SrgbaTuple(
            r as f32 / 255.,
            g as f32 / 255.,
            b as f32 / 255.,
            a as f32 / 255.,
        )
    }
}

impl From<(u8, u8, u8)> for SrgbaTuple {
    fn from((r, g, b): (u8, u8, u8)) -> SrgbaTuple {
        SrgbaTuple(r as f32 / 255., g as f32 / 255., b as f32 / 255., 1.0)
    }
}

impl From<SrgbaTuple> for (f32, f32, f32, f32) {
    fn from(t: SrgbaTuple) -> (f32, f32, f32, f32) {
        (t.0, t.1, t.2, t.3)
    }
}

#[cfg(feature = "std")]
impl From<Color> for SrgbaTuple {
    fn from(color: Color) -> Self {
        Self(
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        )
    }
}
