#![cfg_attr(not(feature = "std"), no_std)]
use crate::line::CellRef;
use alloc::borrow::Cow;
use core::cmp::min;
use finl_unicode::grapheme_clusters::Graphemes;
#[cfg(feature = "use_serde")]
use serde::{Deserialize, Serialize};
use wezterm_cell::color::ColorAttribute;
#[cfg(feature = "use_image")]
use wezterm_cell::image::ImageCell;
use wezterm_cell::{Cell, CellAttributes};
use wezterm_dynamic::{FromDynamic, ToDynamic};

extern crate alloc;
use crate::alloc::borrow::ToOwned;
use crate::alloc::string::ToString;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

pub mod cellcluster;
pub mod change;
pub mod hyperlink;
pub mod line;

pub use self::change::{Change, LineAttribute};
#[cfg(feature = "use_image")]
pub use self::change::{Image, TextureCoordinate};
pub use self::line::Line;

/// Position holds 0-based positioning information, where
/// Absolute(0) is the start of the line or column,
/// Relative(0) is the current position in the line or
/// column and EndRelative(0) is the end position in the
/// line or column.
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Position {
    /// Negative values move up, positive values down, 0 means no change
    Relative(isize),
    /// Relative to the start of the line or top of the screen
    Absolute(usize),
    /// Relative to the end of line or bottom of screen
    EndRelative(usize),
}

#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Hash, Copy, PartialEq, Eq, FromDynamic, ToDynamic)]
pub enum CursorVisibility {
    Hidden,
    Visible,
}

impl Default for CursorVisibility {
    fn default() -> CursorVisibility {
        CursorVisibility::Visible
    }
}

#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromDynamic, ToDynamic)]
pub enum CursorShape {
    Default,
    BlinkingBlock,
    SteadyBlock,
    BlinkingUnderline,
    SteadyUnderline,
    BlinkingBar,
    SteadyBar,
}

impl Default for CursorShape {
    fn default() -> CursorShape {
        CursorShape::Default
    }
}

impl CursorShape {
    pub fn is_blinking(self) -> bool {
        matches!(
            self,
            Self::BlinkingBlock | Self::BlinkingUnderline | Self::BlinkingBar
        )
    }
}

/// SequenceNo indicates a logical position within a stream of changes.
/// The sequence is only meaningful within a given `Surface` instance.
pub type SequenceNo = usize;
pub const SEQ_ZERO: SequenceNo = 0;

/// The `Surface` type represents the contents of a terminal screen.
/// It is not directly connected to a terminal device.
/// It consists of a buffer and a log of changes.  You can accumulate
/// updates to the screen by adding instances of the `Change` enum
/// that describe the updates.
///
/// When ready to render the `Surface` to a `Terminal`, you can use
/// the `get_changes` method to return an optimized stream of `Change`s
/// since the last render and then pass it to an instance of `Renderer`.
///
/// `Surface`s can also be composited together; this is useful when
/// building up a UI with layers or widgets: each widget can be its
/// own `Surface` instance and have its content maintained independently
/// from the other widgets on the screen and can then be copied into
/// the target `Surface` buffer for rendering.
///
/// To support more efficient updates in the composite use case, a
/// `draw_from_screen` method is available; the intent is to have one
/// `Surface` be hold the data that was last rendered, and a second `Surface`
/// of the same size that is repeatedly redrawn from the composite
/// of the widgets.  `draw_from_screen` is used to extract the smallest
/// difference between the updated screen and apply those changes to
/// the render target, and then use `get_changes` to render those without
/// repainting the world on each update.
#[derive(Default, Clone)]
pub struct Surface {
    width: usize,
    height: usize,
    lines: Vec<Line>,
    attributes: CellAttributes,
    xpos: usize,
    ypos: usize,
    seqno: SequenceNo,
    changes: Vec<Change>,
    cursor_shape: Option<CursorShape>,
    cursor_visibility: CursorVisibility,
    cursor_color: ColorAttribute,
    title: String,
}

#[derive(Default)]
struct DiffState {
    changes: Vec<Change>,
    /// Keep track of the cursor position that the change stream
    /// selects for updates so that we can avoid emitting redundant
    /// position changes.
    cursor: Option<(usize, usize)>,
    /// Similarly, we keep track of the cell attributes that we have
    /// activated for change stream to avoid over-emitting.
    /// Tracking the cursor and attributes in this way helps to coalesce
    /// lines of text into simpler strings.
    attr: Option<CellAttributes>,
}

include!("lib/diff_state.rs");
include!("lib/surface_core.rs");
include!("lib/surface_diff.rs");
include!("lib/diff_helpers.rs");

#[cfg(test)]
mod test {
    use super::*;
    use alloc::sync::Arc;
    use wezterm_cell::color::AnsiColor;
    use wezterm_cell::image::ImageData;
    use wezterm_cell::{AttributeChange, Intensity};

    // The  's look a little awkward, but we can't use a plain
    // space in the first chararcter of a multi-line continuation;
    // it gets eaten up and ignored.

    include!("lib/tests_01.rs");
    include!("lib/tests_02.rs");
}
