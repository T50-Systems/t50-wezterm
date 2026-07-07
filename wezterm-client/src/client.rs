use crate::domain::{ClientDomain, ClientDomainConfig};
use crate::pane::ClientPane;
use anyhow::{anyhow, bail, Context};
use async_ossl::AsyncSslStream;
use async_trait::async_trait;
use codec::*;
use config::{configuration, SshDomain, TlsDomainClient, UnixDomain, UnixTarget};
use filedescriptor::FileDescriptor;
use futures::FutureExt;
use mux::client::ClientId;
use mux::connui::ConnectionUI;
use mux::domain::DomainId;
use mux::pane::PaneId;
use mux::ssh::ssh_connect_with_ui;
use mux::Mux;
use openssl::ssl::{SslConnector, SslFiletype, SslMethod};
use openssl::x509::X509;
use portable_pty::Child;
use smol::channel::{bounded, unbounded, Receiver, Sender};
use smol::prelude::*;
use smol::{block_on, Async};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::marker::Unpin;
use std::net::TcpStream;
#[cfg(unix)]
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(windows)]
use std::os::windows::io::{AsRawSocket, AsSocket, BorrowedSocket, RawSocket};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use thiserror::Error;
use wezterm_uds::UnixStream;

#[derive(Error, Debug)]
#[error("Timeout")]
struct Timeout;

#[derive(Error, Debug)]
#[error("ChannelSendError")]
struct ChannelSendError;

enum ReaderMessage {
    SendPdu {
        pdu: Pdu,
        promise: Sender<anyhow::Result<Pdu>>,
    },
    Readable,
}

#[derive(Clone)]
pub struct Client {
    sender: Sender<ReaderMessage>,
    local_domain_id: Option<DomainId>,
    pub client_id: ClientId,
    client_domain_config: ClientDomainConfig,
    pub is_reconnectable: bool,
    pub is_local: bool,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error(
    "Please install the same version of wezterm on both the client and server!\n\
     The server version is {} (codec version {}),\n\
     which is not compatible with our version \n\
     {} (codec version {}).",
    version,
    codec_vers,
    config::wezterm_version(),
    CODEC_VERSION
)]
pub struct IncompatibleVersionError {
    pub version: String,
    pub codec_vers: usize,
}

macro_rules! rpc {
    ($method_name:ident, $request_type:ident, $response_type:ident) => {
        pub async fn $method_name(&self, pdu: $request_type) -> anyhow::Result<$response_type> {
            let start = std::time::Instant::now();
            let result = self.send_pdu(Pdu::$request_type(pdu)).await;
            let elapsed = start.elapsed();
            metrics::histogram!("rpc", "method" => stringify!($method_name)).record(elapsed);
            metrics::counter!("rpc.count", "method" => stringify!($method_name)).increment(1);
            match result {
                Ok(Pdu::$response_type(res)) => Ok(res),
                Ok(_) => bail!("unexpected response {:?}", result),
                Err(err) => Err(err),
            }
        }
    };

    // This variant allows omitting the request parameter; this is useful
    // in the case where the struct is empty and present only for the purpose
    // of typing the request.
    ($method_name:ident, $request_type:ident=(), $response_type:ident) => {
        #[allow(dead_code)]
        pub async fn $method_name(&self) -> anyhow::Result<$response_type> {
            let start = std::time::Instant::now();
            let result = self.send_pdu(Pdu::$request_type($request_type{})).await;
            let elapsed = start.elapsed();
            metrics::histogram!("rpc", "method" => stringify!($method_name)).record(elapsed);
            metrics::counter!("rpc.count", "method" => stringify!($method_name)).increment(1);
            match result {
                Ok(Pdu::$response_type(res)) => Ok(res),
                Ok(_) => bail!("unexpected response {:?}", result),
                Err(err) => Err(err),
            }
        }
    };
}

include!("client/unilateral.rs");
include!("client/thread.rs");
include!("client/streams.rs");
include!("client/reconnectable.rs");
include!("client/client_impl.rs");
