//! The `LineEditor` struct provides line editing facilities similar
//! to those in the unix shell.
//!
//! ```no_run
//! use termwiz::lineedit::{line_editor_terminal, NopLineEditorHost, LineEditor};
//!
//! fn main() -> termwiz::Result<()> {
//!     let mut terminal = line_editor_terminal()?;
//!     let mut editor = LineEditor::new(&mut terminal);
//!     let mut host = NopLineEditorHost::default();
//!
//!     let line = editor.read_line(&mut host)?;
//!     println!("read line: {:?}", line);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Key Bindings
//!
//! The following key bindings are supported:
//!
//! Keystroke     | Action
//! ---------     | ------
//! Ctrl-A, Home  | Move cursor to the beginning of the line
//! Ctrl-E, End   | Move cursor to the end of the line
//! Ctrl-B, Left  | Move cursor one grapheme to the left
//! Ctrl-C        | Cancel the line editor
//! Ctrl-D        | Cancel the line editor with an End-of-File result
//! Ctrl-F, Right | Move cursor one grapheme to the right
//! Ctrl-H, Backspace | Delete the grapheme to the left of the cursor
//! Delete        | Delete the grapheme to the right of the cursor
//! Ctrl-J, Ctrl-M, Enter | Finish line editing and accept the current line
//! Ctrl-K        | Delete from cursor to end of line
//! Ctrl-L        | Move the cursor to the top left, clear screen and repaint
//! Ctrl-R        | Incremental history search mode
//! Ctrl-W        | Delete word leading up to cursor
//! Alt-b, Alt-Left | Move the cursor backwards one word
//! Alt-f, Alt-Right | Move the cursor forwards one word
use crate::caps::{Capabilities, ProbeHints};
use crate::input::{InputEvent, KeyCode, KeyEvent, Modifiers};
use crate::surface::change::ChangeSequence;
use crate::surface::{Change, Position};
use crate::terminal::{new_terminal, Terminal};
use crate::{bail, ensure, Result};

mod actions;
mod buffer;
mod history;
mod host;
pub use actions::{Action, Movement, RepeatCount};
pub use buffer::LineEditBuffer;
pub use history::*;
pub use host::*;

include!("root/types.rs");
include!("root/editor_render.rs");
include!("root/editor_read.rs");
include!("root/editor_actions.rs");
include!("root/editor_loop.rs");
include!("root/terminal.rs");
