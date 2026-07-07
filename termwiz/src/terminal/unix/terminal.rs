/// A unix style terminal
pub struct UnixTerminal {
    read: TtyReadHandle,
    write: TtyWriteHandle,
    saved_termios: Termios,
    renderer: TerminfoRenderer,
    input_parser: InputParser,
    input_queue: VecDeque<InputEvent>,
    sigwinch_id: SigId,
    sigwinch_pipe: UnixStream,
    wake_pipe: UnixStream,
    wake_pipe_write: Arc<Mutex<UnixStream>>,
    caps: Capabilities,
    in_alternate_screen: bool,
}

impl UnixTerminal {
    /// Attempt to create an instance from the stdin and stdout of the
    /// process.  This will fail unless both are associated with a tty.
    /// Note that this will duplicate the underlying file descriptors
    /// and will no longer participate in the stdin/stdout locking
    /// provided by the rust standard library.
    pub fn new_from_stdio(caps: Capabilities) -> Result<UnixTerminal> {
        Self::new_with(caps, &stdin(), &stdout())
    }

    pub fn new_with<A: AsRawFd, B: AsRawFd>(
        caps: Capabilities,
        read: &A,
        write: &B,
    ) -> Result<UnixTerminal> {
        let mut read = TtyReadHandle::new(FileDescriptor::dup(read)?);
        let mut write = TtyWriteHandle::new(FileDescriptor::dup(write)?);
        let saved_termios = write.get_termios()?;
        let renderer = TerminfoRenderer::new(caps.clone());
        let input_parser = InputParser::new();
        let input_queue = VecDeque::new();

        let (sigwinch_pipe, sigwinch_pipe_write) = UnixStream::pair()?;
        let sigwinch_id =
            signal_hook::low_level::pipe::register(libc::SIGWINCH, sigwinch_pipe_write)?;
        sigwinch_pipe.set_nonblocking(true)?;
        let (wake_pipe, wake_pipe_write) = UnixStream::pair()?;
        wake_pipe.set_nonblocking(true)?;
        wake_pipe_write.set_nonblocking(true)?;

        read.set_blocking(Blocking::Wait)?;

        Ok(UnixTerminal {
            caps,
            read,
            write,
            saved_termios,
            renderer,
            input_parser,
            input_queue,
            sigwinch_pipe,
            sigwinch_id,
            wake_pipe,
            wake_pipe_write: Arc::new(Mutex::new(wake_pipe_write)),
            in_alternate_screen: false,
        })
    }

    /// Attempt to explicitly open a handle to the terminal device
    /// (/dev/tty) and build a `UnixTerminal` from there.  This will
    /// yield a terminal even if the stdio streams have been redirected,
    /// provided that the process has an associated controlling terminal.
    pub fn new(caps: Capabilities) -> Result<UnixTerminal> {
        let file = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
        Self::new_with(caps, &file, &file)
    }

    /// Test whether we caught delivery of SIGWINCH.
    /// If so, yield an `InputEvent` with the current size of the tty.
    fn caught_sigwinch(&mut self) -> Result<Option<InputEvent>> {
        let mut buf = [0u8; 64];

        match self.sigwinch_pipe.read(&mut buf) {
            Ok(0) => Ok(None),
            Ok(_) => {
                let size = self.write.get_size()?;
                Ok(Some(InputEvent::Resized {
                    rows: cast(size.ws_row)?,
                    cols: cast(size.ws_col)?,
                }))
            }
            Err(ref e)
                if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::Interrupted =>
            {
                Ok(None)
            }
            Err(e) => bail!("failed to read sigwinch pipe {}", e),
        }
    }
}

#[derive(Clone)]
pub struct UnixTerminalWaker {
    pipe: Arc<Mutex<UnixStream>>,
}

impl UnixTerminalWaker {
    pub fn wake(&self) -> std::result::Result<(), IoError> {
        let mut pipe = self.pipe.lock().unwrap();
        match pipe.write(b"W") {
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => Ok(()),
                _ => Err(e),
            },
            Ok(_) => Ok(()),
        }
    }
}
