impl termwiz::terminal::Terminal for TermWizTerminal {
    fn set_raw_mode(&mut self) -> termwiz::Result<()> {
        use termwiz::escape::csi::{DecPrivateMode, DecPrivateModeCode, Mode, CSI};

        macro_rules! decset {
            ($variant:ident) => {
                write!(
                    self.render_tx,
                    "{}",
                    CSI::Mode(Mode::SetDecPrivateMode(DecPrivateMode::Code(
                        DecPrivateModeCode::$variant
                    )))
                )?;
            };
        }

        decset!(BracketedPaste);
        if self.grab_mouse {
            decset!(AnyEventMouse);
            decset!(SGRMouse);
        }
        self.flush()?;

        Ok(())
    }

    fn set_cooked_mode(&mut self) -> termwiz::Result<()> {
        Ok(())
    }

    fn enter_alternate_screen(&mut self) -> termwiz::Result<()> {
        termwiz::bail!("TermWizTerminalPane has no alt screen");
    }

    fn exit_alternate_screen(&mut self) -> termwiz::Result<()> {
        termwiz::bail!("TermWizTerminalPane has no alt screen");
    }

    fn get_screen_size(&mut self) -> termwiz::Result<ScreenSize> {
        Ok(self.render_tx.screen_size)
    }

    fn set_screen_size(&mut self, _size: ScreenSize) -> termwiz::Result<()> {
        termwiz::bail!("TermWizTerminalPane cannot set screen size");
    }

    fn render(&mut self, changes: &[Change]) -> termwiz::Result<()> {
        self.renderer.render_to(changes, &mut self.render_tx)?;
        Ok(())
    }

    fn flush(&mut self) -> termwiz::Result<()> {
        self.render_tx.render_tx.flush()?;
        Ok(())
    }

    fn poll_input(&mut self, wait: Option<Duration>) -> termwiz::Result<Option<InputEvent>> {
        self.do_input_poll(wait).map(|i| {
            if let Some(InputEvent::Resized { cols, rows }) = i.as_ref() {
                self.render_tx.screen_size.cols = *cols;
                self.render_tx.screen_size.rows = *rows;
            }
            match i {
                // Urgh, we get normalized-to-lowercase CTRL-c,
                // but eg: termwiz and other terminal input expect
                // to get CTRL-C instead.  Adjust for that here.
                Some(InputEvent::Key(KeyEvent {
                    key: KeyCode::Char(c),
                    modifiers: Modifiers::CTRL,
                })) if c.is_ascii_lowercase() => Some(InputEvent::Key(KeyEvent {
                    key: KeyCode::Char(c.to_ascii_uppercase()),
                    modifiers: Modifiers::CTRL,
                })),
                i @ _ => i,
            }
        })
    }

    fn waker(&self) -> TerminalWaker {
        // TODO: TerminalWaker assumes that we're a SystemTerminal but that
        // isn't the case here.
        panic!("TermWizTerminal::waker called!?");
    }
}
