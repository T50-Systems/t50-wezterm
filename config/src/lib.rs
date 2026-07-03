//! Configuration for the gui portion of the terminal

use anyhow::{anyhow, bail, Context, Error};
use lazy_static::lazy_static;
use mlua::Lua;
use ordered_float::NotNan;
use smol::channel::{Receiver, Sender};
use smol::prelude::*;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::fs::DirBuilder;
#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use wezterm_dynamic::{FromDynamic, FromDynamicOptions, ToDynamic, UnknownFieldAction, Value};
use wezterm_term::UnicodeVersion;

mod background;
mod bell;
mod cell;
mod color;
mod config;
mod daemon;
mod exec_domain;
mod font;
mod frontend;
pub mod keyassignment;
mod keys;
pub mod lua;
pub mod meta;
mod scheme_data;
mod serial;
mod ssh;
mod terminal;
mod tls;
mod units;
mod unix;
mod version;
pub mod window;
mod wsl;

pub use crate::config::*;
pub use background::*;
pub use bell::*;
pub use cell::*;
pub use color::*;
pub use daemon::*;
pub use exec_domain::*;
pub use font::*;
pub use frontend::*;
pub use keys::*;
pub use serial::*;
pub use ssh::*;
pub use terminal::*;
pub use tls::*;
pub use units::*;
pub use unix::*;
pub use version::*;
pub use wsl::*;

type ErrorCallback = fn(&str);

include!("root/dynamic.rs");
include!("root/lua_thread.rs");
include!("root/load.rs");
include!("root/inner.rs");
include!("root/handle.rs");
