unsafe fn get_title_log_font(hwnd: HWND, hdc: HDC) -> Option<LOGFONTW> {
    let mut log_font = LOGFONTW::default();
    let theme = OpenThemeData(hwnd, wide_string("HEADER").as_ptr());
    if !theme.is_null() {
        let res = GetThemeFont(
            theme,
            hdc,
            extra_constants::HP_HEADERITEM,
            extra_constants::HIS_NORMAL,
            extra_constants::TMT_CAPTIONFONT,
            &mut log_font,
        );
        if res == S_OK {
            CloseThemeData(theme);
            return Some(log_font);
        }
    }

    let res = GetThemeSysFont(theme, extra_constants::TMT_CAPTIONFONT, &mut log_font);
    if !theme.is_null() {
        CloseThemeData(theme);
    }

    if res == S_OK {
        Some(log_font)
    } else {
        None
    }
}

unsafe fn update_title_font(hwnd: HWND) {
    let hdc = GetDC(hwnd);
    if hdc.is_null() {
        return;
    }

    let mut font = TITLE_FONT.lock().expect("locking title_font");
    if let Some(lf) = get_title_log_font(hwnd, hdc) {
        *font = wezterm_font::locator::gdi::parse_log_font(&lf, hdc).ok();
    }

    ReleaseDC(hwnd, hdc);
}

/// Set up bidirectional pointers:
/// hwnd.USERDATA -> WindowInner
/// WindowInner.hwnd -> hwnd
unsafe fn wm_nccreate(hwnd: HWND, _msg: UINT, _wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
    let create: &CREATESTRUCTW = &*(lparam as *const CREATESTRUCTW);
    let inner = rc_from_pointer(create.lpCreateParams);
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as _);
    inner.borrow_mut().hwnd = HWindow(hwnd);

    None
}

/// Called when the window is being destroyed.
/// Goal is to release the WindowInner reference that was stashed
/// in the window by wm_nccreate.
unsafe fn wm_ncdestroy(
    hwnd: HWND,
    _msg: UINT,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> Option<LRESULT> {
    let raw = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as LPVOID;
    if !raw.is_null() {
        let inner = take_rc_from_pointer(raw);
        let mut inner = inner.borrow_mut();
        inner.events.dispatch(WindowEvent::Destroyed);
        inner.hwnd = HWindow(null_mut());
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
    }

    None
}

fn no_native_title_bar(decorations: WindowDecorations) -> bool {
    decorations == WindowDecorations::RESIZE
        || decorations.contains(WindowDecorations::INTEGRATED_BUTTONS)
}

unsafe fn wm_nccalcsize(hwnd: HWND, _msg: UINT, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
    let inner = rc_from_hwnd(hwnd)?;
    let inner = match inner.try_borrow() {
        Ok(inner) => inner,
        Err(_) => {
            // We've been called recursively and the upper levels
            // own the borrow. Just take the default action
            return None;
        }
    };

    let no_native_title_bar = no_native_title_bar(inner.config.window_decorations);

    if !(wparam == 1 && no_native_title_bar) {
        return None;
    }

    if inner.saved_placement.is_none() {
        let dpi = inner.get_effective_dpi() as u32;
        let frame_x = GetSystemMetricsForDpi(SM_CXFRAME, dpi);
        let frame_y = GetSystemMetricsForDpi(SM_CYFRAME, dpi);
        let padding = GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi);

        let params = (lparam as *mut NCCALCSIZE_PARAMS).as_mut().unwrap();

        let requested_client_rect = &mut params.rgrc[0];

        requested_client_rect.right -= frame_x + padding;
        requested_client_rect.left += frame_x + padding;

        let is_maximized = get_window_state(hwnd) == WindowState::MAXIMIZED;

        // Handle bugged top window border on Windows 10
        if *IS_WIN10 {
            if is_maximized {
                requested_client_rect.top += frame_y + padding;
                requested_client_rect.bottom -= frame_y + padding - 2;
            } else {
                requested_client_rect.top += 1;
                requested_client_rect.bottom -= frame_y - padding;
            }
        } else {
            requested_client_rect.bottom -= frame_y + padding;

            if is_maximized {
                requested_client_rect.top += frame_y + padding;
            }
        }
    }

    Some(0)
}

