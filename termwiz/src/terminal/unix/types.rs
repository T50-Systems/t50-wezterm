use crate::render::RenderTty;
use crate::terminal::ProbeCapabilities;
use crate::{bail, Context, Result};
use filedescriptor::{poll, pollfd, FileDescriptor, POLLIN};
use libc::{self, winsize};
use signal_hook::{self, SigId};
use std::collections::VecDeque;
use std::error::Error as _;
use std::fs::OpenOptions;
use std::io::{stdin, stdout, Error as IoError, ErrorKind, Read, Write};
use std::mem;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use termios::{
    cfmakeraw, tcdrain, tcflush, tcsetattr, Termios, TCIFLUSH, TCIOFLUSH, TCOFLUSH, TCSADRAIN,
    TCSAFLUSH, TCSANOW,
};

use crate::caps::Capabilities;
use crate::escape::csi::{DecPrivateMode, DecPrivateModeCode, Mode, XtermKeyModifierResource, CSI};
use crate::input::{InputEvent, InputParser};
use crate::render::terminfo::TerminfoRenderer;
use crate::surface::Change;
use crate::terminal::{cast, Blocking, ScreenSize, Terminal};

const BUF_SIZE: usize = 4096;

pub enum Purge {
    InputQueue,
    OutputQueue,
    InputAndOutputQueue,
}

pub enum SetAttributeWhen {
    /// changes are applied immediately
    Now,
    /// Apply once the current output queue has drained
    AfterDrainOutputQueue,
    /// Wait for the current output queue to drain, then
    /// discard any unread input
    AfterDrainOutputQueuePurgeInputQueue,
}

pub trait UnixTty {
    fn get_size(&mut self) -> Result<winsize>;
    fn set_size(&mut self, size: winsize) -> Result<()>;
    fn get_termios(&mut self) -> Result<Termios>;
    fn set_termios(&mut self, termios: &Termios, when: SetAttributeWhen) -> Result<()>;
    /// Waits until all written data has been transmitted.
    fn drain(&mut self) -> Result<()>;
    fn purge(&mut self, purge: Purge) -> Result<()>;
}

pub struct TtyReadHandle {
    fd: FileDescriptor,
}

impl TtyReadHandle {
    fn new(fd: FileDescriptor) -> Self {
        Self { fd }
    }

    fn set_blocking(&mut self, blocking: Blocking) -> Result<()> {
        self.fd.set_non_blocking(blocking == Blocking::DoNotWait)?;
        Ok(())
    }
}

impl Read for TtyReadHandle {
    fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, IoError> {
        let size =
            unsafe { libc::read(self.fd.as_raw_fd(), buf.as_mut_ptr() as *mut _, buf.len()) };
        if size == -1 {
            Err(IoError::last_os_error())
        } else {
            Ok(size as usize)
        }
    }
}

pub struct TtyWriteHandle {
    fd: FileDescriptor,
    write_buffer: Vec<u8>,
}

impl TtyWriteHandle {
    fn new(fd: FileDescriptor) -> Self {
        Self {
            fd,
            write_buffer: Vec::with_capacity(BUF_SIZE),
        }
    }

    fn flush_local_buffer(&mut self) -> std::result::Result<(), IoError> {
        if !self.write_buffer.is_empty() {
            self.fd.write_all(&self.write_buffer)?;
            self.write_buffer.clear();
        }
        Ok(())
    }

    fn modify_other_keys(&mut self, level: i64) -> std::io::Result<()> {
        write!(
            self,
            "{}",
            CSI::Mode(Mode::XtermKeyMode {
                resource: XtermKeyModifierResource::OtherKeys,
                value: Some(level),
            })
        )
    }
}

impl Write for TtyWriteHandle {
    fn write(&mut self, buf: &[u8]) -> std::result::Result<usize, IoError> {
        if self.write_buffer.len() + buf.len() > self.write_buffer.capacity() {
            self.flush()?;
        }
        if buf.len() >= self.write_buffer.capacity() {
            self.fd.write(buf)
        } else {
            self.write_buffer.write(buf)
        }
    }

    fn flush(&mut self) -> std::result::Result<(), IoError> {
        self.flush_local_buffer()?;
        self.drain()
            .map_err(|e| IoError::new(ErrorKind::Other, format!("{}", e)))?;
        Ok(())
    }
}

impl RenderTty for TtyWriteHandle {
    fn get_size_in_cells(&mut self) -> Result<(usize, usize)> {
        let size = self.get_size()?;
        Ok((size.ws_col as usize, size.ws_row as usize))
    }
}

impl UnixTty for TtyWriteHandle {
    fn get_size(&mut self) -> Result<winsize> {
        let mut size: winsize = unsafe { mem::zeroed() };
        if unsafe { libc::ioctl(self.fd.as_raw_fd(), libc::TIOCGWINSZ as _, &mut size) } != 0 {
            bail!("failed to ioctl(TIOCGWINSZ): {}", IoError::last_os_error());
        }
        Ok(size)
    }

    fn set_size(&mut self, size: winsize) -> Result<()> {
        if unsafe {
            libc::ioctl(
                self.fd.as_raw_fd(),
                libc::TIOCSWINSZ as _,
                &size as *const _,
            )
        } != 0
        {
            bail!(
                "failed to ioctl(TIOCSWINSZ): {:?}",
                IoError::last_os_error()
            );
        }

        Ok(())
    }

    fn get_termios(&mut self) -> Result<Termios> {
        Termios::from_fd(self.fd.as_raw_fd()).context("get_termios failed")
    }

    fn set_termios(&mut self, termios: &Termios, when: SetAttributeWhen) -> Result<()> {
        let when = match when {
            SetAttributeWhen::Now => TCSANOW,
            SetAttributeWhen::AfterDrainOutputQueue => TCSADRAIN,
            SetAttributeWhen::AfterDrainOutputQueuePurgeInputQueue => TCSAFLUSH,
        };
        tcsetattr(self.fd.as_raw_fd(), when, termios).context("set_termios failed")
    }

    fn drain(&mut self) -> Result<()> {
        tcdrain(self.fd.as_raw_fd()).context("tcdrain failed")
    }

    fn purge(&mut self, purge: Purge) -> Result<()> {
        let param = match purge {
            Purge::InputQueue => TCIFLUSH,
            Purge::OutputQueue => TCOFLUSH,
            Purge::InputAndOutputQueue => TCIOFLUSH,
        };
        tcflush(self.fd.as_raw_fd(), param).context("tcflush failed")
    }
}
