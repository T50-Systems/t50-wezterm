#[derive(Debug, Parser, Clone)]
pub struct PlayCommand {
    /// Explain what is being sent/received
    #[arg(long)]
    explain: bool,

    /// Don't replay, just show the explanation
    #[arg(long, conflicts_with = "explain")]
    explain_only: bool,

    /// Just emit raw escape sequences all at once, with no timing information
    #[arg(long, conflicts_with = "explain")]
    cat: bool,

    cast_file: PathBuf,
}

impl PlayCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut cast_file = BufReader::new(
            std::fs::File::open(&self.cast_file)
                .with_context(|| format!("reading cast file {}", self.cast_file.display()))?,
        );
        let mut header_line = String::new();
        cast_file
            .read_line(&mut header_line)
            .context("reading Header line")?;

        let header: Header = serde_json::from_str(&header_line).context("parsing Header")?;

        if self.cat {
            for line in cast_file.lines() {
                let line = line?;
                let event: Event = serde_json::from_str(&line)?;
                if event.1 != "o" {
                    continue;
                }
                std::io::stdout().write_all(&event.2.as_bytes())?;
            }

            return Ok(());
        }

        let (tx, rx) = channel();
        let mut sent_parser = TWParser::new();
        let mut sent_actions = vec![];

        if self.explain_only {
            for line in cast_file.lines() {
                let line = line?;
                let event: Event = serde_json::from_str(&line)?;
                if event.1 != "o" {
                    continue;
                }
                sent_parser.parse(&event.2.as_bytes(), |act| sent_actions.push(act));
            }
            drop(tx);
        } else {
            let mut tty = Tty::new()?;
            let size = tty.get_size()?;
            if u32::from(size.cols) < header.width || u32::from(size.rows) < header.height {
                anyhow::bail!(
                    "{} was recorded with width={} and height={}
                     but the current screen dimensions {}x{} are
                     too small to display it",
                    self.cast_file.display(),
                    header.width,
                    header.height,
                    size.cols,
                    size.rows
                );
            }

            tty.set_raw()?;

            {
                let mut stdin = tty.reader()?;
                let tx = tx;
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

            let start = Instant::now();

            for line in cast_file.lines() {
                let line = line?;
                let event: Event = serde_json::from_str(&line)?;
                if event.1 != "o" {
                    continue;
                }
                let target = start + Duration::from_secs_f32(event.0);
                let duration = target.saturating_duration_since(Instant::now());
                std::thread::sleep(duration);

                tty.write_all(&event.2.as_bytes())?;
                sent_parser.parse(&event.2.as_bytes(), |act| sent_actions.push(act));
            }

            std::thread::sleep(Duration::from_millis(100));

            tty.set_cooked()?;
        }

        if self.explain || self.explain_only {
            println!("> SENT");
            for s in summarize(sent_actions) {
                println!("\t{:?}", s);
            }
        }

        if !self.explain_only {
            if self.explain {
                println!("< RECV");
            }
            let mut parser = TWParser::new();
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    Message::Stdin(data) => {
                        if self.explain {
                            let answer_back = String::from_utf8_lossy(&data);
                            println!("\t{:?}", answer_back);
                            parser.parse(&data, |action| {
                                println!("\t{:?}", action);
                            });
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }

        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum Summarized {
    Action(Action),
    Print(String),
}

fn summarize(actions: Vec<Action>) -> Vec<Summarized> {
    let mut print = String::new();
    let mut res = vec![];
    for act in actions {
        match act {
            Action::Print(c) => print.push(c),
            act => {
                if !print.is_empty() {
                    res.push(Summarized::Print(print.escape_default().to_string()));
                    print.clear();
                }
                res.push(Summarized::Action(act));
            }
        }
    }
    if !print.is_empty() {
        res.push(Summarized::Print(print.escape_default().to_string()));
    }
    res
}
