use crate::*;
use luahelper::impl_lua_conversion_dynamic;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use termwiz::cell::CellAttributes;
use termwiz::color::ColorSpec as TWColorSpec;
pub use termwiz::color::{AnsiColor, ColorAttribute, RgbColor, SrgbaTuple};
use wezterm_dynamic::{FromDynamic, ToDynamic};
use wezterm_term::color::ColorPalette;

include!("color/rgba.rs");
include!("color/palette.rs");
include!("color/tab_bar.rs");
include!("color/window_frame.rs");
include!("color/scheme.rs");