unsafe fn wm_nchittest(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
    let inner = rc_from_hwnd(hwnd)?;
    let inner = match inner.try_borrow() {
        Ok(inner) => inner,
        Err(_) => {
            // We've been called recursively and the upper levels
            // own the borrow. Just take the default action
            return None;
        }
    };

    let no_native_title_bar = no_native_title_bar(inner.config.window_decorations);
    if !no_native_title_bar {
        return None;
    }

    // Let the default procedure handle resizing areas
    let result = DefWindowProcW(hwnd, msg, wparam, lparam);

    if matches!(
        result,
        HTNOWHERE
            | HTRIGHT
            | HTLEFT
            | HTTOPLEFT
            | HTTOP
            | HTTOPRIGHT
            | HTBOTTOMRIGHT
            | HTBOTTOM
            | HTBOTTOMLEFT
    ) {
        return Some(result);
    }

    // The adjustment in NCCALCSIZE messes with the detection
    // of the top hit area so manually fixing that.
    let dpi = inner.get_effective_dpi() as u32;
    let frame_x = GetSystemMetricsForDpi(SM_CXFRAME, dpi) as isize;
    let frame_y = GetSystemMetricsForDpi(SM_CYFRAME, dpi) as isize;
    let padding = GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi) as isize;

    let coords = mouse_coords(lparam);
    let screen_point = ScreenPoint::new(coords.x, coords.y);
    let cursor_point = screen_to_client(hwnd, screen_point);
    let is_maximized = get_window_state(hwnd) == WindowState::MAXIMIZED;

    // check if mouse is in any of the resize areas (HTTOP, HTBOTTOM, etc)

    let mut client_rect = RECT::default();
    let client_rect_is_valid =
        GetClientRect(hwnd, &mut client_rect) == winapi::shared::minwindef::TRUE;

    // Since we are eating the bottom window frame to deal with a Windows 10 bug,
    // we detect resizing in the window client area as a workaround
    if !is_maximized
        && *IS_WIN10
        && client_rect_is_valid
        && cursor_point.y >= (client_rect.bottom as isize) - (frame_y + padding)
    {
        if cursor_point.x <= (frame_x + padding) {
            return Some(HTBOTTOMLEFT);
        } else if cursor_point.x >= (client_rect.right as isize) - (frame_x + padding) {
            return Some(HTBOTTOMRIGHT);
        } else {
            return Some(HTBOTTOM);
        }
    }

    if !is_maximized && cursor_point.y >= 0 && cursor_point.y < frame_y {
        if cursor_point.x <= (frame_x + padding) {
            return Some(HTTOPLEFT);
        } else if cursor_point.x >= (client_rect.right as isize) - (frame_x + padding) {
            return Some(HTTOPRIGHT);
        } else {
            return Some(HTTOP);
        }
    }

    if let Some(coords) = inner.window_drag_position {
        if coords == screen_point && inner.saved_placement.is_none() {
            return Some(HTCAPTION);
        }
    }

    let use_snap_layouts = !*IS_WIN10;
    if use_snap_layouts {
        if let Some(max) = inner.maximize_button_position {
            if max.contains(screen_point) {
                return Some(HTMAXBUTTON);
            }
        }
    }

    Some(HTCLIENT)
}

fn get_window_state(hwnd: HWND) -> WindowState {
    let mut placement = WINDOWPLACEMENT {
        length: std::mem::size_of::<WINDOWPLACEMENT>() as _,
        ..Default::default()
    };

    let placement =
        if unsafe { GetWindowPlacement(hwnd, &mut placement) } == winapi::shared::minwindef::TRUE {
            placement.showCmd as i32
        } else {
            0
        };

    match placement {
        SW_SHOWMAXIMIZED => WindowState::MAXIMIZED,
        SW_SHOWMINIMIZED => WindowState::HIDDEN,
        _ => unsafe {
            let mut rect = std::mem::zeroed();
            GetWindowRect(hwnd, &mut rect);

            let mut mi: MONITORINFO = std::mem::zeroed();
            mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            GetMonitorInfoW(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST), &mut mi);

            if mi.rcMonitor.left == rect.left
                && mi.rcMonitor.top == rect.top
                && mi.rcMonitor.right == rect.right
                && mi.rcMonitor.bottom == rect.bottom
            {
                WindowState::FULL_SCREEN
            } else {
                WindowState::default()
            }
        },
    }
}

