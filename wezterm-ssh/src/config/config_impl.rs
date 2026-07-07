/// A context for resolving configuration values.
/// Holds a combination of environment and token expansion state,
/// as well as the set of configs that should be consulted.
#[derive(Debug, Clone)]
pub struct Config {
    config_files: Vec<ParsedConfigFile>,
    options: ConfigMap,
    tokens: ConfigMap,
    environment: Option<ConfigMap>,
}

impl Config {
    /// Create a new context without any config files loaded
    pub fn new() -> Self {
        Self {
            config_files: vec![],
            options: ConfigMap::new(),
            tokens: ConfigMap::new(),
            environment: None,
        }
    }

    /// Assign a fake environment map, useful for testing.
    /// The environment is used to expand certain values
    /// from the config.
    pub fn assign_environment(&mut self, env: ConfigMap) {
        self.environment.replace(env);
    }

    /// Assigns token names and expansions for use with a number of
    /// options.  The names and expansions are specified
    /// by `man 5 ssh_config`
    pub fn assign_tokens(&mut self, tokens: ConfigMap) {
        self.tokens = tokens;
    }

    /// Assign the value for an option.
    /// This is logically equivalent to the user specifying command
    /// line options to override config values.
    /// These values take precedence over any values found in config files.
    pub fn set_option<K: AsRef<str>, V: AsRef<str>>(&mut self, key: K, value: V) {
        self.options
            .insert(key.as_ref().to_lowercase(), value.as_ref().to_string());
    }

    /// Parse `config_string` as if it were the contents of an `ssh_config` file,
    /// and add that to the list of configs.
    pub fn add_config_string(&mut self, config_string: &str) {
        self.config_files
            .push(ParsedConfigFile::parse(config_string, None, None));
    }

    /// Open `path`, read its contents and parse it as an `ssh_config` file,
    /// adding that to the list of configs
    pub fn add_config_file<P: AsRef<Path>>(&mut self, path: P) {
        if let Ok(data) = std::fs::read_to_string(path.as_ref()) {
            self.config_files.push(ParsedConfigFile::parse(
                &data,
                path.as_ref().parent(),
                Some(path.as_ref()),
            ));
        }
    }

    /// Convenience method for adding the ~/.ssh/config and system-wide
    /// `/etc/ssh/config` files to the list of configs
    pub fn add_default_config_files(&mut self) {
        if let Some(home) = dirs_next::home_dir() {
            self.add_config_file(home.join(".ssh").join("config"));
        }
        self.add_config_file("/etc/ssh/ssh_config");
        if let Ok(sysdrive) = std::env::var("SystemDrive") {
            self.add_config_file(format!("{}/ProgramData/ssh/ssh_config", sysdrive));
        }
    }

    fn resolve_local_host(&self, include_domain_name: bool) -> String {
        let hostname = if cfg!(test) {
            // Use a fixed and plausible name for the local hostname
            // when running tests.  This isn't an ideal solution, but
            // it is convenient and sufficient at the time of writing
            "localhost".to_string()
        } else {
            gethostname::gethostname().to_string_lossy().to_string()
        };

        if include_domain_name {
            hostname
        } else {
            match hostname.split_once('.') {
                Some((hostname, _domain)) => hostname.to_string(),
                None => hostname,
            }
        }
    }

    fn resolve_local_user(&self) -> String {
        for user in &["USER", "USERNAME"] {
            if let Some(user) = self.resolve_env(user) {
                return user;
            }
        }
        "unknown-user".to_string()
    }

