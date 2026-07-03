#[cfg(all(test, unix))]
mod test {
    use super::*;
    use crate::bail;
    use crate::caps::ProbeHints;
    use crate::color::{AnsiColor, ColorAttribute};
    use crate::escape::parser::Parser;
    use crate::escape::{Action, Esc, EscCode};
    use crate::input::InputEvent;
    use crate::terminal::unix::{Purge, SetAttributeWhen, UnixTty};
    use crate::terminal::{cast, ScreenSize, Terminal, TerminalWaker};
    use libc::winsize;
    use std::io::{Error as IoError, ErrorKind, Read, Result as IoResult, Write};
    use std::mem;
    use std::time::Duration;
    use terminfo;
    use termios::Termios;

    /// Return Capabilities loaded from the included xterm terminfo data
    fn xterm_terminfo() -> Capabilities {
        xterm_terminfo_with_hints(ProbeHints::default())
    }

    fn xterm_terminfo_with_hints(hints: ProbeHints) -> Capabilities {
        // Load our own compiled data so that the tests have an
        // environment that doesn't vary machine by machine.
        let data = include_bytes!("../../data/xterm-256color");
        Capabilities::new_with_hints(hints.terminfo_db(Some(
            terminfo::Database::from_buffer(data.as_ref()).unwrap(),
        )))
        .unwrap()
    }

    fn no_terminfo_all_enabled() -> Capabilities {
        Capabilities::new_with_hints(ProbeHints::default().color_level(Some(ColorLevel::TrueColor)))
            .unwrap()
    }

    struct FakeTty {
        buf: Vec<u8>,
        size: winsize,
        termios: Termios,
    }

    impl FakeTty {
        fn new_with_size(width: usize, height: usize) -> Self {
            let size = winsize {
                ws_col: cast(width).unwrap(),
                ws_row: cast(height).unwrap(),
                ws_xpixel: 0,
                ws_ypixel: 0,
            };
            let buf = Vec::new();
            Self {
                size,
                buf,
                termios: unsafe { mem::zeroed() },
            }
        }
    }
    impl RenderTty for FakeTty {
        fn get_size_in_cells(&mut self) -> Result<(usize, usize)> {
            Ok((self.size.ws_col as usize, self.size.ws_row as usize))
        }
    }

    impl UnixTty for FakeTty {
        fn get_size(&mut self) -> Result<winsize> {
            Ok(self.size.clone())
        }
        fn set_size(&mut self, size: winsize) -> Result<()> {
            self.size = size.clone();
            Ok(())
        }
        fn get_termios(&mut self) -> Result<Termios> {
            Ok(self.termios.clone())
        }
        fn set_termios(&mut self, termios: &Termios, _when: SetAttributeWhen) -> Result<()> {
            self.termios = termios.clone();
            Ok(())
        }
        /// Waits until all written data has been transmitted.
        fn drain(&mut self) -> Result<()> {
            Ok(())
        }
        fn purge(&mut self, _purge: Purge) -> Result<()> {
            Ok(())
        }
    }

    impl Read for FakeTty {
        fn read(&mut self, _buf: &mut [u8]) -> std::result::Result<usize, IoError> {
            Err(IoError::new(ErrorKind::Other, "not implemented"))
        }
    }
    impl Write for FakeTty {
        fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
            self.buf.write(buf)
        }

        fn flush(&mut self) -> IoResult<()> {
            self.buf.flush()
        }
    }

    struct FakeTerm {
        write: FakeTty,
        renderer: TerminfoRenderer,
    }

    impl FakeTerm {
        fn new(caps: Capabilities) -> Self {
            Self::new_with_size(caps, 80, 24)
        }

        fn new_with_size(caps: Capabilities, width: usize, height: usize) -> Self {
            let write = FakeTty::new_with_size(width, height);
            let renderer = TerminfoRenderer::new(caps);
            Self { write, renderer }
        }

        fn parse(&self) -> Vec<Action> {
            let mut p = Parser::new();
            p.parse_as_vec(&self.write.buf)
        }
    }

    impl Terminal for FakeTerm {
        fn set_raw_mode(&mut self) -> Result<()> {
            bail!("not implemented");
        }

        fn set_cooked_mode(&mut self) -> Result<()> {
            bail!("not implemented");
        }

        fn enter_alternate_screen(&mut self) -> Result<()> {
            bail!("not implemented");
        }

        fn exit_alternate_screen(&mut self) -> Result<()> {
            bail!("not implemented");
        }

        fn render(&mut self, changes: &[Change]) -> Result<()> {
            self.renderer.render_to(changes, &mut self.write)
        }

        fn get_screen_size(&mut self) -> Result<ScreenSize> {
            let size = self.write.get_size()?;
            Ok(ScreenSize {
                rows: cast(size.ws_row)?,
                cols: cast(size.ws_col)?,
                xpixel: cast(size.ws_xpixel)?,
                ypixel: cast(size.ws_ypixel)?,
            })
        }

        fn set_screen_size(&mut self, size: ScreenSize) -> Result<()> {
            let size = winsize {
                ws_row: cast(size.rows)?,
                ws_col: cast(size.cols)?,
                ws_xpixel: cast(size.xpixel)?,
                ws_ypixel: cast(size.ypixel)?,
            };

            self.write.set_size(size)
        }

        fn flush(&mut self) -> Result<()> {
            Ok(())
        }

        fn poll_input(&mut self, _wait: Option<Duration>) -> Result<Option<InputEvent>> {
            bail!("not implemented");
        }

        fn waker(&self) -> TerminalWaker {
            unimplemented!();
        }
    }

    #[test]

    include!("tests_01.rs");
    include!("tests_02.rs");
}
