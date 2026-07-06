//! # Terminal Capabilities
//!
//! There are a few problems in the world of terminal capability
//! detection today; the terminal is typically local to a machine
//! running very up to date software, but applications may be running
//! on a remote system with relatively stale system software, or
//! even different operating systems.
//!
//! There are two well-specified standard approaches to querying terminal
//! capabilities: `termcap` and `terminfo`.  The databases for the
//! capabilities are typically deployed with the operating system
//! software and suffers from splay and freshness issues: they may
//! be different on different systems and out of date.
//!
//! A further complication is that the terminal may be connected
//! via a series of intermediaries or multiplexers (mosh, tmux, screen)
//! which hides or perturbs the true capabilities of the terminal.
//!
//! `terminfo` and `termcap` both allow for the user to supply a local
//! override.  `terminfo` needs a locally available compiled database,
//! and that database may have different binary representations on disk
//! depending on the operating system software, making it difficult for
//! users that need to login to many remote machines; the burden is on
//! the user to configure a local profile on many machines or solve the
//! challenge of having a `$HOME/.terminfo` directory that is NFS mountable
//! and readable to all potential machines.
//!
//! `termcap` defines a `TERMCAP` environment variable that can contain
//! overrides and be passed via SSH to the remote systems much more simply
//! than the `terminfo` database, but `termcap` is the old obsolete database
//! and lacks a way to express support for the newer, more interesting
//! features.
//!
//! It's a bit of a mess.
//!
//! There's more: `slang` defined the concept of `COLORTERM` to workaround
//! some of these concerns so that it was possible to influence the
//! availability of relatively recent higher color palette extensions.
//! This allows the user the ability to "know better" about their terminal
//! than the local configuration allows.  `COLORTERM` has the advantage of
//! being able to be passed on to remote systems via SSH.
//!
//! Regarding environment variables: on macOS the two main terminal
//! emulators export `TERM_PROGRAM` and `TERM_PROGRAM_VERSION` into the
//! environment, and if this were transported via SSH and adopted by
//! more terminal emulators then newer software could potentially also
//! look at this information too.
//!
//! Is there or will there ever be an ideal solution to this stuff?
//! Probably not.
//!
//! With all this in mind, this module presents a `Capabilities` struct
//! that holds information about a terminal.   The `new_from_env` method
//! implements some heuristics (a fancy word for guessing) to compute
//! the terminal capabilities, but also offers a `ProbeHints`
//! that can be used by the embedding application to override those choices.
use crate::{builder, Result};
use std::env::var;
use terminfo::{self, capability as cap};

pub mod probed;

/// Environment variable name indicating that color output should be disabled.
/// See <https://no-color.org>
const NO_COLOR_ENV: &str = "NO_COLOR";

include!("root/types.rs");
include!("root/capabilities.rs");
include!("root/helpers.rs");
include!("root/tests.rs");
