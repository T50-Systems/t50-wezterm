use crate::colorease::ColorEase;
use crate::customglyph::{BlockKey, *};
use crate::glyphcache::{CachedGlyph, GlyphCache};
use crate::quad::{
    HeapQuadAllocator, QuadAllocator, QuadImpl, QuadTrait, TripleLayerQuadAllocator,
    TripleLayerQuadAllocatorTrait,
};
use crate::shapecache::*;
use crate::termwindow::render::paint::AllowImage;
use crate::termwindow::{BorrowedShapeCacheKey, RenderState, ShapedInfo, TermWindowNotif};
use crate::utilsprites::RenderMetrics;
use ::window::bitmaps::{TextureCoord, TextureRect, TextureSize};
use ::window::{DeadKeyStatus, PointF, RectF, SizeF, WindowOps};
use anyhow::{anyhow, Context};
use config::{
    BoldBrightening, ConfigHandle, DimensionContext, HorizontalWindowContentAlignment, TextStyle,
    VerticalWindowContentAlignment, VisualBellTarget,
};
use euclid::num::Zero;
use mux::pane::{Pane, PaneId};
use mux::renderable::{RenderableDimensions, StableCursorPosition};
use ordered_float::NotNan;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use termwiz::cellcluster::CellCluster;
use termwiz::hyperlink::Hyperlink;
use termwiz::surface::{CursorShape, CursorVisibility, SequenceNo};
use wezterm_font::shaper::PresentationWidth;
use wezterm_font::units::{IntPixelLength, PixelLength};
use wezterm_font::{ClearShapeCache, GlyphInfo, LoadedFont};
use wezterm_term::color::{ColorAttribute, ColorPalette};
use wezterm_term::{CellAttributes, Line, StableRowIndex};
use window::color::LinearRgba;

pub mod borders;
pub mod corners;
pub mod draw;
pub mod fancy_tab_bar;
pub mod paint;
pub mod pane;
pub mod screen_line;
pub mod split;
pub mod tab_bar;
pub mod window_buttons;

include!("root/types.rs");
include!("root/geometry.rs");
include!("root/cell_colors.rs");
include!("root/shape_cache.rs");
include!("root/free_helpers.rs");
