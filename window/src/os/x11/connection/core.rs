use super::keyboard::{Keyboard, KeyboardWithFallback};
use crate::connection::ConnectionOps;
use crate::os::x11::window::XWindowInner;
use crate::os::x11::xsettings::*;
use crate::os::Connection;
use crate::screen::{ScreenInfo, Screens};
use crate::spawn::*;
use crate::{Appearance, DeadKeyStatus, ScreenRect};
use anyhow::{anyhow, bail, Context as _};
use mio::event::Source;
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Registry, Token};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::os::unix::io::AsRawFd;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use x11::xlib;
use xcb::x::Atom;
use xcb::{dri2, Raw, Xid};

enum ScreenResources {
    Current(xcb::randr::GetScreenResourcesCurrentReply),
    All(xcb::randr::GetScreenResourcesReply),
}

impl ScreenResources {
    fn outputs(&self) -> &[xcb::randr::Output] {
        match self {
            Self::Current(cur) => cur.outputs(),
            Self::All(all) => all.outputs(),
        }
    }

    fn config_timestamp(&self) -> xcb::x::Timestamp {
        match self {
            Self::Current(cur) => cur.config_timestamp(),
            Self::All(all) => all.config_timestamp(),
        }
    }

    pub fn modes(&self) -> &[xcb::randr::ModeInfo] {
        match self {
            Self::Current(cur) => cur.modes(),
            Self::All(all) => all.modes(),
        }
    }
}

pub struct XConnection {
    pub conn: xcb::Connection,
    default_dpi: RefCell<f64>,
    pub(crate) xsettings: RefCell<XSettingsMap>,
    pub screen_num: i32,
    pub root: xcb::x::Window,
    pub keyboard: KeyboardWithFallback,
    pub kbd_ev: u8,
    pub atom_protocols: Atom,
    pub cursor_font_id: xcb::x::Font,
    pub atom_delete: Atom,
    pub atom_utf8_string: Atom,
    pub atom_xsel_data: Atom,
    pub atom_targets: Atom,
    pub atom_clipboard: Atom,
    pub atom_texturilist: Atom,
    pub atom_xmozurl: Atom,
    pub atom_xdndaware: Atom,
    pub atom_xdndtypelist: Atom,
    pub atom_xdndselection: Atom,
    pub atom_xdndenter: Atom,
    pub atom_xdndposition: Atom,
    pub atom_xdndstatus: Atom,
    pub atom_xdndleave: Atom,
    pub atom_xdnddrop: Atom,
    pub atom_xdndfinished: Atom,
    pub atom_xdndactioncopy: Atom,
    pub atom_xdndactionmove: Atom,
    pub atom_xdndactionlink: Atom,
    pub atom_xdndactionask: Atom,
    pub atom_xdndactionprivate: Atom,
    pub atom_gtk_edge_constraints: Atom,
    pub atom_xsettings_selection: Atom,
    pub atom_xsettings_settings: Atom,
    pub atom_manager: Atom,
    pub atom_state_maximized_vert: Atom,
    pub atom_state_maximized_horz: Atom,
    pub atom_state_hidden: Atom,
    pub atom_state_fullscreen: Atom,
    pub atom_net_wm_state: Atom,
    pub atom_motif_wm_hints: Atom,
    pub atom_net_wm_pid: Atom,
    pub atom_net_wm_name: Atom,
    pub atom_net_wm_icon: Atom,
    pub atom_net_move_resize_window: Atom,
    pub atom_net_wm_moveresize: Atom,
    pub atom_net_supported: Atom,
    pub atom_net_supporting_wm_check: Atom,
    pub atom_net_active_window: Atom,
    pub(crate) xrm: RefCell<HashMap<String, String>>,
    pub(crate) windows: RefCell<HashMap<xcb::x::Window, Arc<Mutex<XWindowInner>>>>,
    pub(crate) child_to_parent_id: RefCell<HashMap<xcb::x::Window, xcb::x::Window>>,
    should_terminate: RefCell<bool>,
    pub(crate) visual: xcb::x::Visualtype,
    pub(crate) depth: u8,
    pub(crate) gl_connection: RefCell<Option<Rc<crate::egl::GlConnection>>>,
    pub(crate) ime: RefCell<std::pin::Pin<Box<xcb_imdkit::ImeClient>>>,
    pub(crate) ime_process_event_result: RefCell<anyhow::Result<()>>,
    pub(crate) has_randr: bool,
    pub(crate) atom_names: RefCell<HashMap<Atom, String>>,
    pub(crate) supported: RefCell<HashSet<Atom>>,
    pub(crate) screens: RefCell<Option<Screens>>,
}

