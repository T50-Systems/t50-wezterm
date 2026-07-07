impl SessionInner {
    pub fn run(&mut self) {
        if let Err(err) = self.run_impl() {
            self.tx_event
                .try_send(SessionEvent::Error(format!("{:#}", err)))
                .ok();
        }
    }

    fn run_impl(&mut self) -> anyhow::Result<()> {
        let backend = self
            .config
            .get("wezterm_ssh_backend")
            .map(|s| s.as_str())
            .unwrap_or(
                #[cfg(feature = "libssh-rs")]
                "libssh",
                #[cfg(not(feature = "libssh-rs"))]
                "ssh2",
            );
        match backend {
            #[cfg(feature = "ssh2")]
            "ssh2" => self.run_impl_ssh2(),

            #[cfg(not(feature = "ssh2"))]
            "ssh2" => anyhow::bail!(
                "invalid wezterm_ssh_backend value: {}, not compiled with `ssh2`",
                backend
            ),

            #[cfg(feature = "libssh-rs")]
            "libssh" => self.run_impl_libssh(),

            #[cfg(not(feature = "libssh-rs"))]
            "libssh" => anyhow::bail!(
                "invalid wezterm_ssh_backend value: {}, not compiled with `libssh`",
                backend
            ),

            _ => anyhow::bail!(
                "invalid wezterm_ssh_backend value: {}, expected either `ssh2` or `libssh`",
                backend
            ),
        }
    }

    #[cfg(feature = "libssh-rs")]
    fn run_impl_libssh(&mut self) -> anyhow::Result<()> {
        let hostname = self
            .config
            .get("hostname")
            .ok_or_else(|| anyhow!("hostname not present in config"))?
            .to_string();
        let user = self
            .config
            .get("user")
            .ok_or_else(|| anyhow!("username not present in config"))?
            .to_string();
        let port = self
            .config
            .get("port")
            .ok_or_else(|| anyhow!("port is always set in config loader"))?
            .parse::<u16>()?;

        self.tx_event
            .try_send(SessionEvent::Banner(Some(format!(
                "Using libssh-rs to connect to {}@{}:{}",
                user, hostname, port
            ))))
            .context("notifying user of banner")?;

        let sess = libssh_rs::Session::new()?;
        let verbose = self
            .config
            .get("wezterm_ssh_verbose")
            .map(|s| s.as_str())
            .unwrap_or("false")
            == "true";
        if verbose {
            sess.set_option(libssh_rs::SshOption::LogLevel(libssh_rs::LogLevel::Packet))?;

            /// libssh logs to stderr, but on Windows in the GUI there isn't a valid
            /// stderr for it to log to.
            /// So, we redirect logging via our own log callback and pipe it via
            /// the `log` crate.
            unsafe extern "C" fn log_callback(
                _priority: std::os::raw::c_int,
                function: *const std::os::raw::c_char,
                message: *const std::os::raw::c_char,
                _userdata: *mut std::os::raw::c_void,
            ) {
                use std::ffi::CStr;
                let function = CStr::from_ptr(function).to_string_lossy().to_string();
                let message = CStr::from_ptr(message).to_string_lossy().to_string();

                // The message typically has "function: message" prefixed, which
                // looks redundant when logged with the function prefix by the
                // logging crate.
                // Strip that off!
                let message = match message.strip_prefix(&format!("{}: ", function)) {
                    Some(m) => m,
                    None => &message,
                };

                log::logger().log(
                    &log::Record::builder()
                        .args(format_args!("{}", message))
                        .level(log::Level::Info)
                        .module_path(Some(&function))
                        .target(&format!("libssh::{}", function))
                        .build(),
                );
            }
            unsafe {
                libssh_rs::sys::ssh_set_log_callback(Some(log_callback));
            }
        }
        sess.set_option(libssh_rs::SshOption::Hostname(hostname.clone()))?;
        sess.set_option(libssh_rs::SshOption::User(Some(user)))?;
        sess.set_option(libssh_rs::SshOption::Port(port))?;
        sess.options_parse_config(None)?; // FIXME: overridden config path?
        if let Some(agent) = self.config.get("identityagent") {
            sess.set_option(libssh_rs::SshOption::IdentityAgent(Some(agent.clone())))?;
        }
        if let Some(files) = self.config.get("identityfile") {
            for file in files.split_whitespace() {
                sess.set_option(libssh_rs::SshOption::AddIdentity(file.to_string()))?;
            }
        }
        if let Some(kh) = self.config.get("userknownhostsfile") {
            for file in kh.split_whitespace() {
                sess.set_option(libssh_rs::SshOption::KnownHosts(Some(file.to_string())))?;
                break;
            }
        }
        if let Some(types) = self.config.get("pubkeyacceptedtypes") {
            sess.set_option(libssh_rs::SshOption::PublicKeyAcceptedTypes(
                types.to_string(),
            ))?;
        }
        if let Some(bind_addr) = self.config.get("bindaddress") {
            sess.set_option(libssh_rs::SshOption::BindAddress(bind_addr.to_string()))?;
        }
        if let Some(host_key) = self.config.get("hostkeyalgorithms") {
            sess.set_option(libssh_rs::SshOption::HostKeys(host_key.to_string()))?;
        }

        let (sock, _child) = self.connect_to_host(&hostname, port, verbose)?;
        let raw = {
            #[cfg(unix)]
            {
                use std::os::unix::io::IntoRawFd;
                sock.into_raw_fd()
            }
            #[cfg(windows)]
            {
                use std::os::windows::io::IntoRawSocket;
                sock.into_raw_socket()
            }
        };

        sess.set_option(libssh_rs::SshOption::Socket(raw))?;

        sess.connect()
            .with_context(|| format!("Connecting to {hostname}:{port}"))?;

        let banner = sess.get_server_banner()?;
        self.tx_event
            .try_send(SessionEvent::Banner(Some(banner)))
            .context("notifying user of banner")?;

        self.host_verification_libssh(&sess, &hostname, port)?;
        self.authenticate_libssh(&sess)?;

        if let Ok(banner) = sess.get_issue_banner() {
            self.tx_event
                .try_send(SessionEvent::Banner(Some(banner)))
                .context("notifying user of banner")?;
        }

        self.tx_event
            .try_send(SessionEvent::Authenticated)
            .context("notifying user that session is authenticated")?;

        if let Some("yes") = self.config.get("forwardagent").map(|s| s.as_str()) {
            if self.identity_agent().is_some() {
                sess.enable_accept_agent_forward(true);
            } else {
                log::error!("ForwardAgent is set to yes, but IdentityAgent is not set");
            }
        }
        sess.set_blocking(false);
        let mut sess = SessionWrap::with_libssh(sess);
        self.request_loop(&mut sess)
    }

    #[cfg(feature = "ssh2")]
    fn run_impl_ssh2(&mut self) -> anyhow::Result<()> {
        let verbose = self
            .config
            .get("wezterm_ssh_verbose")
            .map(|s| s.as_str())
            .unwrap_or("false")
            == "true";

        let hostname = self
            .config
            .get("hostname")
            .ok_or_else(|| anyhow!("hostname not present in config"))?
            .to_string();
        let user = self
            .config
            .get("user")
            .ok_or_else(|| anyhow!("username not present in config"))?
            .to_string();
        let port = self
            .config
            .get("port")
            .ok_or_else(|| anyhow!("port is always set in config loader"))?
            .parse::<u16>()?;
        let remote_address = format!("{}:{}", hostname, port);

        self.tx_event
            .try_send(SessionEvent::Banner(Some(format!(
                "Using ssh2 to connect to {}@{}:{}",
                user, hostname, port
            ))))
            .context("notifying user of banner")?;

        let (sock, _child) = self.connect_to_host(&hostname, port, verbose)?;

        let mut sess = ssh2::Session::new()?;
        if verbose {
            sess.trace(ssh2::TraceFlags::all());
        }
        sess.set_blocking(true);
        sess.set_tcp_stream(sock);
        sess.handshake()
            .with_context(|| format!("ssh handshake with {}", remote_address))?;

        self.tx_event
            .try_send(SessionEvent::Banner(sess.banner().map(|s| s.to_string())))
            .context("notifying user of banner")?;

        self.host_verification(&sess, &hostname, port, &remote_address)
            .context("host verification")?;

        self.authenticate(&sess, &user, &hostname)
            .context("authentication")?;

        self.tx_event
            .try_send(SessionEvent::Authenticated)
            .context("notifying user that session is authenticated")?;

        sess.set_blocking(false);

        let mut sess = SessionWrap::with_ssh2(sess);
        self.request_loop(&mut sess)
    }
}
