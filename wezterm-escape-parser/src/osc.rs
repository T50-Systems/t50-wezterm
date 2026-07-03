use crate::color::SrgbaTuple;
pub use crate::hyperlink::Hyperlink;
use crate::{bail, ensure, format_err, Result};
use base64::Engine;
use bitflags::bitflags;
use core::fmt::{Display, Error as FmtError, Formatter, Result as FmtResult};
use core::str;
use core::str::FromStr;
use num_derive::*;
use num_traits::FromPrimitive;
use ordered_float::NotNan;
#[cfg(feature = "std")]
use std::sync::LazyLock;

use crate::allocate::*;

include!("osc/commands.rs");
include!("osc/codes.rs");
include!("osc/finalterm.rs");
include!("osc/iterm_data.rs");
include!("osc/iterm_parse.rs");

#[cfg(test)]
mod test {
    use super::*;

    include!("osc/tests_support.rs");
    include!("osc/tests_01.rs");
    include!("osc/tests_02.rs");
}
