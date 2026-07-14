use crate::domain::DomainId;
use crate::renderable::*;
use crate::ExitBehavior;
use async_trait::async_trait;
use config::keyassignment::{KeyAssignment, ScrollbackEraseMode};
use downcast_rs::{impl_downcast, Downcast};
use parking_lot::MappedMutexGuard;
use rangeset::RangeSet;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;
use termwiz::hyperlink::Rule;
use termwiz::input::KeyboardEncoding;
use termwiz::surface::{Line, SequenceNo};
use url::Url;
use wezterm_dynamic::Value;
use wezterm_term::color::ColorPalette;
use wezterm_term::{
    Clipboard, DownloadHandler, KeyCode, KeyModifiers, MouseEvent, Progress, SemanticZone,
    StableRowIndex, TerminalConfiguration, TerminalSize,
};

static PANE_ID: ::std::sync::atomic::AtomicUsize = ::std::sync::atomic::AtomicUsize::new(0);
pub use wezterm_mux_protocol::pane::{PaneId, Pattern, PatternType, SearchResult};

pub fn alloc_pane_id() -> PaneId {
    PANE_ID.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PerformAssignmentResult {
    /// Continue search for handler
    Unhandled,
    /// Found handler and acted upon the action
    Handled,
    /// Do not perform assignment, but instead treat the key event
    /// as though there was no assignment and run it as a key_down
    /// event.
    BlockAssignmentAndRouteToKeyDown,
}

/// Why a close request is being made
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CloseReason {
    /// The containing window is being closed
    Window,
    /// The containing tab is being close
    Tab,
    /// Just this tab is being closed
    Pane,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicalLine {
    pub physical_lines: Vec<Line>,
    pub logical: Line,
    pub first_row: StableRowIndex,
}

impl LogicalLine {
    pub fn contains_y(&self, y: StableRowIndex) -> bool {
        y >= self.first_row && y < self.first_row + self.physical_lines.len() as StableRowIndex
    }

    pub fn xy_to_logical_x(&self, x: usize, y: StableRowIndex) -> usize {
        let mut offset = 0;
        for (idx, line) in self.physical_lines.iter().enumerate() {
            let phys_y = self.first_row + idx as StableRowIndex;
            if y < phys_y {
                // Eg: trying to drag off the top of the viewport.
                // Their y coordinate precedes our first line, so
                // the only logical x we can return is 0
                return 0;
            }
            if phys_y == y {
                return offset + x;
            }
            offset += line.len();
        }
        // Allow selecting off the end of the line
        offset + x
    }

    pub fn logical_x_to_physical_coord(&self, x: usize) -> (StableRowIndex, usize) {
        let mut y = self.first_row;
        let mut idx = 0;
        for line in &self.physical_lines {
            let x_off = x - idx;
            let line_len = line.len();
            if x_off < line_len {
                return (y, x_off);
            }
            y += 1;
            idx += line_len;
        }
        (y - 1, x - idx + self.physical_lines.last().unwrap().len())
    }
}
