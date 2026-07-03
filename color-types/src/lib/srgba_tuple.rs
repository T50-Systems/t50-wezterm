impl SrgbaTuple {
    /// Construct a color from an X11/SVG/CSS3 color name.
    /// Returns None if the supplied name is not recognized.
    /// The list of names can be found here:
    /// <https://en.wikipedia.org/wiki/X11_color_names>
    pub fn from_named(name: &str) -> Option<Self> {
        #[cfg(feature = "std")]
        {
            return NAMED_COLORS.get(&name.to_ascii_lowercase()).cloned();
        }
        #[cfg(not(feature = "std"))]
        {
            let mut result = None;
            iter_rgb_txt(|candidate, color| {
                if candidate.eq_ignore_ascii_case(name) {
                    result.replace(color);
                    true
                } else {
                    false
                }
            });
            result
        }
    }

    /// Returns self multiplied by the supplied alpha value.
    /// We don't need to linearize for this, as alpha is defined
    /// as being linear even in srgba!
    pub fn mul_alpha(self, alpha: f32) -> Self {
        Self(self.0, self.1, self.2, self.3 * alpha)
    }

    pub fn to_linear(self) -> LinearRgba {
        // See https://docs.rs/palette/0.5.0/src/palette/encoding/srgb.rs.html#43
        fn to_linear(v: f32) -> f32 {
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        }
        // Note that alpha is always linear
        LinearRgba(
            to_linear(self.0),
            to_linear(self.1),
            to_linear(self.2),
            self.3,
        )
    }

    pub fn to_srgb_u8(self) -> (u8, u8, u8, u8) {
        (
            (self.0 * 255.) as u8,
            (self.1 * 255.) as u8,
            (self.2 * 255.) as u8,
            (self.3 * 255.) as u8,
        )
    }

    pub fn to_string(self) -> String {
        if self.3 == 1.0 {
            self.to_rgb_string()
        } else {
            self.to_rgba_string()
        }
    }

    /// Returns a string of the form `#RRGGBB`
    pub fn to_rgb_string(self) -> String {
        format!(
            "#{:02x}{:02x}{:02x}",
            (self.0 * 255.) as u8,
            (self.1 * 255.) as u8,
            (self.2 * 255.) as u8
        )
    }

    pub fn to_rgba_string(self) -> String {
        format!(
            "rgba({}% {}% {}% {}%)",
            (self.0 * 100.),
            (self.1 * 100.),
            (self.2 * 100.),
            (self.3 * 100.)
        )
    }

    /// Returns a string of the form `rgb:RRRR/GGGG/BBBB`
    pub fn to_x11_16bit_rgb_string(self) -> String {
        format!(
            "rgb:{:04x}/{:04x}/{:04x}",
            (self.0 * 65535.) as u16,
            (self.1 * 65535.) as u16,
            (self.2 * 65535.) as u16
        )
    }

    #[cfg(feature = "std")]
    pub fn to_laba(self) -> (f64, f64, f64, f64) {
        Color::new(self.0.into(), self.1.into(), self.2.into(), self.3.into()).to_lab()
    }

    #[cfg(feature = "std")]
    pub fn to_hsla(self) -> (f64, f64, f64, f64) {
        Color::new(self.0.into(), self.1.into(), self.2.into(), self.3.into()).to_hsla()
    }

    #[cfg(feature = "std")]
    pub fn from_hsla(h: f64, s: f64, l: f64, a: f64) -> Self {
        let Color { r, g, b, a } = Color::from_hsla(h, s, l, a);
        Self(r as f32, g as f32, b as f32, a as f32)
    }

    /// Scale the color towards the maximum saturation by factor, a value ranging from 0.0 to 1.0.
    #[cfg(feature = "std")]
    pub fn saturate(&self, factor: f64) -> Self {
        let (h, s, l, a) = self.to_hsla();
        let s = apply_scale(s, factor);
        Self::from_hsla(h, s, l, a)
    }

    /// Increase the saturation by amount, a value ranging from 0.0 to 1.0.
    #[cfg(feature = "std")]
    pub fn saturate_fixed(&self, amount: f64) -> Self {
        let (h, s, l, a) = self.to_hsla();
        let s = apply_fixed(s, amount);
        Self::from_hsla(h, s, l, a)
    }

    /// Scale the color towards the maximum lightness by factor, a value ranging from 0.0 to 1.0
    #[cfg(feature = "std")]
    pub fn lighten(&self, factor: f64) -> Self {
        let (h, s, l, a) = self.to_hsla();
        let l = apply_scale(l, factor);
        Self::from_hsla(h, s, l, a)
    }

    /// Lighten the color by amount, a value ranging from 0.0 to 1.0
    #[cfg(feature = "std")]
    pub fn lighten_fixed(&self, amount: f64) -> Self {
        let (h, s, l, a) = self.to_hsla();
        let l = apply_fixed(l, amount);
        Self::from_hsla(h, s, l, a)
    }

    /// Rotate the hue angle by the specified number of degrees
    #[cfg(feature = "std")]
    pub fn adjust_hue_fixed(&self, amount: f64) -> Self {
        let (h, s, l, a) = self.to_hsla();
        let h = normalize_angle(h + amount);
        Self::from_hsla(h, s, l, a)
    }

    #[cfg(feature = "std")]
    pub fn complement(&self) -> Self {
        self.adjust_hue_fixed(180.)
    }

    #[cfg(feature = "std")]
    pub fn complement_ryb(&self) -> Self {
        self.adjust_hue_fixed_ryb(180.)
    }

    #[cfg(feature = "std")]
    pub fn triad(&self) -> (Self, Self) {
        (self.adjust_hue_fixed(120.), self.adjust_hue_fixed(-120.))
    }

    #[cfg(feature = "std")]
    pub fn square(&self) -> (Self, Self, Self) {
        (
            self.adjust_hue_fixed(90.),
            self.adjust_hue_fixed(270.),
            self.adjust_hue_fixed(180.),
        )
    }

    /// Rotate the hue angle by the specified number of degrees, using
    /// the RYB color wheel
    #[cfg(feature = "std")]
    pub fn adjust_hue_fixed_ryb(&self, amount: f64) -> Self {
        let (h, s, l, a) = self.to_hsla();
        let h = rgb_hue_to_ryb_hue(h);
        let h = normalize_angle(h + amount);
        let h = ryb_huge_to_rgb_hue(h);
        Self::from_hsla(h, s, l, a)
    }

    #[cfg(feature = "std")]
    fn lab_value(&self) -> deltae::LabValue {
        let (l, a, b, _alpha) = self.to_laba();
        deltae::LabValue {
            l: l as f32,
            a: a as f32,
            b: b as f32,
        }
    }

    #[cfg(feature = "std")]
    pub fn delta_e(&self, other: &Self) -> f32 {
        let a = self.lab_value();
        let b = other.lab_value();
        *deltae::DeltaE::new(a, b, deltae::DEMethod::DE2000).value()
    }

    #[cfg(feature = "std")]
    pub fn contrast_ratio(&self, other: &Self) -> f32 {
        self.to_linear().contrast_ratio(&other.to_linear())
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
        self.to_linear()
            .ensure_contrast_ratio(&other.to_linear(), min_ratio)
            .map(|linear| linear.to_srgb())
    }
}
