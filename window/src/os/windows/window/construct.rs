impl Window {
    fn create_window(
        config: ConfigHandle,
        class_name: &str,
        name: &str,
        geometry: ResolvedGeometry,
        lparam: *const RefCell<WindowInner>,
    ) -> anyhow::Result<HWND> {
        let class_name = wide_string(class_name);
        let h_inst = unsafe { GetModuleHandleW(null()) };
        let class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_inst,
            // FIXME: this resource is specific to the wezterm build and this should
            // really be made generic for other sorts of windows.
            // The ID is defined in assets/windows/resource.rc
            hIcon: unsafe { LoadIconW(h_inst, MAKEINTRESOURCEW(0x101)) },
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null(),
            lpszClassName: class_name.as_ptr(),
        };

        if unsafe { RegisterClassW(&class) } == 0 {
            let err = IoError::last_os_error();
            match err.raw_os_error() {
                Some(code)
                    if code == winapi::shared::winerror::ERROR_CLASS_ALREADY_EXISTS as i32 => {}
                _ => return Err(err.into()),
            }
        }

        let decorations = config.window_decorations;
        let style = decorations_to_style(decorations);
        let frame_dpi = get_primary_monitor_dpi();
        let (width, height) =
            adjust_client_to_window_dimensions(style, geometry.width, geometry.height, frame_dpi);

        let (x, y) = match (geometry.x, geometry.y) {
            (Some(x), Some(y)) => (x, y),
            _ => {
                if (style & WS_POPUP) == 0 {
                    (CW_USEDEFAULT, CW_USEDEFAULT)
                } else {
                    // WS_POPUP windows need to specify the initial position.
                    // We pick the middle of the primary monitor

                    unsafe {
                        let mut mi: MONITORINFO = std::mem::zeroed();
                        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                        GetMonitorInfoW(
                            MonitorFromWindow(std::ptr::null_mut(), MONITOR_DEFAULTTOPRIMARY),
                            &mut mi,
                        );

                        let mon_width = mi.rcMonitor.right - mi.rcMonitor.left;
                        let mon_height = mi.rcMonitor.bottom - mi.rcMonitor.top;

                        (
                            mi.rcMonitor.left + (mon_width - width) / 2,
                            mi.rcMonitor.top + (mon_height - height) / 2,
                        )
                    }
                }
            }
        };

        let name = wide_string(name);
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                name.as_ptr(),
                style,
                x,
                y,
                width,
                height,
                null_mut(),
                null_mut(),
                null_mut(),
                std::mem::transmute(lparam),
            )
        };

        if hwnd.is_null() {
            let err = IoError::last_os_error();
            bail!("CreateWindowExW: {}", err);
        }

        // We have to re-apply the styles otherwise they don't
        // completely stick
        schedule_apply_decoration(hwnd, decorations);

        Ok(hwnd)
    }

    pub async fn new_window<F>(
        class_name: &str,
        name: &str,
        geometry: RequestedWindowGeometry,
        config: Option<&ConfigHandle>,
        _font_config: Rc<FontConfiguration>,
        event_handler: F,
    ) -> anyhow::Result<Window>
    where
        F: 'static + FnMut(WindowEvent, &Window),
    {
        let events = WindowEventSender::new(event_handler);

        let config = match config {
            Some(c) => c.clone(),
            None => config::configuration(),
        };
        let appearance = get_appearance();

        let inner = Rc::new(RefCell::new(WindowInner {
            hwnd: HWindow(null_mut()),
            appearance,
            events,
            gl_state: None,
            vscroll_remainder: 0,
            hscroll_remainder: 0,
            keyboard_info: KeyboardLayoutInfo::new(),
            last_size: None,
            in_size_move: false,
            dead_pending: None,
            saved_placement: None,
            track_mouse_leave: false,
            window_drag_position: None,
            maximize_button_position: None,
            config: config.clone(),
            paint_throttled: false,
            invalidated: true,
        }));

        // Careful: `raw` owns a ref to inner, but there is no Drop impl
        let raw = rc_to_pointer(&inner);

        let conn = Connection::get().expect("Connection::init was not called");

        let geometry = conn.resolve_geometry(geometry);

        let hwnd = match Self::create_window(config, class_name, name, geometry, raw) {
            Ok(hwnd) => HWindow(hwnd),
            Err(err) => {
                // Ensure that we drop the extra ref to raw before we return
                drop(unsafe { Rc::from_raw(raw) });
                return Err(err);
            }
        };
        let window_handle = Window(hwnd);
        inner
            .borrow_mut()
            .events
            .assign_window(window_handle.clone());

        apply_theme(hwnd.0);
        enable_blur_behind(hwnd.0);

        // Make window capable of accepting drag and drop
        unsafe {
            DragAcceptFiles(hwnd.0, winapi::shared::minwindef::TRUE);
        }

        conn.windows
            .borrow_mut()
            .insert(hwnd.clone(), Rc::clone(&inner));

        Ok(window_handle)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ShowWindowCommand {
    Normal,
    Minimize,
    Maximize,
}

fn schedule_show_window(hwnd: HWindow, show: ShowWindowCommand) {
    // ShowWindow can call to the window proc and may attempt
    // to lock inner, so we avoid locking it ourselves here
    log::trace!("scheduling ShowWindowCommand {show:?}");
    promise::spawn::spawn(async move {
        unsafe {
            log::trace!("applying ShowWindowCommand {show:?}");
            ShowWindow(
                hwnd.0,
                match show {
                    ShowWindowCommand::Normal => SW_NORMAL,
                    ShowWindowCommand::Minimize => SW_MINIMIZE,
                    ShowWindowCommand::Maximize => SW_MAXIMIZE,
                },
            );
        }
    })
    .detach();
}

impl WindowInner {
    fn close(&mut self) {
        let hwnd = self.hwnd;
        promise::spawn::spawn(async move {
            unsafe {
                DestroyWindow(hwnd.0);
            }
        })
        .detach();
    }

    fn set_cursor(&mut self, cursor: Option<MouseCursor>) {
        apply_mouse_cursor(cursor);
    }

    fn set_window_position(&self, coords: ScreenPoint) {
        let hwnd = self.hwnd.0;
        log::trace!("set_window_position wants {coords:?}");
        promise::spawn::spawn(async move {
            log::trace!("set_window_position apply {coords:?}");
            let mut rect = RECT {
                left: 0,
                bottom: 0,
                right: 0,
                top: 0,
            };
            unsafe {
                GetWindowRect(hwnd, &mut rect);

                let origin = client_to_screen(hwnd, Point::new(0, 0));
                let delta_x = origin.x as i32 - rect.left;
                let delta_y = origin.y as i32 - rect.top;

                MoveWindow(
                    hwnd,
                    coords.x as i32 - delta_x,
                    coords.y as i32 - delta_y,
                    rect_width(&rect),
                    rect_height(&rect),
                    1,
                );
            }
        })
        .detach();
    }

    fn set_title(&mut self, title: &str) {
        let title = wide_string(title);
        unsafe {
            SetWindowTextW(self.hwnd.0, title.as_ptr());
        }
    }

    fn set_text_cursor_position(&mut self, cursor: Rect) {
        self.set_ime_window_position(cursor);
    }

    fn set_ime_window_position(&mut self, cursor: Rect) {
        let imc = ImmContext::get(self.hwnd.0);
        match self.config.ime_preedit_rendering {
            ImePreeditRendering::Builtin => imc.set_candidate_window_position(cursor),
            ImePreeditRendering::System => imc.set_composition_window_position(cursor),
        }
    }

    fn config_did_change(&mut self, config: &ConfigHandle) {
        self.config = config.clone();
        self.apply_decoration();
    }

    fn toggle_fullscreen(&mut self) {
        unsafe {
            let hwnd = self.hwnd.0;
            let style = GetWindowLongW(hwnd, GWL_STYLE);
            let config = self.config.clone();
            if let Some(placement) = self.saved_placement.take() {
                promise::spawn::spawn(async move {
                    let style = decorations_to_style(config.window_decorations);
                    SetWindowLongW(hwnd, GWL_STYLE, style as i32);
                    SetWindowPlacement(hwnd, &placement);
                    SetWindowPos(
                        hwnd,
                        std::ptr::null_mut(),
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE
                            | SWP_NOSIZE
                            | SWP_NOZORDER
                            | SWP_NOOWNERZORDER
                            | SWP_FRAMECHANGED,
                    );
                })
                .detach();
            } else {
                let mut placement: WINDOWPLACEMENT = std::mem::zeroed();
                GetWindowPlacement(hwnd, &mut placement);

                self.saved_placement.replace(placement);
                promise::spawn::spawn(async move {
                    let mut mi: MONITORINFO = std::mem::zeroed();
                    mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                    GetMonitorInfoW(MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY), &mut mi);
                    SetWindowLongW(hwnd, GWL_STYLE, style & !(WS_OVERLAPPEDWINDOW as i32));
                    SetWindowPos(
                        hwnd,
                        HWND_TOP,
                        mi.rcMonitor.left,
                        mi.rcMonitor.top,
                        mi.rcMonitor.right - mi.rcMonitor.left,
                        mi.rcMonitor.bottom - mi.rcMonitor.top,
                        SWP_NOOWNERZORDER | SWP_FRAMECHANGED,
                    );
                })
                .detach();
            }
        }
    }
}

impl HasDisplayHandle for Window {
    fn display_handle(&self) -> Result<DisplayHandle, HandleError> {
        unsafe {
            Ok(DisplayHandle::borrow_raw(RawDisplayHandle::Windows(
                WindowsDisplayHandle::new(),
            )))
        }
    }
}

impl HasWindowHandle for Window {
    fn window_handle(&self) -> Result<WindowHandle, HandleError> {
        let conn = Connection::get().expect("raw_window_handle only callable on main thread");
        let handle = conn.get_window(self.0).expect("window handle invalid!?");

        let inner = handle.borrow();
        let handle = inner.window_handle()?;
        unsafe { Ok(WindowHandle::borrow_raw(handle.as_raw())) }
    }
}
