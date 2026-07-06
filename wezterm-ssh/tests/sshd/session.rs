pub struct SessionWithSshd {
    _sshd: Sshd,
    session: Session,
}

impl std::ops::Deref for SessionWithSshd {
    type Target = Session;
    fn deref(&self) -> &Session {
        &self.session
    }
}

impl std::ops::DerefMut for SessionWithSshd {
    fn deref_mut(&mut self) -> &mut Session {
        &mut self.session
    }
}

#[fixture]
/// Stand up an sshd instance and then connect to it and perform authentication
pub async fn session(#[default(Config::new())] config: Config, sshd: Sshd) -> SessionWithSshd {
    let port = sshd.port;

    // Do not add the default config files; they take the config of the
    // user that is running the tests which can vary wildly and have
    // inappropriate configuration that disrupts the tests.
    // NO: config.add_default_config_files();

    // Load our config to point to ourselves, using current sshd instance's port,
    // generated identity file, and host file
    let mut config = config.for_host("localhost");
    config.insert("port".to_string(), port.to_string());
    config.insert("wezterm_ssh_verbose".to_string(), "true".to_string());

    // If libssh-rs is not loaded (but ssh2 is), then we use ssh2 as the backend
    #[cfg(not(feature = "libssh-rs"))]
    config.insert("wezterm_ssh_backend".to_string(), "ssh2".to_string());

    config.insert(
        "identityagent".to_string(),
        format!("{}", sshd.agent_sock.display()),
    );

    config.insert("user".to_string(), USERNAME.to_string());
    config.insert("identitiesonly".to_string(), "yes".to_string());
    config.insert(
        "pubkeyacceptedtypes".to_string(),
        // Ensure that we have ssh-rsa in the list, as debian9
        // seems unhappy without it
        "ssh-rsa,ssh-ed25519,\
                  rsa-sha2-512,rsa-sha2-256,ecdsa-sha2-nistp521,\
                  ecdsa-sha2-nistp384,ecdsa-sha2-nistp256"
            .to_string(),
    );
    config.insert(
        "identityfile".to_string(),
        sshd.tmp
            .child("id_rsa")
            .path()
            .to_str()
            .expect("Failed to get string path for id_rsa")
            .to_string(),
    );
    config.insert(
        "userknownhostsfile".to_string(),
        sshd.tmp
            .child("known_hosts")
            .path()
            .to_str()
            .expect("Failed to get string path for known_hosts")
            .to_string(),
    );

    // Perform our actual connection
    let (session, events) = Session::connect(config.clone()).expect("Failed to connect to sshd");

    // Perform automated authentication, assuming that we have a publickey with empty password
    while let Ok(event) = events.recv().await {
        match event {
            SessionEvent::Banner(banner) => {
                if let Some(banner) = banner {
                    log::trace!("{}", banner);
                }
            }
            SessionEvent::HostVerify(verify) => {
                eprintln!("{}", verify.message);

                // Automatically verify any host
                verify
                    .answer(true)
                    .await
                    .expect("Failed to send host verification");
            }
            SessionEvent::Authenticate(auth) => {
                if !auth.username.is_empty() {
                    eprintln!("Authentication for {}", auth.username);
                }
                if !auth.instructions.is_empty() {
                    eprintln!("{}", auth.instructions);
                }

                // Reply with empty string to all authentication requests
                let answers = vec![String::new(); auth.prompts.len()];
                auth.answer(answers)
                    .await
                    .expect("Failed to send authenticate response");
            }
            SessionEvent::HostVerificationFailed(failed) => {
                panic!("{}", failed);
            }
            SessionEvent::Error(err) => {
                panic!("{}", err);
            }
            SessionEvent::Authenticated => break,
        }
    }

    SessionWithSshd {
        session,
        _sshd: sshd,
    }
}
