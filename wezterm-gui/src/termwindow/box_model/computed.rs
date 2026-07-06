#[derive(Debug, Clone)]
pub enum ElementContent {
    Text(String),
    Children(Vec<Element>),
    Poly { line_width: isize, poly: SizedPoly },
}

pub struct LayoutContext<'a> {
    pub width: DimensionContext,
    pub height: DimensionContext,
    pub bounds: RectF,
    pub metrics: &'a RenderMetrics,
    pub gl_state: &'a RenderState,
    pub zindex: i8,
}

#[derive(Debug, Clone)]
pub struct ComputedElement {
    pub item_type: Option<UIItemType>,
    pub zindex: i8,
    /// The outer bounds of the element box (its margin)
    pub bounds: RectF,
    /// The outer bounds of the area enclosed by its border
    pub border_rect: RectF,
    pub border: PixelDimension,
    pub border_corners: Option<PixelCorners>,
    pub colors: ElementColors,
    pub hover_colors: Option<ElementColors>,
    /// The outer bounds of the area enclosed by the padding
    pub padding: RectF,
    /// The outer bounds of the content
    pub content_rect: RectF,
    pub baseline: f32,

    pub content: ComputedElementContent,
}

impl ComputedElement {
    pub fn translate(&mut self, delta: euclid::Vector2D<f32, PixelUnit>) {
        self.bounds = self.bounds.translate(delta);
        self.border_rect = self.border_rect.translate(delta);
        self.padding = self.padding.translate(delta);
        self.content_rect = self.content_rect.translate(delta);

        match &mut self.content {
            ComputedElementContent::Children(kids) => {
                for kid in kids {
                    kid.translate(delta)
                }
            }
            ComputedElementContent::Text(_) => {}
            ComputedElementContent::Poly { .. } => {}
        }
    }

    pub fn ui_items(&self) -> Vec<UIItem> {
        let mut items = vec![];
        self.ui_item_impl(&mut items);
        items
    }

    fn ui_item_impl(&self, items: &mut Vec<UIItem>) {
        if let Some(item_type) = &self.item_type {
            items.push(UIItem {
                x: self.bounds.min_x().max(0.) as usize,
                y: self.bounds.min_y().max(0.) as usize,
                width: self.bounds.width().max(0.) as usize,
                height: self.bounds.height().max(0.) as usize,
                item_type: item_type.clone(),
            });
        }

        match &self.content {
            ComputedElementContent::Text(_) => {}
            ComputedElementContent::Children(kids) => {
                for kid in kids {
                    kid.ui_item_impl(items);
                }
            }
            ComputedElementContent::Poly { .. } => {}
        }
    }
}

#[derive(Debug, Clone)]
pub enum ComputedElementContent {
    Text(Vec<ElementCell>),
    Children(Vec<ComputedElement>),
    Poly {
        line_width: isize,
        poly: PixelSizedPoly,
    },
}

#[derive(Debug, Clone)]
pub enum ElementCell {
    Sprite(Sprite),
    Glyph(Rc<CachedGlyph>),
}

#[derive(Debug)]
struct Rects {
    padding: RectF,
    border_rect: RectF,
    bounds: RectF,
    content_rect: RectF,
    translate: euclid::Vector2D<f32, PixelUnit>,
}

impl Element {
    fn compute_rects(&self, context: &LayoutContext, content_rect: RectF) -> Rects {
        let padding = self.padding.to_pixels(context);
        let margin = self.margin.to_pixels(context);
        let border = self.border.to_pixels(context);

        let padding = euclid::rect(
            content_rect.min_x() - padding.left,
            content_rect.min_y() - padding.top,
            content_rect.width() + padding.left + padding.right,
            content_rect.height() + padding.top + padding.bottom,
        );

        let border_rect = euclid::rect(
            padding.min_x() - border.left,
            padding.min_y() - border.top,
            padding.width() + border.left + border.right,
            padding.height() + border.top + border.bottom,
        );

        let bounds = euclid::rect(
            border_rect.min_x() - margin.left,
            border_rect.min_y() - margin.top,
            border_rect.width() + margin.left + margin.right,
            border_rect.height() + margin.top + margin.bottom,
        );
        let translate = euclid::vec2(
            context.bounds.min_x() - bounds.min_x(),
            context.bounds.min_y() - bounds.min_y(),
        );
        Rects {
            padding: padding.translate(translate),
            border_rect: border_rect.translate(translate),
            bounds: bounds.translate(translate),
            content_rect: content_rect.translate(translate),
            translate,
        }
    }
}
