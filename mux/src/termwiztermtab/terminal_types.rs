pub struct TermWizTerminal {
    render_tx: TermWizTerminalRenderTty,
    input_rx: Receiver<InputEvent>,
    renderer: TerminfoRenderer,
    grab_mouse: bool,
}

impl TermWizTerminal {
    pub fn no_grab_mouse_in_raw_mode(&mut self) {
        self.grab_mouse = false;
    }
}

struct TermWizTerminalRenderTty {
    render_tx: BufWriter<FileDescriptor>,
    screen_size: ScreenSize,
}

impl std::io::Write for TermWizTerminalRenderTty {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.render_tx.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.render_tx.flush()
    }
}

impl termwiz::render::RenderTty for TermWizTerminalRenderTty {
    fn get_size_in_cells(&mut self) -> termwiz::Result<(usize, usize)> {
        Ok((self.screen_size.cols, self.screen_size.rows))
    }
}

impl TermWizTerminal {
    fn do_input_poll(&mut self, wait: Option<Duration>) -> termwiz::Result<Option<InputEvent>> {
        if let Some(timeout) = wait {
            match self.input_rx.recv_timeout(timeout) {
                Ok(input) => Ok(Some(input)),
                Err(err) => {
                    if err.is_timeout() {
                        Ok(None)
                    } else {
                        Err(err).context("receive from channel")
                    }
                }
            }
        } else {
            let input = self.input_rx.recv().context("receive from channel")?;
            Ok(Some(input))
        }
    }
}
