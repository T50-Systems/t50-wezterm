use anyhow::{anyhow, Context};
use clap::builder::ValueParser;
use clap::{Parser, ValueEnum, ValueHint};
use clap_complete::{generate as generate_completion, shells, Generator as CompletionGenerator};
use config::{wezterm_version, ConfigHandle};
use mux::Mux;
use std::ffi::OsString;
use std::io::Read;
use termwiz::caps::Capabilities;
use termwiz::escape::esc::{Esc, EscCode};
use termwiz::escape::OneBased;
use termwiz::input::{InputEvent, KeyCode, KeyEvent, Modifiers};
use termwiz::surface::change::Change;
use termwiz::surface::Position;
use termwiz::terminal::{ScreenSize, Terminal};
use umask::UmaskSaver;
use wezterm_gui_subcommands::*;

mod asciicast;
mod cli;

//    let message = "; ❤ 😍🤢\n\x1b[91;mw00t\n\x1b[37;104;m bleet\x1b[0;m.";

include!("root/opts.rs");
include!("root/imgcat.rs");
include!("root/commands.rs");
include!("root/main_run.rs");
