impl SessionInner {
    /// Explicitly and directly connect to the requested host because
    /// neither libssh no libssh2 respect addressfamily, so we must
    /// handle it for ourselves.
    /// If proxy_command is set, then we execute that process for ourselves
    /// too, as proxy commands are not supported by libssh2 and are not supported
    /// on Windows in libssh.
    fn connect_to_host(
        &self,
        hostname: &str,
        port: u16,
        verbose: bool,
    ) -> anyhow::Result<(Socket, Option<KillOnDropChild>)> {
        match self.config.get("proxycommand").map(|s| s.as_str()) {
            Some("none") | None => {}
            Some(proxy_command) => {
                let mut cmd;
                if cfg!(windows) {
                    let comspec = std::env::var("COMSPEC").unwrap_or_else(|_| "cmd".to_string());
                    cmd = std::process::Command::new(comspec);
                    cmd.args(["/c", proxy_command]);
                } else {
                    cmd = std::process::Command::new("sh");
                    cmd.args(["-c", &format!("exec {}", proxy_command)]);
                }

                let (a, b) = socketpair()?;

                cmd.stdin(b.as_stdio()?);
                cmd.stdout(b.as_stdio()?);
                cmd.stderr(std::process::Stdio::inherit());
                let child = cmd
                    .spawn()
                    .with_context(|| format!("spawning ProxyCommand {}", proxy_command))?;

                #[cfg(unix)]
                unsafe {
                    use passfd::FdPassingExt;
                    use std::os::unix::io::{FromRawFd, IntoRawFd};

                    let raw = a.into_raw_fd();
                    let dest = match self.config.get("proxyusefdpass").map(|s| s.as_str()) {
                        Some("yes") => raw.recv_fd()?,
                        _ => raw,
                    };

                    return Ok((Socket::from_raw_fd(dest), Some(KillOnDropChild(child))));
                }
                #[cfg(windows)]
                unsafe {
                    use std::os::windows::io::{FromRawSocket, IntoRawSocket};
                    return Ok((
                        Socket::from_raw_socket(a.into_raw_socket()),
                        Some(KillOnDropChild(child)),
                    ));
                }
            }
        }

        let addr = (hostname, port)
            .to_socket_addrs()?
            .find(|addr| self.filter_sock_addr(addr))
            .with_context(|| format!("resolving address for {}", hostname))?;
        if verbose {
            log::info!("resolved {hostname}:{port} -> {addr:?}");
        }
        let sock = Socket::new(Domain::for_address(addr), Type::STREAM, None)?;
        if let Some(bind_addr) = self.config.get("bindaddress") {
            let bind_addr = (bind_addr.as_str(), 0)
                .to_socket_addrs()?
                .find(|addr| self.filter_sock_addr(addr))
                .with_context(|| format!("resolving bind address {bind_addr:?}"))?;
            if verbose {
                log::info!("binding to {bind_addr:?}");
            }
            sock.bind(&bind_addr.into())
                .with_context(|| format!("binding to {bind_addr:?}"))?;
        }

        sock.connect(&addr.into())
            .with_context(|| format!("Connecting to {hostname}:{port} ({addr:?})"))?;
        Ok((sock, None))
    }

    /// Used to restrict to_socket_addrs results to the address
    /// family specified by the config
    fn filter_sock_addr(&self, addr: &std::net::SocketAddr) -> bool {
        match self.config.get("addressfamily").map(|s| s.as_str()) {
            Some("inet") => addr.is_ipv4(),
            Some("inet6") => addr.is_ipv6(),
            None | Some("any") | Some(_) => true,
        }
    }

    fn do_keepalive(&mut self, sess: &mut SessionWrap) -> anyhow::Result<()> {
        match sess {
            #[cfg(feature = "ssh2")]
            SessionWrap::Ssh2(_sess) => Ok(()),
            #[cfg(feature = "libssh-rs")]
            SessionWrap::LibSsh(sess) => {
                // We implement a very basic keep alive mechanism here;
                // every ServerAliveInterval seconds (if non-zero), we will
                // send an ignore packet.
                // Unlike the openssh client, we do not have a ServerAliveCountMax
                // limit (because it is not clear how we could correctly implement
                // that based on what we can see here in this crate), nor do we
                // explicitly trigger a disconnect if there is an error with
                // the ignore packet.
                if let Some(duration) = self.keep_alive {
                    if self.last_keep_alive.elapsed() >= duration {
                        log::trace!("sending keep alive");
                        self.last_keep_alive = Instant::now();
                        let ignore_me = [0x42; 128];
                        if let Err(err) = sess.sess.send_ignore(&ignore_me) {
                            log::warn!(
                                "Error sending IGNORE packet: {err:#}. Is peer disconnected?"
                            );
                        }
                    }
                }
                Ok(())
            }
        }
    }
}
