#[async_trait(?Send)]
impl WindowOps for Window {
    async fn enable_opengl(&self) -> anyhow::Result<Rc<glium::backend::Context>> {
        let window = self.0;
        promise::spawn::spawn(async move {
            if let Some(handle) = Connection::get().unwrap().get_window(window) {
                let mut inner = handle.borrow_mut();
                inner.enable_opengl()
            } else {
                anyhow::bail!("invalid window");
            }
        })
        .await
    }

    fn notify<T: Any + Send + Sync>(&self, t: T)
    where
        Self: Sized,
    {
        Connection::with_window_inner(self.0, move |inner| {
            inner
                .events
                .dispatch(WindowEvent::Notification(Box::new(t)));
            Ok(())
        });
    }

    fn close(&self) {
        Connection::with_window_inner(self.0, |inner| {
            inner.close();
            Ok(())
        });
    }

    fn show(&self) {
        schedule_show_window(self.0, ShowWindowCommand::Normal);
    }

    fn hide(&self) {
        schedule_show_window(self.0, ShowWindowCommand::Minimize);
    }

    fn focus(&self) {
        let window = self.0;
        let handle = window.0;
        promise::spawn::spawn(async move {
            // In some situation, calling SetForegroundWindow could not bring up the window,
            // This is a little hack which can "steal" the foreground window permission
            // We only call this function in the window creation, so it should be fine.
            // See : https://stackoverflow.com/questions/10740346/setforegroundwindow-only-working-while-visual-studio-is-open
            unsafe {
                let alt_sc = MapVirtualKeyW(VK_MENU as u32, MAPVK_VK_TO_VSC);

                let mut inputs: [INPUT; 2] = [
                    INPUT {
                        type_: INPUT_KEYBOARD,
                        u: Default::default(),
                    },
                    INPUT {
                        type_: INPUT_KEYBOARD,
                        u: Default::default(),
                    },
                ];
                *inputs[0].u.ki_mut() = KEYBDINPUT {
                    wVk: VK_LMENU as u16,
                    wScan: alt_sc as u16,
                    dwFlags: KEYEVENTF_EXTENDEDKEY,
                    dwExtraInfo: 0,
                    time: 0,
                };
                *inputs[1].u.ki_mut() = KEYBDINPUT {
                    wVk: VK_LMENU as u16,
                    wScan: alt_sc as u16,
                    dwFlags: KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP,
                    dwExtraInfo: 0,
                    time: 0,
                };

                // Simulate a key press and release
                SendInput(
                    inputs.len() as u32,
                    inputs.as_mut_ptr(),
                    std::mem::size_of::<INPUT>() as i32,
                );

                SetForegroundWindow(handle);
            }
        })
        .detach();
    }

    fn maximize(&self) {
        schedule_show_window(self.0, ShowWindowCommand::Maximize);
    }

    fn restore(&self) {
        schedule_show_window(self.0, ShowWindowCommand::Normal);
    }

    fn set_cursor(&self, cursor: Option<MouseCursor>) {
        Connection::with_window_inner(self.0, move |inner| {
            inner.set_cursor(cursor);
            Ok(())
        });
    }

    fn invalidate(&self) {
        let hwnd = self.0 .0;
        log::trace!("WindowOps::invalidate calling InvalidateRect");
        unsafe {
            InvalidateRect(hwnd, null(), 0);
        }
    }

    fn set_title(&self, title: &str) {
        let title = title.to_owned();
        Connection::with_window_inner(self.0, move |inner| {
            inner.set_title(&title);
            Ok(())
        });
    }

    fn toggle_fullscreen(&self) {
        Connection::with_window_inner(self.0, move |inner| {
            inner.toggle_fullscreen();
            Ok(())
        });
    }

    fn config_did_change(&self, config: &ConfigHandle) {
        let config = config.clone();
        Connection::with_window_inner(self.0, move |inner| {
            inner.config_did_change(&config);
            Ok(())
        });
    }

    fn set_text_cursor_position(&self, cursor: Rect) {
        Connection::with_window_inner(self.0, move |inner| {
            inner.set_text_cursor_position(cursor);
            Ok(())
        });
    }

