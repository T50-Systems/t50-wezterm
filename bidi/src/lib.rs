#![no_std]
use alloc::borrow::Cow;
use core::ops::Range;
use level::MAX_DEPTH;
use level_stack::{LevelStack, Override};
use log::trace;
use wezterm_dynamic::{FromDynamic, ToDynamic};

extern crate alloc;
use crate::alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

mod bidi_brackets;
mod bidi_class;
mod direction;
mod level;
mod level_stack;

use bidi_brackets::BracketType;
pub use bidi_class::BidiClass;
pub use direction::Direction;
pub use level::Level;

/// Placeholder codepoint index that corresponds to NO_LEVEL
const DELETED: usize = usize::max_value();

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromDynamic, ToDynamic)]
pub enum ParagraphDirectionHint {
    LeftToRight,
    RightToLeft,
    /// Attempt to auto-detect but fall back to LTR
    AutoLeftToRight,
    /// Attempt to auto-detect but fall back to RTL
    AutoRightToLeft,
}

impl Default for ParagraphDirectionHint {
    fn default() -> Self {
        Self::LeftToRight
    }
}

impl ParagraphDirectionHint {
    /// Returns just the direction portion of the hint, independent
    /// of the auto-detection state.
    pub fn direction(self) -> Direction {
        match self {
            ParagraphDirectionHint::AutoLeftToRight | ParagraphDirectionHint::LeftToRight => {
                Direction::LeftToRight
            }
            ParagraphDirectionHint::AutoRightToLeft | ParagraphDirectionHint::RightToLeft => {
                Direction::RightToLeft
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct BidiContext {
    orig_char_types: Vec<BidiClass>,
    char_types: Vec<BidiClass>,
    levels: Vec<Level>,
    base_level: Level,
    runs: Vec<Run>,
    reorder_nsm: bool,
}

/// Represents a formatting character that has been removed by the X9 rule
pub const NO_LEVEL: i8 = -1;

/// A `BidiRun` represents a run which is a contiguous sequence of codepoints
/// from the original paragraph that have been resolved to the same embedding
/// level, and that thus all have the same direction.
///
/// The `range` field encapsulates the starting and ending codepoint indices
/// into the original paragraph.
///
/// Note: while the run sequence has the same level throughout, the X9 portion
/// of the bidi algorithm can logically delete some control characters.
/// I haven't been able to prove to myself that those control characters
/// never manifest in the middle of a run, so it is recommended that you use the `indices`
/// method to skip over any such elements if your shaper doesn't want them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BidiRun {
    /// The direction for this run.  Derived from the level.
    pub direction: Direction,

    /// Embedding level of this run.
    pub level: Level,

    /// The starting and ending codepoint indices for this run
    pub range: Range<usize>,

    /// the list of control codepoint indices that were removed from the text
    /// by the X9 portion of the bidi algorithm.
    // Expected to have low cardinality and be generally empty, so we're
    // using a simple vec for this.
    pub removed_by_x9: Vec<usize>,
}

impl BidiRun {
    pub fn indices<'a>(&'a self) -> impl Iterator<Item = usize> + 'a {
        struct Iter<'a> {
            range: Range<usize>,
            removed_by_x9: &'a [usize],
        }

        impl<'a> Iterator for Iter<'a> {
            type Item = usize;
            fn next(&mut self) -> Option<usize> {
                for idx in self.range.by_ref() {
                    if self.removed_by_x9.iter().any(|&i| i == idx) {
                        // Skip it
                        continue;
                    }
                    return Some(idx);
                }
                None
            }
        }

        Iter {
            range: self.range.clone(),
            removed_by_x9: &self.removed_by_x9,
        }
    }
}

struct RunIter<'a> {
    pos: usize,
    levels: Cow<'a, [Level]>,
    line_range: Range<usize>,
}

impl<'a> Iterator for RunIter<'a> {
    type Item = BidiRun;

    fn next(&mut self) -> Option<BidiRun> {
        loop {
            if self.pos >= self.levels.len() {
                return None;
            }

            let start = self.pos;
            let len = span_len(start, &self.levels);
            self.pos += len;

            let level = self.levels[start];
            if !level.removed_by_x9() {
                let range = start..start + len;

                let mut removed_by_x9 = vec![];
                for idx in range.clone() {
                    if self.levels[idx].removed_by_x9() {
                        removed_by_x9.push(idx + self.line_range.start);
                    }
                }

                return Some(BidiRun {
                    direction: level.direction(),
                    level,
                    range: self.line_range.start + range.start..self.line_range.start + range.end,
                    removed_by_x9,
                });
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReorderedRun {
    /// The direction for this run.  Derived from the level.
    pub direction: Direction,

    /// Embedding level of this run.
    pub level: Level,

    /// The starting and ending codepoint indices for this run
    pub range: Range<usize>,

    /// The indices in their adjusted order
    pub indices: Vec<usize>,
}

fn span_len(start: usize, levels: &[Level]) -> usize {
    let starting_level = levels[start];
    levels
        .iter()
        .skip(start + 1)
        .position(|&l| l != starting_level)
        .unwrap_or(levels.len() - (start + 1))
        + 1
}

include!("lib/context_api.rs");
include!("lib/context_resolve_weak.rs");
include!("lib/context_brackets_neutrals.rs");
include!("lib/context_levels.rs");
include!("lib/context_runs.rs");
include!("lib/bidi_class_impl.rs");
include!("lib/runs.rs");
include!("lib/brackets.rs");
include!("lib/tests.rs");
