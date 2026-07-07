// let () = msg_send! is a common pattern for objc
#![allow(clippy::let_unit_value)]

use super::keycodes::*;
use super::{nsstring, nsstring_to_str};
use crate::clipboard::Clipboard as ClipboardContext;
use crate::connection::ConnectionOps;
use crate::os::macos::menu::{MenuItem, RepresentedItem};
use crate::parameters::{Border, Parameters, TitleBar};
use crate::{
    Clipboard, Connection, DeadKeyStatus, Dimensions, Handled, KeyCode, KeyEvent, Modifiers,
    MouseButtons, MouseCursor, MouseEvent, MouseEventKind, MousePress, Point, RawKeyEvent, Rect,
    RequestedWindowGeometry, ResizeIncrement, ResolvedGeometry, ScreenPoint, Size, ULength,
    WindowDecorations, WindowEvent, WindowEventSender, WindowOps, WindowState,
};
use anyhow::{anyhow, bail, ensure};
use async_trait::async_trait;
use cocoa::appkit::{
    self, CGFloat, NSApplication, NSApplicationActivateIgnoringOtherApps,
    NSApplicationPresentationOptions, NSBackingStoreBuffered, NSEvent, NSEventModifierFlags,
    NSOpenGLContext, NSOpenGLPixelFormat, NSPasteboard, NSRunningApplication, NSScreen, NSView,
    NSViewHeightSizable, NSViewWidthSizable, NSWindow, NSWindowStyleMask,
};
use cocoa::base::*;
use cocoa::foundation::{
    NSArray, NSAutoreleasePool, NSFastEnumeration, NSInteger, NSNotFound, NSPoint, NSRect, NSSize,
    NSString, NSUInteger,
};
use config::window::WindowLevel;
use config::{ConfigHandle, RgbaColor, SrgbaTuple};
use core_foundation::base::{CFTypeID, TCFType};
use core_foundation::bundle::{CFBundleGetBundleWithIdentifier, CFBundleGetFunctionPointerForName};
use core_foundation::data::{CFData, CFDataGetBytePtr, CFDataRef};
use core_foundation::string::{CFString, CFStringRef, UniChar};
use core_foundation::{declare_TCFType, impl_TCFType};
use objc::declare::ClassDecl;
use objc::rc::{StrongPtr, WeakPtr};
use objc::runtime::{Class, Object, Protocol, Sel};
use objc::*;
use promise::Future;
use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, DisplayHandle, HandleError, HasDisplayHandle,
    HasWindowHandle, RawDisplayHandle, RawWindowHandle, WindowHandle,
};
use std::any::Any;
use std::cell::RefCell;
use std::ffi::{c_void, CStr};
use std::path::PathBuf;
use std::ptr::NonNull;
use std::rc::Rc;
use std::str::FromStr;
use std::time::Instant;
use wezterm_font::FontConfiguration;
use wezterm_input_types::{is_ascii_control, IntegratedTitleButtonStyle, KeyboardLedStatus};

#[allow(non_upper_case_globals)]
const NSViewLayerContentsPlacementTopLeft: NSInteger = 11;
#[allow(non_upper_case_globals)]
const NSViewLayerContentsRedrawDuringViewResize: NSInteger = 2;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGSMainConnectionID() -> id;
    fn CGSSetWindowBackgroundBlurRadius(
        connection_id: id,
        window_id: NSInteger,
        radius: i64,
    ) -> i32;
}

fn round_away_from_zerof(value: f64) -> f64 {
    if value > 0. {
        value.max(1.).round()
    } else {
        value.min(-1.).round()
    }
}

