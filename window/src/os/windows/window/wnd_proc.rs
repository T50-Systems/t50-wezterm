unsafe fn drop_files(hwnd: HWND, _msg: UINT, wparam: WPARAM, _lparam: LPARAM) -> Option<LRESULT> {
    let inner = rc_from_hwnd(hwnd)?;
    let h_drop = wparam as HDROP;

    // Get the number of files dropped
    let file_count = DragQueryFileW(h_drop, 0xFFFFFFFF, null_mut(), 0);

    let mut filenames: Vec<PathBuf> = Vec::with_capacity(file_count as usize);

    for idx in 0..file_count {
        // The returned size of buffer is in characters, not including the terminating null character
        let buf_size = DragQueryFileW(h_drop, idx, null_mut(), 0);
        if buf_size > 0 {
            // Windows will truncate the filename and add null terminator if space isn't enough
            let buf_size = buf_size as usize + 1;
            let mut wide_buf = vec![0u16; buf_size];
            DragQueryFileW(h_drop, idx, wide_buf.as_mut_ptr(), wide_buf.len() as u32);
            wide_buf.pop(); // Drops the null terminator
            filenames.push(OsString::from_wide(&wide_buf).into());
        }
    }

    let mut inner = inner.borrow_mut();
    inner.events.dispatch(WindowEvent::DroppedFile(filenames));

    DragFinish(h_drop);
    Some(0)
}

unsafe fn do_wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
    match msg {
        WM_NCCREATE => wm_nccreate(hwnd, msg, wparam, lparam),
        WM_NCDESTROY => wm_ncdestroy(hwnd, msg, wparam, lparam),
        WM_NCCALCSIZE => wm_nccalcsize(hwnd, msg, wparam, lparam),
        WM_NCHITTEST => wm_nchittest(hwnd, msg, wparam, lparam),
        WM_PAINT => wm_paint(hwnd, msg, wparam, lparam),
        WM_ENTERSIZEMOVE | WM_EXITSIZEMOVE => wm_enter_exit_size_move(hwnd, msg, wparam, lparam),
        WM_WINDOWPOSCHANGED => wm_windowposchanged(hwnd, msg, wparam, lparam),
        WM_SETFOCUS => wm_set_focus(hwnd, msg, wparam, lparam),
        WM_KILLFOCUS => wm_kill_focus(hwnd, msg, wparam, lparam),
        WM_DEADCHAR | WM_KEYDOWN | WM_KEYUP | WM_SYSCHAR | WM_CHAR | WM_IME_CHAR | WM_SYSKEYUP
        | WM_SYSKEYDOWN => key(hwnd, msg, wparam, lparam),
        WM_SIZING => {
            // Allow events to be processed during live resize
            crate::spawn::SPAWN_QUEUE.run();
            None
        }
        WM_SETTINGCHANGE | WM_DWMCOMPOSITIONCHANGED => apply_theme(hwnd),
        WM_IME_SETCONTEXT => ime_set_context(hwnd, msg, wparam, lparam),
        WM_IME_COMPOSITION => ime_composition(hwnd, msg, wparam, lparam),
        WM_IME_ENDCOMPOSITION => ime_end_composition(hwnd, msg, wparam, lparam),
        WM_MOUSEMOVE => mouse_move(hwnd, msg, wparam, lparam),
        WM_MOUSELEAVE => mouse_leave(hwnd, msg, wparam, lparam),
        WM_MOUSEHWHEEL | WM_MOUSEWHEEL => mouse_wheel(hwnd, msg, wparam, lparam),
        WM_LBUTTONDBLCLK | WM_RBUTTONDBLCLK | WM_MBUTTONDBLCLK | WM_LBUTTONDOWN | WM_LBUTTONUP
        | WM_RBUTTONDOWN | WM_RBUTTONUP | WM_MBUTTONDOWN | WM_MBUTTONUP => {
            mouse_button(hwnd, msg, wparam, lparam)
        }
        WM_DROPFILES => drop_files(hwnd, msg, wparam, lparam),
        WM_ERASEBKGND => Some(1),
        WM_CLOSE => {
            if let Some(inner) = rc_from_hwnd(hwnd) {
                let mut inner = inner.borrow_mut();
                inner.events.dispatch(WindowEvent::CloseRequested);
                // Don't let it close
                return Some(0);
            }
            None
        }
        _ => {
            if matches!(
                msg,
                WM_NCMOUSEMOVE | WM_NCMOUSELEAVE | WM_NCLBUTTONDOWN | WM_NCLBUTTONDBLCLK
            ) {
                let use_snap_layouts = !*IS_WIN10;
                if use_snap_layouts {
                    return match msg {
                        WM_NCMOUSEMOVE => nc_mouse_move(hwnd, msg, wparam, lparam),
                        WM_NCMOUSELEAVE => mouse_leave(hwnd, msg, wparam, lparam),
                        WM_NCLBUTTONDOWN | WM_NCLBUTTONDBLCLK => {
                            nc_mouse_button(hwnd, msg, wparam, lparam)
                        }
                        _ => None,
                    };
                }
            }

            None
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match std::panic::catch_unwind(|| {
        do_wnd_proc(hwnd, msg, wparam, lparam)
            .unwrap_or_else(|| DefWindowProcW(hwnd, msg, wparam, lparam))
    }) {
        Ok(result) => result,
        Err(e) => {
            log::error!("caught {:?}", e);
            std::process::exit(1)
        }
    }
}
