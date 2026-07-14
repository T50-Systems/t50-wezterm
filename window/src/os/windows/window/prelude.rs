use super::*;
use crate::connection::ConnectionOps;
use crate::parameters::{self, Parameters};
use crate::{
    Appearance, Clipboard, DeadKeyStatus, Dimensions, Handled, KeyCode, KeyEvent, Modifiers,
    MouseButtons, MouseCursor, MouseEvent, MouseEventKind, MousePress, Point, RawKeyEvent, Rect,
    RequestedWindowGeometry, ResolvedGeometry, ScreenPoint, ScreenRect, ULength, WindowDecorations,
    WindowEvent, WindowEventSender, WindowOps, WindowState,
};
use anyhow::{bail, Context};
use async_trait::async_trait;
use config::{ConfigHandle, ImePreeditRendering, SystemBackdrop};
use lazy_static::lazy_static;
use promise::Future;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, Win32WindowHandle, WindowHandle, WindowsDisplayHandle,
};
use shared_library::shared_library;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::ffi::OsString;
use std::io::{self, Error as IoError};
use std::num::NonZeroIsize;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::rc::Rc;
use std::sync::Mutex;
use wezterm_color_types::LinearRgba;
use wezterm_font::FontConfiguration;
use wezterm_input_types::KeyboardLedStatus;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::*;
use winapi::shared::windef::*;
use winapi::shared::winerror::S_OK;
use winapi::um::imm::*;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::shellapi::{DragAcceptFiles, DragFinish, DragQueryFileW, HDROP};
use winapi::um::shellscalingapi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use winapi::um::sysinfoapi::{GetTickCount, GetVersionExW};
use winapi::um::uxtheme::{
    CloseThemeData, GetThemeFont, GetThemeSysFont, OpenThemeData, SetWindowTheme,
};
use winapi::um::wingdi::{LOGFONTW, MAKEPOINTS};
use winapi::um::winnt::OSVERSIONINFOW;
use winapi::um::winuser::*;
use windows::UI::Color as WUIColor;
use windows::UI::ViewManagement::{UIColorType, UISettings};
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

const GCS_RESULTSTR: DWORD = 0x800;
const GCS_COMPSTR: DWORD = 0x8;
const ISC_SHOWUICOMPOSITIONWINDOW: DWORD = 0x80000000;

#[allow(non_snake_case)]
#[repr(C)]
pub struct CANDIDATEFORM {
    dwIndex: DWORD,
    dwStyle: DWORD,
    ptCurrentPos: POINT,
    rcArea: RECT,
}
pub type LPCANDIDATEFORM = *mut CANDIDATEFORM;

extern "system" {
    pub fn ImmGetCompositionStringW(himc: HIMC, index: DWORD, buf: LPVOID, buflen: DWORD) -> LONG;
    pub fn ImmSetCandidateWindow(himc: HIMC, lpCandidate: LPCANDIDATEFORM) -> BOOL;
}

