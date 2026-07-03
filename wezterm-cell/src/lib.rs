#![cfg_attr(not(feature = "std"), no_std)]
//! Model a cell in the terminal display
use crate::color::{ColorAttribute, PaletteIndex};
#[cfg(feature = "use_image")]
use crate::image::ImageCell;
use alloc::sync::Arc;
use core::hash::{Hash, Hasher};
use core::mem;
use finl_unicode::grapheme_clusters::Graphemes;
#[cfg(feature = "use_serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
pub use wezterm_char_props::emoji::Presentation;
use wezterm_char_props::emoji_variation::WCWIDTH_TABLE;
use wezterm_char_props::widechar_width::WcWidth;
use wezterm_dynamic::{FromDynamic, ToDynamic};
pub use wezterm_escape_parser::osc::Hyperlink;

extern crate alloc;
use crate::alloc::string::ToString;
use alloc::boxed::Box;
use alloc::vec::Vec;

pub mod color;
#[cfg(feature = "use_image")]
pub mod image;

include!("lib/attributes.rs");
include!("lib/semantic.rs");
include!("lib/cell_attributes.rs");
include!("lib/teeny_string.rs");
include!("lib/cell.rs");
include!("lib/unicode_width.rs");
include!("lib/attribute_change.rs");
include!("lib/tests.rs");
