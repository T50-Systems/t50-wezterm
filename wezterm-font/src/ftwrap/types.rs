/// Wrapper around std::slice::from_raw_parts that allows for ptr to be
/// null. In the null ptr case, an empty slice is returned.
/// This is necessary because it is common for freetype to encode
/// empty arrays in that way, and rust 1.78 will panic if a null
/// ptr is passed in.
pub(crate) unsafe fn from_raw_parts<'a, T>(ptr: *const T, size: usize) -> &'a [T] {
    if ptr.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(ptr, size)
    }
}

#[derive(Debug)]
pub struct PaletteInfo {
    pub num_palettes: usize,
    /// Note that this may be empty even when num_palettes is non-zero
    pub palettes: Vec<Palette>,
}

#[derive(Debug)]
pub struct Palette {
    pub palette_index: usize,
    pub name: String,
    pub flags: u16,
    pub entry_names: Vec<String>,
}

#[derive(Debug)]
pub struct NameRecord {
    pub platform_id: u16,
    pub encoding_id: u16,
    pub language_id: u16,
    pub name_id: u16,
    pub name: String,
}

pub fn vector_x_y(vector: &FT_Vector) -> (f32, f32) {
    (vector.x.f16d16().to_num(), vector.y.f16d16().to_num())
}

pub fn composite_mode_to_operator(mode: FT_Composite_Mode) -> cairo::Operator {
    use cairo::Operator;
    use FT_Composite_Mode::*;
    match mode {
        FT_COLR_COMPOSITE_CLEAR => Operator::Clear,
        FT_COLR_COMPOSITE_SRC => Operator::Source,
        FT_COLR_COMPOSITE_DEST => Operator::Dest,
        FT_COLR_COMPOSITE_SRC_OVER => Operator::Over,
        FT_COLR_COMPOSITE_DEST_OVER => Operator::DestOver,
        FT_COLR_COMPOSITE_SRC_IN => Operator::In,
        FT_COLR_COMPOSITE_DEST_IN => Operator::DestIn,
        FT_COLR_COMPOSITE_SRC_OUT => Operator::Out,
        FT_COLR_COMPOSITE_DEST_OUT => Operator::DestOut,
        FT_COLR_COMPOSITE_SRC_ATOP => Operator::Atop,
        FT_COLR_COMPOSITE_DEST_ATOP => Operator::DestAtop,
        FT_COLR_COMPOSITE_XOR => Operator::Xor,
        FT_COLR_COMPOSITE_PLUS => Operator::Add,
        FT_COLR_COMPOSITE_SCREEN => Operator::Screen,
        FT_COLR_COMPOSITE_OVERLAY => Operator::Overlay,
        FT_COLR_COMPOSITE_DARKEN => Operator::Darken,
        FT_COLR_COMPOSITE_LIGHTEN => Operator::Lighten,
        FT_COLR_COMPOSITE_COLOR_DODGE => Operator::ColorDodge,
        FT_COLR_COMPOSITE_COLOR_BURN => Operator::ColorBurn,
        FT_COLR_COMPOSITE_HARD_LIGHT => Operator::HardLight,
        FT_COLR_COMPOSITE_SOFT_LIGHT => Operator::SoftLight,
        FT_COLR_COMPOSITE_DIFFERENCE => Operator::Difference,
        FT_COLR_COMPOSITE_EXCLUSION => Operator::Exclusion,
        FT_COLR_COMPOSITE_MULTIPLY => Operator::Multiply,
        FT_COLR_COMPOSITE_HSL_HUE => Operator::HslHue,
        FT_COLR_COMPOSITE_HSL_SATURATION => Operator::HslSaturation,
        FT_COLR_COMPOSITE_HSL_COLOR => Operator::HslColor,
        FT_COLR_COMPOSITE_HSL_LUMINOSITY => Operator::HslLuminosity,
        _ => unreachable!(),
    }
}

fn ft_make_tag(a: u8, b: u8, c: u8, d: u8) -> FT_ULong {
    (a as FT_ULong) << 24 | (b as FT_ULong) << 16 | (c as FT_ULong) << 8 | (d as FT_ULong)
}
