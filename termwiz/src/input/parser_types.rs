#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputState {
    Normal,
    EscapeMaybeAlt,
    Pasting(usize),
}

#[derive(Debug)]
pub struct InputParser {
    key_map: KeyMap<InputEvent>,
    buf: ReadBuffer,
    state: InputState,
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std;
    use winapi::um::winuser;

    fn modifiers_from_ctrl_key_state(state: u32) -> Modifiers {
        use winapi::um::wincon::*;

        let mut mods = Modifiers::NONE;

        if (state & (LEFT_ALT_PRESSED | RIGHT_ALT_PRESSED)) != 0 {
            mods |= Modifiers::ALT;
        }

        if (state & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED)) != 0 {
            mods |= Modifiers::CTRL;
        }

        if (state & SHIFT_PRESSED) != 0 {
            mods |= Modifiers::SHIFT;
        }

        // TODO: we could report caps lock, numlock and scrolllock

        mods
    }
    impl InputParser {
        fn decode_key_record<F: FnMut(InputEvent)>(
            &mut self,
            event: &KEY_EVENT_RECORD,
            callback: &mut F,
        ) {
            // TODO: do we want downs instead of ups?
            if event.bKeyDown == 0 {
                return;
            }

            let key_code = match std::char::from_u32(*unsafe { event.uChar.UnicodeChar() } as u32) {
                Some(unicode) if unicode > '\x00' => {
                    let mut buf = [0u8; 4];
                    self.buf
                        .extend_with(unicode.encode_utf8(&mut buf).as_bytes());
                    self.process_bytes(callback, true);
                    return;
                }
                _ => match event.wVirtualKeyCode as i32 {
                    winuser::VK_CANCEL => KeyCode::Cancel,
                    winuser::VK_BACK => KeyCode::Backspace,
                    winuser::VK_TAB => KeyCode::Tab,
                    winuser::VK_CLEAR => KeyCode::Clear,
                    winuser::VK_RETURN => KeyCode::Enter,
                    winuser::VK_SHIFT => KeyCode::Shift,
                    winuser::VK_CONTROL => KeyCode::Control,
                    winuser::VK_MENU => KeyCode::Menu,
                    winuser::VK_PAUSE => KeyCode::Pause,
                    winuser::VK_CAPITAL => KeyCode::CapsLock,
                    winuser::VK_ESCAPE => KeyCode::Escape,
                    winuser::VK_PRIOR => KeyCode::PageUp,
                    winuser::VK_NEXT => KeyCode::PageDown,
                    winuser::VK_END => KeyCode::End,
                    winuser::VK_HOME => KeyCode::Home,
                    winuser::VK_LEFT => KeyCode::LeftArrow,
                    winuser::VK_RIGHT => KeyCode::RightArrow,
                    winuser::VK_UP => KeyCode::UpArrow,
                    winuser::VK_DOWN => KeyCode::DownArrow,
                    winuser::VK_SELECT => KeyCode::Select,
                    winuser::VK_PRINT => KeyCode::Print,
                    winuser::VK_EXECUTE => KeyCode::Execute,
                    winuser::VK_SNAPSHOT => KeyCode::PrintScreen,
                    winuser::VK_INSERT => KeyCode::Insert,
                    winuser::VK_DELETE => KeyCode::Delete,
                    winuser::VK_HELP => KeyCode::Help,
                    winuser::VK_LWIN => KeyCode::LeftWindows,
                    winuser::VK_RWIN => KeyCode::RightWindows,
                    winuser::VK_APPS => KeyCode::Applications,
                    winuser::VK_SLEEP => KeyCode::Sleep,
                    winuser::VK_NUMPAD0 => KeyCode::Numpad0,
                    winuser::VK_NUMPAD1 => KeyCode::Numpad1,
                    winuser::VK_NUMPAD2 => KeyCode::Numpad2,
                    winuser::VK_NUMPAD3 => KeyCode::Numpad3,
                    winuser::VK_NUMPAD4 => KeyCode::Numpad4,
                    winuser::VK_NUMPAD5 => KeyCode::Numpad5,
                    winuser::VK_NUMPAD6 => KeyCode::Numpad6,
                    winuser::VK_NUMPAD7 => KeyCode::Numpad7,
                    winuser::VK_NUMPAD8 => KeyCode::Numpad8,
                    winuser::VK_NUMPAD9 => KeyCode::Numpad9,
                    winuser::VK_MULTIPLY => KeyCode::Multiply,
                    winuser::VK_ADD => KeyCode::Add,
                    winuser::VK_SEPARATOR => KeyCode::Separator,
                    winuser::VK_SUBTRACT => KeyCode::Subtract,
                    winuser::VK_DECIMAL => KeyCode::Decimal,
                    winuser::VK_DIVIDE => KeyCode::Divide,
                    winuser::VK_F1 => KeyCode::Function(1),
                    winuser::VK_F2 => KeyCode::Function(2),
                    winuser::VK_F3 => KeyCode::Function(3),
                    winuser::VK_F4 => KeyCode::Function(4),
                    winuser::VK_F5 => KeyCode::Function(5),
                    winuser::VK_F6 => KeyCode::Function(6),
                    winuser::VK_F7 => KeyCode::Function(7),
                    winuser::VK_F8 => KeyCode::Function(8),
                    winuser::VK_F9 => KeyCode::Function(9),
                    winuser::VK_F10 => KeyCode::Function(10),
                    winuser::VK_F11 => KeyCode::Function(11),
                    winuser::VK_F12 => KeyCode::Function(12),
                    winuser::VK_F13 => KeyCode::Function(13),
                    winuser::VK_F14 => KeyCode::Function(14),
                    winuser::VK_F15 => KeyCode::Function(15),
                    winuser::VK_F16 => KeyCode::Function(16),
                    winuser::VK_F17 => KeyCode::Function(17),
                    winuser::VK_F18 => KeyCode::Function(18),
                    winuser::VK_F19 => KeyCode::Function(19),
                    winuser::VK_F20 => KeyCode::Function(20),
                    winuser::VK_F21 => KeyCode::Function(21),
                    winuser::VK_F22 => KeyCode::Function(22),
                    winuser::VK_F23 => KeyCode::Function(23),
                    winuser::VK_F24 => KeyCode::Function(24),
                    winuser::VK_NUMLOCK => KeyCode::NumLock,
                    winuser::VK_SCROLL => KeyCode::ScrollLock,
                    winuser::VK_LSHIFT => KeyCode::LeftShift,
                    winuser::VK_RSHIFT => KeyCode::RightShift,
                    winuser::VK_LCONTROL => KeyCode::LeftControl,
                    winuser::VK_RCONTROL => KeyCode::RightControl,
                    winuser::VK_LMENU => KeyCode::LeftMenu,
                    winuser::VK_RMENU => KeyCode::RightMenu,
                    winuser::VK_BROWSER_BACK => KeyCode::BrowserBack,
                    winuser::VK_BROWSER_FORWARD => KeyCode::BrowserForward,
                    winuser::VK_BROWSER_REFRESH => KeyCode::BrowserRefresh,
                    winuser::VK_BROWSER_STOP => KeyCode::BrowserStop,
                    winuser::VK_BROWSER_SEARCH => KeyCode::BrowserSearch,
                    winuser::VK_BROWSER_FAVORITES => KeyCode::BrowserFavorites,
                    winuser::VK_BROWSER_HOME => KeyCode::BrowserHome,
                    winuser::VK_VOLUME_MUTE => KeyCode::VolumeMute,
                    winuser::VK_VOLUME_DOWN => KeyCode::VolumeDown,
                    winuser::VK_VOLUME_UP => KeyCode::VolumeUp,
                    winuser::VK_MEDIA_NEXT_TRACK => KeyCode::MediaNextTrack,
                    winuser::VK_MEDIA_PREV_TRACK => KeyCode::MediaPrevTrack,
                    winuser::VK_MEDIA_STOP => KeyCode::MediaStop,
                    winuser::VK_MEDIA_PLAY_PAUSE => KeyCode::MediaPlayPause,
                    _ => return,
                },
            };
            let mut modifiers = modifiers_from_ctrl_key_state(event.dwControlKeyState);

            let key_code = key_code.normalize_shift_to_upper_case(modifiers);
            if let KeyCode::Char(c) = key_code {
                if c.is_ascii_uppercase() {
                    modifiers.remove(Modifiers::SHIFT);
                }
            }

            let input_event = InputEvent::Key(KeyEvent {
                key: key_code,
                modifiers,
            });
            for _ in 0..event.wRepeatCount {
                callback(input_event.clone());
            }
        }

        fn decode_mouse_record<F: FnMut(InputEvent)>(
            &self,
            event: &MOUSE_EVENT_RECORD,
            callback: &mut F,
        ) {
            use winapi::um::wincon::*;
            let mut buttons = MouseButtons::NONE;

            if (event.dwButtonState & FROM_LEFT_1ST_BUTTON_PRESSED) != 0 {
                buttons |= MouseButtons::LEFT;
            }
            if (event.dwButtonState & RIGHTMOST_BUTTON_PRESSED) != 0 {
                buttons |= MouseButtons::RIGHT;
            }
            if (event.dwButtonState & FROM_LEFT_2ND_BUTTON_PRESSED) != 0 {
                buttons |= MouseButtons::MIDDLE;
            }

            let modifiers = modifiers_from_ctrl_key_state(event.dwControlKeyState);

            if (event.dwEventFlags & MOUSE_WHEELED) != 0 {
                buttons |= MouseButtons::VERT_WHEEL;
                if (event.dwButtonState >> 8) != 0 {
                    buttons |= MouseButtons::WHEEL_POSITIVE;
                }
            } else if (event.dwEventFlags & MOUSE_HWHEELED) != 0 {
                buttons |= MouseButtons::HORZ_WHEEL;
                if (event.dwButtonState >> 8) != 0 {
                    buttons |= MouseButtons::WHEEL_POSITIVE;
                }
            }

            let mouse = InputEvent::Mouse(MouseEvent {
                x: event.dwMousePosition.X as u16,
                y: event.dwMousePosition.Y as u16,
                mouse_buttons: buttons,
                modifiers,
            });

            if (event.dwEventFlags & DOUBLE_CLICK) != 0 {
                callback(mouse.clone());
            }
            callback(mouse);
        }

        fn decode_resize_record<F: FnMut(InputEvent)>(
            &self,
            event: &WINDOW_BUFFER_SIZE_RECORD,
            callback: &mut F,
        ) {
            callback(InputEvent::Resized {
                rows: event.dwSize.Y as usize,
                cols: event.dwSize.X as usize,
            });
        }

        pub fn decode_input_records<F: FnMut(InputEvent)>(
            &mut self,
            records: &[INPUT_RECORD],
            callback: &mut F,
        ) {
            for record in records {
                match record.EventType {
                    KEY_EVENT => {
                        self.decode_key_record(unsafe { record.Event.KeyEvent() }, callback)
                    }
                    MOUSE_EVENT => {
                        self.decode_mouse_record(unsafe { record.Event.MouseEvent() }, callback)
                    }
                    WINDOW_BUFFER_SIZE_EVENT => self.decode_resize_record(
                        unsafe { record.Event.WindowBufferSizeEvent() },
                        callback,
                    ),
                    _ => {}
                }
            }
            self.process_bytes(callback, false);
        }
    }
}

impl Default for InputParser {
    fn default() -> Self {
        Self::new()
    }
}
