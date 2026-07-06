struct OutputHandle {
    handle: FileDescriptor,
    write_buffer: Vec<u8>,
}

impl OutputHandle {
    fn new(handle: FileDescriptor) -> Self {
        Self {
            handle,
            write_buffer: Vec::with_capacity(BUF_SIZE),
        }
    }
}

fn dimensions_from_buffer_info(info: CONSOLE_SCREEN_BUFFER_INFO) -> (usize, usize) {
    let cols = 1 + (info.srWindow.Right - info.srWindow.Left);
    let rows = 1 + (info.srWindow.Bottom - info.srWindow.Top);
    (cols as usize, rows as usize)
}

impl RenderTty for OutputHandle {
    fn get_size_in_cells(&mut self) -> Result<(usize, usize)> {
        let info = self.get_buffer_info()?;
        let (cols, rows) = dimensions_from_buffer_info(info);

        Ok((cols, rows))
    }
}

struct EventHandle {
    handle: OwnedHandle,
}

impl EventHandle {
    fn new() -> IoResult<Self> {
        let handle = unsafe { CreateEventW(ptr::null_mut(), 0, 0, ptr::null_mut()) };
        if handle.is_null() {
            Err(IoError::last_os_error())
        } else {
            Ok(Self {
                handle: unsafe { OwnedHandle::from_raw_handle(handle as *mut _) },
            })
        }
    }

    fn set(&self) -> IoResult<()> {
        let ok = unsafe { SetEvent(self.handle.as_raw_handle() as *mut _) };
        if ok == 0 {
            Err(IoError::last_os_error())
        } else {
            Ok(())
        }
    }
}

// Handle created by `CreateEventW` is safe to be shared.
unsafe impl Sync for EventHandle {}

impl Write for OutputHandle {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        if self.write_buffer.len() + buf.len() > self.write_buffer.capacity() {
            self.flush()?;
        }
        if buf.len() >= self.write_buffer.capacity() {
            self.handle.write(buf)
        } else {
            self.write_buffer.write(buf)
        }
    }

    fn flush(&mut self) -> IoResult<()> {
        if !self.write_buffer.is_empty() {
            self.handle.write_all(&self.write_buffer)?;
            self.write_buffer.clear();
        }
        Ok(())
    }
}

impl ConsoleOutputHandle for OutputHandle {
    fn set_output_mode(&mut self, mode: u32) -> Result<()> {
        if unsafe { consoleapi::SetConsoleMode(self.handle.as_raw_handle() as *mut _, mode) } == 0 {
            bail!("SetConsoleMode failed: {}", IoError::last_os_error());
        }
        Ok(())
    }

    fn get_output_mode(&mut self) -> Result<u32> {
        let mut mode = 0;
        if unsafe { consoleapi::GetConsoleMode(self.handle.as_raw_handle() as *mut _, &mut mode) }
            == 0
        {
            bail!("GetConsoleMode failed: {}", IoError::last_os_error());
        }
        Ok(mode)
    }

    fn set_output_cp(&mut self, cp: u32) -> Result<()> {
        if unsafe { SetConsoleOutputCP(cp) } == 0 {
            bail!("SetConsoleOutputCP failed: {}", IoError::last_os_error());
        }
        Ok(())
    }

    fn get_output_cp(&mut self) -> u32 {
        unsafe { consoleapi::GetConsoleOutputCP() }
    }

    fn fill_char(&mut self, text: char, x: i16, y: i16, len: u32) -> Result<u32> {
        let mut wrote = 0;
        if unsafe {
            FillConsoleOutputCharacterW(
                self.handle.as_raw_handle() as *mut _,
                text as u16,
                len,
                COORD { X: x, Y: y },
                &mut wrote,
            )
        } == 0
        {
            bail!(
                "FillConsoleOutputCharacterW failed: {}",
                IoError::last_os_error()
            );
        }
        Ok(wrote)
    }

    fn fill_attr(&mut self, attr: u16, x: i16, y: i16, len: u32) -> Result<u32> {
        let mut wrote = 0;
        if unsafe {
            FillConsoleOutputAttribute(
                self.handle.as_raw_handle() as *mut _,
                attr,
                len,
                COORD { X: x, Y: y },
                &mut wrote,
            )
        } == 0
        {
            bail!(
                "FillConsoleOutputAttribute failed: {}",
                IoError::last_os_error()
            );
        }
        Ok(wrote)
    }

    fn set_attr(&mut self, attr: u16) -> Result<()> {
        if unsafe { SetConsoleTextAttribute(self.handle.as_raw_handle() as *mut _, attr) } == 0 {
            bail!(
                "SetConsoleTextAttribute failed: {}",
                IoError::last_os_error()
            );
        }
        Ok(())
    }

