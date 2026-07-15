//! Configuration for the gui portion of the terminal

use anyhow::{anyhow, bail, Context, Error};
use lazy_static::lazy_static;
use mlua::Lua;
use smol::channel::{Receiver, Sender};
use smol::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::DirBuilder;
#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use wezterm_dynamic::{FromDynamic, FromDynamicOptions, UnknownFieldAction};
use wezterm_term_api::UnicodeVersion;

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

#[cfg(test)]
mod compatibility_tests {
    use std::any::TypeId;

    fn assert_same_type<T: 'static, U: 'static>() {
        assert_eq!(TypeId::of::<T>(), TypeId::of::<U>());
    }

    #[test]
    fn facade_reexports_preserve_type_identity() {
        assert_same_type::<crate::AudibleBell, wezterm_config_types::AudibleBell>();
        assert_same_type::<crate::Palette, wezterm_config_types::Palette>();
        assert_same_type::<crate::FontAttributes, wezterm_config_types::FontAttributes>();
        assert_same_type::<crate::DefaultCursorStyle, wezterm_config_types::DefaultCursorStyle>();
        assert_same_type::<
            crate::keyassignment::KeyAssignment,
            wezterm_config_types::keyassignment::KeyAssignment,
        >();
        assert_same_type::<crate::window::WindowLevel, wezterm_config_types::window::WindowLevel>();
    }
}
