use crate::terminal::{Alert, Progress};
use crate::terminalstate::{
    default_color_map, CharSet, MouseEncoding, TabStop, UnicodeVersionStackEntry,
};
use crate::{ClipboardSelection, Position, TerminalState, VisibleRowIndex, DCS, ST};
use finl_unicode::grapheme_clusters::Graphemes;
use log::{debug, error};
use num_traits::FromPrimitive;
use ordered_float::NotNan;
use std::fmt::Write;
use std::io::Write as _;
use std::ops::{Deref, DerefMut};
use termwiz::input::KeyboardEncoding;
use unicode_normalization::{is_nfc_quick, IsNormalized, UnicodeNormalization};
use url::Url;
use wezterm_bidi::ParagraphDirectionHint;
use wezterm_cell::{
    grapheme_column_width, is_white_space_grapheme, Cell, CellAttributes, SemanticType,
};
use wezterm_escape_parser::csi::{
    CharacterPath, EraseInDisplay, Keyboard, KittyKeyboardFlags, KittyKeyboardMode,
};
use wezterm_escape_parser::osc::{
    ChangeColorPair, ColorOrQuery, FinalTermSemanticPrompt, ITermProprietary,
    ITermUnicodeVersionOp, Selection,
};
use wezterm_escape_parser::{
    Action, ControlCode, DeviceControlMode, Esc, EscCode, OperatingSystemCommand, CSI,
};

/// A helper struct for implementing `vtparse::VTActor` while compartmentalizing
/// the terminal state and the embedding/host terminal interface
pub(crate) struct Performer<'a> {
    pub state: &'a mut TerminalState,
    print: String,
}

impl<'a> Deref for Performer<'a> {
    type Target = TerminalState;

    fn deref(&self) -> &TerminalState {
        self.state
    }
}

impl<'a> DerefMut for Performer<'a> {
    fn deref_mut(&mut self) -> &mut TerminalState {
        &mut self.state
    }
}

impl<'a> Drop for Performer<'a> {
    fn drop(&mut self) {
        self.flush_print();
    }
}