    /// Resolve the configuration for a given host.
    /// The returned map will expand environment and tokens for options
    /// where that is specified.
    /// Note that in some configurations, the config should be parsed once
    /// to resolve the main configuration, and then based on some options
    /// (such as CanonicalHostname), the tokens should be updated and
    /// the config parsed a second time in order for value expansion
    /// to have the same results as `ssh`.
    pub fn for_host<H: AsRef<str>>(&self, host: H) -> ConfigMap {
        let host = host.as_ref();
        let local_user = self.resolve_local_user();
        let target_user = &local_user;

        let mut result = self.options.clone();
        let mut needs_reparse = false;

        for config in &self.config_files {
            if config.apply_matches(
                host,
                target_user,
                &local_user,
                Context::FirstPass,
                &mut result,
            ) {
                needs_reparse = true;
            }
        }

        if needs_reparse {
            log::debug!(
                "ssh configuration uses options that require two-phase \
                parsing, which isn't supported"
            );
        }

        let mut token_map = self.tokens.clone();
        token_map.insert("%h".to_string(), host.to_string());
        result
            .entry("hostname".to_string())
            .and_modify(|curr| {
                if let Some(tokens) = self.should_expand_tokens("hostname") {
                    self.expand_tokens(curr, tokens, &token_map);
                }
            })
            .or_insert_with(|| host.to_string());
        token_map.insert("%h".to_string(), result["hostname"].to_string());
        token_map.insert("%n".to_string(), host.to_string());
        token_map.insert("%r".to_string(), target_user.to_string());
        token_map.insert(
            "%p".to_string(),
            result
                .get("port")
                .map(|p| p.to_string())
                .unwrap_or_else(|| "22".to_string()),
        );

        for (k, v) in &mut result {
            if let Some(tokens) = self.should_expand_tokens(k) {
                self.expand_tokens(v, tokens, &token_map);
            }

            if self.should_expand_environment(k) {
                self.expand_environment(v);
            }
        }

        result
            .entry("port".to_string())
            .or_insert_with(|| "22".to_string());

        result
            .entry("user".to_string())
            .or_insert_with(|| target_user.clone());

        if !result.contains_key("userknownhostsfile") {
            if let Some(home) = self.resolve_home() {
                result.insert(
                    "userknownhostsfile".to_string(),
                    format!("{}/.ssh/known_hosts {}/.ssh/known_hosts2", home, home,),
                );
            }
        }

        if !result.contains_key("identityfile") {
            if let Some(home) = self.resolve_home() {
                result.insert(
                    "identityfile".to_string(),
                    format!(
                        "{}/.ssh/id_dsa {}/.ssh/id_ecdsa {}/.ssh/id_ed25519 {}/.ssh/id_rsa",
                        home, home, home, home
                    ),
                );
            }
        }

        if !result.contains_key("identityagent") {
            if let Some(sock_path) = self.resolve_env("SSH_AUTH_SOCK") {
                result.insert("identityagent".to_string(), sock_path);
            }
        }

        result
    }

    /// Return true if a given option name is subject to environment variable
    /// expansion.
    fn should_expand_environment(&self, key: &str) -> bool {
        match key {
            "certificatefile" | "controlpath" | "identityagent" | "identityfile"
            | "userknownhostsfile" | "localforward" | "remoteforward" => true,
            _ => false,
        }
    }

    /// Returns a set of tokens that should be expanded for a given option name
    fn should_expand_tokens(&self, key: &str) -> Option<&[&str]> {
        match key {
            "certificatefile" | "controlpath" | "identityagent" | "identityfile"
            | "localforward" | "remotecommand" | "remoteforward" | "userknownkostsfile" => {
                Some(&["%C", "%d", "%h", "%i", "%L", "%l", "%n", "%p", "%r", "%u"])
            }
            "hostname" => Some(&["%h"]),
            "localcommand" => Some(&[
                "%C", "%d", "%h", "%i", "%k", "%L", "%l", "%n", "%p", "%r", "%T", "%u",
            ]),
            "proxycommand" => Some(&["%h", "%n", "%p", "%r"]),
            _ => None,
        }
    }

    /// Resolve the home directory.
    /// For the sake of unit testing, this will look for HOME in the provided
    /// environment override before asking the system for the home directory.
    fn resolve_home(&self) -> Option<String> {
        if let Some(env) = self.environment.as_ref() {
            if let Some(home) = env.get("HOME") {
                return Some(home.to_string());
            }
        }
        if let Some(home) = dirs_next::home_dir() {
            if let Some(home) = home.to_str() {
                return Some(home.to_string());
            }
        }
        None
    }

    fn resolve_uid(&self) -> String {
        #[cfg(test)]
        if let Some(env) = self.environment.as_ref() {
            // For testing purposes only, allow pretending that we
            // have a specific fixed UID so that test expectations
            // are easier to handle with snapshots
            if let Some(uid) = env.get("WEZTERM_SSH_UID") {
                return uid.to_string();
            }
        }

        #[cfg(unix)]
        {
            let uid = unsafe { libc::getuid() };
            return uid.to_string();
        }

        #[cfg(not(unix))]
        {
            String::new()
        }
    }