/// "Blur behind" is the old vista term for a cool blurring
/// effect that the DWM could enable.  Subsequent windows
/// versions have removed the blurring.  We use this call
/// to tell DWM that we set proper alpha channel info as
/// a result of rendering our window content.
fn enable_blur_behind(hwnd: HWND) {
    use winapi::shared::minwindef::*;
    use winapi::um::dwmapi::*;
    use winapi::um::wingdi::*;

    unsafe {
        let region = CreateRectRgn(0, 0, -1, -1);

        let bb = DWM_BLURBEHIND {
            dwFlags: DWM_BB_ENABLE | DWM_BB_BLURREGION,
            fEnable: TRUE,
            hRgnBlur: region,
            fTransitionOnMaximized: FALSE,
        };

        DwmEnableBlurBehindWindow(hwnd, &bb);

        DeleteObject(region as _);
    }
}

fn apply_theme(hwnd: HWND) -> Option<LRESULT> {
    // Check for OS app theme, and set window attributes accordingly.
    // Note that the MS terminal app uses the logic found here for this stuff:
    // https://github.com/microsoft/terminal/blob/9b92986b49bed8cc41fde4d6ef080921c41e6d9e/src/interactivity/win32/windowtheme.cpp#L62
    use winapi::um::dwmapi::{DwmExtendFrameIntoClientArea, DwmSetWindowAttribute};
    use winapi::um::uxtheme::MARGINS;

    #[allow(non_snake_case)]
    type WINDOWCOMPOSITIONATTRIB = u32;
    const WCA_USEDARKMODECOLORS: WINDOWCOMPOSITIONATTRIB = 26;

    #[allow(non_snake_case)]
    #[repr(C)]
    pub struct WINDOWCOMPOSITIONATTRIBDATA {
        Attrib: WINDOWCOMPOSITIONATTRIB,
        pvData: PVOID,
        cbData: winapi::shared::basetsd::SIZE_T,
    }

    shared_library!(User32,
        pub fn SetWindowCompositionAttribute(hwnd: HWND, attrib: *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL,
    );

    const DWMWA_USE_IMMERSIVE_DARK_MODE: DWORD = 20;
    const DWMWA_MICA_EFFECT: DWORD = 1029;
    const DWMWA_SYSTEMBACKDROP_TYPE: DWORD = 38;

    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    #[derive(PartialEq, Eq)]
    #[repr(C)]
    enum ACCENT_STATE {
        ACCENT_DISABLED = 0,
        ACCENT_ENABLE_BLURBEHIND = 3,
        ACCENT_ENABLE_ACRYLICBLURBEHIND = 4,
    }

    #[allow(non_snake_case)]
    #[repr(C)]
    struct ACCENT_POLICY {
        AccentState: u32,
        AccentFlags: u32,
        GradientColour: u32,
        AnimationId: u32,
    }

    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    #[repr(C)]
    enum DWM_SYSTEMBACKDROP_TYPE {
        DWMSBT_AUTO = 0,
        DWMSBT_NONE = 1,
        DWMSBT_MAINWINDOW = 2,      // Mica
        DWMSBT_TRANSIENTWINDOW = 3, // Acrylic
        DWMSBT_TABBEDWINDOW = 4,    // Tabbed
    }

    unsafe {
        update_title_font(hwnd);

        let appearance = get_appearance();
        let theme_string = if appearance == Appearance::Dark {
            "DarkMode_Explorer"
        } else {
            ""
        };

        SetWindowTheme(
            hwnd as _,
            wide_string(theme_string).as_slice().as_ptr(),
            std::ptr::null_mut(),
        );

        let mut enabled: BOOL = if appearance == Appearance::Dark { 1 } else { 0 };
        DwmSetWindowAttribute(
            hwnd as _,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &enabled as *const _ as *const _,
            std::mem::size_of_val(&enabled) as u32,
        );

        if let Ok(user) = User32::open(std::path::Path::new("user32.dll")) {
            (user.SetWindowCompositionAttribute)(
                hwnd,
                &mut WINDOWCOMPOSITIONATTRIBDATA {
                    Attrib: WCA_USEDARKMODECOLORS,
                    pvData: &mut enabled as *mut _ as _,
                    cbData: std::mem::size_of_val(&enabled) as _,
                },
            );
        };

        if let Some(inner) = rc_from_hwnd(hwnd) {
            let mut inner = inner.borrow_mut();

            // Set Acrylic or Mica system Backdrop
            let pv_attribute = match inner.config.win32_system_backdrop {
                SystemBackdrop::Auto => DWM_SYSTEMBACKDROP_TYPE::DWMSBT_AUTO,
                SystemBackdrop::Disable => DWM_SYSTEMBACKDROP_TYPE::DWMSBT_NONE,
                SystemBackdrop::Acrylic => DWM_SYSTEMBACKDROP_TYPE::DWMSBT_TRANSIENTWINDOW,
                SystemBackdrop::Mica => DWM_SYSTEMBACKDROP_TYPE::DWMSBT_MAINWINDOW,
                SystemBackdrop::Tabbed => DWM_SYSTEMBACKDROP_TYPE::DWMSBT_TABBEDWINDOW,
            };

            let margins = match inner.config.window_decorations {
                WindowDecorations::TITLE => -1,
                _ => 0,
            };

            DwmExtendFrameIntoClientArea(
                hwnd,
                &MARGINS {
                    cxLeftWidth: margins,
                    cxRightWidth: margins,
                    cyTopHeight: margins,
                    cyBottomHeight: margins,
                },
            );

            // Apply Acrylic or Mica Backdrop
            if *IS_WIN11_22H2 {
                DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_SYSTEMBACKDROP_TYPE,
                    &pv_attribute as *const _ as _,
                    std::mem::size_of_val(&pv_attribute) as u32,
                );
            } else {
                let mut colour = inner.config.win32_acrylic_accent_color.to_srgb_u8();
                colour.3 = if colour.3 == 0 { 1 } else { colour.3 }; // acrylic doesn't like to have 0 alpha

                let mut policy = ACCENT_POLICY {
                    AccentState: if inner.config.win32_system_backdrop == SystemBackdrop::Acrylic {
                        ACCENT_STATE::ACCENT_ENABLE_ACRYLICBLURBEHIND as _
                    } else {
                        ACCENT_STATE::ACCENT_DISABLED as _
                    },
                    AccentFlags: if inner.config.win32_system_backdrop == SystemBackdrop::Acrylic {
                        2
                    } else {
                        0
                    },
                    GradientColour: (colour.0 as u32)
                        | (colour.1 as u32) << 8
                        | (colour.2 as u32) << 16
                        | (colour.3 as u32) << 24,
                    AnimationId: 0,
                };

                if let Ok(user) = User32::open(std::path::Path::new("user32.dll")) {
                    (user.SetWindowCompositionAttribute)(
                        hwnd,
                        &mut WINDOWCOMPOSITIONATTRIBDATA {
                            Attrib: 0x13,
                            pvData: &mut policy as *mut _ as _,
                            cbData: std::mem::size_of_val(&policy) as _,
                        },
                    );
                }

                if !*IS_WIN10 && !*IS_WIN11_22H2 {
                    // For build versions less than 22h2 but are still win11
                    let mica_enabled: u32 =
                        if inner.config.win32_system_backdrop == SystemBackdrop::Mica {
                            1
                        } else {
                            0
                        };
                    DwmSetWindowAttribute(
                        hwnd,
                        DWMWA_MICA_EFFECT,
                        &mica_enabled as *const _ as _,
                        std::mem::size_of_val(&mica_enabled) as u32,
                    );
                }
            }

            if appearance != inner.appearance {
                inner.appearance = appearance;
                inner
                    .events
                    .dispatch(WindowEvent::AppearanceChanged(appearance));
            }
        }
    }

    None
}
