use crate::color::RgbaColor;
use crate::*;
use bitflags::*;
use enum_display_derive::Display;
use luahelper::impl_lua_conversion_dynamic;
use std::convert::TryFrom;
use std::fmt::Display;
use wezterm_dynamic::{FromDynamic, FromDynamicOptions, ToDynamic, Value};

include!("font/weight.rs");
include!("font/raster_flags.rs");
include!("font/attributes.rs");
include!("font/text_style.rs");
include!("font/rules_and_tests.rs");
