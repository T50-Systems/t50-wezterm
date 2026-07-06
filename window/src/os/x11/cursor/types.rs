use crate::os::x11::xcb_util::*;
use crate::x11::XConnection;
use crate::MouseCursor;
use anyhow::{ensure, Context};
use config::ConfigHandle;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::ffi::OsStr;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use xcb::x::Cursor;
use xcb::Xid;

// X11 classic Cursor glyphs
pub const HAND1: u16 = 58;
pub const SB_H_DOUBLE_ARROW: u16 = 108;
pub const SB_V_DOUBLE_ARROW: u16 = 116;
pub const TOP_LEFT_ARROW: u16 = 132;
pub const TOP_LEFT_CORNER: u16 = 134;
pub const XTERM: u16 = 152;

pub struct XcbCursor {
    pub id: Cursor,
    pub conn: Weak<XConnection>,
}

impl Drop for XcbCursor {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.upgrade() {
            conn.send_request_no_reply_log(&xcb::x::FreeCursor { cursor: self.id });
        }
    }
}

pub struct CursorInfo {
    cursors: HashMap<Option<MouseCursor>, XcbCursor>,
    cursor: Option<MouseCursor>,
    conn: Weak<XConnection>,
    size: Option<u32>,
    theme: Option<String>,
    icon_path: Vec<PathBuf>,
    pict_format_id: Option<xcb::render::Pictformat>,
}

fn icon_path() -> Vec<PathBuf> {
    let path = match std::env::var_os("XCURSOR_PATH") {
        Some(path) => {
            log::trace!("Using $XCURSOR_PATH icon path: {:?}", path);
            path
        }
        None => {
            log::trace!("Constructing default icon path because $XCURSOR_PATH is not set");

            fn add_icons_dir(path: &OsStr, dest: &mut Vec<PathBuf>) {
                for entry in std::env::split_paths(path) {
                    dest.push(entry.join("icons"));
                }
            }

            fn xdg_location(name: &str, def: &str, dest: &mut Vec<PathBuf>) {
                if let Some(var) = std::env::var_os(name) {
                    log::trace!("Using ${} location {:?}", name, var);
                    add_icons_dir(&var, dest);
                } else {
                    log::trace!("Using {} because ${} is not set", def, name);
                    add_icons_dir(OsStr::new(def), dest);
                }
            }

            let mut path = vec![];
            xdg_location("XDG_DATA_HOME", "~/.local/share", &mut path);
            path.push("~/.icons".into());
            xdg_location("XDG_DATA_DIRS", "/usr/local/share:/usr/share", &mut path);
            path.push("/usr/share/pixmaps".into());
            path.push("~/.cursors".into());
            path.push("/usr/share/cursors/xorg-x11".into());
            path.push("/usr/X11R6/lib/X11/icons".into());

            std::env::join_paths(path).expect("failed to compose default xcursor path")
        }
    };

    fn tilde_expand(p: PathBuf) -> PathBuf {
        match p.to_str() {
            Some(s) => {
                if s.starts_with("~/") {
                    if let Some(home) = dirs_next::home_dir() {
                        home.join(&s[2..])
                    } else {
                        p.into()
                    }
                } else {
                    p.into()
                }
            }
            None => p.into(),
        }
    }

    std::env::split_paths(&path).map(tilde_expand).collect()
}

fn cursor_size(xcursor_size: &Option<u32>, map: &HashMap<String, String>) -> u32 {
    if let Some(size) = xcursor_size {
        return *size;
    }

    if let Ok(size) = std::env::var("XCURSOR_SIZE") {
        if let Ok(size) = size.parse::<u32>() {
            return size;
        }
    }

    if let Some(size) = map.get("Xcursor.size") {
        if let Ok(size) = size.parse::<u32>() {
            return size;
        }
    }

    if let Some(dpi) = map.get("Xft.dpi") {
        if let Ok(dpi) = dpi.parse::<u32>() {
            return dpi * 16 / 72;
        }
    }

    // Probably a good default?
    24
}
