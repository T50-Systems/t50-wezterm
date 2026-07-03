//! An implementation of the state machine described by
//! [DEC ANSI Parser](https://vt100.net/emu/dec_ansi_parser), modified to support UTF-8.
//!
//! This is sufficient to broadly categorize ANSI/ECMA-48 escape sequences that are
//! commonly used in terminal emulators.  It does not ascribe semantic meaning to
//! those escape sequences; for example, if you wish to parse the SGR sequence
//! that makes text bold, you will need to know which codes correspond to bold
//! in your implementation of `VTActor`.
//!
//! You may wish to use `termwiz::escape::parser::Parser` in the
//! [termwiz](https://docs.rs/termwiz/) crate if you don't want to have to research
//! all those possible escape sequences for yourself.
#![allow(clippy::upper_case_acronyms)]
#![cfg_attr(not(feature = "std"), no_std)]
use utf8parse::Parser as Utf8Parser;
mod enums;
use crate::enums::*;
mod transitions;

#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

#[cfg(any(feature = "std", feature = "alloc"))]
use alloc::vec::Vec;
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
use heapless::Vec;

use transitions::{ENTRY, EXIT, TRANSITIONS};

include!("lib/lookup.rs");
include!("lib/actor.rs");
include!("lib/parser_types.rs");
include!("lib/parser_impl.rs");
include!("lib/tests.rs");
