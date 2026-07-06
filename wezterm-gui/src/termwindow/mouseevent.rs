use crate::tabbar::{TabBarItem, TabBarZone};
use crate::termwindow::{
    GuiWin, MouseCapture, PositionedSplit, ScrollHit, TermWindowNotif, UIItem, UIItemType, TMB,
};
use ::window::{
    MouseButtons as WMB, MouseCursor, MouseEvent, MouseEventKind as WMEK, MousePress,
    WindowDecorations, WindowOps, WindowState,
};
use config::keyassignment::{KeyAssignment, MouseEventTrigger, SpawnTabDomain};
use config::MouseEventAltScreen;
use mux::pane::{Pane, WithPaneLines};
use mux::tab::SplitDirection;
use mux::Mux;
use mux_lua::MuxPane;
use std::convert::TryInto;
use std::ops::Sub;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use termwiz::hyperlink::Hyperlink;
use termwiz::surface::Line;
use wezterm_dynamic::ToDynamic;
use wezterm_term::input::{MouseButton, MouseEventKind as TMEK};
use wezterm_term::{ClickPosition, LastMouseClick, StableRowIndex};

include!("mouseevent/ui.rs");
include!("mouseevent/tabbar.rs");
include!("mouseevent/terminal.rs");
include!("mouseevent/helpers.rs");
