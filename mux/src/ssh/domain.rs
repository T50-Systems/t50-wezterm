/// Represents a connection to remote host via ssh.
/// The domain is created with the ssh config prior to making the
/// connection.  The connection is established by the first spawn()
/// call.
/// In order to show the authentication dialog inline in that spawned
/// pane, we play some tricks with wrapped versions of the pty, child
/// and the reader and writer instances so that we can inject the
/// interactive setup.  The bulk of that is driven by `connect_ssh_session`.
pub struct RemoteSshDomain {
    session: Mutex<Option<Session>>,
    dom: SshDomain,
    id: DomainId,
    name: String,
}

pub fn ssh_domain_to_ssh_config(ssh_dom: &SshDomain) -> anyhow::Result<ConfigMap> {
    let mut ssh_config = wezterm_ssh::Config::new();
    ssh_config.add_default_config_files();

    let (remote_host_name, port) = {
        let parts: Vec<&str> = ssh_dom.remote_address.split(':').collect();

        if parts.len() == 2 {
            (parts[0], Some(parts[1].parse::<u16>()?))
        } else {
            (ssh_dom.remote_address.as_str(), None)
        }
    };

    let mut ssh_config = ssh_config.for_host(&remote_host_name);
    ssh_config.insert(
        "wezterm_ssh_backend".to_string(),
        match ssh_dom
            .ssh_backend
            .unwrap_or_else(|| config::configuration().ssh_backend)
        {
            SshBackend::Ssh2 => "ssh2",
            SshBackend::LibSsh => "libssh",
        }
        .to_string(),
    );
    for (k, v) in &ssh_dom.ssh_option {
        ssh_config.insert(k.to_string(), v.to_string());
    }

    if let Some(username) = &ssh_dom.username {
        ssh_config.insert("user".to_string(), username.to_string());
    }
    if let Some(port) = port {
        ssh_config.insert("port".to_string(), port.to_string());
    }
    if ssh_dom.no_agent_auth {
        ssh_config.insert("identitiesonly".to_string(), "yes".to_string());
    }
    if let Some("true") = ssh_config.get("wezterm_ssh_verbose").map(|s| s.as_str()) {
        log::info!("Using ssh config: {ssh_config:#?}");
    }
    Ok(ssh_config)
}

impl RemoteSshDomain {
    pub fn with_ssh_domain(dom: &SshDomain) -> anyhow::Result<Self> {
        let id = alloc_domain_id();
        Ok(Self {
            id,
            name: dom.name.clone(),
            session: Mutex::new(None),
            dom: dom.clone(),
        })
    }

    pub fn ssh_config(&self) -> anyhow::Result<ConfigMap> {
        ssh_domain_to_ssh_config(&self.dom)
    }

