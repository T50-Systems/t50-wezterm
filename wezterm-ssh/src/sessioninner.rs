use crate::channelwrap::ChannelWrap;
use crate::config::ConfigMap;
use crate::dirwrap::DirWrap;
use crate::filewrap::FileWrap;
use crate::pty::*;
use crate::session::{Exec, ExecResult, SessionEvent, SessionRequest, SignalChannel};
use crate::sessionwrap::SessionWrap;
use crate::sftp::dir::{Dir, DirId, DirRequest};
use crate::sftp::file::{File, FileId, FileRequest};
use crate::sftp::{OpenWithMode, SftpChannelResult, SftpRequest};
use crate::sftpwrap::SftpWrap;
use anyhow::{anyhow, Context};
use camino::Utf8PathBuf;
use filedescriptor::{
    poll, pollfd, socketpair, AsRawSocketDescriptor, FileDescriptor, POLLIN, POLLOUT,
};
use portable_pty::ExitStatus;
use smol::channel::{bounded, Receiver, Sender, TryRecvError};
use socket2::{Domain, Socket, Type};
use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::net::ToSocketAddrs;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(crate) struct DescriptorState {
    pub fd: Option<FileDescriptor>,
    pub buf: VecDeque<u8>,
}

pub(crate) struct ChannelInfo {
    pub channel_id: ChannelId,
    pub channel: ChannelWrap,
    pub exit: Option<Sender<ExitStatus>>,
    pub exited: bool,
    pub descriptors: [DescriptorState; 3],
}

pub(crate) type ChannelId = usize;

pub(crate) struct SessionInner {
    pub config: ConfigMap,
    pub tx_event: Sender<SessionEvent>,
    pub rx_req: Receiver<SessionRequest>,
    pub channels: HashMap<ChannelId, ChannelInfo>,
    pub files: HashMap<FileId, FileWrap>,
    pub dirs: HashMap<DirId, DirWrap>,
    pub next_channel_id: ChannelId,
    pub next_file_id: FileId,
    pub sender_read: FileDescriptor,
    pub session_was_dropped: bool,
    pub shown_accept_env_error: bool,
    pub last_keep_alive: Instant,
    pub keep_alive: Option<Duration>,
}

impl Drop for SessionInner {
    fn drop(&mut self) {
        log::trace!("Dropping SessionInner");
    }
}

include!("sessioninner/run.rs");
include!("sessioninner/connect.rs");
include!("sessioninner/loop_io.rs");
include!("sessioninner/dispatch.rs");
include!("sessioninner/ops.rs");
include!("sessioninner/helpers.rs");
