//! Parse an ssh_config(5) formatted config file
use regex::{Captures, Regex};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub type ConfigMap = BTreeMap<String, String>;

include!("config/patterns.rs");
include!("config/parsed.rs");
include!("config/config_impl.rs");

#[cfg(test)]
mod test {
    use super::*;
    use k9::snapshot;

    include!("config/test_basic.rs");
    include!("config/test_tokens.rs");
    include!("config/test_match.rs");
    include!("config/test_simple.rs");
}
