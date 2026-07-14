//! This crate provides the core of the virtual terminal emulator implementation
//! used by [wezterm](https://wezterm.org/).  The home for this
//! crate is in the wezterm repo and development is tracked at
//! <https://github.com/wezterm/wezterm/>.
//!
//! It is full featured, providing terminal escape sequence parsing, keyboard
//! and mouse input encoding, a model for the screen cells including scrollback,
//! sixel and iTerm2 image support, OSC 8 Hyperlinks and a wide range of
//! terminal cell attributes.
//!
//! This crate does not provide any kind of gui, nor does it directly
//! manage a PTY; you provide a `std::io::Write` implementation that
//! could connect to a PTY, and supply bytes to the model via the
//! `advance_bytes` method.
//!
//! The entrypoint to the crate is the [Terminal](terminal/struct.Terminal.html)
//! struct.
use anyhow::Error;
use std::ops::{Deref, DerefMut, Range};
use std::str;

pub mod color;
pub mod config;
pub use config::TerminalConfiguration;

pub mod input;
pub use crate::input::*;

pub use wezterm_cell::*;
pub use wezterm_surface::line::*;
pub use wezterm_term_api::{
    intersects_range, CursorPosition, PhysRowIndex, Position, ScrollbackOrVisibleRowIndex,
    SemanticZone, StableRowIndex, VisibleRowIndex, CSI, DCS, OSC, SS3, ST,
};

pub mod screen;
pub use crate::screen::*;

pub mod terminal;
pub use crate::terminal::*;

pub mod terminalstate;
pub use crate::terminalstate::*;

#[cfg(test)]
mod test;
