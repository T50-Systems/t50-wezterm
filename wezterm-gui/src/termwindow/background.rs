use crate::color::LinearRgba;
use crate::glyphcache::LoadState;
use crate::quad::{QuadAllocator, QuadTrait};
use crate::termwindow::RenderState;
use crate::utilsprites::RenderMetrics;
use crate::Dimensions;
use anyhow::Context;
use config::{
    BackgroundHorizontalAlignment, BackgroundLayer, BackgroundRepeat, BackgroundSize,
    BackgroundSource, BackgroundVerticalAlignment, ConfigHandle, DimensionContext, Gradient,
    GradientOrientation,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use termwiz::image::{ImageData, ImageDataType};
use wezterm_term::StableRowIndex;

lazy_static::lazy_static! {
    static ref IMAGE_CACHE: Mutex<HashMap<String, CachedImage>> = Mutex::new(HashMap::new());
    static ref GRADIENT_CACHE: Mutex<Vec<CachedGradient>> = Mutex::new(vec![]);
}

include!("background/cache.rs");
include!("background/load.rs");
include!("background/render.rs");
