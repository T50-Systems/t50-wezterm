#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum SmallColor {
    Default,
    PaletteIndex(PaletteIndex),
}

impl Default for SmallColor {
    fn default() -> Self {
        Self::Default
    }
}

impl Into<ColorAttribute> for SmallColor {
    fn into(self) -> ColorAttribute {
        match self {
            Self::Default => ColorAttribute::Default,
            Self::PaletteIndex(idx) => ColorAttribute::PaletteIndex(idx),
        }
    }
}

/// Holds the attributes for a cell.
/// Most style attributes are stored internally as part of a bitfield
/// to reduce per-cell overhead.
/// The setter methods return a mutable self reference so that they can
/// be chained together.
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq)]
pub struct CellAttributes {
    attributes: u32,
    /// The foreground color
    foreground: SmallColor,
    /// The background color
    background: SmallColor,
    /// Relatively rarely used attributes spill over to a heap
    /// allocated struct in order to keep CellAttributes
    /// smaller in the common case.
    fat: Option<Box<FatAttributes>>,
}

impl core::fmt::Debug for CellAttributes {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        fmt.debug_struct("CellAttributes")
            .field("attributes", &self.attributes)
            .field("intensity", &self.intensity())
            .field("underline", &self.underline())
            .field("blink", &self.blink())
            .field("italic", &self.italic())
            .field("reverse", &self.reverse())
            .field("strikethrough", &self.strikethrough())
            .field("invisible", &self.invisible())
            .field("wrapped", &self.wrapped())
            .field("overline", &self.overline())
            .field("semantic_type", &self.semantic_type())
            .field("foreground", &self.foreground)
            .field("background", &self.background)
            .field("fat", &self.fat)
            .finish()
    }
}

#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Default, Clone, Eq, PartialEq)]
struct FatAttributes {
    /// The hyperlink content, if any
    hyperlink: Option<Arc<Hyperlink>>,
    /// The image data, if any
    #[cfg(feature = "use_image")]
    image: Vec<Box<ImageCell>>,
    /// The color of the underline.  If None, then
    /// the foreground color is to be used
    underline_color: ColorAttribute,
    foreground: ColorAttribute,
    background: ColorAttribute,
}

impl FatAttributes {
    pub fn compute_shape_hash<H: Hasher>(&self, hasher: &mut H) {
        if let Some(link) = &self.hyperlink {
            link.compute_shape_hash(hasher);
        }
        #[cfg(feature = "use_image")]
        for cell in &self.image {
            cell.compute_shape_hash(hasher);
        }
        self.underline_color.hash(hasher);
        self.foreground.hash(hasher);
        self.background.hash(hasher);
    }
}

/// Define getter and setter for the attributes bitfield.
/// The first form is for a simple boolean value stored in
/// a single bit.  The $bitnum parameter specifies which bit.
/// The second form is for an integer value that occupies a range
/// of bits.  The $bitmask and $bitshift parameters define how
/// to transform from the stored bit value to the consumable
/// value.
macro_rules! bitfield {
    ($getter:ident, $setter:ident, $bitnum:expr) => {
        #[inline]
        pub fn $getter(&self) -> bool {
            (self.attributes & (1 << $bitnum)) == (1 << $bitnum)
        }

        #[inline]
        pub fn $setter(&mut self, value: bool) -> &mut Self {
            let attr_value = if value { 1 << $bitnum } else { 0 };
            self.attributes = (self.attributes & !(1 << $bitnum)) | attr_value;
            self
        }
    };

    ($getter:ident, $setter:ident, $bitmask:expr, $bitshift:expr) => {
        #[inline]
        pub fn $getter(&self) -> u32 {
            (self.attributes >> $bitshift) & $bitmask
        }

        #[inline]
        pub fn $setter(&mut self, value: u32) -> &mut Self {
            let clear = !($bitmask << $bitshift);
            let attr_value = (value & $bitmask) << $bitshift;
            self.attributes = (self.attributes & clear) | attr_value;
            self
        }
    };

    ($getter:ident, $setter:ident, $enum:ident, $bitmask:expr, $bitshift:expr) => {
        #[inline]
        pub fn $getter(&self) -> $enum {
            unsafe { mem::transmute(((self.attributes >> $bitshift) & $bitmask) as u8) }
        }

        #[inline]
        pub fn $setter(&mut self, value: $enum) -> &mut Self {
            let value = value as u32;
            let clear = !($bitmask << $bitshift);
            let attr_value = (value & $bitmask) << $bitshift;
            self.attributes = (self.attributes & clear) | attr_value;
            self
        }
    };
}
