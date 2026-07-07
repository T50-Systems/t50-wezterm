use super::*;

impl BlockKey {
    pub fn filter_out_synthetic(glyphs: &mut Vec<char>) {
        let config = config::configuration();
        if config.custom_block_glyphs {
            glyphs.retain(|&c| Self::from_char(c).is_none());
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let mut chars = s.chars();
        let first_char = chars.next()?;
        if chars.next().is_some() {
            None
        } else {
            Self::from_char(first_char)
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        let c = c as u32;
        block_key_from_char_part1::from_char_part1(c)
            .or_else(|| block_key_from_char_part2::from_char_part2(c))
            .or_else(|| block_key_from_char_part3::from_char_part3(c))
            .or_else(|| block_key_from_char_part4::from_char_part4(c))
            .or_else(|| block_key_from_char_part5::from_char_part5(c))
            .or_else(|| block_key_from_char_part6::from_char_part6(c))
            .or_else(|| block_key_from_char_part7::from_char_part7(c))
            .or_else(|| block_key_from_char_part8::from_char_part8(c))
            .or_else(|| block_key_from_char_part9::from_char_part9(c))
            .or_else(|| block_key_from_char_part10::from_char_part10(c))
            .or_else(|| block_key_from_char_part11::from_char_part11(c))
    }

    pub fn from_cell_iter(cell: termwiz::surface::line::CellRef) -> Option<Self> {
        let mut chars = cell.str().chars();
        let first_char = chars.next()?;
        if chars.next().is_some() {
            None
        } else {
            Self::from_char(first_char)
        }
    }

    pub fn from_cell(cell: &termwiz::cell::Cell) -> Option<Self> {
        let mut chars = cell.str().chars();
        let first_char = chars.next()?;
        if chars.next().is_some() {
            None
        } else {
            Self::from_char(first_char)
        }
    }
}
