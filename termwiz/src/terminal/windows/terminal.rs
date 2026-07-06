pub struct WindowsTerminal {
    input_handle: InputHandle,
    output_handle: OutputHandle,
    waker_handle: Arc<EventHandle>,
    saved_input_mode: u32,
    saved_output_mode: u32,
    renderer: Renderer,
    input_parser: InputParser,
    input_queue: VecDeque<InputEvent>,
    saved_input_cp: u32,
    saved_output_cp: u32,
    in_alternate_screen: bool,
    caps: Capabilities,
}

impl Drop for WindowsTerminal {
    fn drop(&mut self) {
        if matches!(&self.renderer, Renderer::Terminfo(_)) {
            macro_rules! decreset {
                ($variant:ident) => {
                    write!(
                        self.output_handle,
                        "{}",
                        CSI::Mode(Mode::ResetDecPrivateMode(DecPrivateMode::Code(
                            DecPrivateModeCode::$variant
                        )))
                    )
                    .unwrap();
                };
            }
            self.render(&[Change::CursorVisibility(
                crate::surface::CursorVisibility::Visible,
            )])
            .ok();
            decreset!(BracketedPaste);
            decreset!(SGRMouse);
            decreset!(AnyEventMouse);
        }

        self.exit_alternate_screen().unwrap();
        self.output_handle.flush().unwrap();
        self.input_handle
            .set_input_mode(self.saved_input_mode)
            .expect("failed to restore console input mode");
        self.input_handle
            .set_input_cp(self.saved_input_cp)
            .expect("failed to restore console input codepage");
        self.output_handle
            .set_output_mode(self.saved_output_mode)
            .expect("failed to restore console output mode");
        self.output_handle
            .set_output_cp(self.saved_output_cp)
            .expect("failed to restore console output codepage");
    }
}

impl WindowsTerminal {
    /// Attempt to create an instance from the stdin and stdout of the
    /// process.  This will fail unless both are associated with a tty.
    /// Note that this will duplicate the underlying file descriptors
    /// and will no longer participate in the stdin/stdout locking
    /// provided by the rust standard library.
    pub fn new_from_stdio(caps: Capabilities) -> Result<Self> {
        Self::new_with(caps, stdin(), stdout())
    }

    /// Create an instance using the provided capabilities, read and write
    /// handles. The read and write handles must be tty handles of this
    /// will return an error.
    pub fn new_with<A: Read + IsTty + AsRawHandle, B: Write + IsTty + AsRawHandle>(
        caps: Capabilities,
        read: A,
        write: B,
    ) -> Result<Self> {
        if !read.is_tty() || !write.is_tty() {
            bail!("stdin and stdout must both be tty handles");
        }

        let mut input_handle = InputHandle {
            handle: FileDescriptor::dup(&read)?,
        };
        let mut output_handle = OutputHandle::new(FileDescriptor::dup(&write)?);
        let waker_handle = Arc::new(EventHandle::new()?);

        let saved_input_mode = input_handle.get_input_mode()?;
        let saved_output_mode = output_handle.get_output_mode()?;
        let saved_input_cp = input_handle.get_input_cp();
        let saved_output_cp = output_handle.get_output_cp();

        // Test whether we have a virtual terminal capable
        // console device by attempting to set the appropriate flags.
        let virtual_terminal_available = output_handle
            .set_output_mode(
                saved_output_mode
                    | ENABLE_VIRTUAL_TERMINAL_PROCESSING
                    | DISABLE_NEWLINE_AUTO_RETURN,
            )
            .is_ok();

        // Allow opting out of that processing
        fn bypass_virtual_terminal() -> bool {
            if let Ok(t) = std::env::var("TERMWIZ_BYPASS_VIRTUAL_TERMINAL") {
                t == "1"
            } else {
                false
            }
        }

        let renderer = if caps.terminfo_db().is_some() {
            Renderer::Terminfo(TerminfoRenderer::new(caps.clone()))
        } else if virtual_terminal_available && !bypass_virtual_terminal() {
            Renderer::Terminfo(TerminfoRenderer::new(caps.clone().apply_builtin_terminfo()))
        } else {
            Renderer::Windows(WindowsConsoleRenderer::new(caps.clone()))
        };
        let input_parser = InputParser::new();

        let mut terminal = Self {
            input_handle,
            output_handle,
            waker_handle,
            saved_input_mode,
            saved_output_mode,
            saved_input_cp,
            saved_output_cp,
            renderer,
            input_parser,
            input_queue: VecDeque::new(),
            in_alternate_screen: false,
            caps,
        };

        terminal.input_handle.set_input_cp(CP_UTF8)?;
        terminal.output_handle.set_output_cp(CP_UTF8)?;

        // We already enabled this for output, but let's also turn it
        // on for input here now.
        terminal.enable_virtual_terminal_processing_if_needed()?;

        Ok(terminal)
    }

    fn enable_virtual_terminal_processing_if_needed(&mut self) -> Result<()> {
        match &self.renderer {
            Renderer::Terminfo(_) => self.enable_virtual_terminal_processing(),
            Renderer::Windows(_) => Ok(()),
        }
    }

    /// Attempt to explicitly open handles to a console device (CONIN$,
    /// CONOUT$). This should yield the terminal already associated with
    /// the process, even if stdio streams have been redirected.
    pub fn new(caps: Capabilities) -> Result<Self> {
        let read = OpenOptions::new().read(true).write(true).open("CONIN$")?;
        let write = OpenOptions::new().read(true).write(true).open("CONOUT$")?;
        Self::new_with(caps, read, write)
    }

    pub fn enable_virtual_terminal_processing(&mut self) -> Result<()> {
        let mode = self.output_handle.get_output_mode()?;
        self.output_handle.set_output_mode(
            mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING | DISABLE_NEWLINE_AUTO_RETURN,
        )?;

        let mode = self.input_handle.get_input_mode()?;
        self.input_handle
            .set_input_mode(mode | ENABLE_VIRTUAL_TERMINAL_INPUT)?;
        Ok(())
    }
}
