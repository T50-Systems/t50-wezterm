use crate::scripting::guiwin::GuiWin;
use crate::spawn::SpawnWhere;
use crate::termwindow::TermWindowNotif;
use crate::TermWindow;
use ::window::*;
use anyhow::{Context, Error};
use config::keyassignment::{KeyAssignment, SpawnCommand};
use config::{ConfigSubscription, NotificationHandling};
use mux::client::ClientId;
use mux::window::WindowId as MuxWindowId;
use mux::{Mux, MuxNotification};
use promise::{Future, Promise};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use wezterm_term::{Alert, ClipboardSelection};
use wezterm_toast_notification::*;

pub struct GuiFrontEnd {
    connection: Rc<Connection>,
    switching_workspaces: RefCell<bool>,
    spawned_mux_window: RefCell<HashSet<MuxWindowId>>,
    known_windows: RefCell<BTreeMap<Window, MuxWindowId>>,
    client_id: Arc<ClientId>,
    config_subscription: RefCell<Option<ConfigSubscription>>,
}

impl Drop for GuiFrontEnd {
    fn drop(&mut self) {
        ::window::shutdown();
    }
}
