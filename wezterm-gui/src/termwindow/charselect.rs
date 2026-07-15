use crate::overlay::selector::{matcher_pattern, matcher_score};
use crate::termwindow::box_model::*;
use crate::termwindow::modal::Modal;
use crate::termwindow::render::corners::{
    BOTTOM_LEFT_ROUNDED_CORNER, BOTTOM_RIGHT_ROUNDED_CORNER, TOP_LEFT_ROUNDED_CORNER,
    TOP_RIGHT_ROUNDED_CORNER,
};
use crate::termwindow::DimensionContext;
use crate::utilsprites::RenderMetrics;
use crate::TermWindow;
use config::keyassignment::{
    CharSelectArguments, CharSelectGroup, ClipboardCopyDestination, KeyAssignment,
};
use wezterm_config_types::Dimension;
use emojis::{Emoji, Group};
use frecency::Frecency;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::path::PathBuf;
use termwiz::input::Modifiers;
use wezterm_term::{KeyCode, KeyModifiers, MouseEvent};
use window::color::LinearRgba;

include!("charselect/types.rs");
include!("charselect/aliases.rs");
include!("charselect/selector.rs");
include!("charselect/modal.rs");
