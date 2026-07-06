use crate::resize_increment_calculator::ResizeIncrementCalculator;
use crate::utilsprites::RenderMetrics;
use ::window::{Dimensions, ResizeIncrement, Window, WindowOps, WindowState};
use config::{ConfigHandle, DimensionContext};
use mux::Mux;
use std::rc::Rc;
use wezterm_font::FontConfiguration;
use wezterm_term::TerminalSize;

#[derive(Debug, Clone, Copy)]
pub struct RowsAndCols {
    pub rows: usize,
    pub cols: usize,
}

#[derive(Debug)]
pub enum ScaleChange {
    Absolute(f64),
    Relative(f64),
}