    fn build_command(
        &self,
        pane_id: PaneId,
        command: Option<CommandBuilder>,
        command_dir: Option<String>,
    ) -> anyhow::Result<(Option<String>, HashMap<String, String>)> {
        let config = config::configuration();
        let cmd = match command {
            Some(mut cmd) => {
                config.apply_cmd_defaults(&mut cmd, self.dom.default_prog.as_ref(), None);
                cmd
            }
            None => config.build_prog(None, self.dom.default_prog.as_ref(), None)?,
        };
        let mut env: HashMap<String, String> = cmd
            .iter_extra_env_as_str()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        // FIXME: this isn't useful without a way to talk to the remote mux.
        // One option is to forward the mux via unix domain, another is to
        // embed the mux protocol in an escape sequence and just use the
        // existing terminal connection
        env.insert("WEZTERM_REMOTE_PANE".to_string(), pane_id.to_string());

        fn build_env_command(
            dir: Option<String>,
            cmd: &CommandBuilder,
            env: &HashMap<String, String>,
        ) -> anyhow::Result<String> {
            // "Soft" chdir: if it doesn't exist then it doesn't matter
            let cd_cmd = if let Some(dir) = dir {
                format!("cd {};", shell_words::quote(&dir))
            } else if let Some(dir) = cmd.get_cwd() {
                let dir = dir.to_str().context("converting cwd to string")?;
                format!("cd {};", shell_words::quote(&dir))
            } else {
                String::new()
            };

            let mut env_cmd = vec!["env".to_string()];

            for (k, v) in env {
                env_cmd.push(format!("{}={}", k, v));
            }

            let cmd = if cmd.is_default_prog() {
                // We'd like to spawn a login shell, but since we are invoking env
                // we end up in a regular shell.
                // This guff tries to find a reasonably portable way to execute
                // the shell as a login shell.
                // Per: <https://unix.stackexchange.com/a/666850/123914>
                // the most portable way is to use perl, but in case perl is not
                // installed, zsh, bash and ksh all support `exec -a`.
                // Other shells may support `exec -a` but there isn't a simple
                // way to test for them, so we assume that if we have one of those
                // three that we can use it, otherwise we fall back to just running
                // the shell directly.
                let login_shell = "command -v perl > /dev/null && \
                  exec perl -e 'use File::Basename; $shell = basename($ENV{SHELL}); exec {$ENV{SHELL}} \"-$shell\"'; \
                  case \"$SHELL\" in */zsh|*/bash|*/ksh ) exec -a \"-$(basename $SHELL)\" $SHELL ;; esac ; \
                  exec $SHELL";

                format!("$SHELL -c {}", shell_words::quote(login_shell))
            } else {
                cmd.as_unix_command_line()?
            };

            Ok(cd_cmd + &shell_words::join(env_cmd) + " " + &cmd)
        }

        let command_line = match (cmd.is_default_prog(), self.dom.assume_shell, command_dir) {
            (_, Shell::Posix, dir) => Some(build_env_command(dir, &cmd, &env)?),
            (true, _, _) => None,
            (false, _, _) => Some(cmd.as_unix_command_line()?),
        };

        Ok((command_line, env))
    }

    async fn start_new_session(
        &self,
        command_line: Option<String>,
        env: HashMap<String, String>,
        size: TerminalSize,
    ) -> anyhow::Result<StartNewSessionResult> {
        let (session, events) = Session::connect(self.ssh_config().context("obtain ssh config")?)
            .context("connect to ssh server")?;
        self.session.lock().unwrap().replace(session.clone());

        // We get to establish the session!
        //
        // Since we want spawn to return the Pane in which
        // we'll carry out interactive auth, we generate
        // some shim/wrapper versions of the pty, child
        // and reader/writer.

        let (stdout_read, stdout_write) = socketpair()?;
        let (reader_tx, reader_rx) = channel();
        let (stdin_read, stdin_write) = socketpair()?;
        let (writer_tx, writer_rx) = channel();

        let pty_reader = PtyReader {
            reader: Box::new(stdout_read),
            rx: reader_rx,
        };

        let pty_writer = PtyWriter {
            writer: Box::new(stdin_write),
            rx: writer_rx,
        };
        let writer = Box::new(pty_writer);

        let (child_tx, child_rx) = channel();

        let child = Box::new(WrappedSshChild {
            status: None,
            rx: child_rx,
            exited: None,
            killer: WrappedSshChildKiller {
                inner: Arc::new(Mutex::new(KillerInner {
                    killer: None,
                    pending_kill: false,
                })),
            },
        });

        let (pty_tx, pty_rx) = channel();

        let size = Arc::new(Mutex::new(size));

        let pty = Box::new(WrappedSshPty {
            inner: RefCell::new(WrappedSshPtyInner::Connecting {
                size: Arc::clone(&size),
                reader: Some(pty_reader),
                connected: pty_rx,
            }),
        });

        // And with those created, we can now spawn a new thread
        // to perform the blocking (from its perspective) terminal
        // UI to carry out any authentication.
        let mut stdout_write = BufWriter::new(stdout_write);
        std::thread::spawn(move || {
            if let Err(err) = connect_ssh_session(
                session,
                events,
                stdin_read,
                writer_tx,
                &mut stdout_write,
                reader_tx,
                child_tx,
                pty_tx,
                size,
                command_line,
                env,
            ) {
                let _ = write!(stdout_write, "{:#}", err);
                log::error!("Failed to connect ssh: {:#}", err);
            }
            let _ = stdout_write.flush();
        });

        Ok(StartNewSessionResult { pty, child, writer })
    }
}

struct StartNewSessionResult {
    pty: Box<dyn portable_pty::MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send>,
    writer: BoxedWriter,
}
