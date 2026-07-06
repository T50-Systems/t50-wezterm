/// Decode OpenBSD `vis` encoded strings
/// See: https://github.com/tmux/tmux/blob/486ce9b09855ae30a2bf5e576cb6f7ad37792699/compat/unvis.c
fn unvis_bytes(s: &[u8]) -> Result<Vec<u8>> {
    enum State {
        Ground,
        Start,
        Meta,
        Meta1,
        Ctrl(u8),
        Octal2(u8),
        Octal3(u8),
    }

    let mut state = State::Ground;
    let mut result: Vec<u8> = vec![];
    let mut bytes = s.iter();

    fn is_octal(b: u8) -> bool {
        b >= b'0' && b <= b'7'
    }

    fn unvis_byte(b: u8, state: &mut State, result: &mut Vec<u8>) -> Result<bool> {
        match state {
            State::Ground => {
                if b == b'\\' {
                    *state = State::Start;
                } else {
                    result.push(b);
                }
            }

            State::Start => {
                match b {
                    b'\\' => {
                        result.push(b'\\');
                        *state = State::Ground;
                    }
                    b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' => {
                        let value = b - b'0';
                        *state = State::Octal2(value);
                    }
                    b'M' => {
                        *state = State::Meta;
                    }
                    b'^' => {
                        *state = State::Ctrl(0);
                    }
                    b'n' => {
                        result.push(b'\n');
                        *state = State::Ground;
                    }
                    b'r' => {
                        result.push(b'\r');
                        *state = State::Ground;
                    }
                    b'b' => {
                        result.push(b'\x08');
                        *state = State::Ground;
                    }
                    b'a' => {
                        result.push(b'\x07');
                        *state = State::Ground;
                    }
                    b'v' => {
                        result.push(b'\x0b');
                        *state = State::Ground;
                    }
                    b't' => {
                        result.push(b'\t');
                        *state = State::Ground;
                    }
                    b'f' => {
                        result.push(b'\x0c');
                        *state = State::Ground;
                    }
                    b's' => {
                        result.push(b' ');
                        *state = State::Ground;
                    }
                    b'E' => {
                        result.push(b'\x1b');
                        *state = State::Ground;
                    }
                    b'\n' => {
                        // Hidden newline
                        // result.push(b'\n');
                        *state = State::Ground;
                    }
                    b'$' => {
                        // Hidden marker
                        *state = State::Ground;
                    }
                    _ => {
                        // Invalid syntax
                        bail!("Invalid \\ escape: {}", b);
                    }
                }
            }

            State::Meta => {
                if b == b'-' {
                    *state = State::Meta1;
                } else if b == b'^' {
                    *state = State::Ctrl(0o200);
                } else {
                    bail!("invalid \\M escape: {}", b);
                }
            }

            State::Meta1 => {
                result.push(b | 0o200);
                *state = State::Ground;
            }

            State::Ctrl(c) => {
                if b == b'?' {
                    result.push(*c | 0o177);
                } else {
                    result.push((b & 0o37) | *c);
                }
                *state = State::Ground;
            }

            State::Octal2(prior) => {
                if is_octal(b) {
                    // It's the second in a 2 or 3 byte octal sequence
                    let value = (*prior << 3) + (b - b'0');
                    *state = State::Octal3(value);
                } else {
                    // Prior character was a single octal value
                    result.push(*prior);
                    *state = State::Ground;
                    // re-process the current byte
                    return Ok(true);
                }
            }

            State::Octal3(prior) => {
                if is_octal(b) {
                    // It's the third in a 3 byte octal sequence
                    let value = (*prior << 3) + (b - b'0');
                    result.push(value);
                    *state = State::Ground;
                } else {
                    // Prior was a 2-byte octal sequence
                    result.push(*prior);
                    *state = State::Ground;
                    // re-process the current byte
                    return Ok(true);
                }
            }
        }
        // Don't process this byte again
        Ok(false)
    }

    while let Some(&b) = bytes.next() {
        let again = unvis_byte(b, &mut state, &mut result)?;
        if again {
            unvis_byte(b, &mut state, &mut result)?;
        }
    }

    Ok(result)
}

pub fn unvis(s: &str) -> Result<String> {
    let bytes = s.as_bytes();

    let result = unvis_bytes(bytes)?;

    String::from_utf8(result)
        .map_err(|err| format_err!("Unescaped string is not valid UTF8: {}", err))
}
