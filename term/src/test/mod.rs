//! Various tests of the terminal model and escape sequence
//! processing routines.

use super::*;
mod api_compat;
mod c0;
use bitflags::bitflags;
mod c1;
mod csi;
mod image;
// mod selection; FIXME: port to render layer
use crate::color::ColorPalette;
use k9::assert_equal as assert_eq;
use std::sync::{Arc, Mutex};
use wezterm_escape_parser::csi::{Edit, EraseInDisplay, EraseInLine};
use wezterm_escape_parser::{OneBased, OperatingSystemCommand, CSI};
use wezterm_surface::{CursorShape, CursorVisibility, SequenceNo, SEQ_ZERO};

include!("root/support.rs");
include!("root/tests_01.rs");
include!("root/tests_02.rs");
include!("root/tests_03.rs");
