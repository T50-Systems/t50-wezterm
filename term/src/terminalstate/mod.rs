// The range_plus_one lint can't see when the LHS is not compatible with
// and inclusive range
#![allow(clippy::range_plus_one)]
use super::*;
use crate::color::{ColorPalette, RgbColor};
use crate::config::{BidiMode, NewlineCanon};
use log::debug;
use num_traits::ToPrimitive;
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::num::NonZeroUsize;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use terminfo::{Database, Value};
use termwiz::input::KeyboardEncoding;
use url::Url;
use wezterm_bidi::ParagraphDirectionHint;
use wezterm_cell::image::ImageData;
use wezterm_cell::UnicodeVersion;
use wezterm_escape_parser::csi::{
    Cursor, CursorStyle, DecPrivateMode, DecPrivateModeCode, Device, Edit, EraseInDisplay,
    EraseInLine, Mode, Sgr, TabulationClear, TerminalMode, TerminalModeCode, Window, XtSmGraphics,
    XtSmGraphicsAction, XtSmGraphicsItem, XtSmGraphicsStatus, XtermKeyModifierResource,
};
use wezterm_escape_parser::{OneBased, OperatingSystemCommand, CSI};
use wezterm_surface::{CursorShape, CursorVisibility, SequenceNo};

mod colors;
mod core;
mod csi_cursor_sgr;
mod csi_mode;
mod csi_mode_part1;
mod csi_mode_part2;
mod csi_mode_part3;
mod csi_window_edit;
mod device;
mod image;
mod iterm;
mod keyboard;
mod kitty;
mod layout;
mod mouse;
pub(crate) mod performer;
mod sixel;
mod threaded_writer;
use crate::terminalstate::image::*;
use crate::terminalstate::kitty::*;
pub(crate) use colors::default_color_map;
use threaded_writer::ThreadedWriter;
lazy_static::lazy_static! {
    static ref DB: Database = {
        let data = include_bytes!("../../../termwiz/data/wezterm");
        Database::from_buffer(&data[..]).unwrap()
    };
}

