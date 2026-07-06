// suppress inscrutable useless_attribute clippy that shows up when
// using derive(FromPrimitive)
#![allow(clippy::useless_attribute)]
#![allow(clippy::upper_case_acronyms)]
#![cfg_attr(not(feature = "std"), no_std)]
//! This module provides the ability to parse escape sequences and attach
//! semantic meaning to them.  It can also encode the semantic values as
//! escape sequences.  It provides encoding and decoding functionality
//! only; it does not provide terminal emulation facilities itself.
#[cfg(feature = "tmux_cc")]
use crate::tmux_cc::Event;
use core::fmt::{Display, Formatter, Result as FmtResult, Write as FmtWrite};
use num_derive::*;
use wezterm_color_types::LinearRgba;

#[cfg_attr(not(feature = "std"), macro_use)]
extern crate alloc;

mod allocate;
use allocate::*;

pub mod apc;
pub mod color;
pub mod csi;
pub mod error;
pub mod esc;
pub mod hyperlink;
pub mod osc;
pub mod parser;
#[cfg(feature = "tmux_cc")]
pub mod tmux_cc;

pub use self::apc::KittyImage;
pub use self::csi::CSI;
pub use self::error::{Error, Result};
pub use self::esc::{Esc, EscCode};
pub use self::osc::OperatingSystemCommand;

use vtparse::CsiParam;

include!("root/action.rs");
include!("root/device_control.rs");
include!("root/sixel.rs");
include!("root/control_code.rs");
include!("root/one_based.rs");
