unsafe fn wm_enter_exit_size_move(
    hwnd: HWND,
    msg: UINT,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> Option<LRESULT> {
    let mut should_size = false;
    if let Some(inner) = rc_from_hwnd(hwnd) {
        let mut inner = inner.borrow_mut();
        inner.in_size_move = msg == WM_ENTERSIZEMOVE;
        should_size = !inner.in_size_move;
    }

    if should_size {
        wm_size(hwnd, 0, 0, 0)?;
    }

    Some(0)
}

/// We handle WM_WINDOWPOSCHANGED and dispatch directly to our wm_size as it
/// is a bit more efficient than letting DefWindowProcW parse this and
/// trigger WM_SIZE.
unsafe fn wm_windowposchanged(
    hwnd: HWND,
    _msg: UINT,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> Option<LRESULT> {
    // let pos = &*(lparam as *const WINDOWPOS);
    wm_size(hwnd, 0, 0, 0)?;
    Some(0)
}

unsafe fn wm_size(hwnd: HWND, _msg: UINT, _wparam: WPARAM, _lparam: LPARAM) -> Option<LRESULT> {
    let mut should_paint = false;
    let mut should_pump = false;

    if let Some(inner) = rc_from_hwnd(hwnd) {
        let mut inner = inner.borrow_mut();
        should_paint = inner.check_and_call_resize_if_needed();
        should_pump = inner.in_size_move;
    }

    if should_paint {
        wm_paint(hwnd, 0, 0, 0)?;
        if should_pump {
            crate::spawn::SPAWN_QUEUE.run();
        }
    }

    None
}

unsafe fn wm_set_focus(
    hwnd: HWND,
    _msg: UINT,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> Option<LRESULT> {
    rc_from_hwnd(hwnd)?
        .borrow_mut()
        .events
        .dispatch(WindowEvent::FocusChanged(true));
    None
}

unsafe fn wm_kill_focus(
    hwnd: HWND,
    _msg: UINT,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> Option<LRESULT> {
    rc_from_hwnd(hwnd)?
        .borrow_mut()
        .events
        .dispatch(WindowEvent::FocusChanged(false));
    None
}

unsafe fn wm_paint(hwnd: HWND, _msg: UINT, _wparam: WPARAM, _lparam: LPARAM) -> Option<LRESULT> {
    let inner = rc_from_hwnd(hwnd)?;
    let mut inner = inner.borrow_mut();

    if inner.paint_throttled {
        inner.invalidated = true;
        return Some(0);
    }

    let mut ps = PAINTSTRUCT {
        fErase: 0,
        fIncUpdate: 0,
        fRestore: 0,
        hdc: std::ptr::null_mut(),
        rcPaint: RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        rgbReserved: [0; 32],
    };
    let _ = BeginPaint(hwnd, &mut ps);
    // Do nothing right now
    EndPaint(hwnd, &mut ps);

    inner.invalidated = false;
    // Ask the app to repaint in a bit
    inner.events.dispatch(WindowEvent::NeedRepaint);

    inner.paint_throttled = true;
    let window_id = inner.hwnd;
    let max_fps = inner.config.max_fps;
    promise::spawn::spawn(async move {
        async_io::Timer::after(std::time::Duration::from_millis(1000 / max_fps as u64)).await;
        Connection::with_window_inner(window_id, move |inner| {
            inner.paint_throttled = false;
            if inner.invalidated {
                InvalidateRect(inner.hwnd.0, null(), 0);
            }
            Ok(())
        });
    })
    .detach();

    Some(0)
}

fn mods_and_buttons(wparam: WPARAM) -> (Modifiers, MouseButtons) {
    let mut modifiers = Modifiers::default();
    let mut buttons = MouseButtons::default();
    if wparam & MK_CONTROL != 0 {
        modifiers |= Modifiers::CTRL;
    }
    if wparam & MK_SHIFT != 0 {
        modifiers |= Modifiers::SHIFT;
    }
    if unsafe { GetKeyState(VK_MENU) } as u16 & 0x8000 != 0 {
        modifiers |= Modifiers::ALT;
    }
    if wparam & MK_LBUTTON != 0 {
        buttons |= MouseButtons::LEFT;
    }
    if wparam & MK_MBUTTON != 0 {
        buttons |= MouseButtons::MIDDLE;
    }
    if wparam & MK_RBUTTON != 0 {
        buttons |= MouseButtons::RIGHT;
    }
    // TODO: XBUTTON1 and XBUTTON2?
    (modifiers, buttons)
}

fn mouse_coords(lparam: LPARAM) -> Point {
    let point = MAKEPOINTS(lparam as _);
    Point::new(point.x as _, point.y as _)
}

fn nc_mouse_coords(hwnd: HWND, lparam: LPARAM) -> Point {
    let point = MAKEPOINTS(lparam as _);
    let point = ScreenPoint::new(point.x as _, point.y as _);
    screen_to_client(hwnd, point)
}

fn screen_to_client(hwnd: HWND, point: ScreenPoint) -> Point {
    let mut point = POINT {
        x: point.x.try_into().unwrap(),
        y: point.y.try_into().unwrap(),
    };
    unsafe { ScreenToClient(hwnd, &mut point as *mut _) };
    Point::new(point.x.try_into().unwrap(), point.y.try_into().unwrap())
}

fn client_to_screen(hwnd: HWND, point: Point) -> ScreenPoint {
    let mut point = POINT {
        x: point.x.try_into().unwrap(),
        y: point.y.try_into().unwrap(),
    };
    unsafe { ClientToScreen(hwnd, &mut point as *mut _) };
    ScreenPoint::new(point.x.try_into().unwrap(), point.y.try_into().unwrap())
}

fn apply_mouse_cursor(cursor: Option<MouseCursor>) {
    match cursor {
        None => unsafe {
            SetCursor(null_mut());
        },
        Some(cursor) => unsafe {
            SetCursor(LoadCursorW(
                null_mut(),
                match cursor {
                    MouseCursor::Arrow => IDC_ARROW,
                    MouseCursor::Hand => IDC_HAND,
                    MouseCursor::Text => IDC_IBEAM,
                    MouseCursor::SizeUpDown => IDC_SIZENS,
                    MouseCursor::SizeLeftRight => IDC_SIZEWE,
                },
            ));
        },
    }
}

unsafe fn mouse_button(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
    let inner = rc_from_hwnd(hwnd)?;
    // To support dragging the window, capture when the left
    // button goes down and release when it goes up.
    // Without this, the drag state can be confused when dragging
    // the mouse up outside of the client area.
    if msg == WM_LBUTTONDOWN {
        SetCapture(hwnd);
    } else if msg == WM_LBUTTONUP {
        ReleaseCapture();
    }
    let (modifiers, mouse_buttons) = mods_and_buttons(wparam);
    let coords = mouse_coords(lparam);
    let event = MouseEvent {
        kind: match msg {
            WM_LBUTTONDOWN => MouseEventKind::Press(MousePress::Left),
            WM_LBUTTONUP => MouseEventKind::Release(MousePress::Left),
            WM_RBUTTONDOWN => MouseEventKind::Press(MousePress::Right),
            WM_RBUTTONUP => MouseEventKind::Release(MousePress::Right),
            WM_MBUTTONDOWN => MouseEventKind::Press(MousePress::Middle),
            WM_MBUTTONUP => MouseEventKind::Release(MousePress::Middle),
            _ => return None,
        },
        coords,
        screen_coords: client_to_screen(hwnd, coords),
        mouse_buttons,
        modifiers,
    };
    inner
        .borrow_mut()
        .events
        .dispatch(WindowEvent::MouseEvent(event));
    Some(0)
}

unsafe fn nc_mouse_button(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Option<LRESULT> {
    let inner = rc_from_hwnd(hwnd)?;

    let no_native_title_bar = no_native_title_bar(inner.borrow().config.window_decorations);
    if !no_native_title_bar {
        // Don't mess with this event unless we're doing our own custom
        // titlebar
        return None;
    }

    // To support dragging the window, capture when the left
    // button goes down and release when it goes up.
    // Without this, the drag state can be confused when dragging
    // the mouse up outside of the client area.

    if msg == WM_LBUTTONDOWN {
        SetCapture(hwnd);
    } else if msg == WM_LBUTTONUP {
        ReleaseCapture();
    }

    if wparam != HTMAXBUTTON as usize {
        return None;
    }

    let (modifiers, mouse_buttons) = mods_and_buttons(0);
    let coords = nc_mouse_coords(hwnd, lparam);

    let event = MouseEvent {
        kind: match msg {
            WM_NCLBUTTONDOWN | WM_NCLBUTTONDBLCLK => MouseEventKind::Press(MousePress::Left),
            _ => return None,
        },
        coords,
        screen_coords: client_to_screen(hwnd, coords),
        mouse_buttons,
        modifiers,
    };
    inner
        .borrow_mut()
        .events
        .dispatch(WindowEvent::MouseEvent(event));
    Some(0)
}

unsafe fn mouse_move(hwnd: HWND, _msg: UINT, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
    let inner = rc_from_hwnd(hwnd)?;
    let mut inner = inner.borrow_mut();

    if !inner.track_mouse_leave {
        inner.track_mouse_leave = true;

        let mut trk = TRACKMOUSEEVENT {
            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE,
            hwndTrack: hwnd,
            dwHoverTime: 0,
        };

        inner.track_mouse_leave = TrackMouseEvent(&mut trk) == winapi::shared::minwindef::TRUE;
    }

    let (modifiers, mouse_buttons) = mods_and_buttons(wparam);
    let coords = mouse_coords(lparam);
    let event = MouseEvent {
        kind: MouseEventKind::Move,
        coords,
        screen_coords: client_to_screen(hwnd, coords),
        mouse_buttons,
        modifiers,
    };

    inner.events.dispatch(WindowEvent::MouseEvent(event));
    Some(0)
}

unsafe fn nc_mouse_move(hwnd: HWND, _msg: UINT, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
    let inner = rc_from_hwnd(hwnd)?;
    let mut inner = inner.borrow_mut();

    if !inner.track_mouse_leave {
        inner.track_mouse_leave = true;

        let mut trk = TRACKMOUSEEVENT {
            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE | TME_NONCLIENT,
            hwndTrack: hwnd,
            dwHoverTime: 0,
        };

        inner.track_mouse_leave = TrackMouseEvent(&mut trk) == winapi::shared::minwindef::TRUE;
    }

    if wparam != HTMAXBUTTON as usize {
        return None;
    }

    let (modifiers, mouse_buttons) = mods_and_buttons(0);
    let coords = nc_mouse_coords(hwnd, lparam);

    let event = MouseEvent {
        kind: MouseEventKind::Move,
        coords,
        screen_coords: client_to_screen(hwnd, coords),
        mouse_buttons,
        modifiers,
    };

    inner.events.dispatch(WindowEvent::MouseEvent(event));
    inner.events.dispatch(WindowEvent::NeedRepaint);

    Some(0)
}

unsafe fn mouse_leave(hwnd: HWND, _msg: UINT, _wparam: WPARAM, _lparam: LPARAM) -> Option<LRESULT> {
    let inner = rc_from_hwnd(hwnd)?;
    let mut inner = inner.borrow_mut();

    inner.track_mouse_leave = false;
    inner.events.dispatch(WindowEvent::MouseLeave);

    Some(0)
}

lazy_static! {
    static ref WHEEL_SCROLL_LINES: i16 = read_scroll_speed("WheelScrollLines").unwrap_or(3);
    static ref WHEEL_SCROLL_CHARS: i16 = read_scroll_speed("WheelScrollChars").unwrap_or(3);
}

fn read_scroll_speed(name: &str) -> io::Result<i16> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let desktop = hkcu.open_subkey("Control Panel\\Desktop")?;
    desktop
        .get_value::<String, _>(name)
        .and_then(|v| v.parse().map_err(|_| io::ErrorKind::InvalidData.into()))
}

unsafe fn mouse_wheel(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
    let inner = rc_from_hwnd(hwnd)?;
    let (modifiers, mouse_buttons) = mods_and_buttons(wparam);
    // Wheel events return screen coordinates!
    let coords = mouse_coords(lparam);
    let screen_coords = ScreenPoint::new(coords.x, coords.y);
    let coords = screen_to_client(hwnd, screen_coords);
    let delta = GET_WHEEL_DELTA_WPARAM(wparam);
    let scaled_delta = if msg == WM_MOUSEWHEEL {
        delta * (*WHEEL_SCROLL_LINES)
    } else {
        delta * (*WHEEL_SCROLL_CHARS)
    };
    let mut position = scaled_delta / WHEEL_DELTA;
    let remainder = scaled_delta % WHEEL_DELTA;
    let event = MouseEvent {
        kind: if msg == WM_MOUSEHWHEEL {
            let mut inner = inner.borrow_mut();
            if inner.hscroll_remainder.signum() != remainder.signum() {
                // Reset remainder when changing scroll direction
                inner.hscroll_remainder = 0;
            }
            inner.hscroll_remainder += remainder;
            position += inner.hscroll_remainder / WHEEL_DELTA;
            inner.hscroll_remainder %= WHEEL_DELTA;
            log::trace!(
                "mouse_hwheel delta={} scaled={} remainder={} pos={}",
                delta,
                scaled_delta,
                inner.hscroll_remainder,
                position
            );
            if position == 0 {
                return Some(0);
            }
            MouseEventKind::HorzWheel(position)
        } else {
            let mut inner = inner.borrow_mut();
            if inner.vscroll_remainder.signum() != remainder.signum() {
                // Reset remainder when changing scroll direction
                inner.vscroll_remainder = 0;
            }
            inner.vscroll_remainder += remainder;
            position += inner.vscroll_remainder / WHEEL_DELTA;
            inner.vscroll_remainder %= WHEEL_DELTA;
            log::trace!(
                "mouse_wheel delta={} scaled={} remainder={} pos={}",
                delta,
                scaled_delta,
                inner.vscroll_remainder,
                position
            );
            if position == 0 {
                return Some(0);
            }
            MouseEventKind::VertWheel(position)
        },
        coords,
        screen_coords,
        mouse_buttons,
        modifiers,
    };
    inner
        .borrow_mut()
        .events
        .dispatch(WindowEvent::MouseEvent(event));
    Some(0)
}