impl std::ops::Deref for XConnection {
    type Target = xcb::Connection;

    fn deref(&self) -> &xcb::Connection {
        &self.conn
    }
}

impl Source for XConnection {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interest: Interest,
    ) -> std::io::Result<()> {
        SourceFd(&self.conn.as_raw_fd()).register(registry, token, interest)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interest: Interest,
    ) -> std::io::Result<()> {
        SourceFd(&self.conn.as_raw_fd()).reregister(registry, token, interest)
    }

    fn deregister(&mut self, registry: &Registry) -> std::io::Result<()> {
        SourceFd(&self.conn.as_raw_fd()).deregister(registry)
    }
}

fn window_id_from_event(event: &xcb::Event) -> Option<xcb::x::Window> {
    match event {
        xcb::Event::X(xcb::x::Event::Expose(e)) => Some(e.window()),
        xcb::Event::X(xcb::x::Event::ConfigureNotify(e)) => Some(e.window()),
        xcb::Event::Present(xcb::present::Event::ConfigureNotify(e)) => Some(e.window()),
        xcb::Event::X(xcb::x::Event::KeyPress(e)) => Some(e.event()),
        xcb::Event::X(xcb::x::Event::KeyRelease(e)) => Some(e.event()),
        xcb::Event::X(xcb::x::Event::MotionNotify(e)) => Some(e.event()),
        xcb::Event::X(xcb::x::Event::ButtonPress(e)) => Some(e.event()),
        xcb::Event::X(xcb::x::Event::ButtonRelease(e)) => Some(e.event()),
        xcb::Event::X(xcb::x::Event::ClientMessage(e)) => Some(e.window()),
        xcb::Event::X(xcb::x::Event::DestroyNotify(e)) => Some(e.window()),
        xcb::Event::X(xcb::x::Event::SelectionClear(e)) => Some(e.owner()),
        xcb::Event::X(xcb::x::Event::SelectionNotify(e)) => Some(e.requestor()),
        xcb::Event::X(xcb::x::Event::SelectionRequest(e)) => Some(e.owner()),
        xcb::Event::X(xcb::x::Event::PropertyNotify(e)) => Some(e.window()),
        xcb::Event::X(xcb::x::Event::FocusIn(e)) => Some(e.event()),
        xcb::Event::X(xcb::x::Event::FocusOut(e)) => Some(e.event()),
        xcb::Event::X(xcb::x::Event::LeaveNotify(e)) => Some(e.event()),
        _ => None,
    }
}

/// Returns the name of the window manager
fn get_wm_name(
    conn: &xcb::Connection,
    root: xcb::x::Window,
    atom_net_supporting_wm_check: Atom,
    atom_net_wm_name: Atom,
    atom_utf8_string: Atom,
) -> anyhow::Result<String> {
    let reply = conn
        .wait_for_reply(conn.send_request(&xcb::x::GetProperty {
            delete: false,
            window: root,
            property: atom_net_supporting_wm_check,
            r#type: xcb::x::ATOM_WINDOW,
            long_offset: 0,
            long_length: 4,
        }))
        .context("GetProperty _NET_SUPPORTING_WM_CHECK")?;

    let wm_window = match reply.value::<xcb::x::Window>().get(0) {
        Some(w) => *w,
        None => anyhow::bail!("empty list of windows"),
    };

    let reply = conn
        .wait_for_reply(conn.send_request(&xcb::x::GetProperty {
            delete: false,
            window: wm_window,
            property: atom_net_wm_name,
            r#type: atom_utf8_string,
            long_offset: 0,
            long_length: 1024,
        }))
        .context("GetProperty _NET_WM_NAME from window manager")?;
    Ok(String::from_utf8_lossy(reply.value::<u8>()).to_string())
