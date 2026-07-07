//! This file is derived from the ConceptFrame implementation
//! in smithay_client_toolkit 0.11 which is Copyright (c) 2018 Victor Berger
//! and provided under the terms of the MIT license.

use crate::os::wayland::pointer::make_theme_manager;
use config::{ConfigHandle, RgbaColor, WindowFrameConfig};
use smithay_client_toolkit::output::{add_output_listener, with_output_info, OutputListener};
use smithay_client_toolkit::seat::pointer::{ThemeManager, ThemedPointer};
use smithay_client_toolkit::shm::DoubleMemPool;
use smithay_client_toolkit::window::{ButtonState, Frame, FrameRequest, State, WindowState};
use std::cell::RefCell;
use std::cmp::max;
use std::rc::Rc;
use std::sync::Mutex;
use tiny_skia::{
    ColorU8, FillRule, Paint, PathBuilder, PixmapMut, PixmapPaint, PixmapRef, Rect, Stroke,
    Transform,
};
use wayland_client::protocol::{
    wl_compositor, wl_output, wl_pointer, wl_seat, wl_shm, wl_subcompositor, wl_subsurface,
    wl_surface,
};
use wayland_client::{Attached, DispatchData, Main};
use wezterm_color_types::SrgbaTuple;
use wezterm_font::{FontConfiguration, FontMetrics, GlyphInfo, RasterizedGlyph};
use wezterm_input_types::WindowDecorations;

fn color_to_paint(c: RgbaColor) -> Paint<'static> {
    let mut paint = Paint::default();
    let (red, green, blue, alpha) = c.as_rgba_u8();
    paint.set_color_rgba8(blue, green, red, alpha);
    paint.anti_alias = true;
    paint
}

/*
 * Drawing theme definitions
 */

const BORDER_SIZE: u32 = 12;
const HEADER_SIZE: u32 = 30;

/// Configuration for ConceptFrame
#[derive(Clone)]
pub struct ConceptConfig {
    pub font_config: Option<Rc<FontConfiguration>>,
    pub config: ConfigHandle,
}

impl Default for ConceptConfig {
    fn default() -> Self {
        Self {
            font_config: None,
            config: config::configuration(),
        }
    }
}

impl ConceptConfig {
    pub fn colors(&self) -> &WindowFrameConfig {
        &self.config.window_frame
    }
}

/*
 * Utilities
 */