    /// Perform token substitution
    fn expand_tokens(&self, value: &mut String, tokens: &[&str], token_map: &ConfigMap) {
        let orig_value = value.to_string();
        for &t in tokens {
            if let Some(v) = token_map.get(t) {
                *value = value.replace(t, v);
            } else if t == "%i" {
                *value = value.replace(t, &self.resolve_uid());
            } else if t == "%u" {
                *value = value.replace(t, &self.resolve_local_user());
            } else if t == "%l" {
                *value = value.replace(t, &self.resolve_local_host(false));
            } else if t == "%L" {
                *value = value.replace(t, &self.resolve_local_host(true));
            } else if t == "%d" {
                if let Some(home) = self.resolve_home() {
                    let mut items = value
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>();
                    for item in &mut items {
                        if item.starts_with("~/") {
                            item.replace_range(0..1, &home);
                        } else {
                            *item = item.replace(t, &home);
                        }
                    }
                    *value = items.join(" ");
                }
            } else if t == "%j" {
                // %j: The contents of the ProxyJump option, or the empty string if this option is unset
                // We don't directly support ProxyJump, and this %j token referencing
                // may technically put this into two-phase evaluation territory which
                // we don't support.
                // Let's silently gloss over this and treat this token as the empty
                // string.
                // Someone in the future will probably curse this.
                *value = value.replace(t, "");
            } else if t == "%T" {
                // %T: The local tun(4) or tap(4) network interface assigned if tunnel
                // forwarding was requested, or "NONE" otherwise.
                // We don't support this function, so it is always NONE
                *value = value.replace(t, "NONE");
            } else if t == "%C" && value.contains("%C") {
                // %C: Hash of %l%h%p%r%j
                use sha2::Digest;
                let mut c_value = "%l%h%p%r%j".to_string();
                self.expand_tokens(&mut c_value, tokens, token_map);
                let hashed = hex::encode(sha2::Sha256::digest(&c_value.as_bytes()));
                *value = value.replace("%C", &hashed);
            } else if value.contains(t) {
                log::warn!("Unsupported token {t} when evaluating `{orig_value}`");
            }
        }

        *value = value.replace("%%", "%");
    }

    /// Resolve an environment variable; if an override is set use that,
    /// otherwise read from the real environment.
    fn resolve_env(&self, name: &str) -> Option<String> {
        if let Some(env) = self.environment.as_ref() {
            env.get(name).cloned()
        } else {
            std::env::var(name).ok()
        }
    }

    /// Look for `${NAME}` and substitute the value of the `NAME` env var
    /// into the provided string.
    fn expand_environment(&self, value: &mut String) {
        let re = Regex::new(r#"\$\{([a-zA-Z_][a-zA-Z_0-9]+)\}"#).unwrap();
        *value = re
            .replace_all(value, |caps: &Captures| -> String {
                if let Some(rep) = self.resolve_env(&caps[1]) {
                    rep
                } else {
                    caps[0].to_string()
                }
            })
            .to_string();
    }

    /// Returns the list of file names that were loaded as part of parsing
    /// the ssh config
    pub fn loaded_config_files(&self) -> Vec<PathBuf> {
        let mut files = vec![];

        for config in &self.config_files {
            for file in &config.loaded_files {
                if !files.contains(file) {
                    files.push(file.to_path_buf());
                }
            }
        }

        files
    }

    /// Returns the list of host names that have defined ssh config entries.
    /// The host names are literal (non-pattern), non-negated hosts extracted
    /// from `Host` and `Match` stanzas in the ssh config.
    pub fn enumerate_hosts(&self) -> Vec<String> {
        let mut hosts = vec![];

        for config in &self.config_files {
            for group in &config.groups {
                for c in &group.criteria {
                    if let Criteria::Host(patterns) = c {
                        for pattern in patterns {
                            if pattern.is_literal && !pattern.negated {
                                if !hosts.contains(&pattern.original) {
                                    hosts.push(pattern.original.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        hosts
    }
}
