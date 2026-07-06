use anyhow::{anyhow, bail, ensure, Error};
use std::ffi::c_void;
use std::rc::Rc;

include!("egl/ffi.rs");
include!("egl/types.rs");
include!("egl/wrapper.rs");
include!("egl/state.rs");
