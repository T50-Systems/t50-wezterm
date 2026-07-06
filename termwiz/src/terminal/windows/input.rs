struct InputHandle {
    handle: FileDescriptor,
}

impl Read for InputHandle {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.handle.read(buf)
    }
}

impl ConsoleInputHandle for InputHandle {
    fn set_input_mode(&mut self, mode: u32) -> Result<()> {
        if unsafe { consoleapi::SetConsoleMode(self.handle.as_raw_handle() as *mut _, mode) } == 0 {
            bail!("SetConsoleMode failed: {}", IoError::last_os_error());
        }
        Ok(())
    }

    fn get_input_mode(&mut self) -> Result<u32> {
        let mut mode = 0;
        if unsafe { consoleapi::GetConsoleMode(self.handle.as_raw_handle() as *mut _, &mut mode) }
            == 0
        {
            bail!("GetConsoleMode failed: {}", IoError::last_os_error());
        }
        Ok(mode)
    }

    fn set_input_cp(&mut self, cp: u32) -> Result<()> {
        if unsafe { SetConsoleCP(cp) } == 0 {
            bail!("SetConsoleCP failed: {}", IoError::last_os_error());
        }
        Ok(())
    }

    fn get_input_cp(&mut self) -> u32 {
        unsafe { consoleapi::GetConsoleCP() }
    }

    fn get_number_of_input_events(&mut self) -> Result<usize> {
        let mut num = 0;
        if unsafe {
            consoleapi::GetNumberOfConsoleInputEvents(
                self.handle.as_raw_handle() as *mut _,
                &mut num,
            )
        } == 0
        {
            bail!(
                "GetNumberOfConsoleInputEvents failed: {}",
                IoError::last_os_error()
            );
        }
        Ok(num as usize)
    }

    fn read_console_input(&mut self, num_events: usize) -> Result<Vec<INPUT_RECORD>> {
        let mut res = Vec::with_capacity(num_events);
        let empty_record: INPUT_RECORD = unsafe { mem::zeroed() };
        res.resize(num_events, empty_record);

        let mut num = 0;

        if unsafe {
            consoleapi::ReadConsoleInputW(
                self.handle.as_raw_handle() as *mut _,
                res.as_mut_ptr(),
                num_events as u32,
                &mut num,
            )
        } == 0
        {
            bail!("ReadConsoleInput failed: {}", IoError::last_os_error());
        }

        unsafe { res.set_len(num as usize) };
        Ok(res)
    }
}