    fn set_cursor_position(&mut self, x: i16, y: i16) -> Result<()> {
        if unsafe {
            SetConsoleCursorPosition(self.handle.as_raw_handle() as *mut _, COORD { X: x, Y: y })
        } == 0
        {
            bail!(
                "SetConsoleCursorPosition(x={}, y={}) failed: {}",
                x,
                y,
                IoError::last_os_error()
            );
        }
        Ok(())
    }

    fn get_buffer_contents(&mut self) -> Result<Vec<CHAR_INFO>> {
        let info = self.get_buffer_info()?;

        let cols = info.dwSize.X as usize;
        let rows = 1 + info.srWindow.Bottom as usize - info.srWindow.Top as usize;

        let mut res = vec![
            CHAR_INFO {
                Attributes: 0,
                Char: unsafe { mem::zeroed() }
            };
            cols * rows
        ];
        let mut read_region = SMALL_RECT {
            Left: 0,
            Right: info.dwSize.X - 1,
            Top: info.srWindow.Top,
            Bottom: info.srWindow.Bottom,
        };
        unsafe {
            if ReadConsoleOutputW(
                self.handle.as_raw_handle() as *mut _,
                res.as_mut_ptr(),
                COORD {
                    X: cols as i16,
                    Y: rows as i16,
                },
                COORD { X: 0, Y: 0 },
                &mut read_region,
            ) == 0
            {
                bail!("ReadConsoleOutputW failed: {}", IoError::last_os_error());
            }
        }
        Ok(res)
    }

    fn set_buffer_contents(&mut self, buffer: &[CHAR_INFO]) -> Result<()> {
        let info = self.get_buffer_info()?;

        let cols = info.dwSize.X as usize;
        let rows = 1 + info.srWindow.Bottom as usize - info.srWindow.Top as usize;
        ensure!(
            rows * cols == buffer.len(),
            "buffer size doesn't match screen size"
        );

        let mut write_region = SMALL_RECT {
            Left: 0,
            Right: info.dwSize.X - 1,
            Top: info.srWindow.Top,
            Bottom: info.srWindow.Bottom,
        };

        unsafe {
            if WriteConsoleOutputW(
                self.handle.as_raw_handle() as *mut _,
                buffer.as_ptr(),
                COORD {
                    X: cols as i16,
                    Y: rows as i16,
                },
                COORD { X: 0, Y: 0 },
                &mut write_region,
            ) == 0
            {
                bail!("WriteConsoleOutputW failed: {}", IoError::last_os_error());
            }
        }
        Ok(())
    }

    fn get_buffer_info(&mut self) -> Result<CONSOLE_SCREEN_BUFFER_INFO> {
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { mem::zeroed() };
        let ok = unsafe {
            GetConsoleScreenBufferInfo(self.handle.as_raw_handle() as *mut _, &mut info as *mut _)
        };
        if ok == 0 {
            bail!(
                "GetConsoleScreenBufferInfo failed: {}",
                IoError::last_os_error()
            );
        }
        Ok(info)
    }

    fn set_viewport(&mut self, left: i16, top: i16, right: i16, bottom: i16) -> Result<()> {
        let rect = SMALL_RECT {
            Left: left,
            Top: top,
            Right: right,
            Bottom: bottom,
        };
        if unsafe { SetConsoleWindowInfo(self.handle.as_raw_handle() as *mut _, 1, &rect) } == 0 {
            bail!("SetConsoleWindowInfo failed: {}", IoError::last_os_error());
        }
        Ok(())
    }

    fn scroll_region(
        &mut self,
        left: i16,
        top: i16,
        right: i16,
        bottom: i16,
        dx: i16,
        dy: i16,
        attr: u16,
    ) -> Result<()> {
        let scroll_rect = SMALL_RECT {
            Left: max(left, left - dx),
            Top: max(top, top - dy),
            Right: min(right, right - dx),
            Bottom: min(bottom, bottom - dy),
        };
        let clip_rect = SMALL_RECT {
            Left: left,
            Top: top,
            Right: right,
            Bottom: bottom,
        };
        let fill = unsafe {
            let mut fill = CHAR_INFO {
                Char: mem::zeroed(),
                Attributes: attr,
            };
            *fill.Char.UnicodeChar_mut() = ' ' as u16;
            fill
        };
        if unsafe {
            ScrollConsoleScreenBufferW(
                self.handle.as_raw_handle() as *mut _,
                &scroll_rect,
                &clip_rect,
                COORD {
                    X: max(left, left + dx),
                    Y: max(left, top + dy),
                },
                &fill,
            )
        } == 0
        {
            bail!(
                "ScrollConsoleScreenBufferW failed: {}",
                IoError::last_os_error()
            );
        }
        Ok(())
    }
}
