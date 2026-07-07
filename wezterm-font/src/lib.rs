use crate::db::FontDatabase;
use crate::locator::{new_locator, FontLocator};
use crate::parser::ParsedFont;
use crate::rasterizer::{new_rasterizer, FontRasterizer};
use crate::shaper::{new_shaper, FontShaper, PresentationWidth};
use anyhow::{Context, Error};
use config::{
    configuration, BoldBrightening, ConfigHandle, DisplayPixelGeometry, FontAttributes,
    FontRasterizerSelection, FontStretch, FontStyle, FontWeight, TextStyle,
};
use rangeset::RangeSet;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::rc::{Rc, Weak};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use termwiz::cell::Presentation;
use thiserror::Error;
use wezterm_bidi::Direction;
use wezterm_term::{CellAttributes, Intensity};
use wezterm_toast_notification::ToastNotification;

mod hbwrap;

pub mod db;
pub mod ftwrap;
pub mod locator;
pub mod parser;
pub mod rasterizer;
pub mod shaper;
pub mod units;

#[cfg(all(unix, not(target_os = "macos")))]
pub mod fcwrap;

pub use crate::rasterizer::RasterizedGlyph;
pub use crate::shaper::{FallbackIdx, FontMetrics, GlyphInfo};

include!("root/loaded_font.rs");
include!("root/fallback.rs");
include!("root/font_config_types.rs");
include!("root/font_config_build.rs");
include!("root/font_config_resolve.rs");
include!("root/font_configuration.rs");
