use crate::selection::{SelectionCoordinate, SelectionRange, SelectionX};
use crate::termwindow::keyevent::KeyTableArgs;
use crate::termwindow::{TermWindow, TermWindowNotif};
use config::keyassignment::{
    CopyModeAssignment, KeyAssignment, ScrollbackEraseMode, SelectionMode,
};
use mux::domain::DomainId;
use mux::pane::{
    CachePolicy, ForEachPaneLogicalLine, LogicalLine, Pane, PaneId, Pattern, PatternType,
    PerformAssignmentResult, SearchResult, WithPaneLines,
};
use mux::renderable::*;
use mux::tab::TabId;
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
use rangeset::RangeSet;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;
use termwiz::cell::{Cell, CellAttributes};
use termwiz::color::AnsiColor;
use termwiz::lineedit::{LineEditBuffer, Movement};
use termwiz::surface::{CursorVisibility, SequenceNo, SEQ_ZERO};
use unicode_segmentation::*;
use url::Url;
use wezterm_term::color::ColorPalette;
use wezterm_term::{
    unicode_column_width, Clipboard, KeyCode, KeyModifiers, Line, MouseEvent, SemanticType,
    StableRowIndex, TerminalSize,
};
use window::WindowOps;

lazy_static::lazy_static! {
    static ref SAVED_PATTERN: Mutex<HashMap<TabId, Pattern>> = Mutex::new(HashMap::new());
}

const SEARCH_CHUNK_SIZE: StableRowIndex = 1000;

pub struct CopyOverlay {
    delegate: Arc<dyn Pane>,
    render: Arc<Mutex<CopyRenderable>>,
    writer: Mutex<SearchOverlayPatternWriter>,
}

#[derive(Copy, Clone, Debug)]
struct PendingJump {
    forward: bool,
    prev_char: bool,
}

#[derive(Copy, Clone, Debug)]
struct Jump {
    forward: bool,
    prev_char: bool,
    target: char,
}

struct CopyRenderable {
    cursor: StableCursorPosition,
    delegate: Arc<dyn Pane>,
    start: Option<SelectionCoordinate>,
    selection_mode: SelectionMode,
    viewport: Option<StableRowIndex>,
    /// We use this to cancel ourselves later
    window: ::window::Window,

    /// The text that the user entered
    pattern_type: PatternType,
    search_line: LineEditBuffer,
    /// The most recently queried set of matches
    results: Vec<SearchResult>,
    by_line: HashMap<StableRowIndex, Vec<MatchResult>>,
    last_result_seqno: SequenceNo,
    last_bar_pos: Option<StableRowIndex>,
    dirty_results: RangeSet<StableRowIndex>,
    width: usize,
    height: usize,
    editing_search: bool,
    result_pos: Option<usize>,
    tab_id: TabId,
    /// Used to debounce queries while the user is typing
    typing_cookie: usize,
    searching: Option<Searching>,
    pending_jump: Option<PendingJump>,
    last_jump: Option<Jump>,
}

struct Searching {
    remain: StableRowIndex,
}

#[derive(Debug)]
struct MatchResult {
    range: Range<usize>,
    result_index: usize,
}

struct Dimensions {
    vertical_gap: isize,
    dims: RenderableDimensions,
    top: StableRowIndex,
}

#[derive(Debug)]
pub struct CopyModeParams {
    pub pattern: Pattern,
    pub editing_search: bool,
}
