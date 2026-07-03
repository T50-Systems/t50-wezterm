#![allow(clippy::range_plus_one)]
use super::renderstate::*;
use super::utilsprites::RenderMetrics;
use crate::colorease::ColorEase;
use crate::frontend::{front_end, try_front_end};
use crate::inputmap::InputMap;
use crate::overlay::{
    confirm_close_pane, confirm_close_tab, confirm_close_window, confirm_quit_program, launcher,
    start_overlay, start_overlay_pane, CopyModeParams, CopyOverlay, LauncherArgs, LauncherFlags,
    QuickSelectOverlay,
};
use crate::resize_increment_calculator::ResizeIncrementCalculator;
use crate::scripting::guiwin::GuiWin;
use crate::scrollbar::*;
use crate::selection::Selection;
use crate::shapecache::*;
use crate::tabbar::{TabBarItem, TabBarState};
use crate::termwindow::background::{
    load_background_image, reload_background_image, LoadedBackgroundLayer,
};
use crate::termwindow::keyevent::{KeyTableArgs, KeyTableState};
use crate::termwindow::modal::Modal;
use crate::termwindow::render::paint::AllowImage;
use crate::termwindow::render::{
    CachedLineState, LineQuadCacheKey, LineQuadCacheValue, LineToEleShapeCacheKey,
    LineToElementShapeItem,
};
use crate::termwindow::webgpu::WebGpuState;
use ::wezterm_term::input::{ClickPosition, MouseButton as TMB};
use ::window::*;
use anyhow::{anyhow, ensure, Context};
use config::keyassignment::{
    Confirmation, KeyAssignment, LauncherActionArgs, PaneDirection, Pattern, PromptInputLine,
    QuickSelectArguments, RotationDirection, SpawnCommand, SplitSize,
};
use config::window::WindowLevel;
use config::{
    configuration, AudibleBell, ConfigHandle, Dimension, DimensionContext, FrontEndSelection,
    GeometryOrigin, GuiPosition, TermConfig, WindowCloseConfirmation,
};
use lfucache::*;
use mlua::{FromLua, LuaSerdeExt, UserData, UserDataFields};
use mux::pane::{
    CachePolicy, CloseReason, Pane, PaneId, Pattern as MuxPattern, PerformAssignmentResult,
};
use mux::renderable::RenderableDimensions;
use mux::tab::{
    PositionedPane, PositionedSplit, SplitDirection, SplitRequest, SplitSize as MuxSplitSize, Tab,
    TabId,
};
use mux::window::WindowId as MuxWindowId;
use mux::{Mux, MuxNotification};
use mux_lua::MuxPane;
use smol::channel::Sender;
use smol::Timer;
use std::cell::{RefCell, RefMut};
use std::collections::{HashMap, LinkedList};
use std::ops::Add;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use termwiz::hyperlink::Hyperlink;
use termwiz::surface::SequenceNo;
use wezterm_dynamic::Value;
use wezterm_font::FontConfiguration;
use wezterm_term::color::ColorPalette;
use wezterm_term::input::LastMouseClick;
use wezterm_term::{Alert, Progress, StableRowIndex, TerminalConfiguration, TerminalSize};

pub mod background;
pub mod box_model;
pub mod charselect;
pub mod clipboard;
pub mod keyevent;
pub mod modal;
mod mouseevent;
pub mod palette;
pub mod paneselect;
mod prevcursor;
pub mod render;
pub mod resize;
mod selection;
pub mod spawn;
pub mod webgpu;
use crate::spawn::SpawnWhere;
use prevcursor::PrevCursorPos;

include!("root/support.rs");
include!("root/bootstrap.rs");
include!("root/impl_part_01.rs");
include!("root/impl_part_02.rs");
include!("root/impl_part_03.rs");
include!("root/impl_part_04.rs");
include!("root/impl_part_05.rs");
include!("root/impl_part_06.rs");
include!("root/impl_part_07.rs");
include!("root/impl_key_assignment_helpers.rs");
include!("root/impl_part_08.rs");
include!("root/impl_part_09.rs");
include!("root/drop.rs");