fn round_away_from_zero(value: f64) -> i16 {
    if value > 0. {
        value.max(1.).round() as i16
    } else {
        value.min(-1.).round() as i16
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ImeDisposition {
    /// Nothing happened
    None,
    /// IME triggered an action
    Acted,
    /// We decided to continue with key dispatch
    Continue,
}

#[repr(C)]
struct NSRange(cocoa::foundation::NSRange);

#[derive(Debug)]
#[repr(C)]
struct NSRangePointer(*mut NSRange);

impl std::fmt::Debug for NSRange {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        fmt.debug_struct("NSRange")
            .field("location", &self.0.location)
            .field("length", &self.0.length)
            .finish()
    }
}

unsafe impl objc::Encode for NSRange {
    fn encode() -> objc::Encoding {
        let encoding = format!(
            "{{NSRange={}{}}}",
            NSUInteger::encode().as_str(),
            NSUInteger::encode().as_str()
        );
        unsafe { objc::Encoding::from_str(&encoding) }
    }
}

unsafe impl objc::Encode for NSRangePointer {
    fn encode() -> objc::Encoding {
        unsafe { objc::Encoding::from_str(&format!("^{}", NSRange::encode().as_str())) }
    }
}

impl NSRange {
    fn new(location: u64, length: u64) -> Self {
        Self(cocoa::foundation::NSRange { location, length })
    }
}

#[derive(Clone)]
pub enum BackendImpl {
    Cgl(Rc<cglbits::GlState>),
    Egl(Rc<crate::egl::GlState>),
}

impl BackendImpl {
    pub fn update(&self) {
        if let Self::Cgl(be) = self {
            be.update();
        }
    }
}

#[derive(Clone)]
pub struct GlContextPair {
    pub context: Rc<glium::backend::Context>,
    pub backend: BackendImpl,
}

impl GlContextPair {
    /// on macOS we first try to initialize EGL by dynamically loading it.
    /// The system doesn't provide an EGL implementation, but the ANGLE
    /// project (and MetalANGLE) both provide implementations.
    /// The ANGLE EGL implementation wants a CALayer descendant passed
    /// as the EGLNativeWindowType.
    pub fn create(view: id) -> anyhow::Result<Self> {
        let behavior = if cfg!(debug_assertions) {
            glium::debug::DebugCallbackBehavior::DebugMessageOnError
        } else {
            glium::debug::DebugCallbackBehavior::Ignore
        };

        // Let's first try to initialize EGL...
        let (context, backend) = match if config::configuration().prefer_egl {
            // ANGLE wants a layer, so tell the view to create one.
            // Importantly, we must set its scale to 1.0 prior to initializing
            // EGL to prevent undesirable scaling.
            let layer: id;
            unsafe {
                let _: () = msg_send![view, setWantsLayer: YES];
                layer = msg_send![view, layer];
                let _: () = msg_send![layer, setContentsScale: 1.0f64];
                let _: () = msg_send![layer, setOpaque: NO];
            };

            let conn = Connection::get().unwrap();

            let state = match conn.gl_connection.borrow().as_ref() {
                None => crate::egl::GlState::create(None, layer as *const c_void),
                Some(glconn) => crate::egl::GlState::create_with_existing_connection(
                    glconn,
                    layer as *const c_void,
                ),
            };

            if state.is_ok() {
                conn.gl_connection
                    .borrow_mut()
                    .replace(Rc::clone(state.as_ref().unwrap().get_connection()));

                // ANGLE will create a CAMetalLayer as a sublayer of our provided
                // layer.  Even though CALayer defaults to !opaque, CAMetalLayer
                // defaults to opaque, so we need to find that layer and fix
                // the opacity so that our alpha values are respected.
                unsafe {
                    let sublayers: id = msg_send![layer, sublayers];
                    let layer_count = sublayers.count();
                    for i in 0..layer_count {
                        let layer = sublayers.objectAtIndex(i);
                        let _: () = msg_send![layer, setOpaque: NO];
                    }
                }
            }

            state
        } else {
            Err(anyhow!("prefers not to use EGL"))
        } {
            Ok(backend) => {
                let backend = Rc::new(backend);
                let context =
                    unsafe { glium::backend::Context::new(Rc::clone(&backend), true, behavior) }?;
                (context, BackendImpl::Egl(backend))
            }
            // ... and then fallback to the deprecated platform provided CGL
            Err(err) => {
                log::debug!("EGL init failed: {:#}, falling back to CGL", err);
                let backend = Rc::new(cglbits::GlState::create(view)?);
                let context =
                    unsafe { glium::backend::Context::new(Rc::clone(&backend), true, behavior) }?;
                (context, BackendImpl::Cgl(backend))
            }
        };

        Ok(Self { context, backend })
    }
}

mod cglbits {
    use super::*;

    pub struct GlState {
        _pixel_format: StrongPtr,
        gl_context: StrongPtr,
    }

    impl GlState {
        pub fn create(view: id) -> anyhow::Result<Self> {
            log::trace!("Calling NSOpenGLPixelFormat::initWithAttributes");
            let pixel_format = unsafe {
                StrongPtr::new(NSOpenGLPixelFormat::alloc(nil).initWithAttributes_(&[
                    appkit::NSOpenGLPFAOpenGLProfile as u32,
                    appkit::NSOpenGLProfileVersion3_2Core as u32,
                    appkit::NSOpenGLPFAClosestPolicy as u32,
                    appkit::NSOpenGLPFAColorSize as u32,
                    32,
                    appkit::NSOpenGLPFAAlphaSize as u32,
                    8,
                    appkit::NSOpenGLPFADepthSize as u32,
                    24,
                    appkit::NSOpenGLPFAStencilSize as u32,
                    8,
                    appkit::NSOpenGLPFAAllowOfflineRenderers as u32,
                    appkit::NSOpenGLPFAAccelerated as u32,
                    appkit::NSOpenGLPFADoubleBuffer as u32,
                    0,
                ]))
            };
            log::trace!("NSOpenGLPixelFormat::initWithAttributes returned");
            ensure!(
                !pixel_format.is_null(),
                "failed to create NSOpenGLPixelFormat"
            );

            // Allow using retina resolutions; without this we're forced into low res
            // and the system will scale us up, resulting in blurry rendering
            unsafe {
                let _: () = msg_send![view, setWantsBestResolutionOpenGLSurface: YES];
            }

            let gl_context = unsafe {
                StrongPtr::new(
                    NSOpenGLContext::alloc(nil).initWithFormat_shareContext_(*pixel_format, nil),
                )
            };
            ensure!(!gl_context.is_null(), "failed to create NSOpenGLContext");

            unsafe {
                let opaque: cgl::GLint = 0;
                gl_context.setValues_forParameter_(
                    &opaque,
                    cocoa::appkit::NSOpenGLContextParameter::NSOpenGLCPSurfaceOpacity,
                );

                gl_context.setView_(view);

                // Explicitly disable vsync; we'll manage throttling frames at
                // the application level
                let swap_interval: cgl::GLint = 0;
                gl_context.setValues_forParameter_(
                    &swap_interval,
                    cocoa::appkit::NSOpenGLContextParameter::NSOpenGLCPSwapInterval,
                );
            }

            Ok(Self {
                _pixel_format: pixel_format,
                gl_context,
            })
        }

        /// Calls NSOpenGLContext update; we need to do this on resize
        pub fn update(&self) {
            unsafe {
                let _: () = msg_send![*self.gl_context, update];
            }
        }
    }

    unsafe impl glium::backend::Backend for GlState {
        fn resize(&self, _: (u32, u32)) {
            todo!()
        }

        fn swap_buffers(&self) -> Result<(), glium::SwapBuffersError> {
            unsafe {
                let pool = NSAutoreleasePool::new(nil);
                self.gl_context.flushBuffer();
                let _: () = msg_send![pool, release];
            }
            Ok(())
        }

        unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
            let symbol_name: CFString = FromStr::from_str(symbol).unwrap();
            let framework_name: CFString = FromStr::from_str("com.apple.opengl").unwrap();
            let framework = CFBundleGetBundleWithIdentifier(framework_name.as_concrete_TypeRef());
            let symbol =
                CFBundleGetFunctionPointerForName(framework, symbol_name.as_concrete_TypeRef());
            symbol as *const _
        }

        fn get_framebuffer_dimensions(&self) -> (u32, u32) {
            unsafe {
                let view = self.gl_context.view();
                let frame = NSView::frame(view);
                let backing_frame = NSView::convertRectToBacking(view, frame);
                (
                    backing_frame.size.width as u32,
                    backing_frame.size.height as u32,
                )
            }
        }

        fn is_current(&self) -> bool {
            unsafe {
                let pool = NSAutoreleasePool::new(nil);
                let current = NSOpenGLContext::currentContext(nil);
                let res = if current != nil {
                    let is_equal: BOOL = msg_send![current, isEqual: *self.gl_context];
                    is_equal != NO
                } else {
                    false
                };
                let _: () = msg_send![pool, release];
                res
            }
        }

        unsafe fn make_current(&self) {
            let _: () = msg_send![*self.gl_context, update];
            self.gl_context.makeCurrentContext();
        }
    }
}

