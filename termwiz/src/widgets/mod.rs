// Ideally this would be scoped to WidgetId, but I can't seem to find the
// right place for it to take effect
#![allow(clippy::new_without_default)]
use crate::color::ColorAttribute;
use crate::input::InputEvent;
use crate::surface::{Change, CursorShape, CursorVisibility, Position, SequenceNo, Surface};
use crate::Result;
use fnv::FnvHasher;
use std::collections::{HashMap, VecDeque};
use std::hash::BuildHasherDefault;

/// fnv is a more appropriate hasher for the WidgetIds we use in this module.
type FnvHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FnvHasher>>;

pub mod layout;

include!("root/types.rs");
include!("root/graph.rs");
include!("root/ui_events.rs");
include!("root/ui_render.rs");
include!("root/tests.rs");
