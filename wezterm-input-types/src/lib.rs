#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "serde")]
use ::serde::*;
use alloc::sync::Arc;
use bitflags::*;
use core::convert::TryFrom;
use core::fmt::Write;
use core::sync::atomic::AtomicBool;
#[cfg(feature = "std")]
use std::sync::LazyLock;
use wezterm_dynamic::{FromDynamic, ToDynamic};

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};

#[cfg(feature = "std")]
use std::collections::HashMap;

pub struct PixelUnit;
pub struct ScreenPixelUnit;
pub type Point = euclid::Point2D<isize, PixelUnit>;
pub type PointF = euclid::Point2D<f32, PixelUnit>;
pub type ScreenPoint = euclid::Point2D<isize, ScreenPixelUnit>;

include!("lib/key_code.rs");
include!("lib/modifiers.rs");
include!("lib/phys_key_code.rs");
include!("lib/mouse.rs");
include!("lib/key_event_helpers.rs");
include!("lib/key_event.rs");
include!("lib/csi_u.rs");
include!("lib/decorations.rs");
include!("lib/ui_key_cap.rs");

#[cfg(test)]
mod test {
    use super::*;

    include!("lib/tests_01.rs");
    include!("lib/tests_02.rs");
    include!("lib/tests_03.rs");
}
