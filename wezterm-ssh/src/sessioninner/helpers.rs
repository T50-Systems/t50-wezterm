fn write_from_buf<W: Write>(w: &mut W, buf: &mut VecDeque<u8>) -> std::io::Result<()> {
    match w.write(buf.make_contiguous()) {
        Ok(len) => {
            buf.drain(0..len);
            Ok(())
        }
        Err(err) => {
            if err.kind() == std::io::ErrorKind::WouldBlock {
                return Ok(());
            }
            Err(err)
        }
    }
}

fn read_into_buf<R: Read>(r: &mut R, buf: &mut VecDeque<u8>) -> std::io::Result<()> {
    let current_len = buf.len();
    buf.resize(buf.capacity(), 0);
    let target_buf = &mut buf.make_contiguous()[current_len..];
    match r.read(target_buf) {
        Ok(len) => {
            buf.resize(current_len + len, 0);
            if len == 0 {
                Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "EOF",
                ))
            } else {
                Ok(())
            }
        }
        Err(err) => {
            buf.resize(current_len, 0);

            if err.kind() == std::io::ErrorKind::WouldBlock {
                return Ok(());
            }
            Err(err)
        }
    }
}

/// A little helper to ensure that the Result returned by `f()`
/// is routed via a Sender
fn dispatch<T, F>(reply: Sender<T>, f: F, what: &str) -> anyhow::Result<bool>
where
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    if let Err(err) = reply.try_send(f()) {
        log::error!("{}: {:#}", what, err);
    }
    Ok(true)
}

/// A little helper to ensure the Child process is killed on Drop.
struct KillOnDropChild(std::process::Child);

impl Drop for KillOnDropChild {
    fn drop(&mut self) {
        if let Err(err) = self.0.kill() {
            log::error!("Error killing ProxyCommand: {}", err);
        }
        if let Err(err) = self.0.wait() {
            log::error!("Error waiting for ProxyCommand to finish: {}", err);
        }
    }
}
