use super::*;
use crate::bitmaps::*;
use crate::connection::ConnectionOps;
use crate::os::{xkeysyms, Connection, Window};
use crate::{
    Appearance, Clipboard, DeadKeyStatus, Dimensions, MouseButtons, MouseCursor, MouseEvent,
    MouseEventKind, MousePress, Point, Rect, RequestedWindowGeometry, ResizeIncrement,
    ResolvedGeometry, ScreenPoint, ScreenRect, WindowDecorations, WindowEvent, WindowEventSender,
    WindowOps, WindowState,
};
use anyhow::{anyhow, Context as _};
use async_trait::async_trait;
use config::ConfigHandle;
use promise::{Future, Promise};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle, XcbDisplayHandle, XcbWindowHandle,
};
use std::any::Any;
use std::convert::TryInto;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};
use url::Url;
use wezterm_font::FontConfiguration;
use wezterm_input_types::{KeyCode, KeyEvent, KeyboardLedStatus, Modifiers};
use xcb::x::{Atom, PropMode};
use xcb::{Event, Xid};

#[derive(Default)]
struct CopyAndPaste {
    clipboard_owned: Option<String>,
    primary_selection_owned: Option<String>,
    clipboard_request: Option<Promise<String>>,
    selection_request: Option<Promise<String>>,
    time: u32,
}

impl CopyAndPaste {
    fn clipboard(&self, clipboard: Clipboard) -> &Option<String> {
        match clipboard {
            Clipboard::PrimarySelection => &self.primary_selection_owned,
            Clipboard::Clipboard => &self.clipboard_owned,
        }
    }

    fn clipboard_mut(&mut self, clipboard: Clipboard) -> &mut Option<String> {
        match clipboard {
            Clipboard::PrimarySelection => &mut self.primary_selection_owned,
            Clipboard::Clipboard => &mut self.clipboard_owned,
        }
    }

    fn request_mut(&mut self, clipboard: Clipboard) -> &mut Option<Promise<String>> {
        match clipboard {
            Clipboard::PrimarySelection => &mut self.selection_request,
            Clipboard::Clipboard => &mut self.clipboard_request,
        }
    }
}

struct DragAndDrop {
    src_window: Option<xcb::x::Window>,
    src_types: Vec<Atom>,
    src_action: Atom,
    time: u32,
    target_type: Atom,
    target_action: Atom,
}

impl Default for DragAndDrop {
    fn default() -> DragAndDrop {
        DragAndDrop {
            src_window: None,
            src_types: Vec::new(),
            src_action: xcb::x::ATOM_NONE,
            time: 0,
            target_type: xcb::x::ATOM_NONE,
            target_action: xcb::x::ATOM_NONE,
        }
    }
}

pub(crate) struct XWindowInner {
    pub window_id: xcb::x::Window,
    pub child_id: xcb::x::Window,
    conn: Weak<XConnection>,
    pub events: WindowEventSender,
    width: u16,
    height: u16,
    last_wm_state: WindowState,
    dpi: f64,
    cursors: CursorInfo,
    copy_and_paste: CopyAndPaste,
    drag_and_drop: DragAndDrop,
    config: ConfigHandle,
    appearance: Appearance,
    title: String,
    pub has_focus: Option<bool>,
    verify_focus: bool,
    last_cursor_position: Rect,
    invalidated: bool,
    paint_throttled: bool,
    pending: Vec<WindowEvent>,
    sure_about_geometry: bool,
    current_mouse_event: Option<MouseEvent>,
    window_drag_position: Option<ScreenPoint>,
    dragging: bool,
    outstanding_configure_requests: usize,
    pending_finished_resizes: usize,
}

/// <https://specifications.freedesktop.org/wm-spec/wm-spec-latest.html#idm46409506331616>
const _NET_WM_MOVERESIZE_MOVE: u32 = 8;
const _NET_WM_MOVERESIZE_CANCEL: u32 = 11;

impl Drop for XWindowInner {
    fn drop(&mut self) {
        if self.window_id != xcb::x::Window::none() {
            if let Some(conn) = self.conn.upgrade() {
                self.conn()
                    .conn()
                    .flush()
                    .context("flush pending requests prior to issuing DestroyWindow")
                    .ok();
                conn.send_request_no_reply_log(&xcb::x::DestroyWindow {
                    window: self.child_id,
                });
                conn.send_request_no_reply_log(&xcb::x::DestroyWindow {
                    window: self.window_id,
                });
            }
        }
    }
}

impl HasDisplayHandle for XWindowInner {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        if let Some(conn) = self.conn.upgrade() {
            let handle =
                XcbDisplayHandle::new(NonNull::new(conn.conn.get_raw_conn() as _), conn.screen_num);
            unsafe { Ok(DisplayHandle::borrow_raw(RawDisplayHandle::Xcb(handle))) }
        } else {
            Err(HandleError::Unavailable)
        }
    }
}

impl HasWindowHandle for XWindowInner {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let mut handle =
            XcbWindowHandle::new(NonZeroU32::new(self.child_id.resource_id()).expect("non-zero"));
        handle.visual_id = NonZeroU32::new(self.conn.upgrade().unwrap().visual.visual_id());
        unsafe { Ok(WindowHandle::borrow_raw(RawWindowHandle::Xcb(handle))) }
    }
}

impl XWindowInner {
    include!("window/inner_core.rs");
    include!("window/inner_events.rs");
    include!("window/inner_selection.rs");
}

include!("window/window_construct.rs");
include!("window/inner_ops.rs");
include!("window/window_ops.rs");
include!("window/tail.rs");