    fn set_inner_size(&self, width: usize, height: usize) {
        Connection::with_window_inner(self.0, move |inner| {
            let hwnd = inner.hwnd;
            let decorations = inner.config.window_decorations;
            promise::spawn::spawn(async move {
                log::trace!("set_inner_size called with {width}x{height}");
                let frame_dpi = unsafe { GetDpiForWindow(hwnd.0) };
                let (width, height) = adjust_client_to_window_dimensions(
                    decorations_to_style(decorations),
                    width,
                    height,
                    frame_dpi,
                );
                let window_state = get_window_state(hwnd.0);
                if window_state.can_resize() {
                    log::trace!("set_inner_size now calling SetWindowPos with {width}x{height}");
                    unsafe {
                        SetWindowPos(
                            hwnd.0,
                            hwnd.0,
                            0,
                            0,
                            width,
                            height,
                            SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOZORDER,
                        );
                        wm_paint(hwnd.0, 0, 0, 0);
                        if let Some(inner) = rc_from_hwnd(hwnd.0) {
                            let mut inner = inner.borrow_mut();
                            inner.events.dispatch(WindowEvent::SetInnerSizeCompleted);
                        }
                    }
                } else {
                    log::trace!(
                        "ignoring set_inner_size({width}, {height}) call \
                                because window_state is {window_state:?}"
                    );
                }
            })
            .detach();
            Ok(())
        });
    }

    fn set_maximize_button_position(&self, coords: ScreenRect) {
        Connection::with_window_inner(self.0, move |inner| {
            inner.maximize_button_position = Some(coords);
            Ok(())
        });
    }

    fn set_window_position(&self, coords: ScreenPoint) {
        Connection::with_window_inner(self.0, move |inner| {
            inner.set_window_position(coords);
            Ok(())
        });
    }

    fn get_clipboard(&self, _clipboard: Clipboard) -> Future<String> {
        Future::result(
            clipboard_win::get_clipboard_string()
                .map(|s| s.replace("\r\n", "\n"))
                .context("Error getting clipboard"),
        )
    }

    fn set_clipboard(&self, _clipboard: Clipboard, text: String) {
        clipboard_win::set_clipboard_string(&text).ok();
    }

    fn set_window_drag_position(&self, coords: ScreenPoint) {
        Connection::with_window_inner(self.0, move |inner| {
            inner.window_drag_position = Some(coords);

            Ok(())
        });
    }

    fn get_os_parameters(
        &self,
        config: &ConfigHandle,
        window_state: WindowState,
    ) -> anyhow::Result<Option<Parameters>> {
        let hwnd = self.0 .0;
        anyhow::ensure!(!hwnd.is_null(), "HWND is null");

        let has_focus = unsafe { GetFocus() } == hwnd;
        let is_full_screen = window_state.contains(WindowState::FULL_SCREEN);

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let use_accent = hkcu
            .open_subkey("SOFTWARE\\Microsoft\\Windows\\DWM")?
            .get_value::<u32, _>("ColorPrevalence")?;
        let settings = UISettings::new()?;
        let top_border_color = if has_focus {
            if use_accent == 1 {
                wuicolor_to_linearrgba(settings.GetColorValue(UIColorType::Accent)?)
            } else {
                if *IS_WIN10 {
                    LinearRgba(0.01, 0.01, 0.01, 0.67)
                } else {
                    LinearRgba(0.026, 0.026, 0.026, 0.5)
                }
            }
        } else {
            if *IS_WIN10 {
                LinearRgba(0.024, 0.024, 0.024, 0.5)
            } else {
                LinearRgba(0.028, 0.028, 0.028, 0.5)
            }
        };

        const BASE_BORDER: ULength = ULength::new(0);
        let is_resize = config.window_decorations == WindowDecorations::RESIZE;

        let title_font = {
            let font = TITLE_FONT.lock().expect("locking title_font");
            (*font).clone()
        };

        Ok(Some(Parameters {
            title_bar: parameters::TitleBar {
                padding_left: ULength::new(0),
                padding_right: ULength::new(0),
                height: None,
                font_and_size: title_font,
            },
            border_dimensions: Some(parameters::Border {
                top: if is_resize && !*IS_WIN10 && !is_full_screen {
                    BASE_BORDER + ULength::new(1)
                } else {
                    BASE_BORDER
                },
                left: BASE_BORDER,
                bottom: if is_resize && *IS_WIN10 && !is_full_screen {
                    BASE_BORDER + ULength::new(2)
                } else {
                    BASE_BORDER
                },
                right: BASE_BORDER,
                color: top_border_color,
            }),
        }))
    }
}
