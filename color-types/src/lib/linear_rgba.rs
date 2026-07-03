/// A pixel value encoded as linear RGBA values in f32 format (range: 0.0-1.0)
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct LinearRgba(pub f32, pub f32, pub f32, pub f32);

impl Eq for LinearRgba {}

impl Hash for LinearRgba {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.to_ne_bytes().hash(state);
        self.1.to_ne_bytes().hash(state);
        self.2.to_ne_bytes().hash(state);
        self.3.to_ne_bytes().hash(state);
    }
}

impl From<(f32, f32, f32, f32)> for LinearRgba {
    fn from((r, g, b, a): (f32, f32, f32, f32)) -> Self {
        Self(r, g, b, a)
    }
}

impl From<[f32; 4]> for LinearRgba {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Self(r, g, b, a)
    }
}

impl From<LinearRgba> for [f32; 4] {
    fn from(val: LinearRgba) -> Self {
        [val.0, val.1, val.2, val.3]
    }
}

impl LinearRgba {
    /// Convert SRGBA u8 components to LinearRgba.
    /// Note that alpha in SRGBA colorspace is already linear,
    /// so this only applies gamma correction to RGB.
    pub fn with_srgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self(
            srgb8_to_linear_f32(red),
            srgb8_to_linear_f32(green),
            srgb8_to_linear_f32(blue),
            rgb_to_linear_f32(alpha),
        )
    }

    /// Convert linear RGBA u8 components to LinearRgba (f32)
    pub fn with_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self(
            rgb_to_linear_f32(red),
            rgb_to_linear_f32(green),
            rgb_to_linear_f32(blue),
            rgb_to_linear_f32(alpha),
        )
    }

    /// Create using the provided f32 components in the range 0.0-1.0
    pub const fn with_components(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self(red, green, blue, alpha)
    }

    pub const TRANSPARENT: Self = Self::with_components(0., 0., 0., 0.);

    /// Returns true if this color is fully transparent
    pub fn is_fully_transparent(self) -> bool {
        self.3 == 0.0
    }

    /// Returns self, except when self is transparent, in which case returns other
    pub fn when_fully_transparent(self, other: Self) -> Self {
        if self.is_fully_transparent() {
            other
        } else {
            self
        }
    }

    /// Returns self multiplied by the supplied alpha value
    pub fn mul_alpha(self, alpha: f32) -> Self {
        Self(self.0, self.1, self.2, self.3 * alpha)
    }

    /// Convert to an SRGB u32 pixel
    pub fn srgba_pixel(self) -> SrgbaPixel {
        SrgbaPixel::rgba(
            linear_f32_to_srgb8(self.0),
            linear_f32_to_srgb8(self.1),
            linear_f32_to_srgb8(self.2),
            (self.3 * 255.) as u8,
        )
    }

    /// Returns the individual RGBA channels as f32 components 0.0-1.0
    pub fn tuple(self) -> (f32, f32, f32, f32) {
        (self.0, self.1, self.2, self.3)
    }

    pub fn to_srgb(self) -> SrgbaTuple {
        // Note that alpha is always linear
        SrgbaTuple(
            linear_f32_to_srgbf32(self.0),
            linear_f32_to_srgbf32(self.1),
            linear_f32_to_srgbf32(self.2),
            self.3,
        )
    }

    #[cfg(feature = "std")]
    pub fn relative_luminance(&self) -> f32 {
        0.2126 * self.0 + 0.7152 * self.1 + 0.0722 * self.2
    }

    #[cfg(feature = "std")]
    pub fn contrast_ratio(&self, other: &Self) -> f32 {
        let lum_a = self.relative_luminance();
        let lum_b = other.relative_luminance();
        Self::lum_contrast_ratio(lum_a, lum_b)
    }

    #[cfg(feature = "std")]
    fn lum_contrast_ratio(lum_a: f32, lum_b: f32) -> f32 {
        let a = lum_a + 0.05;
        let b = lum_b + 0.05;
        if a > b {
            a / b
        } else {
            b / a
        }
    }

    #[cfg(feature = "std")]
    fn to_oklaba(&self) -> [f32; 4] {
        let (r, g, b, alpha) = (self.0, self.1, self.2, self.3);
        let l_ = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
        let m_ = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
        let s_ = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
        let l = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
        let a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
        let b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;
        [l, a, b, alpha]
    }

    #[cfg(feature = "std")]
    fn from_oklaba(l: f32, a: f32, b: f32, alpha: f32) -> Self {
        let l_ = (l + 0.3963377774 * a + 0.2158037573 * b).powi(3);
        let m_ = (l - 0.1055613458 * a - 0.0638541728 * b).powi(3);
        let s_ = (l - 0.0894841775 * a - 1.2914855480 * b).powi(3);

        let r = 4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_;
        let g = -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_;
        let b = -0.0041960863 * l_ - 0.7034186147 * m_ + 1.7076147010 * s_;

        Self(r, g, b, alpha)
    }

    /// Assuming that `self` represents the foreground color
    /// and `other` represents the background color, if the
    /// contrast ratio is below min_ratio, returns Some color
    /// that equals or exceeds the min_ratio to use as an alternative
    /// foreground color.
    /// If the ratio is already suitable, returns None; the caller should
    /// continue to use `self` as the foreground color.
    #[cfg(feature = "std")]
    pub fn ensure_contrast_ratio(&self, other: &Self, min_ratio: f32) -> Option<Self> {
        if self == other {
            // Intentionally the same color, don't try to fixup
            return None;
        }

        let fg_lum = self.relative_luminance();
        let bg_lum = other.relative_luminance();
        let ratio = Self::lum_contrast_ratio(fg_lum, bg_lum);
        if ratio >= min_ratio {
            // Already has desired ratio or better
            return None;
        }

        let [_fg_l, fg_a, fg_b, fg_alpha] = self.to_oklaba();

        let reduced_lum = ((bg_lum + 0.05) / min_ratio - 0.05).clamp(0.05, 1.0);
        let reduced_col = Self::from_oklaba(reduced_lum, fg_a, fg_b, fg_alpha);
        let reduced_ratio = reduced_col.contrast_ratio(other);

        let increased_lum = ((bg_lum + 0.05) * min_ratio - 0.05).clamp(0.05, 1.0);
        let increased_col = Self::from_oklaba(increased_lum, fg_a, fg_b, fg_alpha);
        let increased_ratio = reduced_col.contrast_ratio(other);

        // Prefer the reduced luminance version if the fg is dimmer than bg
        if fg_lum < bg_lum {
            if reduced_ratio >= min_ratio {
                return Some(reduced_col);
            }
        }
        // Otherwise, let's find a satisfactory alternative
        if increased_ratio >= min_ratio {
            return Some(increased_col);
        }
        if reduced_ratio >= min_ratio {
            return Some(reduced_col);
        }

        // Didn't find one that satifies the min_ratio, but did we find
        // one that is better than the existing ratio?
        if reduced_ratio > ratio {
            return Some(reduced_col);
        }
        if increased_ratio > ratio {
            return Some(increased_col);
        }

        // What they had was as good as it gets
        None
    }
}
