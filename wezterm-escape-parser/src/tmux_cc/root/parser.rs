pub struct Parser {
    buffer: Vec<u8>,
    begun: Option<Guarded>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            buffer: vec![],
            begun: None,
        }
    }

    pub fn advance_byte(&mut self, c: u8) -> Result<Option<Event>> {
        if c == b'\n' {
            self.process_line()
        } else {
            self.buffer.push(c);
            Ok(None)
        }
    }

    pub fn advance_string(&mut self, s: &str) -> Result<Vec<Event>> {
        self.advance_bytes(s.as_bytes())
    }

    pub fn advance_bytes(&mut self, bytes: &[u8]) -> Result<Vec<Event>> {
        let mut events = vec![];
        for (i, &b) in bytes.iter().enumerate() {
            match self.advance_byte(b) {
                Ok(option_event) => {
                    if let Some(e) = option_event {
                        events.push(e);
                    }
                }
                Err(err) => {
                    // concat remained bytes after digested bytes
                    bail!("{}{}", err, String::from_utf8_lossy(&bytes[i..]));
                }
            }
        }
        Ok(events)
    }

    fn process_guarded_line(&mut self) -> Result<Option<Event>> {
        let line = std::str::from_utf8(&self.buffer)?;
        let result = match parse_line(&self.buffer) {
            Ok(Event::End {
                timestamp,
                number,
                flags,
            }) => {
                if let Some(begun) = self.begun.take() {
                    if begun.timestamp == timestamp
                        && begun.number == number
                        && begun.flags == flags
                    {
                        Some(Event::Guarded(begun))
                    } else {
                        log::error!("mismatched %end; expected {:?} but got {}", begun, line);
                        None
                    }
                } else {
                    log::error!("unexpected %end with no %begin ({})", line);
                    None
                }
            }
            Ok(Event::Error {
                timestamp,
                number,
                flags,
            }) => {
                if let Some(mut begun) = self.begun.take() {
                    if begun.timestamp == timestamp
                        && begun.number == number
                        && begun.flags == flags
                    {
                        begun.error = true;
                        Some(Event::Guarded(begun))
                    } else {
                        log::error!("mismatched %error; expected {:?} but got {}", begun, line);
                        None
                    }
                } else {
                    log::error!("unexpected %error with no %begin ({})", line);
                    None
                }
            }
            _ => {
                let begun = self
                    .begun
                    .as_mut()
                    .ok_or_else(|| format_err!("missing begun"))?;
                begun.output.push_str(line);
                begun.output.push('\n');
                None
            }
        };
        self.buffer.clear();
        return Ok(result);
    }

    fn process_line(&mut self) -> Result<Option<Event>> {
        if self.buffer.last() == Some(&b'\r') {
            self.buffer.pop();
        }
        if self.begun.is_some() {
            return self.process_guarded_line();
        }

        let result = match parse_line(&self.buffer) {
            Ok(Event::Begin {
                timestamp,
                number,
                flags,
            }) => {
                if self.begun.is_some() {
                    log::error!(
                        "expected %end or %error before %begin ({})",
                        String::from_utf8_lossy(&self.buffer)
                    );
                }
                self.begun.replace(Guarded {
                    timestamp,
                    number,
                    flags,
                    error: false,
                    output: String::new(),
                });
                None
            }
            Ok(event) => Some(event),
            Err(err) => {
                log::error!("Unrecognized tmux cc line: {}", err);
                bail!("{}", String::from_utf8_lossy(&self.buffer));
            }
        };

        self.buffer.clear();
        Ok(result)
    }
}
