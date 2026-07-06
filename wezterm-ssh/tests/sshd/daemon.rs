pub struct Sshd {
    child: Child,

    /// Port that sshd is listening on
    pub port: u16,

    /// Temporary directory used to hold resources for sshd such as its config, keys, and log
    pub tmp: TempDir,

    agent_sock: PathBuf,
    _agent: SshAgent,
}

impl Sshd {
    pub fn spawn(mut config: SshdConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let _ = env_logger::Builder::new()
            .is_test(true)
            .filter_level(log::LevelFilter::Trace)
            .try_init();

        let tmp = TempDir::new()?;

        // Spawn an agent. It doesn't strictly belong to the daemon in normal
        // operation, but we take care of it together here in the test context
        // because this is where we know the temp dir for the test-specific fixture
        let agent_sock = tmp.join("agent.sock");
        let agent = SshAgent::with_sock(&agent_sock)?;

        // ssh-keygen -t rsa -f $ROOT/id_rsa -N "" -q
        let id_rsa_file = tmp.child("id_rsa");
        assert!(
            SshKeygen::generate_rsa(id_rsa_file.path(), "")?,
            "Failed to ssh-keygen id_rsa"
        );

        // cp $ROOT/id_rsa.pub $ROOT/authorized_keys
        let authorized_keys_file = tmp.child("authorized_keys");
        std::fs::copy(
            id_rsa_file.path().with_extension("pub"),
            authorized_keys_file.path(),
        )?;

        // ssh-keygen -t rsa -f $ROOT/ssh_host_rsa_key -N "" -q
        let ssh_host_rsa_key_file = tmp.child("ssh_host_rsa_key");
        assert!(
            SshKeygen::generate_rsa(ssh_host_rsa_key_file.path(), "")?,
            "Failed to ssh-keygen ssh_host_rsa_key"
        );

        config.set_authorized_keys_file(id_rsa_file.path().with_extension("pub"));
        config.set_host_key(ssh_host_rsa_key_file.path());

        let sshd_pid_file = tmp.child("sshd.pid");
        config.set_pid_file(sshd_pid_file.path());

        // Generate $ROOT/sshd_config based on config
        let sshd_config_file = tmp.child("sshd_config");
        let config_string = config.to_string();
        sshd_config_file.write_str(&config_string)?;
        eprintln!("{config_string}");

        let sshd_log_file = tmp.child("sshd.log");

        let (child, port) = Self::try_spawn_next(sshd_config_file.path(), sshd_log_file.path())
            .expect("No open port available for sshd");

        Ok(Self {
            child,
            port,
            tmp,
            _agent: agent,
            agent_sock,
        })
    }

    fn try_spawn_next(
        config_path: impl AsRef<Path>,
        log_path: impl AsRef<Path>,
    ) -> IoResult<(Child, u16)> {
        let mut err = None;

        for _ in 0..100 {
            let port = allocate_port();

            match Self::try_spawn(port, config_path.as_ref(), log_path.as_ref()) {
                // If successful, return our spawned server child process
                Ok(child) => return Ok((child, port)),

                Err(x) => {
                    err.replace(x);
                }
            }
        }

        Err(err.unwrap())
    }

    fn try_spawn(
        port: u16,
        config_path: impl AsRef<Path>,
        log_path: impl AsRef<Path>,
    ) -> IoResult<Child> {
        let mut child = Command::new(BIN_PATH_STR)
            .arg("-D")
            .arg("-p")
            .arg(port.to_string())
            .arg("-f")
            .arg(config_path.as_ref())
            .arg("-E")
            .arg(log_path.as_ref())
            .spawn()
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("spawning {} failed {:#}", BIN_PATH_STR, e),
                )
            })?;

        for _ in 0..10 {
            // Wait until the port is up
            std::thread::sleep(Duration::from_millis(100));

            // If the server exited already, then we know something is wrong!
            if let Some(exit_status) = child.try_wait()? {
                let output = child.wait_with_output()?;
                let code = exit_status.code();
                let msg = format!(
                    "{}\n{}",
                    String::from_utf8(output.stdout).unwrap(),
                    String::from_utf8(output.stderr).unwrap(),
                );

                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "{} failed [{}]: {}",
                        BIN_PATH_STR,
                        code.map(|x| x.to_string())
                            .unwrap_or_else(|| String::from("???")),
                        msg
                    ),
                ));
            }

            // If the port is up, then we're good!
            if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
                return Ok(child);
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "ran out of ports when spawning sshd",
        ))
    }
}

impl Drop for Sshd {
    /// Kills server upon drop
    fn drop(&mut self) {
        let _ = self.child.kill();

        // NOTE: Should wait to ensure that the process does not become a zombie
        let _ = self.child.wait();
    }
}

#[fixture]
/// Stand up a singular sshd session and hold onto it for the lifetime
/// of our tests, returning a reference to it with each fixture ref
pub fn sshd() -> Sshd {
    Sshd::spawn(Default::default()).unwrap()
}