pub(crate) struct WindowInner {
    view: StrongPtr,
    window: StrongPtr,
    config: ConfigHandle,
}

fn function_key_to_keycode(function_key: char) -> KeyCode {
    // FIXME: CTRL-C is 0x3, should it be normalized to C here
    // using the unmod string?  Or should be normalize the 0x3
    // as the canonical representation of that input?
    match function_key as u16 {
        appkit::NSUpArrowFunctionKey => KeyCode::UpArrow,
        appkit::NSDownArrowFunctionKey => KeyCode::DownArrow,
        appkit::NSLeftArrowFunctionKey => KeyCode::LeftArrow,
        appkit::NSRightArrowFunctionKey => KeyCode::RightArrow,
        appkit::NSHomeFunctionKey => KeyCode::Home,
        appkit::NSEndFunctionKey => KeyCode::End,
        appkit::NSPageUpFunctionKey => KeyCode::PageUp,
        appkit::NSPageDownFunctionKey => KeyCode::PageDown,
        appkit::NSClearLineFunctionKey => KeyCode::NumLock,
        value @ appkit::NSF1FunctionKey..=appkit::NSF35FunctionKey => {
            KeyCode::Function((value - appkit::NSF1FunctionKey + 1) as u8)
        }
        appkit::NSInsertFunctionKey => KeyCode::Insert,
        appkit::NSDeleteFunctionKey => KeyCode::Char('\u{7f}'),
        appkit::NSPrintScreenFunctionKey => KeyCode::PrintScreen,
        appkit::NSScrollLockFunctionKey => KeyCode::ScrollLock,
        appkit::NSPauseFunctionKey => KeyCode::Pause,
        appkit::NSBreakFunctionKey => KeyCode::Cancel,
        appkit::NSPrintFunctionKey => KeyCode::Print,
        _ => KeyCode::Char(function_key),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Window {
    id: usize,
    ns_window: *mut Object,
    ns_view: *mut Object,
}

unsafe impl Send for Window {}
unsafe impl Sync for Window {}

fn set_window_position(window: *mut Object, coords: ScreenPoint) {
    unsafe {
        let cartesian = screen_point_to_cartesian(coords);
        let frame = NSWindow::frame(window);
        let content_frame = NSWindow::contentRectForFrameRect_(window, frame);
        let delta_x = content_frame.origin.x - frame.origin.x;
        let delta_y = content_frame.origin.y - frame.origin.y;
        let point = NSPoint::new(
            cartesian.x as f64 - delta_x,
            cartesian.y as f64 - delta_y - content_frame.size.height,
        );
        NSWindow::setFrameOrigin_(window, point);
    }
}