pub(crate) struct TabStop {
    tabs: Vec<bool>,
    tab_width: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CharSet {
    Ascii,
    Uk,
    DecLineDrawing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MouseEncoding {
    X10,
    Utf8,
    SGR,
    SgrPixels,
}

impl TabStop {
    fn new(screen_width: usize, tab_width: usize) -> Self {
        let mut tabs = Vec::with_capacity(screen_width);

        for i in 0..screen_width {
            tabs.push((i % tab_width) == 0);
        }
        Self { tabs, tab_width }
    }

    fn set_tab_stop(&mut self, col: usize) {
        self.tabs[col] = true;
    }

    fn find_prev_tab_stop(&self, col: usize) -> Option<usize> {
        for i in (0..col.min(self.tabs.len())).rev() {
            if self.tabs[i] {
                return Some(i);
            }
        }
        None
    }

    fn find_next_tab_stop(&self, col: usize) -> Option<usize> {
        for i in col + 1..self.tabs.len() {
            if self.tabs[i] {
                return Some(i);
            }
        }
        None
    }

    /// Respond to the terminal resizing.
    /// If the screen got bigger, we need to expand the tab stops
    /// into the new columns with the appropriate width.
    fn resize(&mut self, screen_width: usize) {
        let current = self.tabs.len();
        if screen_width > current {
            for i in current..screen_width {
                self.tabs.push((i % self.tab_width) == 0);
            }
        }
    }

    fn clear(&mut self, to_clear: TabulationClear, col: usize, log_unknown_escape_sequences: bool) {
        match to_clear {
            TabulationClear::ClearCharacterTabStopAtActivePosition => {
                if let Some(t) = self.tabs.get_mut(col) {
                    *t = false;
                }
            }
            // If we want to exactly match VT100/xterm behavior, then
            // we cannot honor ClearCharacterTabStopsAtActiveLine.
            TabulationClear::ClearAllCharacterTabStops => {
                // | TabulationClear::ClearCharacterTabStopsAtActiveLine
                for t in &mut self.tabs {
                    *t = false;
                }
            }
            _ => {
                if log_unknown_escape_sequences {
                    log::warn!("unhandled TabulationClear {:?}", to_clear);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SavedCursor {
    position: CursorPosition,
    wrap_next: bool,
    pen: CellAttributes,
    dec_origin_mode: bool,
    g0_charset: CharSet,
    g1_charset: CharSet,
    // TODO: selective_erase when supported
}

struct ScreenOrAlt {
    /// The primary screen + scrollback
    screen: Screen,
    /// The alternate screen; no scrollback
    alt_screen: Screen,
    /// Tells us which screen is active
    alt_screen_is_active: bool,
}

impl Deref for ScreenOrAlt {
    type Target = Screen;

    fn deref(&self) -> &Screen {
        if self.alt_screen_is_active {
            &self.alt_screen
        } else {
            &self.screen
        }
    }
}

impl DerefMut for ScreenOrAlt {
    fn deref_mut(&mut self) -> &mut Screen {
        if self.alt_screen_is_active {
            &mut self.alt_screen
        } else {
            &mut self.screen
        }
    }
}

impl ScreenOrAlt {
    pub fn new(
        size: TerminalSize,
        config: &Arc<dyn TerminalConfiguration>,
        seqno: SequenceNo,
        bidi_mode: BidiMode,
    ) -> Self {
        let screen = Screen::new(size, config, true, seqno, bidi_mode);
        let alt_screen = Screen::new(size, config, false, seqno, bidi_mode);

        Self {
            screen,
            alt_screen,
            alt_screen_is_active: false,
        }
    }

    pub fn resize(
        &mut self,
        size: TerminalSize,
        cursor_main: CursorPosition,
        cursor_alt: CursorPosition,
        seqno: SequenceNo,
        is_conpty: bool,
    ) -> (CursorPosition, CursorPosition) {
        let cursor_main = self.screen.resize(size, cursor_main, seqno, is_conpty);
        let cursor_alt = self.alt_screen.resize(size, cursor_alt, seqno, is_conpty);
        (cursor_main, cursor_alt)
    }

    pub fn activate_alt_screen(&mut self, seqno: SequenceNo) {
        self.alt_screen_is_active = true;
        self.dirty_top_phys_rows(seqno);
    }

    pub fn activate_primary_screen(&mut self, seqno: SequenceNo) {
        self.alt_screen_is_active = false;
        self.dirty_top_phys_rows(seqno);
    }

    // When switching between alt and primary screen, we implicitly change
    // the content associated with StableRowIndex 0..num_rows.  The muxer
    // use case needs to know to invalidate its cache, so we mark those rows
    // as dirty.
    fn dirty_top_phys_rows(&mut self, seqno: SequenceNo) {
        let num_rows = self.screen.physical_rows;
        for line_idx in 0..num_rows {
            self.screen
                .line_mut(line_idx)
                .update_last_change_seqno(seqno);
        }
    }

    pub fn is_alt_screen_active(&self) -> bool {
        self.alt_screen_is_active
    }

    pub fn saved_cursor(&mut self) -> &mut Option<SavedCursor> {
        if self.alt_screen_is_active {
            &mut self.alt_screen.saved_cursor
        } else {
            &mut self.screen.saved_cursor
        }
    }

    pub fn full_reset(&mut self) {
        self.screen.full_reset();
        self.alt_screen.full_reset();
    }
}

/// Manages the state for the terminal
pub struct TerminalState {
    config: Arc<dyn TerminalConfiguration>,

    screen: ScreenOrAlt,
    /// The current set of attributes in effect for the next
    /// attempt to print to the display
    pen: CellAttributes,
    /// The current cursor position, relative to the top left
    /// of the screen.  0-based index.
    cursor: CursorPosition,

    /// if true, implicitly move to the next line on the next
    /// printed character
    wrap_next: bool,

    clear_semantic_attribute_on_newline: bool,

    /// If true, writing a character inserts a new cell
    insert: bool,

    /// https://vt100.net/docs/vt510-rm/DECAWM.html
    dec_auto_wrap: bool,

    /// Reverse Wraparound Mode
    reverse_wraparound_mode: bool,

    /// Reverse video mode
    reverse_video_mode: bool,

    /// https://vt100.net/docs/vt510-rm/DECOM.html
    /// When OriginMode is enabled, cursor is constrained to the
    /// scroll region and its position is relative to the scroll
    /// region.
    dec_origin_mode: bool,

    /// The scroll region
    top_and_bottom_margins: Range<VisibleRowIndex>,
    left_and_right_margins: Range<usize>,
    left_and_right_margin_mode: bool,

    /// When set, modifies the sequence of bytes sent for keys
    /// designated as cursor keys.  This includes various navigation
    /// keys.  The code in key_down() is responsible for interpreting this.
    application_cursor_keys: bool,
    modify_other_keys: Option<i64>,

    dec_ansi_mode: bool,

    /// https://vt100.net/dec/ek-vt38t-ug-001.pdf#page=132 has a
    /// discussion on what sixel dispay mode (DECSDM) does.
    sixel_display_mode: bool,
    use_private_color_registers_for_each_graphic: bool,

    /// Graphics mode color register map.
    color_map: HashMap<u16, RgbColor>,

    /// When set, modifies the sequence of bytes sent for keys
    /// in the numeric keypad portion of the keyboard.
    application_keypad: bool,

    /// When set, pasting the clipboard should bracket the data with
    /// designated marker characters.
    bracketed_paste: bool,

    /// Movement events enabled
    any_event_mouse: bool,
    focus_tracking: bool,
    /// X10 (legacy), SGR, and SGR-Pixels style mouse tracking and
    /// reporting is enabled
    mouse_encoding: MouseEncoding,
    mouse_tracking: bool,
    /// Button events enabled
    button_event_mouse: bool,
    current_mouse_buttons: Vec<MouseButton>,
    last_mouse_move: Option<MouseEvent>,
    cursor_visible: bool,

    keyboard_encoding: KeyboardEncoding,
    /// Support for US, UK, and DEC Special Graphics
    g0_charset: CharSet,
    g1_charset: CharSet,
    shift_out: bool,

    newline_mode: bool,

    tabs: TabStop,

    /// The terminal title string (OSC 2)
    title: String,
    /// The icon title string (OSC 1)
    icon_title: Option<String>,
    progress: Progress,

    palette: Option<ColorPalette>,

    pixel_width: usize,
    pixel_height: usize,
    dpi: u32,

    clipboard: Option<Arc<dyn Clipboard>>,
    device_control_handler: Option<Box<dyn DeviceControlHandler>>,
    alert_handler: Option<Box<dyn AlertHandler>>,
    download_handler: Option<Arc<dyn DownloadHandler>>,

    current_dir: Option<Url>,

    term_program: String,
    term_version: String,

    writer: BufWriter<ThreadedWriter>,

    image_cache: lru::LruCache<[u8; 32], Arc<ImageData>>,
    sixel_scrolls_right: bool,

    user_vars: HashMap<String, String>,

    kitty_img: KittyImageState,
    seqno: SequenceNo,

    /// The unicode version that is in effect
    unicode_version: UnicodeVersion,
    unicode_version_stack: Vec<UnicodeVersionStackEntry>,

    enable_conpty_quirks: bool,
    /// On Windows, the ConPTY layer emits an OSC sequence to
    /// set the title shortly after it starts up.
    /// We don't want that, so we use this flag to remember
    /// whether we want to skip it or not.
    suppress_initial_title_change: bool,

    accumulating_title: Option<String>,

    /// seqno when we last lost focus
    lost_focus_seqno: SequenceNo,
    /// seqno when we last emitted Alert::OutputSinceFocusLost
    lost_focus_alerted_seqno: SequenceNo,
    focused: bool,

    /// True if lines should be marked as bidi-enabled, and thus
    /// have the renderer apply the bidi algorithm.
    /// true is equivalent to "implicit" bidi mode as described in
    /// <https://terminal-wg.pages.freedesktop.org/bidi/recommendation/basic-modes.html>
    /// If none, then the default value specified by the config is used.
    bidi_enabled: Option<bool>,
    /// When set, specifies the bidi direction information that should be
    /// applied to lines.
    /// If none, then the default value specified by the config is used.
    bidi_hint: Option<ParagraphDirectionHint>,
}

#[derive(Debug)]
struct UnicodeVersionStackEntry {
    vers: UnicodeVersion,
    label: Option<String>,
}
