/// Describes an event that may need to be processed by the widget
pub enum WidgetEvent {
    Input(InputEvent),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CursorShapeAndPosition {
    pub shape: CursorShape,
    pub coords: ParentRelativeCoords,
    pub color: ColorAttribute,
    pub visibility: CursorVisibility,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

pub struct RenderArgs<'a> {
    /// The id of the current widget
    pub id: WidgetId,
    pub is_focused: bool,
    pub cursor: &'a mut CursorShapeAndPosition,
    pub surface: &'a mut Surface,
}

/// UpdateArgs provides access to the widget and UI state during
/// a call to `Widget::update_state`
pub struct UpdateArgs<'a> {
    /// The id of the current widget
    pub id: WidgetId,
    pub cursor: &'a mut CursorShapeAndPosition,
}

/// Implementing the `Widget` trait allows for defining a potentially
/// interactive component in a UI layout.
pub trait Widget {
    /// Draw the widget to the RenderArgs::surface, and optionally
    /// update RenderArgs::cursor to reflect the cursor position and
    /// display attributes.
    fn render(&mut self, args: &mut RenderArgs);

    /// Override this to have your widget specify its layout constraints.
    /// You may wish to have your widget constructor receive a `Constraints`
    /// instance to make this more easily configurable in more generic widgets.
    fn get_size_constraints(&self) -> layout::Constraints {
        Default::default()
    }

    /// Override this to allow your widget to respond to keyboard, mouse and
    /// other widget events.
    /// Return `true` if your widget handled the event, or `false` to allow
    /// the event to propagate to the widget parent.
    fn process_event(&mut self, _event: &WidgetEvent, _args: &mut UpdateArgs) -> bool {
        false
    }
}

/// Relative to the top left of the parent container
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParentRelativeCoords {
    pub x: usize,
    pub y: usize,
}

impl ParentRelativeCoords {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl From<(usize, usize)> for ParentRelativeCoords {
    fn from(coords: (usize, usize)) -> ParentRelativeCoords {
        ParentRelativeCoords::new(coords.0, coords.1)
    }
}

/// Relative to the top left of the screen
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScreenRelativeCoords {
    pub x: usize,
    pub y: usize,
}

impl ScreenRelativeCoords {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn offset_by(&self, rel: &ParentRelativeCoords) -> Self {
        Self {
            x: self.x + rel.x,
            y: self.y + rel.y,
        }
    }
}

static WIDGET_ID: ::std::sync::atomic::AtomicUsize = ::std::sync::atomic::AtomicUsize::new(0);

/// The `WidgetId` uniquely describes an instance of a widget.
/// Creating a new `WidgetId` generates a new unique identifier which can
/// be safely copied and moved around; each copy refers to the same widget.
/// The intent is that you set up the identifiers once and re-use them,
/// rather than generating new ids on each iteration of the UI loop so that
/// the widget state is maintained correctly by the Ui.
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct WidgetId(usize);

impl WidgetId {
    pub fn new() -> Self {
        WidgetId(WIDGET_ID.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed))
    }
}

impl Default for WidgetId {
    fn default() -> Self {
        Self::new()
    }
}
