/// A Pattern in a `Host` list
#[derive(Debug, PartialEq, Eq, Clone)]
struct Pattern {
    negated: bool,
    pattern: String,
    original: String,
    is_literal: bool,
}

/// Compile a glob style pattern string into a regex pattern string
fn wildcard_to_pattern(s: &str) -> (String, bool) {
    let mut pattern = String::new();
    let mut is_literal = true;
    pattern.push('^');
    for c in s.chars() {
        if c == '*' {
            pattern.push_str(".*");
            is_literal = false;
        } else if c == '?' {
            pattern.push('.');
            is_literal = false;
        } else {
            let s = regex::escape(&c.to_string());
            pattern.push_str(&s);
        }
    }
    pattern.push('$');
    (pattern, is_literal)
}

impl Pattern {
    /// Returns true if this pattern matches the provided hostname
    fn match_text(&self, hostname: &str) -> bool {
        if let Ok(re) = Regex::new(&self.pattern) {
            re.is_match(hostname)
        } else {
            false
        }
    }

    fn new(text: &str, negated: bool) -> Self {
        let (pattern, is_literal) = wildcard_to_pattern(text);
        Self {
            pattern,
            is_literal,
            negated,
            original: text.to_string(),
        }
    }

    /// Returns true if hostname matches the
    /// condition specified by a list of patterns
    fn match_group(hostname: &str, patterns: &[Self]) -> bool {
        for pat in patterns {
            if pat.match_text(hostname) {
                // We got a definitive name match.
                // If it was an exlusion then we've been told
                // that this doesn't really match, otherwise
                // we got one that we were looking for
                return !pat.negated;
            }
        }
        false
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
enum Criteria {
    Host(Vec<Pattern>),
    Exec(String),
    OriginalHost(Vec<Pattern>),
    User(Vec<Pattern>),
    LocalUser(Vec<Pattern>),
    All,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Context {
    FirstPass,
    Canonical,
    Final,
}

/// Represents `Host pattern,list` stanza in the config,
/// and the options that it logically contains
#[derive(Debug, PartialEq, Eq, Clone)]
struct MatchGroup {
    criteria: Vec<Criteria>,
    context: Context,
    options: ConfigMap,
}

impl MatchGroup {
    fn is_match(&self, hostname: &str, user: &str, local_user: &str, context: Context) -> bool {
        if self.context != context {
            return false;
        }
        for c in &self.criteria {
            match c {
                Criteria::Host(patterns) => {
                    if !Pattern::match_group(hostname, patterns) {
                        return false;
                    }
                }
                Criteria::Exec(_) => {
                    log::warn!("Match Exec is not implemented");
                }
                Criteria::OriginalHost(patterns) => {
                    if !Pattern::match_group(hostname, patterns) {
                        return false;
                    }
                }
                Criteria::User(patterns) => {
                    if !Pattern::match_group(user, patterns) {
                        return false;
                    }
                }
                Criteria::LocalUser(patterns) => {
                    if !Pattern::match_group(local_user, patterns) {
                        return false;
                    }
                }
                Criteria::All => {
                    // Always matches
                }
            }
        }
        true
    }
}