lazy_static! {
    static ref IS_WIN10: bool = {
        let osver = OSVERSIONINFOW {
            dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as _,
            ..Default::default()
        };

        if unsafe { GetVersionExW(&osver as *const _ as _) } == winapi::shared::minwindef::TRUE {
            osver.dwBuildNumber < 22000
        } else {
            true
        }
    };
    static ref IS_WIN11_22H2: bool = {
        let osver = OSVERSIONINFOW {
            dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as _,
            ..Default::default()
        };

        if unsafe { GetVersionExW(&osver as *const _ as _) } == winapi::shared::minwindef::TRUE {
            osver.dwBuildNumber >= 22621
        } else {
            true
        }
    };
    static ref TITLE_FONT: Mutex<Option<parameters::FontAndSize>> = Mutex::new(None);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub(crate) struct HWindow(HWND);
unsafe impl Send for HWindow {}
unsafe impl Sync for HWindow {}

pub(crate) struct WindowInner {
    /// Non-owning reference to the window handle
    hwnd: HWindow,
    events: WindowEventSender,
    gl_state: Option<Rc<glium::backend::Context>>,
    /// Fraction of mouse scroll
    hscroll_remainder: i16,
    vscroll_remainder: i16,

    last_size: Option<Dimensions>,
    in_size_move: bool,
    dead_pending: Option<(Modifiers, u32)>,
    saved_placement: Option<WINDOWPLACEMENT>,
    track_mouse_leave: bool,
    window_drag_position: Option<ScreenPoint>,
    maximize_button_position: Option<ScreenRect>,

    keyboard_info: KeyboardLayoutInfo,
    appearance: Appearance,

    config: ConfigHandle,
    paint_throttled: bool,
    invalidated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Window(HWindow);

fn wuicolor_to_linearrgba(color: WUIColor) -> LinearRgba {
    LinearRgba::with_srgba(color.R, color.G, color.B, 255)
}

fn rect_width(r: &RECT) -> i32 {
    r.right - r.left
}

fn rect_height(r: &RECT) -> i32 {
    r.bottom - r.top
}

fn adjust_client_to_window_dimensions(
    style: u32,
    width: usize,
    height: usize,
    dpi: u32,
) -> (i32, i32) {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: width as _,
        bottom: height as _,
    };
    unsafe { AdjustWindowRectExForDpi(&mut rect, style, 0, 0, dpi) };

    (rect_width(&rect), rect_height(&rect))
}

fn rc_to_pointer(arc: &Rc<RefCell<WindowInner>>) -> *const RefCell<WindowInner> {
    let cloned = Rc::clone(arc);
    Rc::into_raw(cloned)
}

fn rc_from_pointer(lparam: LPVOID) -> Rc<RefCell<WindowInner>> {
    // Turn it into an Rc
    let arc = unsafe { Rc::from_raw(std::mem::transmute(lparam)) };
    // Add a ref for the caller
    let cloned = Rc::clone(&arc);

    // We must not drop this ref though; turn it back into a raw pointer!
    let _ = Rc::into_raw(arc);

    cloned
}

fn rc_from_hwnd(hwnd: HWND) -> Option<Rc<RefCell<WindowInner>>> {
    let raw = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as LPVOID };
    if raw.is_null() {
        None
    } else {
        Some(rc_from_pointer(raw))
    }
}

fn take_rc_from_pointer(lparam: LPVOID) -> Rc<RefCell<WindowInner>> {
    unsafe { Rc::from_raw(std::mem::transmute(lparam)) }
}

fn callback_behavior() -> glium::debug::DebugCallbackBehavior {
    if cfg!(debug_assertions) && false
    /* https://github.com/glium/glium/issues/1885 */
    {
        glium::debug::DebugCallbackBehavior::DebugMessageOnError
    } else {
        glium::debug::DebugCallbackBehavior::Ignore
    }
}

impl HasDisplayHandle for WindowInner {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        unsafe {
            Ok(DisplayHandle::borrow_raw(RawDisplayHandle::Windows(
                WindowsDisplayHandle::new(),
            )))
        }
    }
}

impl HasWindowHandle for WindowInner {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let mut handle =
            Win32WindowHandle::new(NonZeroIsize::new(self.hwnd.0 as _).expect("non-zero"));
        handle.hinstance = NonZeroIsize::new(unsafe { GetModuleHandleW(null()) } as _);
        unsafe { Ok(WindowHandle::borrow_raw(RawWindowHandle::Win32(handle))) }
    }
}

impl WindowInner {
    fn enable_opengl(&mut self) -> anyhow::Result<Rc<glium::backend::Context>> {
        let conn = Connection::get().unwrap();

        let gl_state = if self.config.prefer_egl {
            match conn.gl_connection.borrow().as_ref() {
                None => crate::egl::GlState::create(None, self.hwnd.0),
                Some(glconn) => {
                    crate::egl::GlState::create_with_existing_connection(glconn, self.hwnd.0)
                }
            }
        } else {
            Err(anyhow::anyhow!("Config says to avoid EGL"))
        }
        .and_then(|egl| unsafe {
            log::trace!("Initialized EGL!");
            conn.gl_connection
                .borrow_mut()
                .replace(Rc::clone(egl.get_connection()));
            let backend = Rc::new(egl);
            Ok(glium::backend::Context::new(
                backend,
                true,
                callback_behavior(),
            )?)
        })
        .or_else(|err| {
            log::trace!("EGL init failed {:?}, fall back to WGL", err);
            super::wgl::GlState::create(self.hwnd.0).and_then(|state| unsafe {
                Ok(glium::backend::Context::new(
                    Rc::new(state),
                    true,
                    callback_behavior(),
                )?)
            })
        })?;

        self.gl_state.replace(gl_state.clone());

        Ok(gl_state)
    }

