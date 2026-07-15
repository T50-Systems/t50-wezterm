pub use wezterm_custom_glyph::*;

pub fn filter_out_synthetic(glyphs: &mut Vec<char>) {
    let config = config::configuration();
    if config.custom_block_glyphs {
        glyphs.retain(|&c| BlockKey::from_char(c).is_none());
    }
}
