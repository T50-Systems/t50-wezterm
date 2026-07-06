#[cfg(windows)]
mod win {
    use super::*;
    use filedescriptor::AsRawFileDescriptor;
    use std::fs::OpenOptions;
    use std::os::windows::io::AsRawHandle;
    use winapi::um::consoleapi::*;
    use winapi::um::wincon::*;
    use winapi::um::winnls::CP_UTF8;

    pub struct WinTty {
        saved_input: u32,
        saved_output: u32,
        saved_cp: u32,
        read: FileDescriptor,
        write: FileDescriptor,
    }

    impl WinTty {
        pub fn new() -> anyhow::Result<Self> {
            let read =
                FileDescriptor::new(OpenOptions::new().read(true).write(true).open("CONIN$")?);
            let write =
                FileDescriptor::new(OpenOptions::new().read(true).write(true).open("CONOUT$")?);

            let mut saved_input = 0;
            let mut saved_output = 0;
            let saved_cp;
            unsafe {
                GetConsoleMode(read.as_raw_file_descriptor() as *mut _, &mut saved_input);
                GetConsoleMode(write.as_raw_file_descriptor() as *mut _, &mut saved_output);
                saved_cp = GetConsoleOutputCP();
                SetConsoleOutputCP(CP_UTF8);
            }

            Ok(Self {
                saved_input,
                saved_output,
                saved_cp,
                read,
                write,
            })
        }

        pub fn set_cooked(&mut self) -> anyhow::Result<()> {
            unsafe {
                SetConsoleOutputCP(self.saved_cp);
                SetConsoleMode(self.read.as_raw_handle() as *mut _, self.saved_input);
                SetConsoleMode(self.write.as_raw_handle() as *mut _, self.saved_output);
            }
            Ok(())
        }

        pub fn set_raw(&mut self) -> anyhow::Result<()> {
            unsafe {
                SetConsoleMode(
                    self.read.as_raw_file_descriptor() as *mut _,
                    ENABLE_VIRTUAL_TERMINAL_INPUT,
                );
                SetConsoleMode(
                    self.write.as_raw_file_descriptor() as *mut _,
                    ENABLE_PROCESSED_OUTPUT
                        | ENABLE_WRAP_AT_EOL_OUTPUT
                        | ENABLE_VIRTUAL_TERMINAL_PROCESSING
                        | DISABLE_NEWLINE_AUTO_RETURN,
                );
            }
            Ok(())
        }

        pub fn get_size(&self) -> anyhow::Result<PtySize> {
            let mut info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { std::mem::zeroed() };
            let ok = unsafe {
                GetConsoleScreenBufferInfo(
                    self.write.as_raw_handle() as *mut _,
                    &mut info as *mut _,
                )
            };
            if ok == 0 {
                anyhow::bail!(
                    "GetConsoleScreenBufferInfo failed: {}",
                    std::io::Error::last_os_error()
                );
            }

            let cols = 1 + (info.srWindow.Right - info.srWindow.Left);
            let rows = 1 + (info.srWindow.Bottom - info.srWindow.Top);

            Ok(PtySize {
                rows: rows as u16,
                cols: cols as u16,
                pixel_width: 0,
                pixel_height: 0,
            })
        }

        pub fn reader(&self) -> anyhow::Result<FileDescriptor> {
            Ok(self.read.try_clone()?)
        }

        pub fn write_all(&mut self, data: &[u8]) -> anyhow::Result<()> {
            Ok(self.write.write_all(data)?)
        }
    }

    impl Drop for WinTty {
        fn drop(&mut self) {
            let _ = self.set_cooked();
        }
    }
}