    fn get_effective_dpi(&self) -> usize {
        let actual_dpi = unsafe { GetDpiForWindow(self.hwnd.0) } as f64;

        if self.config.dpi_by_screen.is_empty() {
            return self.config.dpi.unwrap_or(actual_dpi) as usize;
        }

        unsafe {
            let mut mi: MONITORINFOEXW = std::mem::zeroed();
            mi.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
            let mon = MonitorFromWindow(self.hwnd.0, MONITOR_DEFAULTTONEAREST);
            GetMonitorInfoW(mon, &mut mi as *mut MONITORINFOEXW as *mut MONITORINFO);

            if let Ok(info) = crate::os::windows::connection::ScreenInfoHelper::new() {
                let name = info.monitor_name(&mi);
                if let Some(dpi) = self.config.dpi_by_screen.get(&name).copied() {
                    return dpi as usize;
                }
            }

            actual_dpi as usize
        }
    }

    /// Check if we need to generate a resize callback.
    /// Calls resize if needed.
    /// Returns true if we did.
    fn check_and_call_resize_if_needed(&mut self) -> bool {
        /*
        if self.gl_state.is_none() {
            // Don't cache state or generate resize callbacks until
            // we've set up opengl, otherwise we can miss propagating
            // some state during the initial window setup that results
            // in the window dimensions being out of sync with the dpi
            // when eg: the system display settings are set to 200%
            // scale factor.
            return false;
        }
        */

        let mut rect = RECT {
            left: 0,
            bottom: 0,
            right: 0,
            top: 0,
        };
        unsafe {
            GetClientRect(self.hwnd.0, &mut rect);
        }
        let pixel_width = rect_width(&rect) as usize;
        let pixel_height = rect_height(&rect) as usize;

        let current_dims = Dimensions {
            pixel_width,
            pixel_height,
            dpi: self.get_effective_dpi(),
        };

        let same = self
            .last_size
            .as_ref()
            .map(|&dims| dims == current_dims)
            .unwrap_or(false);
        self.last_size.replace(current_dims);

        if !same {
            self.set_ime_window_position(Rect::default());

            self.events.dispatch(WindowEvent::Resized {
                dimensions: current_dims,
                window_state: get_window_state(self.hwnd.0),
                live_resizing: self.in_size_move,
            });
        }

        !same
    }

    fn apply_decoration(&mut self) {
        let hwnd = self.hwnd.0;
        schedule_apply_decoration(hwnd, self.config.window_decorations);
    }
}

fn schedule_apply_decoration(hwnd: HWND, decorations: WindowDecorations) {
    promise::spawn::spawn(async move {
        apply_decoration_immediate(hwnd, decorations);
    })
    .detach();
}

fn apply_decoration_immediate(hwnd: HWND, decorations: WindowDecorations) {
    match rc_from_hwnd(hwnd) {
        Some(inner) => {
            if inner.borrow().saved_placement.is_some() {
                // We are full screen; ignore it for now
                return;
            }
        }
        None => return,
    };

    unsafe {
        let orig_style = GetWindowLongW(hwnd, GWL_STYLE);
        let style = decorations_to_style(decorations);
        let new_style = (orig_style & !(WS_OVERLAPPEDWINDOW as i32)) | style as i32;
        SetWindowLongW(hwnd, GWL_STYLE, new_style);
        SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_NOACTIVATE
                | SWP_NOMOVE
                | SWP_NOSIZE
                | SWP_NOZORDER
                | SWP_NOOWNERZORDER
                | SWP_FRAMECHANGED,
        );
        apply_theme(hwnd);
    }
}

fn decorations_to_style(decorations: WindowDecorations) -> u32 {
    if decorations == WindowDecorations::RESIZE {
        WS_OVERLAPPEDWINDOW
    } else if decorations == WindowDecorations::TITLE {
        WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX
    } else if decorations == WindowDecorations::NONE {
        WS_POPUP
    } else if decorations == WindowDecorations::TITLE | WindowDecorations::RESIZE {
        WS_OVERLAPPEDWINDOW
    } else {
        WS_OVERLAPPEDWINDOW
    }
}

fn get_primary_monitor_dpi() -> u32 {
    let primary = unsafe { MonitorFromWindow(null_mut(), MONITOR_DEFAULTTOPRIMARY) };
    assert!(!primary.is_null(), "MonitorFromWindow() returned NULL");
    let mut dpi_x = USER_DEFAULT_SCREEN_DPI as u32;
    let mut dpi_y = USER_DEFAULT_SCREEN_DPI as u32;
    unsafe { GetDpiForMonitor(primary, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) };
    dpi_x
}
