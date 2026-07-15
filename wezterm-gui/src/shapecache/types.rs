use crate::customglyph::BlockKey;
use crate::glyphcache::CachedGlyph;
use wezterm_config_types::TextStyle;
use std::rc::Rc;
use wezterm_font::shaper::GlyphInfo;
use wezterm_font::units::*;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ShapeCacheKey {
    pub style: TextStyle,
    pub text: String,
}

#[derive(Debug, PartialEq)]
pub struct GlyphPosition {
    pub glyph_idx: u32,
    pub num_cells: u8,
    pub x_offset: PixelLength,
    pub bearing_x: f32,
    pub bitmap_pixel_width: u32,
}

#[derive(Debug)]
pub struct ShapedInfo {
    pub glyph: Rc<CachedGlyph>,
    pub pos: GlyphPosition,
    pub block_key: Option<BlockKey>,
}

impl ShapedInfo {
    /// Process the results from the shaper, stitching together glyph
    /// and positioning information
    pub fn process(infos: &[GlyphInfo], glyphs: &[Rc<CachedGlyph>]) -> Vec<ShapedInfo> {
        let mut pos: Vec<ShapedInfo> = Vec::with_capacity(infos.len());

        for (info, glyph) in infos.iter().zip(glyphs.iter()) {
            pos.push(ShapedInfo {
                pos: GlyphPosition {
                    glyph_idx: info.glyph_pos,
                    bitmap_pixel_width: glyph
                        .texture
                        .as_ref()
                        .map_or(0, |t| t.coords.width() as u32),
                    num_cells: info.num_cells,
                    x_offset: info.x_offset,
                    bearing_x: glyph.bearing_x.get() as f32,
                },
                glyph: Rc::clone(glyph),
                block_key: info.only_char.and_then(BlockKey::from_char),
            });
        }
        pos
    }
}

/// We'd like to avoid allocating when resolving from the cache
/// so this is the borrowed version of ShapeCacheKey.
/// It's a bit involved to make this work; more details can be
/// found in the excellent guide here:
/// <https://github.com/sunshowers/borrow-complex-key-example/blob/master/src/lib.rs>
#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BorrowedShapeCacheKey<'a> {
    pub style: &'a TextStyle,
    pub text: &'a str,
}

impl<'a> BorrowedShapeCacheKey<'a> {
    pub fn to_owned(&self) -> ShapeCacheKey {
        ShapeCacheKey {
            style: self.style.clone(),
            text: self.text.to_owned(),
        }
    }
}

pub trait ShapeCacheKeyTrait: std::fmt::Debug {
    fn key<'k>(&'k self) -> BorrowedShapeCacheKey<'k>;
}

impl ShapeCacheKeyTrait for ShapeCacheKey {
    fn key<'k>(&'k self) -> BorrowedShapeCacheKey<'k> {
        BorrowedShapeCacheKey {
            style: &self.style,
            text: &self.text,
        }
    }
}

impl<'a> ShapeCacheKeyTrait for BorrowedShapeCacheKey<'a> {
    fn key<'k>(&'k self) -> BorrowedShapeCacheKey<'k> {
        *self
    }
}

impl<'a> std::borrow::Borrow<dyn ShapeCacheKeyTrait + 'a> for ShapeCacheKey {
    fn borrow(&self) -> &(dyn ShapeCacheKeyTrait + 'a) {
        self
    }
}

impl<'a> PartialEq for dyn ShapeCacheKeyTrait + 'a {
    fn eq(&self, other: &Self) -> bool {
        self.key().eq(&other.key())
    }
}

impl<'a> Eq for dyn ShapeCacheKeyTrait + 'a {}

impl<'a> std::hash::Hash for dyn ShapeCacheKeyTrait + 'a {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().hash(state)
    }
}
