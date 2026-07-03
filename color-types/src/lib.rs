#![cfg_attr(not(feature = "std"), no_std)]

use core::hash::{Hash, Hasher};
use core::str::FromStr;
#[cfg(feature = "std")]
use csscolorparser::Color;
#[cfg(not(feature = "std"))]
#[allow(unused)]
use num_traits::float::Float;
#[cfg(feature = "use_serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::sync::LazyLock;
use wezterm_dynamic::{FromDynamic, FromDynamicOptions, ToDynamic, Value};

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "std")]
static SRGB_TO_F32_TABLE: LazyLock<[f32; 256]> = LazyLock::new(generate_srgb8_to_linear_f32_table);
#[cfg(feature = "std")]
static F32_TO_U8_TABLE: LazyLock<[u32; 104]> = LazyLock::new(generate_linear_f32_to_srgb8_table);
#[cfg(feature = "std")]
static RGB_TO_SRGB_TABLE: LazyLock<[u8; 256]> = LazyLock::new(generate_rgb_to_srgb8_table);
#[cfg(feature = "std")]
static RGB_TO_F32_TABLE: LazyLock<[f32; 256]> = LazyLock::new(generate_rgb_to_linear_f32_table);

include!("lib/gamma_tables.rs");
include!("lib/srgba_pixel.rs");
include!("lib/named_colors.rs");
include!("lib/srgba_tuple.rs");
include!("lib/color_math.rs");
include!("lib/srgba_parse.rs");
include!("lib/linear_rgba.rs");
include!("lib/tests.rs");
