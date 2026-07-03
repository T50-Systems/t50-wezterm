use crate::commands::CommandDef;
use config::keyassignment::{
    ClipboardCopyDestination, ClipboardPasteSource, KeyAssignment, KeyTableEntry, KeyTables,
    MouseEventTrigger, SelectionMode,
};
use config::{ConfigHandle, MouseEventAltScreen, MouseEventTriggerMods};
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;
use wezterm_dynamic::{ToDynamic, Value};
use wezterm_term::input::MouseButton;
use window::{KeyCode, Modifiers, PhysKeyCode, UIKeyCapRendering};

include!("inputmap/types.rs");
include!("inputmap/build.rs");
include!("inputmap/lookup.rs");
include!("inputmap/format.rs");
