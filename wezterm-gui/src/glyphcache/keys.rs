use super::utilsprites::RenderMetrics;
use crate::customglyph::*;
use crate::renderstate::RenderContext;
use crate::termwindow::render::paint::AllowImage;
use ::window::bitmaps::atlas::{Atlas, OutOfTextureSpace, Sprite};
use ::window::bitmaps::{BitmapImage, Image, ImageTexture, Texture2d};
use ::window::color::SrgbaPixel;
use ::window::{Point, Rect};
use anyhow::Context;
use config::{AllowSquareGlyphOverflow, TextStyle};
use euclid::num::Zero;
use image::{
    AnimationDecoder, DynamicImage, Frame, Frames, ImageDecoder, ImageFormat, ImageResult, Limits,
};
use lfucache::LfuCache;
use ordered_float::NotNan;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Seek;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender, TryRecvError};
use std::sync::{Arc, LazyLock, MutexGuard};
use std::time::{Duration, Instant};
use termwiz::color::RgbColor;
use termwiz::image::{ImageData, ImageDataType};
use termwiz::surface::CursorShape;
use wezterm_blob_leases::{BlobLease, BlobManager, BoxedReader};
use wezterm_font::units::*;
use wezterm_font::{FontConfiguration, GlyphInfo, LoadedFont, LoadedFontId};
use wezterm_term::Underline;

static FRAME_ERROR_REPORTED: AtomicBool = AtomicBool::new(false);

/// We only want to report a frame error once at error level, because
/// if it is triggering it is likely in a animated image and will continue
/// to trigger multiple times per second as the frames are cycled.
fn report_frame_error<S: Into<String>>(message: S) {
    if FRAME_ERROR_REPORTED.load(Ordering::Relaxed) {
        log::debug!("{}", message.into());
    } else {
        log::error!("{}", message.into());
        FRAME_ERROR_REPORTED.store(true, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadState {
    Loading,
    Loaded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellMetricKey {
    pub pixel_width: u16,
    pub pixel_height: u16,
}

impl From<&RenderMetrics> for CellMetricKey {
    fn from(metrics: &RenderMetrics) -> CellMetricKey {
        CellMetricKey {
            pixel_width: metrics.cell_size.width as u16,
            pixel_height: metrics.cell_size.height as u16,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SizedBlockKey {
    pub block: BlockKey,
    pub size: CellMetricKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font_idx: usize,
    pub glyph_pos: u32,
    pub num_cells: u8,
    pub style: TextStyle,
    pub followed_by_space: bool,
    pub metric: CellMetricKey,
    pub id: LoadedFontId,
}

/// We'd like to avoid allocating when resolving from the cache
/// so this is the borrowed version of GlyphKey.
/// It's a bit involved to make this work; more details can be
/// found in the excellent guide here:
/// <https://github.com/sunshowers/borrow-complex-key-example/blob/master/src/lib.rs>
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BorrowedGlyphKey<'a> {
    pub font_idx: usize,
    pub glyph_pos: u32,
    pub num_cells: u8,
    pub style: &'a TextStyle,
    pub followed_by_space: bool,
    pub metric: CellMetricKey,
    pub id: LoadedFontId,
}

impl<'a> BorrowedGlyphKey<'a> {
    fn to_owned(&self) -> GlyphKey {
        GlyphKey {
            font_idx: self.font_idx,
            glyph_pos: self.glyph_pos,
            num_cells: self.num_cells,
            style: self.style.clone(),
            followed_by_space: self.followed_by_space,
            metric: self.metric,
            id: self.id,
        }
    }
}

trait GlyphKeyTrait {
    fn key<'k>(&'k self) -> BorrowedGlyphKey<'k>;
}

impl GlyphKeyTrait for GlyphKey {
    fn key<'k>(&'k self) -> BorrowedGlyphKey<'k> {
        BorrowedGlyphKey {
            font_idx: self.font_idx,
            glyph_pos: self.glyph_pos,
            num_cells: self.num_cells,
            style: &self.style,
            followed_by_space: self.followed_by_space,
            metric: self.metric,
            id: self.id,
        }
    }
}

impl<'a> GlyphKeyTrait for BorrowedGlyphKey<'a> {
    fn key<'k>(&'k self) -> BorrowedGlyphKey<'k> {
        *self
    }
}

impl<'a> std::borrow::Borrow<dyn GlyphKeyTrait + 'a> for GlyphKey {
    fn borrow(&self) -> &(dyn GlyphKeyTrait + 'a) {
        self
    }
}

impl<'a> PartialEq for dyn GlyphKeyTrait + 'a {
    fn eq(&self, other: &Self) -> bool {
        self.key().eq(&other.key())
    }
}

impl<'a> Eq for dyn GlyphKeyTrait + 'a {}

impl<'a> std::hash::Hash for dyn GlyphKeyTrait + 'a {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().hash(state)
    }
}

/// Caches a rendered glyph.
/// The image data may be None for whitespace glyphs.
pub struct CachedGlyph {
    pub has_color: bool,
    pub brightness_adjust: f32,
    pub x_offset: PixelLength,
    pub y_offset: PixelLength,
    pub x_advance: PixelLength,
    pub bearing_x: PixelLength,
    pub bearing_y: PixelLength,
    pub texture: Option<Sprite>,
    pub scale: f64,
}

impl std::fmt::Debug for CachedGlyph {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        fmt.debug_struct("CachedGlyph")
            .field("has_color", &self.has_color)
            .field("x_advance", &self.x_advance)
            .field("x_offset", &self.x_offset)
            .field("y_offset", &self.y_offset)
            .field("bearing_x", &self.bearing_x)
            .field("bearing_y", &self.bearing_y)
            .field("scale", &self.scale)
            .field("texture", &self.texture)
            .finish()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct LineKey {
    strike_through: bool,
    underline: Underline,
    overline: bool,
    size: CellMetricKey,
}
