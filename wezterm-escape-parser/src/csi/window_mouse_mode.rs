#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseButton {
    Button1Press,
    Button2Press,
    Button3Press,
    Button4Press,
    Button5Press,
    Button6Press,
    Button7Press,
    Button1Release,
    Button2Release,
    Button3Release,
    Button4Release,
    Button5Release,
    Button6Release,
    Button7Release,
    Button1Drag,
    Button2Drag,
    Button3Drag,
    None,
}

impl From<MouseButton> for MouseButtons {
    fn from(button: MouseButton) -> MouseButtons {
        match button {
            MouseButton::Button1Press | MouseButton::Button1Drag => MouseButtons::LEFT,
            MouseButton::Button2Press | MouseButton::Button2Drag => MouseButtons::MIDDLE,
            MouseButton::Button3Press | MouseButton::Button3Drag => MouseButtons::RIGHT,
            MouseButton::Button4Press => MouseButtons::VERT_WHEEL | MouseButtons::WHEEL_POSITIVE,
            MouseButton::Button5Press => MouseButtons::VERT_WHEEL,
            MouseButton::Button6Press => MouseButtons::HORZ_WHEEL | MouseButtons::WHEEL_POSITIVE,
            MouseButton::Button7Press => MouseButtons::HORZ_WHEEL,
            _ => MouseButtons::NONE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Window {
    DeIconify,
    Iconify,
    MoveWindow {
        x: i64,
        y: i64,
    },
    ResizeWindowPixels {
        width: Option<i64>,
        height: Option<i64>,
    },
    RaiseWindow,
    LowerWindow,
    RefreshWindow,
    ResizeWindowCells {
        width: Option<i64>,
        height: Option<i64>,
    },
    RestoreMaximizedWindow,
    MaximizeWindow,
    MaximizeWindowVertically,
    MaximizeWindowHorizontally,
    UndoFullScreenMode,
    ChangeToFullScreenMode,
    ToggleFullScreen,
    ReportWindowState,
    ReportWindowPosition,
    ReportTextAreaPosition,
    ReportTextAreaSizePixels,
    ReportWindowSizePixels,
    ReportScreenSizePixels,
    ReportCellSizePixels,
    ReportCellSizePixelsResponse {
        width: Option<i64>,
        height: Option<i64>,
    },
    ReportTextAreaSizeCells,
    ReportScreenSizeCells,
    ReportIconLabel,
    ReportWindowTitle,
    PushIconAndWindowTitle,
    PushIconTitle,
    PushWindowTitle,
    PopIconAndWindowTitle,
    PopIconTitle,
    PopWindowTitle,
    /// DECRQCRA; used by esctest
    ChecksumRectangularArea {
        request_id: i64,
        page_number: i64,
        top: OneBased,
        left: OneBased,
        bottom: OneBased,
        right: OneBased,
    },
}

fn numstr_or_empty(x: &Option<i64>) -> String {
    match x {
        Some(x) => format!("{}", x),
        None => "".to_owned(),
    }
}

impl Display for Window {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Window::DeIconify => write!(f, "1t"),
            Window::Iconify => write!(f, "2t"),
            Window::MoveWindow { x, y } => write!(f, "3;{};{}t", x, y),
            Window::ResizeWindowPixels { width, height } => write!(
                f,
                "4;{};{}t",
                numstr_or_empty(height),
                numstr_or_empty(width),
            ),
            Window::RaiseWindow => write!(f, "5t"),
            Window::LowerWindow => write!(f, "6t"),
            Window::RefreshWindow => write!(f, "7t"),
            Window::ResizeWindowCells { width, height } => write!(
                f,
                "8;{};{}t",
                numstr_or_empty(height),
                numstr_or_empty(width),
            ),
            Window::RestoreMaximizedWindow => write!(f, "9;0t"),
            Window::MaximizeWindow => write!(f, "9;1t"),
            Window::MaximizeWindowVertically => write!(f, "9;2t"),
            Window::MaximizeWindowHorizontally => write!(f, "9;3t"),
            Window::UndoFullScreenMode => write!(f, "10;0t"),
            Window::ChangeToFullScreenMode => write!(f, "10;1t"),
            Window::ToggleFullScreen => write!(f, "10;2t"),
            Window::ReportWindowState => write!(f, "11t"),
            Window::ReportWindowPosition => write!(f, "13t"),
            Window::ReportTextAreaPosition => write!(f, "13;2t"),
            Window::ReportTextAreaSizePixels => write!(f, "14t"),
            Window::ReportWindowSizePixels => write!(f, "14;2t"),
            Window::ReportScreenSizePixels => write!(f, "15t"),
            Window::ReportCellSizePixels => write!(f, "16t"),
            Window::ReportCellSizePixelsResponse { width, height } => write!(
                f,
                "6;{};{}t",
                numstr_or_empty(height),
                numstr_or_empty(width),
            ),
            Window::ReportTextAreaSizeCells => write!(f, "18t"),
            Window::ReportScreenSizeCells => write!(f, "19t"),
            Window::ReportIconLabel => write!(f, "20t"),
            Window::ReportWindowTitle => write!(f, "21t"),
            Window::PushIconAndWindowTitle => write!(f, "22;0t"),
            Window::PushIconTitle => write!(f, "22;1t"),
            Window::PushWindowTitle => write!(f, "22;2t"),
            Window::PopIconAndWindowTitle => write!(f, "23;0t"),
            Window::PopIconTitle => write!(f, "23;1t"),
            Window::PopWindowTitle => write!(f, "23;2t"),
            Window::ChecksumRectangularArea {
                request_id,
                page_number,
                top,
                left,
                bottom,
                right,
            } => write!(
                f,
                "{};{};{};{};{};{}*y",
                request_id, page_number, top, left, bottom, right,
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseReport {
    SGR1006 {
        x: u16,
        y: u16,
        button: MouseButton,
        modifiers: Modifiers,
    },
    SGR1016 {
        x_pixels: u16,
        y_pixels: u16,
        button: MouseButton,
        modifiers: Modifiers,
    },
}

impl Display for MouseReport {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            MouseReport::SGR1006 {
                x,
                y,
                button,
                modifiers,
            } => {
                let mut b = 0;
                if (*modifiers & Modifiers::SHIFT) != Modifiers::NONE {
                    b |= 4;
                }
                if (*modifiers & Modifiers::ALT) != Modifiers::NONE {
                    b |= 8;
                }
                if (*modifiers & Modifiers::CTRL) != Modifiers::NONE {
                    b |= 16;
                }
                b |= match button {
                    MouseButton::Button1Press | MouseButton::Button1Release => 0,
                    MouseButton::Button2Press | MouseButton::Button2Release => 1,
                    MouseButton::Button3Press | MouseButton::Button3Release => 2,
                    MouseButton::Button4Press | MouseButton::Button4Release => 64,
                    MouseButton::Button5Press | MouseButton::Button5Release => 65,
                    MouseButton::Button6Press | MouseButton::Button6Release => 66,
                    MouseButton::Button7Press | MouseButton::Button7Release => 67,
                    MouseButton::Button1Drag => 32,
                    MouseButton::Button2Drag => 33,
                    MouseButton::Button3Drag => 34,
                    MouseButton::None => 35,
                };
                let trailer = match button {
                    MouseButton::Button1Press
                    | MouseButton::Button2Press
                    | MouseButton::Button3Press
                    | MouseButton::Button4Press
                    | MouseButton::Button5Press
                    | MouseButton::Button1Drag
                    | MouseButton::Button2Drag
                    | MouseButton::Button3Drag
                    | MouseButton::None => 'M',
                    _ => 'm',
                };
                write!(f, "<{};{};{}{}", b, x, y, trailer)
            }
            MouseReport::SGR1016 {
                x_pixels,
                y_pixels,
                button,
                modifiers,
            } => {
                let mut b = 0;
                if (*modifiers & Modifiers::SHIFT) != Modifiers::NONE {
                    b |= 4;
                }
                if (*modifiers & Modifiers::ALT) != Modifiers::NONE {
                    b |= 8;
                }
                if (*modifiers & Modifiers::CTRL) != Modifiers::NONE {
                    b |= 16;
                }
                b |= match button {
                    MouseButton::Button1Press | MouseButton::Button1Release => 0,
                    MouseButton::Button2Press | MouseButton::Button2Release => 1,
                    MouseButton::Button3Press | MouseButton::Button3Release => 2,
                    MouseButton::Button4Press | MouseButton::Button4Release => 64,
                    MouseButton::Button5Press | MouseButton::Button5Release => 65,
                    MouseButton::Button6Press | MouseButton::Button6Release => 66,
                    MouseButton::Button7Press | MouseButton::Button7Release => 67,
                    MouseButton::Button1Drag => 32,
                    MouseButton::Button2Drag => 33,
                    MouseButton::Button3Drag => 34,
                    MouseButton::None => 35,
                };
                let trailer = match button {
                    MouseButton::Button1Press
                    | MouseButton::Button2Press
                    | MouseButton::Button3Press
                    | MouseButton::Button4Press
                    | MouseButton::Button5Press
                    | MouseButton::Button1Drag
                    | MouseButton::Button2Drag
                    | MouseButton::Button3Drag
                    | MouseButton::None => 'M',
                    _ => 'm',
                };
                write!(f, "<{};{};{}{}", b, x_pixels, y_pixels, trailer)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XtermKeyModifierResource {
    Keyboard,
    CursorKeys,
    FunctionKeys,
    OtherKeys,
}

impl XtermKeyModifierResource {
    pub fn parse(value: i64) -> Option<Self> {
        Some(match value {
            0 => XtermKeyModifierResource::Keyboard,
            1 => XtermKeyModifierResource::CursorKeys,
            2 => XtermKeyModifierResource::FunctionKeys,
            4 => XtermKeyModifierResource::OtherKeys,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    SetDecPrivateMode(DecPrivateMode),
    ResetDecPrivateMode(DecPrivateMode),
    SaveDecPrivateMode(DecPrivateMode),
    RestoreDecPrivateMode(DecPrivateMode),
    QueryDecPrivateMode(DecPrivateMode),
    SetMode(TerminalMode),
    ResetMode(TerminalMode),
    QueryMode(TerminalMode),
    XtermKeyMode {
        resource: XtermKeyModifierResource,
        value: Option<i64>,
    },
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        macro_rules! emit {
            ($flag:expr, $mode:expr) => {{
                let value = match $mode {
                    DecPrivateMode::Code(mode) => mode.to_u16().ok_or_else(|| FmtError)?,
                    DecPrivateMode::Unspecified(mode) => *mode,
                };
                write!(f, "?{}{}", value, $flag)
            }};
        }
        macro_rules! emit_mode {
            ($flag:expr, $mode:expr) => {{
                let value = match $mode {
                    TerminalMode::Code(mode) => mode.to_u16().ok_or_else(|| FmtError)?,
                    TerminalMode::Unspecified(mode) => *mode,
                };
                write!(f, "{}{}", value, $flag)
            }};
        }
        match self {
            Mode::SetDecPrivateMode(mode) => emit!("h", mode),
            Mode::ResetDecPrivateMode(mode) => emit!("l", mode),
            Mode::SaveDecPrivateMode(mode) => emit!("s", mode),
            Mode::RestoreDecPrivateMode(mode) => emit!("r", mode),
            Mode::QueryDecPrivateMode(DecPrivateMode::Code(mode)) => {
                write!(f, "?{}$p", mode.to_u16().ok_or_else(|| FmtError)?)
            }
            Mode::QueryDecPrivateMode(DecPrivateMode::Unspecified(mode)) => {
                write!(f, "?{}$p", mode)
            }
            Mode::SetMode(mode) => emit_mode!("h", mode),
            Mode::ResetMode(mode) => emit_mode!("l", mode),
            Mode::QueryMode(TerminalMode::Code(mode)) => {
                write!(f, "?{}$p", mode.to_u16().ok_or_else(|| FmtError)?)
            }
            Mode::QueryMode(TerminalMode::Unspecified(mode)) => write!(f, "?{}$p", mode),
            Mode::XtermKeyMode { resource, value } => {
                write!(
                    f,
                    ">{}",
                    match resource {
                        XtermKeyModifierResource::Keyboard => 0,
                        XtermKeyModifierResource::CursorKeys => 1,
                        XtermKeyModifierResource::FunctionKeys => 2,
                        XtermKeyModifierResource::OtherKeys => 4,
                    }
                )?;
                if let Some(value) = value {
                    write!(f, ";{}", value)?;
                } else {
                    write!(f, ";")?;
                }
                write!(f, "m")
            }
        }
    }
}
