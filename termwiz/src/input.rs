//! This module provides an InputParser struct to help with parsing
//! input received from a terminal.
use crate::bail;
use crate::error::Result;
use crate::escape::csi::{KittyKeyboardFlags, MouseReport};
use crate::escape::parser::Parser;
use crate::escape::{Action, CSI};
use crate::keymap::{Found, KeyMap};
use crate::readbuf::ReadBuffer;
#[cfg(feature = "use_serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use wezterm_input_types::ctrl_mapping;

pub use wezterm_input_types::Modifiers;

pub const CSI: &str = "\x1b[";
pub const SS3: &str = "\x1bO";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardEncoding {
    Xterm,
    /// <http://www.leonerd.org.uk/hacks/fixterms/>
    CsiU,
    /// <https://github.com/microsoft/terminal/blob/main/doc/specs/%234999%20-%20Improved%20keyboard%20handling%20in%20Conpty.md>
    Win32,
    /// <https://sw.kovidgoyal.net/kitty/keyboard-protocol/>
    Kitty(KittyKeyboardFlags),
}

/// Specifies terminal modes/configuration that can influence how a KeyCode
/// is encoded when being sent to and application via the pty.
#[derive(Debug, Clone, Copy)]
pub struct KeyCodeEncodeModes {
    pub encoding: KeyboardEncoding,
    pub application_cursor_keys: bool,
    pub newline_mode: bool,
    pub modify_other_keys: Option<i64>,
}

#[cfg(windows)]
use winapi::um::wincon::{
    INPUT_RECORD, KEY_EVENT, KEY_EVENT_RECORD, MOUSE_EVENT, MOUSE_EVENT_RECORD,
    WINDOW_BUFFER_SIZE_EVENT, WINDOW_BUFFER_SIZE_RECORD,
};

pub use wezterm_escape_parser::csi::MouseButtons;

#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    PixelMouse(PixelMouseEvent),
    /// Detected that the user has resized the terminal
    Resized {
        cols: usize,
        rows: usize,
    },
    /// For terminals that support Bracketed Paste mode,
    /// pastes are collected and reported as this variant.
    Paste(String),
    /// The program has woken the input thread.
    Wake,
}

#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseEvent {
    pub x: u16,
    pub y: u16,
    pub mouse_buttons: MouseButtons,
    pub modifiers: Modifiers,
}

#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelMouseEvent {
    pub x_pixels: u16,
    pub y_pixels: u16,
    pub mouse_buttons: MouseButtons,
    pub modifiers: Modifiers,
}

#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    /// Which key was pressed
    pub key: KeyCode,

    /// Which modifiers are down
    pub modifiers: Modifiers,
}

/// Which key is pressed.  Not all of these are probable to appear
/// on most systems.  A lot of this list is @wez trawling docs and
/// making an entry for things that might be possible in this first pass.
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// The decoded unicode character
    Char(char),

    Hyper,
    Super,
    Meta,

    /// Ctrl-break on windows
    Cancel,
    Backspace,
    Tab,
    Clear,
    Enter,
    Shift,
    Escape,
    LeftShift,
    RightShift,
    Control,
    LeftControl,
    RightControl,
    Alt,
    LeftAlt,
    RightAlt,
    Menu,
    LeftMenu,
    RightMenu,
    Pause,
    CapsLock,
    PageUp,
    PageDown,
    End,
    Home,
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,
    Select,
    Print,
    Execute,
    PrintScreen,
    Insert,
    Delete,
    Help,
    LeftWindows,
    RightWindows,
    Applications,
    Sleep,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    Multiply,
    Add,
    Separator,
    Subtract,
    Decimal,
    Divide,
    /// F1-F24 are possible
    Function(u8),
    NumLock,
    ScrollLock,
    Copy,
    Cut,
    Paste,
    BrowserBack,
    BrowserForward,
    BrowserRefresh,
    BrowserStop,
    BrowserSearch,
    BrowserFavorites,
    BrowserHome,
    VolumeMute,
    VolumeDown,
    VolumeUp,
    MediaNextTrack,
    MediaPrevTrack,
    MediaStop,
    MediaPlayPause,
    ApplicationLeftArrow,
    ApplicationRightArrow,
    ApplicationUpArrow,
    ApplicationDownArrow,
    KeyPadHome,
    KeyPadEnd,
    KeyPadPageUp,
    KeyPadPageDown,
    KeyPadBegin,

    #[doc(hidden)]
    InternalPasteStart,
    #[doc(hidden)]
    InternalPasteEnd,
}

include!("input/key_code.rs");
include!("input/encode_helpers.rs");
include!("input/parser_types.rs");
include!("input/parser_map.rs");
include!("input/parser_decode.rs");

#[cfg(test)]
mod test {
    use super::*;

    const NO_MORE: bool = false;
    const MAYBE_MORE: bool = true;

    include!("input/tests_01.rs");
    include!("input/tests_02.rs");
}
