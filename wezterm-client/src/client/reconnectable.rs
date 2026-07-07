impl Reconnectable {
    fn new(config: ClientDomainConfig, stream: Option<Box<dyn AsyncReadAndWrite>>) -> Self {
        Self {
            config,
            stream,
            tls_creds: None,
        }
    }

    fn tls_creds_path(&self) -> anyhow::Result<PathBuf> {
        let path = config::pki_dir()?.join(self.config.name());
        std::fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn tls_creds_ca_path(&self) -> anyhow::Result<PathBuf> {
        Ok(self.tls_creds_path()?.join("ca.pem"))
    }

    fn tls_creds_cert_path(&self) -> anyhow::Result<PathBuf> {
        Ok(self.tls_creds_path()?.join("cert.pem"))
    }

    fn take_stream(&mut self) -> Option<Box<dyn AsyncReadAndWrite>> {
        self.stream.take()
    }

    fn is_local(&mut self) -> bool {
        matches!(&self.config, ClientDomainConfig::Unix(_))
    }

    fn reconnectable(&mut self) -> bool {
        match &self.config {
            // It doesn't make sense to reconnect to a unix socket; we only
            // get disconnected it it dies, so respawning it would not preserve
            // the set of tabs and we'd have confusing and inconsistent state
            ClientDomainConfig::Unix(_) => false,
            ClientDomainConfig::Tls(_) => true,
            // It *does* make sense to reconnect with an ssh session, but we
            // need to grow some smarts about whether the disconnect was because
            // we sent CTRL-D to close the last session, or whether it was a network
            // level disconnect, because we will otherwise throw up authentication
            // dialogs that would be annoying
            ClientDomainConfig::Ssh(_) => false,
        }
    }

    fn connect(
        &mut self,
        initial: bool,
        ui: &mut ConnectionUI,
        no_auto_start: bool,
    ) -> anyhow::Result<()> {
        match self.config.clone() {
            ClientDomainConfig::Unix(unix_dom) => {
                self.unix_connect(unix_dom, initial, ui, no_auto_start)
            }
            ClientDomainConfig::Tls(tls) => self.tls_connect(tls, initial, ui),
            ClientDomainConfig::Ssh(ssh) => self.ssh_connect(ssh, initial, ui),
        }
    }

    /// Resolve the path to wezterm for the remote system.
    /// We can't simply derive this from the current executable because
    /// we are being asked to produce a path for the remote system and
    /// we don't really know anything about it.
    /// `path` comes from the SshDoman::remote_wezterm_path option; if set
    /// then the user has told us where to look.
    /// Otherwise, we have to rely on the `PATH` environment for the remote
    /// system, and we don't know if it is even running unix, or whether
    /// any given shell syntax will help us provide a more meaningful
    /// message to the user.
    fn wezterm_bin_path(path: &Option<String>) -> String {
        path.as_deref().unwrap_or("wezterm").to_string()
    }

    fn ssh_connect(
        &mut self,
        ssh_dom: SshDomain,
        initial: bool,
        ui: &mut ConnectionUI,
    ) -> anyhow::Result<()> {
        let ssh_config = mux::ssh::ssh_domain_to_ssh_config(&ssh_dom)?;

        let sess = ssh_connect_with_ui(ssh_config, ui)?;
        let proxy_bin = Self::wezterm_bin_path(&ssh_dom.remote_wezterm_path);

        let cmd = if let Some(cmd) = ssh_dom.override_proxy_command.clone() {
            cmd
        } else if initial {
            format!("{} cli --prefer-mux proxy", proxy_bin)
        } else {
            format!("{} cli --prefer-mux --no-auto-start proxy", proxy_bin)
        };
        ui.output_str(&format!("Running: {}\n", cmd));
        log::debug!("going to run {}", cmd);

        let exec = smol::block_on(sess.exec(&cmd, None))?;

        let mut stderr = exec.stderr;
        std::thread::spawn(move || {
            let mut buf = [0u8; 1024];
            while let Ok(len) = stderr.read(&mut buf) {
                if len == 0 {
                    break;
                } else {
                    let stderr = &buf[0..len];
                    log::error!("ssh stderr: {}", String::from_utf8_lossy(stderr));
                }
            }
        });

        // This is a bit gross, but it helps to surface errors in running
        // the proxy, and prevents us from hanging forever after the process
        // has died
        let mut child = exec.child;
        std::thread::spawn(move || match child.wait() {
            Err(err) => log::error!("waiting on {} failed: {:#}", cmd, err),
            Ok(status) if !status.success() => log::error!("{}: {}", cmd, status),
            _ => {}
        });

        let stream: Box<dyn AsyncReadAndWrite> = Box::new(Async::new(SshStream {
            stdin: exec.stdin,
            stdout: exec.stdout,
        })?);
        self.stream.replace(stream);
        Ok(())
    }

    fn unix_connect(
        &mut self,
        unix_dom: UnixDomain,
        initial: bool,
        ui: &mut ConnectionUI,
        no_auto_start: bool,
    ) -> anyhow::Result<()> {
        let target = unix_dom.target();
        ui.output_str(&format!("Connect to {:?}\n", target));
        log::trace!("connect to {:?}", target);

        let max_attempts = if no_auto_start { Some(1) } else { None };

        let stream = match unix_connect_with_retry(&target, false, max_attempts) {
            Ok(stream) => stream,
            Err(e) => {
                if no_auto_start || unix_dom.no_serve_automatically || !initial {
                    bail!("failed to connect to {:?}: {}", target, e);
                }
                log::warn!(
                    "While connecting to {:?}: {}.  Will try spawning the server.",
                    target,
                    e
                );
                ui.output_str(&format!("Error: {}.  Will try spawning server.\n", e));

                let argv = unix_dom.serve_command()?;

                let mut cmd = std::process::Command::new(&argv[0]);
                cmd.args(&argv[1..]);

                #[cfg(unix)]
                if let Some(mask) = umask::UmaskSaver::saved_umask() {
                    unsafe {
                        cmd.pre_exec(move || {
                            libc::umask(mask);
                            Ok(())
                        });
                    }
                }

                log::warn!("Running: {:?}", cmd);
                ui.output_str(&format!("Running: {:?}\n", cmd));

                let child = cmd
                    .spawn()
                    .with_context(|| format!("while spawning {:?}", cmd))?;
                std::thread::spawn(move || match child.wait_with_output() {
                    Ok(out) => {
                        if let Ok(stdout) = std::str::from_utf8(&out.stdout) {
                            if !stdout.is_empty() {
                                log::warn!("stdout: {}", stdout);
                            }
                        }
                        if let Ok(stderr) = std::str::from_utf8(&out.stderr) {
                            if !stderr.is_empty() {
                                log::warn!("stderr: {}", stderr);
                            }
                        }
                    }
                    Err(err) => {
                        log::error!("spawn: {:#}", err);
                    }
                });

                unix_connect_with_retry(&target, true, None).with_context(|| {
                    format!("(after spawning server) failed to connect to {:?}", target)
                })?
            }
        };

        ui.output_str("Connected!\n");
        stream.set_read_timeout(Some(unix_dom.read_timeout))?;
        stream.set_write_timeout(Some(unix_dom.write_timeout))?;
        let stream: Box<dyn AsyncReadAndWrite> = Box::new(Async::new(stream)?);
        self.stream.replace(stream);
        Ok(())
    }

    pub fn tls_connect(
        &mut self,
        tls_client: TlsDomainClient,
        _initial: bool,
        ui: &mut ConnectionUI,
    ) -> anyhow::Result<()> {
        openssl::init();

        let remote_address = &tls_client.remote_address;

        let remote_host_name = remote_address.split(':').next().ok_or_else(|| {
            anyhow!(
                "expected mux_server_remote_address to have the form 'host:port', but have {}",
                remote_address
            )
        })?;

        // If we are reconnecting and already bootstrapped via SSH, let's see if
        // we can connect using those same credentials and avoid running through
        // the SSH authentication flow.
        if let Some(Ok(_)) = tls_client.ssh_parameters() {
            match self.try_connect(&tls_client, ui, &remote_address, remote_host_name) {
                Ok(stream) => {
                    self.stream.replace(stream);
                    return Ok(());
                }
                Err(err) => {
                    if let Some(ioerr) = err.root_cause().downcast_ref::<std::io::Error>() {
                        match ioerr.kind() {
                            std::io::ErrorKind::ConnectionRefused => {
                                // Server isn't up yet; let's proceed with bootstrap
                            }
                            _ => {
                                // If it is an IO error that implies that we had an issue
                                // reaching or otherwise talking to the remote host.
                                // Re-attempting the SSH bootstrap most likely will not
                                // succeed so we let this bubble up.
                                return Err(err);
                            }
                        }
                    }
                    ui.output_str(&format!(
                        "Failed to reuse creds: {:?}\nWill retry bootstrap via SSH\n",
                        err
                    ));
                }
            }
        }

        if let Some(Ok(ssh_params)) = tls_client.ssh_parameters() {
            if self.tls_creds.is_none() {
                // We need to bootstrap via an ssh session

                let mut ssh_config = wezterm_ssh::Config::new();
                ssh_config.add_default_config_files();

                let mut fields = ssh_params.host_and_port.split(':');
                let host = fields
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("no host component somehow"))?;
                let port = fields.next();

                let mut ssh_config = ssh_config.for_host(host);
                if let Some(username) = &ssh_params.username {
                    ssh_config.insert("user".to_string(), username.to_string());
                }
                if let Some(port) = port {
                    ssh_config.insert("port".to_string(), port.to_string());
                }

                let sess = ssh_connect_with_ui(ssh_config, ui)?;

                let creds = ui.run_and_log_error(|| {
                    // The `tlscreds` command will start the server if needed and then
                    // obtain client credentials that we can use for tls.
                    let cmd = format!(
                        "{} cli tlscreds",
                        Self::wezterm_bin_path(&tls_client.remote_wezterm_path)
                    );

                    ui.output_str(&format!("Running: {}\n", cmd));
                    let mut exec = smol::block_on(sess.exec(&cmd, None))
                        .with_context(|| format!("executing `{}` on remote host", cmd))?;

                    log::debug!("waiting for command to finish");
                    let status = exec.child.wait()?;
                    if !status.success() {
                        anyhow::bail!("{} failed", cmd);
                    }

                    drop(exec.stdin);

                    let mut stderr = exec.stderr;
                    thread::spawn(move || {
                        // stderr is ideally empty
                        let mut err = String::new();
                        let _ = stderr.read_to_string(&mut err);
                        if !err.is_empty() {
                            log::error!("remote: `{}` stderr -> `{}`", cmd, err);
                        }
                    });

                    let creds = match Pdu::decode(exec.stdout)
                        .context("reading tlscreds response")?
                        .pdu
                    {
                        Pdu::GetTlsCredsResponse(creds) => creds,
                        _ => bail!("unexpected response to tlscreds"),
                    };

                    // Save the credentials to disk, as that is currently the easiest
                    // way to get them into openssl.  Ideally we'd keep these entirely
                    // in memory.
                    std::fs::write(&self.tls_creds_ca_path()?, creds.ca_cert_pem.as_bytes())?;
                    std::fs::write(
                        &self.tls_creds_cert_path()?,
                        creds.client_cert_pem.as_bytes(),
                    )?;
                    log::info!("got TLS creds");
                    Ok(creds)
                })?;
                self.tls_creds.replace(creds);
            }
        }

        let cloned_ui = ui.clone();
        let stream = cloned_ui.run_and_log_error({
            || self.try_connect(&tls_client, ui, &remote_address, remote_host_name)
        })?;
        self.stream.replace(stream);
        Ok(())
    }

    fn try_connect(
        &mut self,
        tls_client: &TlsDomainClient,
        ui: &mut ConnectionUI,
        remote_address: &str,
        remote_host_name: &str,
    ) -> anyhow::Result<Box<dyn AsyncReadAndWrite>> {
        let mut connector = SslConnector::builder(SslMethod::tls())?;

        let cert_file = match tls_client.pem_cert.clone() {
            Some(cert) => cert,
            None => self.tls_creds_cert_path()?,
        };

        connector
            .set_certificate_file(&cert_file, SslFiletype::PEM)
            .context(format!(
                "set_certificate_file to {} for TLS client",
                cert_file.display()
            ))?;

        if let Some(chain_file) = tls_client.pem_ca.as_ref() {
            connector
                .set_certificate_chain_file(&chain_file)
                .context(format!(
                    "set_certificate_chain_file to {} for TLS client",
                    chain_file.display()
                ))?;
        }

        let key_file = match tls_client.pem_private_key.clone() {
            Some(key) => key,
            None => self.tls_creds_cert_path()?,
        };
        connector
            .set_private_key_file(&key_file, SslFiletype::PEM)
            .context(format!(
                "set_private_key_file to {} for TLS client",
                key_file.display()
            ))?;

        fn load_cert(name: &Path) -> anyhow::Result<X509> {
            let cert_bytes = std::fs::read(name)?;
            log::trace!("loaded {}", name.display());
            Ok(X509::from_pem(&cert_bytes)?)
        }
        for name in &tls_client.pem_root_certs {
            if name.is_dir() {
                for entry in std::fs::read_dir(name)? {
                    if let Ok(cert) = load_cert(&entry?.path()) {
                        connector.cert_store_mut().add_cert(cert).ok();
                    }
                }
            } else {
                connector.cert_store_mut().add_cert(load_cert(name)?)?;
            }
        }

        if let Ok(ca_path) = self.tls_creds_ca_path() {
            if ca_path.exists() {
                connector.cert_store_mut().add_cert(load_cert(&ca_path)?)?;
            }
        }

        let connector = connector.build();
        let connector = connector
            .configure()?
            .verify_hostname(!tls_client.accept_invalid_hostnames);

        ui.output_str(&format!("Connecting to {} using TLS\n", remote_address));
        let stream = TcpStream::connect(remote_address)
            .with_context(|| format!("connecting to {}", remote_address))?;
        stream.set_nodelay(true)?;
        stream.set_write_timeout(Some(tls_client.write_timeout))?;
        stream.set_read_timeout(Some(tls_client.read_timeout))?;

        let stream = Box::new(Async::new(AsyncSslStream::new(
            connector
                .connect(
                    tls_client
                        .expected_cn
                        .as_deref()
                        .unwrap_or(remote_host_name),
                    stream,
                )
                .with_context(|| {
                    format!(
                        "SslConnector for {} with host name {}",
                        remote_address, remote_host_name,
                    )
                })?,
        ))?);
        ui.output_str("TLS Connected!\n");
        Ok(stream)
    }
}
