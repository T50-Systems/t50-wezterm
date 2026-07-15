//! encode and decode the frames for the mux protocol.
//! The frames include the length of a PDU as well as an identifier
//! that informs us how to decode it.  The length, ident and serial
//! number are encoded using a variable length integer encoding.
//! Rather than rely solely on serde to serialize and deserialize an
//! enum, we encode the enum variants with a version/identifier tag
//! for ourselves.  This will make it a little easier to manage
//! client and server instances that are built from different versions
//! of this code; in this way the client and server can more gracefully
//! manage unknown enum variants.
#![allow(dead_code)]
#![allow(clippy::range_plus_one)]

use anyhow::{bail, Context as _, Error};
use config::keyassignment::{PaneDirection, ScrollbackEraseMode};
use portable_pty::CommandBuilder;
use rangeset::*;
use serde::{Deserialize, Serialize};
use smol::io::AsyncWriteExt;
use smol::prelude::*;
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Cursor;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;
use termwiz::hyperlink::Hyperlink;
use termwiz::image::{ImageData, TextureCoordinate};
use termwiz::surface::{Line, SequenceNo};
use thiserror::Error;
use wezterm_mux_protocol::client::{ClientId, ClientInfo};
use wezterm_mux_protocol::pane::{PaneId, Pattern, SearchResult};
use wezterm_mux_protocol::renderable::{RenderableDimensions, StableCursorPosition};
use wezterm_mux_protocol::tab::{PaneNode, SerdeUrl, SplitRequest, TabId};
use wezterm_mux_protocol::window::WindowId;
use wezterm_term_api::color::ColorPalette;
use wezterm_term_api::{Alert, ClipboardSelection, StableRowIndex, TerminalSize};

include!("lib/raw.rs");
include!("lib/pdu.rs");
include!("lib/messages_01.rs");
include!("lib/messages_02.rs");
include!("lib/tests.rs");