const HEAD: usize = 0;
const TOP: usize = 1;
const BOTTOM: usize = 2;
const LEFT: usize = 3;
const RIGHT: usize = 4;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Location {
    None,
    Head,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
    Button(UIButton),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum UIButton {
    Minimize,
    Maximize,
    Close,
}

struct Part {
    surface: wl_surface::WlSurface,
    subsurface: wl_subsurface::WlSubsurface,
}

pub(crate) struct SurfaceUserData {
    scale_factor: i32,
    outputs: Vec<(wl_output::WlOutput, i32, OutputListener)>,
}

impl SurfaceUserData {
    fn new() -> Self {
        SurfaceUserData {
            scale_factor: 1,
            outputs: Vec::new(),
        }
    }

    pub(crate) fn enter<F>(
        &mut self,
        output: wl_output::WlOutput,
        surface: wl_surface::WlSurface,
        callback: &Option<Rc<RefCell<F>>>,
    ) where
        F: FnMut(i32, wl_surface::WlSurface, DispatchData) + 'static,
    {
        let output_scale = with_output_info(&output, |info| info.scale_factor).unwrap_or(1);
        let my_surface = surface.clone();
        // Use a UserData to safely share the callback with the other thread
        let my_callback = wayland_client::UserData::new();
        if let Some(ref cb) = callback {
            my_callback.set(|| cb.clone());
        }
        let listener = add_output_listener(&output, move |output, info, ddata| {
            let mut user_data = my_surface
                .as_ref()
                .user_data()
                .get::<Mutex<SurfaceUserData>>()
                .unwrap()
                .lock()
                .unwrap();
            // update the scale factor of the relevant output
            for (ref o, ref mut factor, _) in user_data.outputs.iter_mut() {
                if o.as_ref().equals(output.as_ref()) {
                    if info.obsolete {
                        // an output that no longer exists is marked by a scale factor of -1
                        *factor = -1;
                    } else {
                        *factor = info.scale_factor;
                    }
                    break;
                }
            }
            // recompute the scale factor with the new info
            let callback = my_callback.get::<Rc<RefCell<F>>>().cloned();
            let old_scale_factor = user_data.scale_factor;
            let new_scale_factor = user_data.recompute_scale_factor();
            drop(user_data);
            if let Some(ref cb) = callback {
                if old_scale_factor != new_scale_factor {
                    (&mut *cb.borrow_mut())(new_scale_factor, surface.clone(), ddata);
                }
            }
        });
        self.outputs.push((output, output_scale, listener));
    }

    pub(crate) fn leave(&mut self, output: &wl_output::WlOutput) {
        self.outputs
            .retain(|(ref output2, _, _)| !output.as_ref().equals(output2.as_ref()));
    }

    fn recompute_scale_factor(&mut self) -> i32 {
        let mut new_scale_factor = 1;
        self.outputs.retain(|&(_, output_scale, _)| {
            if output_scale > 0 {
                new_scale_factor = ::std::cmp::max(new_scale_factor, output_scale);
                true
            } else {
                // cleanup obsolete output
                false
            }
        });
        if self.outputs.is_empty() {
            // don't update the scale factor if we are not displayed on any output
            return self.scale_factor;
        }
        self.scale_factor = new_scale_factor;
        new_scale_factor
    }
}

/// Returns the current suggested scale factor of a surface.
///
/// Panics if the surface was not created using `create_surface`
fn get_surface_scale_factor(surface: &wl_surface::WlSurface) -> i32 {
    surface
        .as_ref()
        .user_data()
        .get::<Mutex<SurfaceUserData>>()
        .expect("SCTK: Surface was not created by SCTK.")
        .lock()
        .unwrap()
        .scale_factor
}

fn setup_surface<F>(
    surface: Main<wl_surface::WlSurface>,
    callback: Option<F>,
) -> Attached<wl_surface::WlSurface>
where
    F: FnMut(i32, wl_surface::WlSurface, DispatchData) + 'static,
{
    let callback = callback.map(|c| Rc::new(RefCell::new(c)));
    surface.quick_assign(move |surface, event, ddata| {
        let mut user_data = surface
            .as_ref()
            .user_data()
            .get::<Mutex<SurfaceUserData>>()
            .unwrap()
            .lock()
            .unwrap();
        match event {
            wl_surface::Event::Enter { output } => {
                // Passing the callback to be added to output listener
                user_data.enter(output, surface.detach(), &callback);
            }
            wl_surface::Event::Leave { output } => {
                user_data.leave(&output);
            }
            _ => unreachable!(),
        };
        let old_scale_factor = user_data.scale_factor;
        let new_scale_factor = user_data.recompute_scale_factor();
        drop(user_data);
        if let Some(ref cb) = callback {
            if old_scale_factor != new_scale_factor {
                (&mut *cb.borrow_mut())(new_scale_factor, surface.detach(), ddata);
            }
        }
    });
    surface
        .as_ref()
        .user_data()
        .set_threadsafe(|| Mutex::new(SurfaceUserData::new()));
    surface.into()
}

impl Part {
    fn new(
        parent: &wl_surface::WlSurface,
        compositor: &Attached<wl_compositor::WlCompositor>,
        subcompositor: &Attached<wl_subcompositor::WlSubcompositor>,
        inner: Option<Rc<RefCell<Inner>>>,
    ) -> Part {
        let surface = if let Some(inner) = inner {
            setup_surface(
                compositor.create_surface(),
                Some(
                    move |dpi, surface: wl_surface::WlSurface, ddata: DispatchData| {
                        surface.set_buffer_scale(dpi);
                        surface.commit();
                        (&mut inner.borrow_mut().implem)(FrameRequest::Refresh, 0, ddata);
                    },
                ),
            )
        } else {
            setup_surface(
                compositor.create_surface(),
                Some(
                    move |dpi, surface: wl_surface::WlSurface, _ddata: DispatchData| {
                        surface.set_buffer_scale(dpi);
                        surface.commit();
                    },
                ),
            )
        };

        let surface = surface.detach();

        let subsurface = subcompositor.get_subsurface(&surface, parent);

        Part {
            surface,
            subsurface: subsurface.detach(),
        }
    }
}

impl Drop for Part {
    fn drop(&mut self) {
        self.subsurface.destroy();
        self.surface.destroy();
    }
}

struct PointerUserData {
    location: Location,
    position: (f64, f64),
    seat: wl_seat::WlSeat,
}

/*
 * The core frame
 */

struct Inner {
    parts: Vec<Part>,
    size: (u32, u32),
    resizable: bool,
    theme_over_surface: bool,
    implem: Box<dyn FnMut(FrameRequest, u32, DispatchData)>,
    maximized: bool,
    fullscreened: bool,
}

impl Inner {
    fn find_surface(&self, surface: &wl_surface::WlSurface) -> Location {
        if self.parts.is_empty() {
            return Location::None;
        }

        if surface.as_ref().equals(self.parts[HEAD].surface.as_ref()) {
            Location::Head
        } else if surface.as_ref().equals(self.parts[TOP].surface.as_ref()) {
            Location::Top
        } else if surface.as_ref().equals(self.parts[BOTTOM].surface.as_ref()) {
            Location::Bottom
        } else if surface.as_ref().equals(self.parts[LEFT].surface.as_ref()) {
            Location::Left
        } else if surface.as_ref().equals(self.parts[RIGHT].surface.as_ref()) {
            Location::Right
        } else {
            Location::None
        }
    }
}

fn precise_location(old: Location, width: u32, x: f64, y: f64) -> Location {
    match old {
        Location::Head | Location::Button(_) => find_button(x, y, width),

        Location::Top | Location::TopLeft | Location::TopRight => {
            if x <= f64::from(BORDER_SIZE) {
                Location::TopLeft
            } else if x >= f64::from(width + BORDER_SIZE) {
                Location::TopRight
            } else {
                Location::Top
            }
        }

        Location::Bottom | Location::BottomLeft | Location::BottomRight => {
            if x <= f64::from(BORDER_SIZE) {
                Location::BottomLeft
            } else if x >= f64::from(width + BORDER_SIZE) {
                Location::BottomRight
            } else {
                Location::Bottom
            }
        }

        other => other,
    }
}

fn find_button(x: f64, y: f64, w: u32) -> Location {
    if (w >= HEADER_SIZE)
        && (x >= f64::from(w - HEADER_SIZE))
        && (x <= f64::from(w))
        && (y <= f64::from(HEADER_SIZE))
        && (y >= f64::from(0))
    {
        // first button
        Location::Button(UIButton::Close)
    } else if (w >= 2 * HEADER_SIZE)
        && (x >= f64::from(w - 2 * HEADER_SIZE))
        && (x <= f64::from(w - HEADER_SIZE))
        && (y <= f64::from(HEADER_SIZE))
        && (y >= f64::from(0))
    {
        // second button
        Location::Button(UIButton::Maximize)
    } else if (w >= 3 * HEADER_SIZE)
        && (x >= f64::from(w - 3 * HEADER_SIZE))
        && (x <= f64::from(w - 2 * HEADER_SIZE))
        && (y <= f64::from(HEADER_SIZE))
        && (y >= f64::from(0))
    {
        // third button
        Location::Button(UIButton::Minimize)
    } else {
        Location::Head
    }
}

include!("frame/concept.rs");

impl Frame for ConceptFrame {
    type Error = ::std::io::Error;
    type Config = ConceptConfig;

    include!("frame/frame_impl_setup.rs");
    include!("frame/frame_impl_redraw.rs");
    include!("frame/frame_impl_tail.rs");
}

include!("frame/tail.rs");
