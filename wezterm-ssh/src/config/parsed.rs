/// Holds the ordered set of parsed options.
/// The config file semantics are that the first matching value
/// for a given option takes precedence
#[derive(Debug, PartialEq, Eq, Clone)]
struct ParsedConfigFile {
    /// options that appeared before any `Host` stanza
    options: ConfigMap,
    /// options inside a `Host` stanza
    groups: Vec<MatchGroup>,
    /// list of loaded file names
    loaded_files: Vec<PathBuf>,
}

impl ParsedConfigFile {
    fn parse(s: &str, cwd: Option<&Path>, source_file: Option<&Path>) -> Self {
        let mut options = ConfigMap::new();
        let mut groups = vec![];
        let mut loaded_files = vec![];

        if let Some(source) = source_file {
            loaded_files.push(source.to_path_buf());
        }

        Self::parse_impl(s, cwd, &mut options, &mut groups, &mut loaded_files);

        Self {
            options,
            groups,
            loaded_files,
        }
    }

    fn do_include(
        pattern: &str,
        cwd: Option<&Path>,
        options: &mut ConfigMap,
        groups: &mut Vec<MatchGroup>,
        loaded_files: &mut Vec<PathBuf>,
    ) {
        match filenamegen::Glob::new(&pattern) {
            Ok(g) => {
                match cwd
                    .as_ref()
                    .map(|p| p.to_path_buf())
                    .or_else(|| std::env::current_dir().ok())
                {
                    Some(cwd) => {
                        for path in g.walk(&cwd) {
                            let path = if path.is_absolute() {
                                path
                            } else {
                                cwd.join(path)
                            };
                            match std::fs::read_to_string(&path) {
                                Ok(data) => {
                                    loaded_files.push(path.clone());
                                    Self::parse_impl(
                                        &data,
                                        Some(&cwd),
                                        options,
                                        groups,
                                        loaded_files,
                                    );
                                }
                                Err(err) => {
                                    log::error!(
                                        "error expanding `Include {}`: unable to open {}: {:#}",
                                        pattern,
                                        path.display(),
                                        err
                                    );
                                }
                            }
                        }
                    }
                    None => {
                        log::error!(
                            "error expanding `Include {}`: unable to determine cwd",
                            pattern
                        );
                    }
                }
            }
            Err(err) => {
                log::error!("error expanding `Include {}`: {:#}", pattern, err);
            }
        }
    }

    fn parse_impl(
        s: &str,
        cwd: Option<&Path>,
        options: &mut ConfigMap,
        groups: &mut Vec<MatchGroup>,
        loaded_files: &mut Vec<PathBuf>,
    ) {
        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(sep) = line.find(|c: char| c == '=' || c.is_whitespace()) {
                let (k, v) = line.split_at(sep);
                let k = k.trim().to_lowercase();
                let v = v[1..].trim();

                let v = if v.starts_with('"') && v.ends_with('"') {
                    &v[1..v.len() - 1]
                } else {
                    v
                };

                fn parse_pattern_list(v: &str) -> Vec<Pattern> {
                    let mut patterns = vec![];
                    for p in v.split(',') {
                        let p = p.trim();
                        if p.starts_with('!') {
                            patterns.push(Pattern::new(&p[1..], true));
                        } else {
                            patterns.push(Pattern::new(p, false));
                        }
                    }
                    patterns
                }
                fn parse_whitespace_pattern_list(v: &str) -> Vec<Pattern> {
                    let mut patterns = vec![];
                    for p in v.split_ascii_whitespace() {
                        let p = p.trim();
                        if p.starts_with('!') {
                            patterns.push(Pattern::new(&p[1..], true));
                        } else {
                            patterns.push(Pattern::new(p, false));
                        }
                    }
                    patterns
                }

                if k == "include" {
                    Self::do_include(v, cwd, options, groups, loaded_files);
                    continue;
                }

                if k == "host" {
                    let patterns = parse_whitespace_pattern_list(v);
                    groups.push(MatchGroup {
                        criteria: vec![Criteria::Host(patterns)],
                        options: ConfigMap::new(),
                        context: Context::FirstPass,
                    });
                    continue;
                }

                if k == "match" {
                    let mut criteria = vec![];
                    let mut context = Context::FirstPass;

                    let mut tokens = v.split_ascii_whitespace();

                    while let Some(cname) = tokens.next() {
                        match cname.to_lowercase().as_str() {
                            "all" => {
                                criteria.push(Criteria::All);
                            }
                            "canonical" => {
                                context = Context::Canonical;
                            }
                            "final" => {
                                context = Context::Final;
                            }
                            "exec" => {
                                criteria.push(Criteria::Exec(
                                    tokens.next().unwrap_or("false").to_string(),
                                ));
                            }
                            "host" => {
                                criteria.push(Criteria::Host(parse_pattern_list(
                                    tokens.next().unwrap_or(""),
                                )));
                            }
                            "originalhost" => {
                                criteria.push(Criteria::OriginalHost(parse_pattern_list(
                                    tokens.next().unwrap_or(""),
                                )));
                            }
                            "user" => {
                                criteria.push(Criteria::User(parse_pattern_list(
                                    tokens.next().unwrap_or(""),
                                )));
                            }
                            "localuser" => {
                                criteria.push(Criteria::LocalUser(parse_pattern_list(
                                    tokens.next().unwrap_or(""),
                                )));
                            }
                            _ => break,
                        }
                    }

                    groups.push(MatchGroup {
                        criteria,
                        options: ConfigMap::new(),
                        context,
                    });
                    continue;
                }

                fn add_option(options: &mut ConfigMap, k: String, v: &str) {
                    // first option wins in ssh_config, except for identityfile
                    // which explicitly allows multiple entries to combine together
                    let is_identity_file = k == "identityfile";
                    options
                        .entry(k)
                        .and_modify(|e| {
                            if is_identity_file {
                                e.push(' ');
                                e.push_str(v);
                            }
                        })
                        .or_insert_with(|| v.to_string());
                }

                if let Some(group) = groups.last_mut() {
                    add_option(&mut group.options, k, v);
                } else {
                    add_option(options, k, v);
                }
            }
        }
    }

    /// Apply configuration values that match the specified hostname to target,
    /// but only if a given key is not already present in target, because the
    /// semantics are that the first match wins
    fn apply_matches(
        &self,
        hostname: &str,
        user: &str,
        local_user: &str,
        context: Context,
        target: &mut ConfigMap,
    ) -> bool {
        let mut needs_reparse = false;

        for (k, v) in &self.options {
            target.entry(k.to_string()).or_insert_with(|| v.to_string());
        }
        for group in &self.groups {
            if group.context != Context::FirstPass {
                needs_reparse = true;
            }
            if group.is_match(hostname, user, local_user, context) {
                for (k, v) in &group.options {
                    target.entry(k.to_string()).or_insert_with(|| v.to_string());
                }
            }
        }

        needs_reparse
    }
}
