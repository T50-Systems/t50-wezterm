use crate::escape::csi::{DecPrivateMode, DecPrivateModeCode, Mode, CSI};
use crate::istty::IsTty;
use crate::terminal::ProbeCapabilities;
use crate::{bail, ensure, format_err, Result};
use filedescriptor::{FileDescriptor, OwnedHandle};
use std::cmp::{max, min};
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::{stdin, stdout, Error as IoError, Read, Result as IoResult, Write};
use std::os::windows::io::{AsRawHandle, FromRawHandle};
use std::sync::Arc;
use std::time::Duration;
use std::{mem, ptr};
use winapi::shared::winerror::WAIT_TIMEOUT;
use winapi::um::consoleapi;
use winapi::um::synchapi::{CreateEventW, SetEvent, WaitForMultipleObjects};
use winapi::um::winbase::{INFINITE, WAIT_FAILED, WAIT_OBJECT_0};
use winapi::um::wincon::{
    FillConsoleOutputAttribute, FillConsoleOutputCharacterW, GetConsoleScreenBufferInfo,
    ReadConsoleOutputW, ScrollConsoleScreenBufferW, SetConsoleCP, SetConsoleCursorPosition,
    SetConsoleOutputCP, SetConsoleScreenBufferSize, SetConsoleTextAttribute, SetConsoleWindowInfo,
    WriteConsoleOutputW, CHAR_INFO, CONSOLE_SCREEN_BUFFER_INFO, COORD, DISABLE_NEWLINE_AUTO_RETURN,
    ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, ENABLE_MOUSE_INPUT, ENABLE_PROCESSED_INPUT,
    ENABLE_VIRTUAL_TERMINAL_INPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING, ENABLE_WINDOW_INPUT,
    INPUT_RECORD, SMALL_RECT,
};
use winapi::um::winnls::CP_UTF8;

use crate::caps::Capabilities;
use crate::input::{InputEvent, InputParser};
use crate::render::terminfo::TerminfoRenderer;
use crate::render::windows::WindowsConsoleRenderer;
use crate::render::RenderTty;
use crate::surface::Change;
use crate::terminal::{cast, ScreenSize, Terminal};

const BUF_SIZE: usize = 128;

include!("windows/traits.rs");
include!("windows/input.rs");
include!("windows/output.rs");
include!("windows/terminal.rs");
include!("windows/terminal_impl.rs");
