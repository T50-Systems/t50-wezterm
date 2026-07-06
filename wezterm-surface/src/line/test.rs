#![cfg(test)]

use super::*;
use crate::hyperlink::{Hyperlink, Rule};
use crate::line::clusterline::ClusteredLine;
use crate::SEQ_ZERO;
use alloc::sync::Arc;
use k9::assert_equal as assert_eq;
use wezterm_cell::{Cell, CellAttributes};

include!("test/basic.rs");
include!("test/wrap_and_attrs.rs");
include!("test/append.rs");
include!("test/line_new.rs");
