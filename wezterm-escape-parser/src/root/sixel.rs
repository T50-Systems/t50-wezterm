/// See <https://vt100.net/docs/vt3xx-gp/chapter14.html>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sixel {
    /// Specifies the numerator for the pixel aspect ratio
    pub pan: i64,

    /// Specifies the denominator for the pixel aspect ratio
    pub pad: i64,

    /// How wide the image is, in pixels
    pub pixel_width: Option<u32>,

    /// How tall the image is, in pixels,
    pub pixel_height: Option<u32>,

    /// When true, pixels with 0 value are left at their
    /// present color, otherwise, they are set to the background
    /// color.
    pub background_is_transparent: bool,

    /// The horizontal spacing between pixels
    pub horizontal_grid_size: Option<i64>,

    /// The sixel data
    pub data: Vec<SixelData>,
}

impl Sixel {
    /// Returns the width, height of the image
    pub fn dimensions(&self) -> (u32, u32) {
        if let (Some(w), Some(h)) = (self.pixel_width, self.pixel_height) {
            return (w, h);
        }

        // Compute it by evaluating the sixel data
        let mut max_x = 0;
        let mut max_y = 0;
        let mut x: u32 = 0;
        let mut rows: u32 = 1;

        for d in &self.data {
            match d {
                SixelData::Data(_) => {
                    max_y = max_y.max(rows * 6);
                    x = x.saturating_add(1);
                    max_x = max_x.max(x);
                }
                SixelData::Repeat { repeat_count, .. } => {
                    max_y = max_y.max(rows * 6);
                    x = x.saturating_add(*repeat_count);
                    max_x = max_x.max(x);
                }
                SixelData::SelectColorMapEntry(_)
                | SixelData::DefineColorMapRGB { .. }
                | SixelData::DefineColorMapHSL { .. } => {}
                SixelData::NewLine => {
                    max_x = max_x.max(x);
                    x = 0;
                    rows = rows.saturating_add(1);
                }
                SixelData::CarriageReturn => {
                    max_x = max_x.max(x);
                    x = 0;
                }
            }
        }

        (max_x, max_y)
    }
}

impl Display for Sixel {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        if self.pixel_width.is_some() {
            write!(
                f,
                "\x1bP;{}{}q\"{};{};{};{}",
                if self.background_is_transparent { 1 } else { 0 },
                match self.horizontal_grid_size {
                    Some(h) => format!(";{}", h),
                    None => "".to_string(),
                },
                self.pan,
                self.pad,
                self.pixel_width.unwrap_or(0),
                self.pixel_height.unwrap_or(0)
            )?;
        } else {
            write!(
                f,
                "\x1bP{};{}{}q",
                match (self.pan, self.pad) {
                    (2, 1) => 0,
                    (5, 1) => 2,
                    (3, 1) => 3,
                    (1, 1) => 7,
                    _ => {
                        log::error!("bad pad/pan combo: {:?}", self);
                        return Err(core::fmt::Error);
                    }
                },
                if self.background_is_transparent { 1 } else { 0 },
                match self.horizontal_grid_size {
                    Some(h) => format!(";{}", h),
                    None => "".to_string(),
                },
            )?;
        }
        for d in &self.data {
            d.fmt(f)?;
        }
        // The sixel data itself doesn't contain the ST
        // write!(f, "\x1b\\")?;
        Ok(())
    }
}

/// A decoded 6-bit sixel value.
/// Each sixel represents a six-pixel tall bitmap where
/// the least significant bit is the topmost bit.
pub type SixelValue = u8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SixelData {
    /// A single sixel value
    Data(SixelValue),

    /// Run-length encoding; allows repeating a sixel value
    /// the specified number of times
    Repeat { repeat_count: u32, data: SixelValue },

    /// Set the specified color map entry to the specified
    /// linear RGB color value
    DefineColorMapRGB {
        color_number: u16,
        rgb: crate::color::RgbColor,
    },

    DefineColorMapHSL {
        color_number: u16,
        /// 0 to 360 degrees
        hue_angle: u16,
        /// 0 to 100
        lightness: u8,
        /// 0 to 100
        saturation: u8,
    },

    /// Select the numbered color from the color map entry
    SelectColorMapEntry(u16),

    /// Move the x position to the left page border of the
    /// current sixel line.
    CarriageReturn,

    /// Move the x position to the left page border and
    /// the y position down to the next sixel line.
    NewLine,
}

impl Display for SixelData {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::Data(value) => write!(f, "{}", (value + 0x3f) as char),
            Self::Repeat { repeat_count, data } => {
                write!(f, "!{}{}", repeat_count, (data + 0x3f) as char)
            }
            Self::DefineColorMapRGB { color_number, rgb } => {
                let LinearRgba(r, g, b, _) = rgb.to_linear_tuple_rgba();
                write!(
                    f,
                    "#{};2;{};{};{}",
                    color_number,
                    (r * 100.) as u8,
                    (g * 100.) as u8,
                    (b * 100.0) as u8
                )
            }
            Self::DefineColorMapHSL {
                color_number,
                hue_angle,
                lightness,
                saturation,
            } => write!(
                f,
                "#{};1;{};{};{}",
                color_number, hue_angle, lightness, saturation
            ),
            Self::SelectColorMapEntry(n) => write!(f, "#{}", n),
            Self::CarriageReturn => write!(f, "$"),
            Self::NewLine => write!(f, "-"),
        }
    }
}
