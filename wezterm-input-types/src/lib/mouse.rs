bitflags! {
    #[derive(Default)]
    pub struct MouseButtons: u8 {
        const NONE = 0;
        #[allow(clippy::identity_op)]
        const LEFT = 1<<0;
        const RIGHT = 1<<1;
        const MIDDLE = 1<<2;
        const X1 = 1<<3;
        const X2 = 1<<4;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MousePress {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseEventKind {
    Move,
    Press(MousePress),
    Release(MousePress),
    VertWheel(i16),
    HorzWheel(i16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    /// Coordinates of the mouse relative to the top left of the window
    pub coords: Point,
    /// The mouse position in screen coordinates
    pub screen_coords: crate::ScreenPoint,
    pub mouse_buttons: MouseButtons,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone)]
pub struct Handled(Arc<AtomicBool>);

impl Handled {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    pub fn set_handled(&self) {
        self.0.store(true, core::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_handled(&self) -> bool {
        self.0.load(core::sync::atomic::Ordering::Relaxed)
    }
}

impl PartialEq for Handled {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for Handled {}

/// A key event prior to any dead key or IME composition
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RawKeyEvent {
    pub key: KeyCode,
    pub modifiers: Modifiers,
    pub leds: KeyboardLedStatus,

    /// The physical location of the key on an ANSI-Standard US layout
    pub phys_code: Option<PhysKeyCode>,
    /// The OS and hardware dependent key code for the key
    pub raw_code: u32,

    /// The *other* OS and hardware dependent key code for the key
    #[cfg(windows)]
    pub scan_code: u32,

    /// How many times this key repeats
    pub repeat_count: u16,

    /// If true, this is a key down rather than a key up event
    pub key_is_down: bool,
    pub handled: Handled,
}

impl RawKeyEvent {
    /// Mark the event as handled, in order to prevent additional
    /// processing.
    pub fn set_handled(&self) {
        self.handled.set_handled();
    }

    /// <https://sw.kovidgoyal.net/kitty/keyboard-protocol/#functional-key-definitions>
    #[deny(warnings)]
    fn kitty_function_code(&self) -> Option<u32> {
        use KeyCode::*;
        Some(match self.key {
            // Tab => 9,
            // Backspace => 127,
            // CapsLock => 57358,
            // ScrollLock => 57359,
            // NumLock => 57360,
            // PrintScreen => 57361,
            // Pause => 57362,
            // Menu => 57363,
            Function(n) if n >= 13 && n <= 35 => 57376 + n as u32 - 13,
            Numpad(n) => n as u32 + 57399,
            Decimal => 57409,
            Divide => 57410,
            Multiply => 57411,
            Subtract => 57412,
            Add => 57413,
            // KeypadEnter => 57414,
            // KeypadEquals => 57415,
            Separator => 57416,
            ApplicationLeftArrow => 57417,
            ApplicationRightArrow => 57418,
            ApplicationUpArrow => 57419,
            ApplicationDownArrow => 57420,
            KeyPadHome => 57423,
            KeyPadEnd => 57424,
            KeyPadBegin => 57427,
            KeyPadPageUp => 57421,
            KeyPadPageDown => 57422,
            Insert => 57425,
            // KeypadDelete => 57426,
            MediaPlayPause => 57430,
            MediaStop => 57432,
            MediaNextTrack => 57435,
            MediaPrevTrack => 57436,
            VolumeDown => 57436,
            VolumeUp => 57439,
            VolumeMute => 57440,
            LeftShift => 57441,
            LeftControl => 57442,
            LeftAlt => 57443,
            LeftWindows => 57444,
            RightShift => 57447,
            RightControl => 57448,
            RightAlt => 57449,
            RightWindows => 57450,
            _ => match &self.phys_code {
                Some(phys) => {
                    use PhysKeyCode::*;

                    match *phys {
                        Escape => 27,
                        Return => 13,
                        Tab => 9,
                        Backspace => 127,
                        CapsLock => 57358,
                        // ScrollLock => 57359,
                        NumLock => 57360,
                        // PrintScreen => 57361,
                        // Pause => 57362,
                        // Menu => 57363,
                        F13 => 57376,
                        F14 => 57377,
                        F15 => 57378,
                        F16 => 57379,
                        F17 => 57380,
                        F18 => 57381,
                        F19 => 57382,
                        F20 => 57383,
                        F21 => 57384,
                        F22 => 57385,
                        F23 => 57386,
                        F24 => 57387,
                        /*
                        F25 => 57388,
                        F26 => 57389,
                        F27 => 57390,
                        F28 => 57391,
                        F29 => 57392,
                        F30 => 57393,
                        F31 => 57394,
                        F32 => 57395,
                        F33 => 57396,
                        F34 => 57397,
                        */
                        Keypad0 => 57399,
                        Keypad1 => 57400,
                        Keypad2 => 57401,
                        Keypad3 => 57402,
                        Keypad4 => 57403,
                        Keypad5 => 57404,
                        Keypad6 => 57405,
                        Keypad7 => 57406,
                        Keypad8 => 57407,
                        Keypad9 => 57408,
                        KeypadDecimal => 57409,
                        KeypadDivide => 57410,
                        KeypadMultiply => 57411,
                        KeypadSubtract => 57412,
                        KeypadAdd => 57413,
                        KeypadEnter => 57414,
                        KeypadEquals => 57415,
                        // KeypadSeparator => 57416,
                        // ApplicationLeftArrow => 57417,
                        // ApplicationRightArrow => 57418,
                        // ApplicationUpArrow => 57419,
                        // ApplicationDownArrow => 57420,
                        // KeyPadHome => 57423,
                        // KeyPadEnd => 57424,
                        // KeyPadBegin => 57427,
                        // KeyPadPageUp => 57421,
                        // KeyPadPageDown => 57422,
                        Insert => 57425,
                        // KeypadDelete => 57426,
                        // MediaPlayPause => 57430,
                        // MediaStop => 57432,
                        // MediaNextTrack => 57435,
                        // MediaPrevTrack => 57436,
                        VolumeDown => 57436,
                        VolumeUp => 57439,
                        VolumeMute => 57440,
                        LeftShift => 57441,
                        LeftControl => 57442,
                        LeftAlt => 57443,
                        LeftWindows => 57444,
                        RightShift => 57447,
                        RightControl => 57448,
                        RightAlt => 57449,
                        RightWindows => 57450,
                        _ => return None,
                    }
                }
                _ => return None,
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    /// Which key was pressed.
    /// This is the potentially processed/composed version
    /// of the input.
    pub key: KeyCode,
    /// Which modifiers are down
    pub modifiers: Modifiers,

    pub leds: KeyboardLedStatus,

    /// How many times this key repeats
    pub repeat_count: u16,

    /// If true, this is a key down rather than a key up event
    pub key_is_down: bool,

    /// If triggered from a raw key event, here it is.
    pub raw: Option<RawKeyEvent>,

    #[cfg(windows)]
    pub win32_uni_char: Option<char>,
}
