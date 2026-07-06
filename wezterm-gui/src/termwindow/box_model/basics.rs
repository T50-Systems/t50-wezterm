use crate::color::LinearRgba;
use crate::customglyph::{BlockKey, Poly};
use crate::glyphcache::CachedGlyph;
use crate::quad::{QuadImpl, QuadTrait, TripleLayerQuadAllocator, TripleLayerQuadAllocatorTrait};
use crate::termwindow::{
    ColorEase, MouseCapture, RenderState, TermWindowNotif, UIItem, UIItemType,
};
use crate::utilsprites::RenderMetrics;
use ::window::{RectF, WindowOps};
use anyhow::anyhow;
use config::{Dimension, DimensionContext};
use finl_unicode::grapheme_clusters::Graphemes;
use std::cell::RefCell;
use std::rc::Rc;
use termwiz::cell::{grapheme_column_width, Presentation};
use termwiz::surface::Line;
use wezterm_font::units::PixelUnit;
use wezterm_font::LoadedFont;
use wezterm_term::color::{ColorAttribute, ColorPalette};
use window::bitmaps::atlas::Sprite;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlign {
    Top,
    Bottom,
    Middle,
}

impl Default for VerticalAlign {
    fn default() -> VerticalAlign {
        VerticalAlign::Top
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayType {
    Block,
    Inline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Float {
    None,
    Right,
}

impl Default for Float {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PixelDimension {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PixelSizedPoly {
    pub poly: &'static [Poly],
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SizedPoly {
    pub poly: &'static [Poly],
    pub width: Dimension,
    pub height: Dimension,
}

impl SizedPoly {
    pub fn to_pixels(&self, context: &LayoutContext) -> PixelSizedPoly {
        PixelSizedPoly {
            poly: self.poly,
            width: self.width.evaluate_as_pixels(context.width),
            height: self.height.evaluate_as_pixels(context.height),
        }
    }

    pub fn none() -> Self {
        Self {
            poly: &[],
            width: Dimension::default(),
            height: Dimension::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PixelCorners {
    pub top_left: PixelSizedPoly,
    pub top_right: PixelSizedPoly,
    pub bottom_left: PixelSizedPoly,
    pub bottom_right: PixelSizedPoly,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Corners {
    pub top_left: SizedPoly,
    pub top_right: SizedPoly,
    pub bottom_left: SizedPoly,
    pub bottom_right: SizedPoly,
}

impl Corners {
    pub fn to_pixels(&self, context: &LayoutContext) -> PixelCorners {
        PixelCorners {
            top_left: self.top_left.to_pixels(context),
            top_right: self.top_right.to_pixels(context),
            bottom_left: self.bottom_left.to_pixels(context),
            bottom_right: self.bottom_right.to_pixels(context),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BoxDimension {
    pub left: Dimension,
    pub top: Dimension,
    pub right: Dimension,
    pub bottom: Dimension,
}

impl BoxDimension {
    pub const fn new(dim: Dimension) -> Self {
        Self {
            left: dim,
            top: dim,
            right: dim,
            bottom: dim,
        }
    }

    pub fn to_pixels(&self, context: &LayoutContext) -> PixelDimension {
        PixelDimension {
            left: self.left.evaluate_as_pixels(context.width),
            top: self.top.evaluate_as_pixels(context.height),
            right: self.right.evaluate_as_pixels(context.width),
            bottom: self.bottom.evaluate_as_pixels(context.height),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InheritableColor {
    Inherited,
    Color(LinearRgba),
    Animated {
        color: LinearRgba,
        alt_color: LinearRgba,
        ease: Rc<RefCell<ColorEase>>,
        one_shot: bool,
    },
}

impl Default for InheritableColor {
    fn default() -> Self {
        Self::Inherited
    }
}

impl From<LinearRgba> for InheritableColor {
    fn from(color: LinearRgba) -> InheritableColor {
        InheritableColor::Color(color)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BorderColor {
    pub left: LinearRgba,
    pub top: LinearRgba,
    pub right: LinearRgba,
    pub bottom: LinearRgba,
}

impl BorderColor {
    pub const fn new(color: LinearRgba) -> Self {
        Self {
            left: color,
            top: color,
            right: color,
            bottom: color,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ElementColors {
    pub border: BorderColor,
    pub bg: InheritableColor,
    pub text: InheritableColor,
}

struct ResolvedColor {
    color: LinearRgba,
    alt_color: LinearRgba,
    mix_value: f32,
}

impl ResolvedColor {
    fn apply(&self, quad: &mut QuadImpl) {
        quad.set_fg_color(self.color);
        quad.set_alt_color_and_mix_value(self.alt_color, self.mix_value);
    }
}

impl From<LinearRgba> for ResolvedColor {
    fn from(color: LinearRgba) -> Self {
        Self {
            color,
            alt_color: color,
            mix_value: 0.,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    pub item_type: Option<UIItemType>,
    pub vertical_align: VerticalAlign,
    pub zindex: i8,
    pub display: DisplayType,
    pub float: Float,
    pub padding: BoxDimension,
    pub margin: BoxDimension,
    pub border: BoxDimension,
    pub border_corners: Option<Corners>,
    pub colors: ElementColors,
    pub hover_colors: Option<ElementColors>,
    pub font: Rc<LoadedFont>,
    pub content: ElementContent,
    pub presentation: Option<Presentation>,
    pub line_height: Option<f64>,
    pub max_width: Option<Dimension>,
    pub min_width: Option<Dimension>,
    pub min_height: Option<Dimension>,
}

impl Element {
    pub fn new(font: &Rc<LoadedFont>, content: ElementContent) -> Self {
        Self {
            item_type: None,
            zindex: 0,
            display: DisplayType::Inline,
            float: Float::None,
            padding: BoxDimension::default(),
            margin: BoxDimension::default(),
            border: BoxDimension::default(),
            border_corners: None,
            vertical_align: VerticalAlign::default(),
            colors: ElementColors::default(),
            hover_colors: None,
            font: Rc::clone(font),
            content,
            presentation: None,
            line_height: None,
            max_width: None,
            min_width: None,
            min_height: None,
        }
    }

    pub fn with_line(font: &Rc<LoadedFont>, line: &Line, palette: &ColorPalette) -> Self {
        let mut content: Vec<Element> = vec![];
        let mut prior_attr = None;

        for cluster in line.cluster(None) {
            // Clustering may introduce cluster boundaries when the text hasn't actually
            // changed style. Undo that here.
            // There's still an issue where the style does actually change and we
            // subsequently don't clip the element.
            // <https://github.com/wezterm/wezterm/issues/2560>
            if let Some(prior) = content.last_mut() {
                let (fg, bg) = prior_attr.as_ref().unwrap();
                if cluster.attrs.background() == *bg && cluster.attrs.foreground() == *fg {
                    if let ElementContent::Text(t) = &mut prior.content {
                        t.push_str(&cluster.text);
                        continue;
                    }
                }
            }

            let child =
                Element::new(font, ElementContent::Text(cluster.text)).colors(ElementColors {
                    border: BorderColor::default(),
                    bg: if cluster.attrs.background() == ColorAttribute::Default {
                        InheritableColor::Inherited
                    } else {
                        palette
                            .resolve_bg(cluster.attrs.background())
                            .to_linear()
                            .into()
                    },
                    text: if cluster.attrs.foreground() == ColorAttribute::Default {
                        InheritableColor::Inherited
                    } else {
                        palette
                            .resolve_fg(cluster.attrs.foreground())
                            .to_linear()
                            .into()
                    },
                });

            content.push(child);
            prior_attr.replace((cluster.attrs.foreground(), cluster.attrs.background()));
        }

        Self::new(font, ElementContent::Children(content))
    }

    pub fn vertical_align(mut self, align: VerticalAlign) -> Self {
        self.vertical_align = align;
        self
    }

    pub fn item_type(mut self, item_type: UIItemType) -> Self {
        self.item_type.replace(item_type);
        self
    }

    pub fn display(mut self, display: DisplayType) -> Self {
        self.display = display;
        self
    }

    pub fn float(mut self, float: Float) -> Self {
        self.float = float;
        self
    }

    pub fn colors(mut self, colors: ElementColors) -> Self {
        self.colors = colors;
        self
    }

    pub fn hover_colors(mut self, colors: Option<ElementColors>) -> Self {
        self.hover_colors = colors;
        self
    }

    pub fn line_height(mut self, line_height: Option<f64>) -> Self {
        self.line_height = line_height;
        self
    }

    pub fn zindex(mut self, zindex: i8) -> Self {
        self.zindex = zindex;
        self
    }

    pub fn padding(mut self, padding: BoxDimension) -> Self {
        self.padding = padding;
        self
    }

    pub fn border(mut self, border: BoxDimension) -> Self {
        self.border = border;
        self
    }

    pub fn border_corners(mut self, corners: Option<Corners>) -> Self {
        self.border_corners = corners;
        self
    }

    pub fn margin(mut self, margin: BoxDimension) -> Self {
        self.margin = margin;
        self
    }

    pub fn max_width(mut self, width: Option<Dimension>) -> Self {
        self.max_width = width;
        self
    }

    pub fn min_width(mut self, width: Option<Dimension>) -> Self {
        self.min_width = width;
        self
    }

    pub fn min_height(mut self, height: Option<Dimension>) -> Self {
        self.min_height = height;
        self
    }
}
