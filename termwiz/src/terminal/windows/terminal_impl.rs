#[derive(Clone)]
pub struct WindowsTerminalWaker {
    handle: Arc<EventHandle>,
}

impl WindowsTerminalWaker {
    pub fn wake(&self) -> IoResult<()> {
        self.handle.set()?;
        Ok(())
    }
}

impl Terminal for WindowsTerminal {
    fn set_raw_mode(&mut self) -> Result<()> {
        let mode = self.output_handle.get_output_mode()?;
        self.output_handle
            .set_output_mode(mode | DISABLE_NEWLINE_AUTO_RETURN)
            .ok();

        let mode = self.input_handle.get_input_mode()?;

        self.input_handle.set_input_mode(
            (mode & !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT))
                | ENABLE_MOUSE_INPUT
                | ENABLE_WINDOW_INPUT,
        )?;

        if matches!(&self.renderer, Renderer::Terminfo(_)) {
            macro_rules! decset {
                ($variant:ident) => {
                    write!(
                        self.output_handle,
                        "{}",
                        CSI::Mode(Mode::SetDecPrivateMode(DecPrivateMode::Code(
                            DecPrivateModeCode::$variant
                        )))
                    )?;
                };
            }

            if self.caps.bracketed_paste() {
                decset!(BracketedPaste);
            }
            if self.caps.mouse_reporting() {
                decset!(AnyEventMouse);
                decset!(SGRMouse);
            }
            self.output_handle.flush()?;
        }

        Ok(())
    }

    fn set_cooked_mode(&mut self) -> Result<()> {
        let mode = self.output_handle.get_output_mode()?;
        self.output_handle
            .set_output_mode(mode & !DISABLE_NEWLINE_AUTO_RETURN)
            .ok();

        let mode = self.input_handle.get_input_mode()?;

        self.input_handle.set_input_mode(
            (mode & !(ENABLE_MOUSE_INPUT | ENABLE_WINDOW_INPUT))
                | ENABLE_ECHO_INPUT
                | ENABLE_LINE_INPUT
                | ENABLE_PROCESSED_INPUT,
        )
    }

    fn enter_alternate_screen(&mut self) -> Result<()> {
        if matches!(&self.renderer, Renderer::Terminfo(_)) {
            if !self.in_alternate_screen {
                write!(
                    self.output_handle,
                    "{}",
                    CSI::Mode(Mode::SetDecPrivateMode(DecPrivateMode::Code(
                        DecPrivateModeCode::ClearAndEnableAlternateScreen
                    )))
                )?;
                self.in_alternate_screen = true;
            }
        } else {
            // TODO: Implement using CreateConsoleScreenBuffer and
            // SetConsoleActiveScreenBuffer.
        }
        Ok(())
    }

    fn exit_alternate_screen(&mut self) -> Result<()> {
        // TODO: Implement using SetConsoleActiveScreenBuffer.
        if matches!(&self.renderer, Renderer::Terminfo(_)) {
            if self.in_alternate_screen {
                write!(
                    self.output_handle,
                    "{}",
                    CSI::Mode(Mode::ResetDecPrivateMode(DecPrivateMode::Code(
                        DecPrivateModeCode::ClearAndEnableAlternateScreen
                    )))
                )?;
                self.in_alternate_screen = false;
            }
        } else {
            // TODO: Implement using CreateConsoleScreenBuffer and
            // SetConsoleActiveScreenBuffer.
        }
        Ok(())
    }

    fn get_screen_size(&mut self) -> Result<ScreenSize> {
        let info = self.output_handle.get_buffer_info()?;
        let (cols, rows) = dimensions_from_buffer_info(info);

        Ok(ScreenSize {
            rows: cast(rows)?,
            cols: cast(cols)?,
            xpixel: 0,
            ypixel: 0,
        })
    }

    fn probe_capabilities(&mut self) -> Option<ProbeCapabilities> {
        Some(ProbeCapabilities::new(
            &mut self.input_handle,
            &mut self.output_handle,
        ))
    }

    fn set_screen_size(&mut self, size: ScreenSize) -> Result<()> {
        // FIXME: take into account the visible window size here;
        // this probably changes the size of everything including scrollback
        let size = COORD {
            X: cast(size.cols)?,
            Y: cast(size.rows)?,
        };
        let handle = self.output_handle.handle.as_raw_handle();
        if unsafe { SetConsoleScreenBufferSize(handle as *mut _, size) } != 1 {
            bail!(
                "failed to SetConsoleScreenBufferSize: {}",
                IoError::last_os_error()
            );
        }
        Ok(())
    }

    fn render(&mut self, changes: &[Change]) -> Result<()> {
        match &mut self.renderer {
            Renderer::Terminfo(r) => r.render_to(changes, &mut self.output_handle),
            Renderer::Windows(r) => r.render_to(changes, &mut self.output_handle),
        }
    }

    fn flush(&mut self) -> Result<()> {
        self.output_handle
            .flush()
            .map_err(|e| format_err!("flush failed: {}", e))
    }

    fn poll_input(&mut self, wait: Option<Duration>) -> Result<Option<InputEvent>> {
        loop {
            if let Some(event) = self.input_queue.pop_front() {
                return Ok(Some(event));
            }

            let mut pending = self.input_handle.get_number_of_input_events()?;

            if pending == 0 {
                let mut handles = [
                    self.input_handle.handle.as_raw_handle() as *mut _,
                    self.waker_handle.handle.as_raw_handle() as *mut _,
                ];
                let result = unsafe {
                    WaitForMultipleObjects(
                        2,
                        handles.as_mut_ptr(),
                        0,
                        wait.map(|wait| wait.as_millis() as u32).unwrap_or(INFINITE),
                    )
                };
                if result == WAIT_OBJECT_0 + 0 {
                    pending = self.input_handle.get_number_of_input_events()?;
                } else if result == WAIT_OBJECT_0 + 1 {
                    return Ok(Some(InputEvent::Wake));
                } else if result == WAIT_FAILED {
                    bail!(
                        "failed to WaitForMultipleObjects: {}",
                        IoError::last_os_error()
                    );
                } else if result == WAIT_TIMEOUT {
                    return Ok(None);
                } else {
                    return Ok(None);
                }
            }

            let records = self.input_handle.read_console_input(pending)?;

            let input_queue = &mut self.input_queue;
            self.input_parser
                .decode_input_records(&records, &mut |evt| input_queue.push_back(evt));
        }
    }

    fn waker(&self) -> WindowsTerminalWaker {
        WindowsTerminalWaker {
            handle: self.waker_handle.clone(),
        }
    }
}
