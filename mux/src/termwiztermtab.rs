//! a tab hosting a termwiz terminal applet
//! The idea is to use these when wezterm needs to request
//! input from the user as part of eg: setting up an ssh
//! session.

use crate::domain::{alloc_domain_id, Domain, DomainId, DomainState};
use crate::pane::{
    alloc_pane_id, CachePolicy, CloseReason, ForEachPaneLogicalLine, LogicalLine, Pane, PaneId,
    WithPaneLines,
};
use crate::renderable::*;
use crate::tab::Tab;
use crate::window::WindowId;
use crate::Mux;
use anyhow::bail;
use async_trait::async_trait;
use config::keyassignment::ScrollbackEraseMode;
use crossbeam::channel::{unbounded as channel, Receiver, Sender};
use filedescriptor::{FileDescriptor, Pipe};
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
use portable_pty::*;
use rangeset::RangeSet;
use std::io::{BufWriter, Write};
use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;
use termwiz::input::{InputEvent, KeyEvent, Modifiers, MouseEvent as TermWizMouseEvent};
use termwiz::render::terminfo::TerminfoRenderer;
use termwiz::surface::{Change, Line, SequenceNo};
use termwiz::terminal::{ScreenSize, TerminalWaker};
use termwiz::Context;
use url::Url;
use wezterm_term::color::ColorPalette;
use wezterm_term::{
    KeyCode, KeyModifiers, MouseEvent, StableRowIndex, TerminalConfiguration, TerminalSize,
};

include!("termwiztermtab/domain.rs");
include!("termwiztermtab/pane.rs");
include!("termwiztermtab/terminal_types.rs");
include!("termwiztermtab/terminal_impl.rs");
include!("termwiztermtab/allocate_run.rs");
