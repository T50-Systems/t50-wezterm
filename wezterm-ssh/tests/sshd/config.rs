#[derive(Debug)]
pub struct SshdConfig(HashMap<String, Vec<String>>);

impl Default for SshdConfig {
    fn default() -> Self {
        let mut config = Self::new();

        config.set_authentication_methods(vec!["publickey".to_string()]);
        config.set_use_privilege_separation(false);
        config.set_subsystem(true, true);
        config.set_use_pam(true);
        config.set_x11_forwarding(true);
        config.set_print_motd(true);
        config.set_permit_tunnel(true);
        config.set_kbd_interactive_authentication(true);
        config.set_allow_tcp_forwarding(true);
        config.set_max_startups(500, None);
        config.set_strict_modes(false);

        config
    }
}

impl SshdConfig {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn set_authentication_methods(&mut self, methods: Vec<String>) {
        self.0.insert("AuthenticationMethods".to_string(), methods);
    }

    pub fn set_authorized_keys_file(&mut self, path: impl AsRef<Path>) {
        self.0.insert(
            "AuthorizedKeysFile".to_string(),
            vec![path.as_ref().to_string_lossy().to_string()],
        );
    }

    pub fn set_host_key(&mut self, path: impl AsRef<Path>) {
        self.0.insert(
            "HostKey".to_string(),
            vec![path.as_ref().to_string_lossy().to_string()],
        );
    }

    pub fn set_pid_file(&mut self, path: impl AsRef<Path>) {
        self.0.insert(
            "PidFile".to_string(),
            vec![path.as_ref().to_string_lossy().to_string()],
        );
    }

    pub fn set_subsystem(&mut self, sftp: bool, internal_sftp: bool) {
        let mut values = Vec::new();
        if sftp {
            values.push("sftp".to_string());
        }
        if internal_sftp {
            values.push("internal-sftp".to_string());
        }

        self.0.insert("Subsystem".to_string(), values);
    }

    pub fn set_use_pam(&mut self, yes: bool) {
        self.0.insert("UsePAM".to_string(), Self::yes_value(yes));
    }

    pub fn set_x11_forwarding(&mut self, yes: bool) {
        self.0
            .insert("X11Forwarding".to_string(), Self::yes_value(yes));
    }

    pub fn set_use_privilege_separation(&mut self, yes: bool) {
        self.0
            .insert("UsePrivilegeSeparation".to_string(), Self::yes_value(yes));
    }

    pub fn set_print_motd(&mut self, yes: bool) {
        self.0.insert("PrintMotd".to_string(), Self::yes_value(yes));
    }

    pub fn set_permit_tunnel(&mut self, yes: bool) {
        self.0
            .insert("PermitTunnel".to_string(), Self::yes_value(yes));
    }

    pub fn set_kbd_interactive_authentication(&mut self, yes: bool) {
        self.0.insert(
            "KbdInteractiveAuthentication".to_string(),
            Self::yes_value(yes),
        );
    }

    pub fn set_allow_tcp_forwarding(&mut self, yes: bool) {
        self.0
            .insert("AllowTcpForwarding".to_string(), Self::yes_value(yes));
    }

    pub fn set_max_startups(&mut self, start: u16, rate_full: Option<(u16, u16)>) {
        let value = format!(
            "{}{}",
            start,
            rate_full
                .map(|(r, f)| format!(":{}:{}", r, f))
                .unwrap_or_default(),
        );

        self.0.insert("MaxStartups".to_string(), vec![value]);
    }

    pub fn set_strict_modes(&mut self, yes: bool) {
        self.0
            .insert("StrictModes".to_string(), Self::yes_value(yes));
    }

    fn yes_value(yes: bool) -> Vec<String> {
        vec![Self::yes_string(yes)]
    }

    fn yes_string(yes: bool) -> String {
        Self::yes_str(yes).to_string()
    }

    const fn yes_str(yes: bool) -> &'static str {
        if yes {
            "yes"
        } else {
            "no"
        }
    }
}

impl std::fmt::Display for SshdConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (keyword, values) in self.0.iter() {
            writeln!(
                f,
                "{} {}",
                keyword,
                values
                    .iter()
                    .map(|v| {
                        let v = v.trim();
                        if v.contains(|c: char| c.is_whitespace()) {
                            format!("\"{}\"", v)
                        } else {
                            v.to_string()
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            )?;
        }
        Ok(())
    }
}
