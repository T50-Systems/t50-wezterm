#[derive(Debug)]
enum Message {
    /// Input from the user
    Stdin(Vec<u8>),
    /// Output from the child tty
    Stdout(Vec<u8>),
    /// Child process terminated
    Terminated(portable_pty::ExitStatus),
}

#[derive(Debug, Parser, Clone)]
pub struct RecordCommand {
    /// Start in the specified directory, instead of
    /// the default_cwd defined by your wezterm configuration
    #[arg(long)]
    cwd: Option<std::path::PathBuf>,

    /// Save asciicast to the specified file, instead of
    /// using a random file name in the temp directory
    #[arg(short)]
    outfile: Option<std::path::PathBuf>,

    /// Start prog instead of the default_prog defined by your
    /// wezterm configuration
    #[arg(value_parser)]
    prog: Vec<OsString>,
}

impl RecordCommand {
    pub fn run(&self, config: ConfigHandle) -> anyhow::Result<()> {
        let prog = self.prog.iter().map(|s| s.as_os_str()).collect::<Vec<_>>();

        let mut tty = Tty::new()?;
        let size = tty.get_size()?;

        let header = Header::new(&config, size, &prog);

        let (cast_file, cast_file_name) = match self.outfile.as_ref() {
            Some(outfile) => (
                std::fs::File::options()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(outfile)?,
                outfile.clone(),
            ),
            None => {
                tempfile::Builder::new()
                    .prefix("wezterm-recording-")
                    // We use a .txt suffix for convenice when uploading to GH
                    .suffix(".cast.txt")
                    .tempfile()?
                    .keep()?
            }
        };
        let mut cast_file = BufWriter::new(cast_file);
        writeln!(cast_file, "{}", serde_json::to_string(&header)?)?;

        let pty_system = native_pty_system();
        let pair = pty_system.openpty(size)?;

        let cmd = config.build_prog(
            if self.prog.is_empty() {
                None
            } else {
                Some(prog)
            },
            config.default_prog.as_ref(),
            self.cwd.as_ref().or(config.default_cwd.as_ref()),
        )?;

        let mut child = pair.slave.spawn_command(cmd)?;
        drop(pair.slave);
        let mut child_output = pair.master.try_clone_reader()?;

        tty.set_raw()?;

        let (tx, rx) = channel();

        {
            let tx = tx.clone();
            std::thread::spawn(move || -> anyhow::Result<()> {
                let mut buf = [0u8; 8192];
                loop {
                    let size = child_output.read(&mut buf)?;
                    if size == 0 {
                        break;
                    }
                    tx.send(Message::Stdout(buf[0..size].to_vec()))?;
                }
                Ok(())
            });
        }

        {
            let mut stdin = tty.reader()?;
            let tx = tx.clone();
            std::thread::spawn(move || -> anyhow::Result<()> {
                let mut buf = [0u8; 8192];
                loop {
                    let size = stdin.read(&mut buf)?;
                    if size == 0 {
                        break;
                    }
                    tx.send(Message::Stdin(buf[0..size].to_vec()))?;
                }
                Ok(())
            });
        }

        {
            let tx = tx;
            std::thread::spawn(move || -> anyhow::Result<()> {
                let status = child.wait()?;
                tx.send(Message::Terminated(status))?;
                Ok(())
            });
        }

        let mut child_status = None;
        let first_output = Instant::now();
        let mut buffer = vec![];
        let mut writer = pair.master.take_writer()?;

        for msg in rx {
            match msg {
                Message::Stdin(data) => {
                    writer.write_all(&data)?;
                }
                Message::Stdout(mut data) => {
                    let elapsed = first_output.elapsed().as_secs_f32();
                    tty.write_all(&data)?;

                    // The end of the data may be an incomplete utf8 sequence
                    // that straddles the buffer boundary.  JSON requires strings
                    // to be utf-8 so we need to send the currently-valid portions
                    // through to the .cast file and buffer up the remainder
                    buffer.append(&mut data);
                    match std::str::from_utf8(&buffer) {
                        Ok(valid) => {
                            Event::log_output(&mut cast_file, elapsed, valid)?;
                            buffer.clear();
                        }
                        Err(error) => {
                            let valid_len = error.valid_up_to();
                            Event::log_output(&mut cast_file, elapsed, unsafe {
                                std::str::from_utf8_unchecked(&buffer[0..valid_len])
                            })?;

                            buffer.drain(0..valid_len);

                            if let Some(invalid_sequence_length) = error.error_len() {
                                // Invalid sequence: skip it
                                buffer.drain(0..invalid_sequence_length);
                            }
                        }
                    }
                }
                Message::Terminated(status) => {
                    child_status.replace(status);
                    break;
                }
            }
        }

        tty.set_cooked()?;
        eprintln!("Child status: {:?}", child_status);
        cast_file.flush()?;
        eprintln!("*** Finished recording to {}", cast_file_name.display());

        Ok(())
    }
}
