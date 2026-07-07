use crate::connui::ConnectionUI;
use crate::domain::{alloc_domain_id, Domain, DomainId, DomainState, WriterWrapper};
use crate::localpane::LocalPane;
use crate::pane::{alloc_pane_id, Pane, PaneId};
use crate::Mux;
use anyhow::{anyhow, bail, Context};
use async_trait::async_trait;
use config::{Shell, SshBackend, SshDomain};
use filedescriptor::{poll, pollfd, socketpair, AsRawSocketDescriptor, FileDescriptor, POLLIN};
use portable_pty::cmdbuilder::CommandBuilder;
use portable_pty::{ChildKiller, ExitStatus, MasterPty, PtySize};
use smol::channel::{bounded, Receiver as AsyncReceiver};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::io::{BufWriter, Read, Write};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use termwiz::cell::{unicode_column_width, AttributeChange, Intensity};
use termwiz::input::{InputEvent, InputParser};
use termwiz::lineedit::*;
use termwiz::render::terminfo::TerminfoRenderer;
use termwiz::surface::{Change, LineAttribute};
use termwiz::terminal::{ScreenSize, Terminal, TerminalWaker};
use wezterm_ssh::{
    ConfigMap, HostVerificationFailed, Session, SessionEvent, SshChildProcess, SshPty,
};
use wezterm_term::TerminalSize;

include!("ssh/ui.rs");
include!("ssh/domain.rs");
include!("ssh/connect.rs");
include!("ssh/domain_impl.rs");
include!("ssh/child.rs");
include!("ssh/pty.rs");
