//! Rendering of Changes using terminfo
use crate::caps::{Capabilities, ColorLevel};
use crate::cell::{AttributeChange, Blink, CellAttributes, Intensity, Underline};
use crate::color::{ColorAttribute, ColorSpec};
use crate::escape::csi::{Cursor, Edit, EraseInDisplay, EraseInLine, Sgr, CSI};
use crate::escape::esc::EscCode;
use crate::escape::osc::OperatingSystemCommand;
#[cfg(feature = "use_image")]
use crate::escape::osc::{ITermDimension, ITermFileData, ITermProprietary};
use crate::escape::{Esc, OneBased};
#[cfg(feature = "use_image")]
use crate::image::{ImageDataType, TextureCoordinate};
use crate::render::RenderTty;
use crate::surface::{Change, CursorShape, CursorVisibility, LineAttribute, Position};
use crate::Result;
use std::io::Write;
use terminfo::{capability as cap, Capability as TermInfoCapability};

include!("terminfo/renderer_types.rs");
include!("terminfo/renderer_attrs.rs");
include!("terminfo/renderer_cursor.rs");
include!("terminfo/renderer_render.rs");
include!("terminfo/tests_support.rs");
