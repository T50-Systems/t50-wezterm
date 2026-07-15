use crate::commands::CommandDef;
use config::keyassignment::{
    ClipboardCopyDestination, ClipboardPasteSource, KeyAssignment, KeyTable, KeyTableEntry,
    KeyTables, MouseEventTrigger, SelectionMode,
};
use config::{ConfigHandle, MouseEventAltScreen, MouseEventTriggerMods};
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;
use wezterm_dynamic::{ToDynamic, Value};
use wezterm_term_api::MouseButton;
use wezterm_input_types::{KeyCode, Modifiers, PhysKeyCode, UIKeyCapRendering};

#[derive(Debug, Clone)]
pub struct OverlayKeyTables {
    pub copy_mode: KeyTable,
    pub search_mode: KeyTable,
}

include!("inputmap/types.rs");
include!("inputmap/build.rs");
include!("inputmap/lookup.rs");
include!("inputmap/format.rs");
