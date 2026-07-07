#[derive(Error, Debug, Clone, PartialEq, Eq)]
enum NotReconnectableError {
    #[error("Client was destroyed")]
    ClientWasDestroyed,
}

fn client_thread(
    reconnectable: &mut Reconnectable,
    local_domain_id: Option<DomainId>,
    rx: &mut Receiver<ReaderMessage>,
) -> anyhow::Result<()> {
    block_on(client_thread_async(reconnectable, local_domain_id, rx))
}

async fn client_thread_async(
    reconnectable: &mut Reconnectable,
    local_domain_id: Option<DomainId>,
    rx: &mut Receiver<ReaderMessage>,
) -> anyhow::Result<()> {
    let mut next_serial = 1u64;

    struct Promises {
        map: HashMap<u64, Sender<anyhow::Result<Pdu>>>,
    }

    impl Promises {
        fn fail_all(&mut self, reason: &str) {
            log::trace!("failing all promises: {}", reason);
            for (_, promise) in self.map.drain() {
                let _ = promise.try_send(Err(anyhow!("{}", reason)));
            }
        }
    }

    impl Drop for Promises {
        fn drop(&mut self) {
            self.fail_all("Client was destroyed");
        }
    }
    let mut promises = Promises {
        map: HashMap::new(),
    };

    let mut stream = reconnectable.take_stream().unwrap();

    loop {
        let rx_msg = rx.recv();
        let wait_for_read = stream
            .wait_for_readable()
            .map(|_| Ok(ReaderMessage::Readable));

        match smol::future::or(rx_msg, wait_for_read).await {
            Ok(ReaderMessage::SendPdu { pdu, promise }) => {
                let serial = next_serial;
                next_serial += 1;
                promises.map.insert(serial, promise);

                pdu.encode_async(&mut stream, serial)
                    .await
                    .context("encoding a PDU to send to the server")?;
                stream.flush().await.context("flushing PDU to server")?;
            }
            Ok(ReaderMessage::Readable) => {
                match Pdu::decode_async(&mut stream, Some(next_serial)).await {
                    Ok(decoded) => {
                        log::debug!(
                            "decoded serial {} {}",
                            decoded.serial,
                            decoded.pdu.pdu_name()
                        );
                        if decoded.serial == 0 {
                            process_unilateral(local_domain_id, decoded)
                                .context("processing unilateral PDU from server")
                                .map_err(|e| {
                                    log::error!("process_unilateral: {:?}", e);
                                    e
                                })?;
                        } else if let Some(promise) = promises.map.remove(&decoded.serial) {
                            if promise.try_send(Ok(decoded.pdu)).is_err() {
                                return Err(NotReconnectableError::ClientWasDestroyed.into());
                            }
                        } else {
                            let reason =
                                format!("got serial {:?} without a corresponding promise", decoded);
                            promises.fail_all(&reason);
                            anyhow::bail!("{}", reason);
                        }
                    }
                    Err(err) => {
                        let reason = format!("Error while decoding response pdu: {:#}", err);
                        log::error!("{}", reason);
                        promises.fail_all(&reason);
                        return Err(err).context("Error while decoding response pdu");
                    }
                }
            }
            Err(_) => {
                return Err(NotReconnectableError::ClientWasDestroyed.into());
            }
        }
    }
}

pub fn unix_connect_with_retry(
    target: &UnixTarget,
    just_spawned: bool,
    max_attempts: Option<u64>,
) -> anyhow::Result<UnixStream> {
    let mut error = None;

    if just_spawned {
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    let max_attempts = max_attempts.unwrap_or(10);

    for iter in 0..max_attempts {
        if iter > 0 {
            std::thread::sleep(std::time::Duration::from_millis(iter * 50));
        }
        match target {
            UnixTarget::Socket(path) => match UnixStream::connect(path) {
                Ok(stream) => return Ok(stream),
                Err(err) => {
                    error =
                        Some(Err(err).with_context(|| format!("connecting to {}", path.display())))
                }
            },
            UnixTarget::Proxy(argv) => {
                let mut cmd = std::process::Command::new(&argv[0]);
                cmd.args(&argv[1..]);

                let (a, b) = filedescriptor::socketpair()?;

                cmd.stdin(b.as_stdio()?);
                cmd.stdout(b.as_stdio()?);
                cmd.stderr(std::process::Stdio::inherit());
                let mut child = cmd
                    .spawn()
                    .with_context(|| format!("spawning proxy command {:?}", cmd))?;

                error.take();

                // Grace period to detect whether connection failed
                for _ in 0..5 {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            error = Some(Err(anyhow!(
                                "{:?} exited already with status {:?}",
                                cmd,
                                status
                            )));
                            continue;
                        }
                        Ok(None) => {
                            error.take();
                        }
                        Err(err) => {
                            error =
                                Some(Err(err).context(format!("spawning proxy command {:?}", cmd)));
                            continue;
                        }
                    }
                }

                if error.is_none() {
                    #[cfg(unix)]
                    unsafe {
                        use std::os::unix::io::{FromRawFd, IntoRawFd};
                        return Ok(UnixStream::from_raw_fd(a.into_raw_fd()));
                    }
                    #[cfg(windows)]
                    unsafe {
                        use std::os::windows::io::{FromRawSocket, IntoRawSocket};
                        return Ok(UnixStream::from_raw_socket(a.into_raw_socket()));
                    }
                }
            }
        }
    }

    error.expect("only get here after at least one unix fail")
}
