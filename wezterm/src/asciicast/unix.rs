#[cfg(unix)]
mod unix {
    use super::*;
    use std::os::unix::io::AsRawFd;
    use termios::{cfmakeraw, tcsetattr, Termios, TCSAFLUSH};

    pub struct UnixTty {
        tty: FileDescriptor,
        termios: Termios,
    }

    fn get_termios(fd: &FileDescriptor) -> anyhow::Result<Termios> {
        Termios::from_fd(fd.as_raw_fd()).context("get_termios failed")
    }

    fn set_termios(
        fd: &FileDescriptor,
        termios: &Termios,
        mode: libc::c_int,
    ) -> anyhow::Result<()> {
        tcsetattr(fd.as_raw_fd(), mode, termios).context("set_termios failed")
    }

    impl UnixTty {
        pub fn new() -> anyhow::Result<Self> {
            let tty = FileDescriptor::new(
                std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open("/dev/tty")?,
            );
            let termios = get_termios(&tty)?;

            Ok(Self { tty, termios })
        }

        pub fn set_raw(&mut self) -> anyhow::Result<()> {
            let mut termios = get_termios(&self.tty)?;
            cfmakeraw(&mut termios);
            set_termios(&self.tty, &termios, TCSAFLUSH)
        }

        pub fn set_cooked(&mut self) -> anyhow::Result<()> {
            set_termios(&self.tty, &self.termios, TCSAFLUSH)
        }

        pub fn get_size(&self) -> anyhow::Result<PtySize> {
            let mut size = std::mem::MaybeUninit::<libc::winsize>::uninit();
            if unsafe { libc::ioctl(self.tty.as_raw_fd(), libc::TIOCGWINSZ as _, &mut size) } != 0 {
                anyhow::bail!(
                    "failed to ioctl(TIOCGWINSZ): {:#}",
                    std::io::Error::last_os_error()
                );
            }

            let size = unsafe { size.assume_init() };

            Ok(PtySize {
                rows: size.ws_row.into(),
                cols: size.ws_col.into(),
                pixel_width: size.ws_xpixel.into(),
                pixel_height: size.ws_ypixel.into(),
            })
        }

        pub fn reader(&self) -> anyhow::Result<FileDescriptor> {
            Ok(self.tty.try_clone()?)
        }

        pub fn write_all(&mut self, data: &[u8]) -> anyhow::Result<()> {
            Ok(self.tty.write_all(data)?)
        }
    }

    impl Drop for UnixTty {
        fn drop(&mut self) {
            let _ = self.set_cooked();
        }
    }
}
