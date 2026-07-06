#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, FromDynamic, ToDynamic)]
pub enum DisplayPixelGeometry {
    #[default]
    RGB,
    BGR,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, FromDynamic, ToDynamic)]
pub enum FreeTypeLoadTarget {
    /// This corresponds to the default hinting algorithm, optimized
    /// for standard gray-level rendering.
    #[default]
    Normal,
    /// A lighter hinting algorithm for non-monochrome modes. Many
    /// generated glyphs are more fuzzy but better resemble its
    /// original shape. A bit like rendering on Mac OS X.  This target
    /// implies FT_LOAD_FORCE_AUTOHINT.
    Light,
    /// Strong hinting algorithm that should only be used for
    /// monochrome output. The result is probably unpleasant if the
    /// glyph is rendered in non-monochrome modes.
    Mono,
    /// A variant of Normal optimized for horizontally decimated LCD displays.
    HorizontalLcd,
    /// A variant of Normal optimized for vertically decimated LCD displays.
    VerticalLcd,
}

bitflags! {
    // Note that these are strongly coupled with deps/freetype/src/lib.rs,
    // but we can't directly reference that from here without making config
    // depend on freetype.
    #[derive(FromDynamic, ToDynamic)]
    #[dynamic(try_from="String", into="String")]
    pub struct FreeTypeLoadFlags: u32 {
        /// FT_LOAD_DEFAULT
        const DEFAULT = 0;
        /// Disable hinting. This generally generates ‘blurrier’
        /// bitmap glyph when the glyph is rendered in any of the
        /// anti-aliased modes.
        const NO_HINTING = 2;
        const NO_BITMAP = 8;
        /// Indicates that the auto-hinter is preferred over the
        /// font’s native hinter.
        const FORCE_AUTOHINT = 32;
        const MONOCHROME = 4096;
        /// Disable auto-hinter.
        const NO_AUTOHINT = 32768;
        const NO_SVG = 16777216;
        const SVG_ONLY = 8388608;
    }
}

impl FreeTypeLoadFlags {
    pub fn default_hidpi() -> Self {
        Self::NO_HINTING
    }
}

impl Default for FreeTypeLoadFlags {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl From<FreeTypeLoadFlags> for String {
    fn from(val: FreeTypeLoadFlags) -> Self {
        val.to_string()
    }
}

impl From<&FreeTypeLoadFlags> for String {
    fn from(val: &FreeTypeLoadFlags) -> Self {
        val.to_string()
    }
}

impl ToString for FreeTypeLoadFlags {
    fn to_string(&self) -> String {
        let mut s = vec![];
        if *self == Self::DEFAULT {
            s.push("DEFAULT");
        }
        if self.contains(Self::NO_HINTING) {
            s.push("NO_HINTING");
        }
        if self.contains(Self::NO_BITMAP) {
            s.push("NO_BITMAP");
        }
        if self.contains(Self::NO_SVG) {
            s.push("NO_SVG");
        }
        if self.contains(Self::SVG_ONLY) {
            s.push("SVG_ONLY");
        }
        if self.contains(Self::FORCE_AUTOHINT) {
            s.push("FORCE_AUTOHINT");
        }
        if self.contains(Self::MONOCHROME) {
            s.push("MONOCHROME");
        }
        if self.contains(Self::NO_AUTOHINT) {
            s.push("NO_AUTOHINT");
        }
        s.join("|")
    }
}

impl TryFrom<String> for FreeTypeLoadFlags {
    type Error = String;
    fn try_from(s: String) -> Result<Self, String> {
        let mut flags = FreeTypeLoadFlags::empty();

        for ele in s.split('|') {
            let ele = ele.trim();
            match ele {
                "DEFAULT" => flags |= Self::DEFAULT,
                "NO_HINTING" => flags |= Self::NO_HINTING,
                "NO_BITMAP" => flags |= Self::NO_BITMAP,
                "NO_SVG" => flags |= Self::NO_SVG,
                "SVG_ONLY" => flags |= Self::SVG_ONLY,
                "FORCE_AUTOHINT" => flags |= Self::FORCE_AUTOHINT,
                "MONOCHROME" => flags |= Self::MONOCHROME,
                "NO_AUTOHINT" => flags |= Self::NO_AUTOHINT,
                _ => {
                    return Err(format!("invalid FreeTypeLoadFlags `{}` in `{}`", ele, s));
                }
            }
        }

        Ok(flags)
    }
}
