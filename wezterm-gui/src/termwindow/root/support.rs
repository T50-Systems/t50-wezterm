const ATLAS_SIZE: usize = 128;

lazy_static::lazy_static! {
    static ref WINDOW_CLASS: Mutex<String> = Mutex::new(wezterm_gui_subcommands::DEFAULT_WINDOW_CLASS.to_owned());
    static ref POSITION: Mutex<Option<GuiPosition>> = Mutex::new(None);
}

pub const ICON_DATA: &'static [u8] = include_bytes!("../../../../assets/icon/terminal.png");

pub fn set_window_position(pos: GuiPosition) {
    POSITION.lock().unwrap().replace(pos);
}

pub fn set_window_class(cls: &str) {
    *WINDOW_CLASS.lock().unwrap() = cls.to_owned();
}

pub fn get_window_class() -> String {
    WINDOW_CLASS.lock().unwrap().clone()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MouseCapture {
    UI,
    TerminalPane(PaneId),
}

/// Type used together with Window::notify to do something in the
/// context of the window-specific event loop
pub enum TermWindowNotif {
    InvalidateShapeCache,
    PerformAssignment {
        pane_id: PaneId,
        assignment: KeyAssignment,
        tx: Option<Sender<anyhow::Result<()>>>,
    },
    SetLeftStatus(String),
    SetRightStatus(String),
    SetSecondaryBar {
        left: String,
        center: String,
        right: String,
    },
    GetDimensions(Sender<(Dimensions, WindowState)>),
    GetSelectionForPane {
        pane_id: PaneId,
        tx: Sender<String>,
    },
    GetEffectiveConfig(Sender<ConfigHandle>),
    FinishWindowEvent {
        name: String,
        again: bool,
    },
    GetConfigOverrides(Sender<wezterm_dynamic::Value>),
    SetConfigOverrides(wezterm_dynamic::Value),
    CancelOverlayForPane(PaneId),
    CancelOverlayForTab {
        tab_id: TabId,
        pane_id: Option<PaneId>,
    },
    MuxNotification(MuxNotification),
    EmitStatusUpdate,
    Apply(Box<dyn FnOnce(&mut TermWindow) + Send + Sync>),
    SwitchToMuxWindow(MuxWindowId),
    SetInnerSize {
        width: usize,
        height: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UIItemType {
    TabBar(TabBarItem),
    CloseTab(usize),
    AboveScrollThumb,
    ScrollThumb,
    BelowScrollThumb,
    Split(PositionedSplit),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UIItem {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub item_type: UIItemType,
}

impl UIItem {
    pub fn hit_test(&self, x: isize, y: isize) -> bool {
        x >= self.x as isize
            && x < (self.x + self.width) as isize
            && y >= self.y as isize
            && y < (self.y + self.height) as isize
    }
}

#[cfg(test)]
mod ui_item_hit_test_tests {
    use super::{TabBarItem, UIItem, UIItemType};

    fn tab_bar_item() -> UIItem {
        UIItem {
            x: 10,
            y: 20,
            width: 30,
            height: 5,
            item_type: UIItemType::TabBar(TabBarItem::PaneStatus {
                pane_id: 7,
                active: false,
            }),
        }
    }

    #[test]
    fn hit_test_accepts_points_inside_half_open_bounds() {
        let item = tab_bar_item();

        assert!(item.hit_test(10, 20));
        assert!(item.hit_test(39, 24));
    }

    #[test]
    fn hit_test_rejects_points_just_outside_bounds() {
        let item = tab_bar_item();

        assert!(!item.hit_test(9, 20));
        assert!(!item.hit_test(40, 20));
        assert!(!item.hit_test(10, 19));
        assert!(!item.hit_test(10, 25));
    }
}

#[derive(Clone, Default)]
pub struct SemanticZoneCache {
    seqno: SequenceNo,
    zones: Vec<StableRowIndex>,
}

pub struct OverlayState {
    pub pane: Arc<dyn Pane>,
    pub key_table_state: KeyTableState,
}

#[derive(Default)]
pub struct PaneState {
    /// If is_some(), the top row of the visible screen.
    /// Otherwise, the viewport is at the bottom of the
    /// scrollback.
    viewport: Option<StableRowIndex>,
    selection: Selection,
    /// If is_some(), rather than display the actual tab
    /// contents, we're overlaying a little internal application
    /// tab.  We'll also route input to it.
    pub overlay: Option<OverlayState>,

    bell_start: Option<Instant>,
    pub mouse_terminal_coords: Option<(ClickPosition, StableRowIndex)>,
}

/// Data used when synchronously formatting pane and window titles
#[derive(Debug, Clone)]
pub struct TabInformation {
    pub tab_id: TabId,
    pub tab_index: usize,
    pub is_active: bool,
    pub is_last_active: bool,
    pub active_pane: Option<PaneInformation>,
    pub window_id: MuxWindowId,
    pub tab_title: String,
}

impl UserData for TabInformation {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("tab_id", |_, this| Ok(this.tab_id));
        fields.add_field_method_get("tab_index", |_, this| Ok(this.tab_index));
        fields.add_field_method_get("is_active", |_, this| Ok(this.is_active));
        fields.add_field_method_get("is_last_active", |_, this| Ok(this.is_last_active));
        fields.add_field_method_get("active_pane", |_, this| {
            if let Some(pane) = &this.active_pane {
                Ok(Some(pane.clone()))
            } else {
                Ok(None)
            }
        });
        fields.add_field_method_get("panes", |_, this| {
            let mux = Mux::get();
            let mut panes = vec![];
            if let Some(tab) = mux.get_tab(this.tab_id) {
                panes = tab
                    .iter_panes()
                    .iter()
                    .map(TermWindow::pos_pane_to_pane_info)
                    .collect();
            }
            Ok(panes)
        });
        fields.add_field_method_get("window_id", |_, this| Ok(this.window_id));
        fields.add_field_method_get("tab_title", |_, this| Ok(this.tab_title.clone()));
        fields.add_field_method_get("window_title", |_, this| {
            let mux = Mux::get();
            let window = mux.get_window(this.window_id).ok_or_else(|| {
                mlua::Error::external(format!("window {} not found", this.window_id))
            })?;
            Ok(window.get_title().to_string())
        });
    }
}

/// Data used when synchronously formatting pane and window titles
#[derive(Debug, Clone)]
pub struct PaneInformation {
    pub pane_id: PaneId,
    pub pane_index: usize,
    pub is_active: bool,
    pub is_zoomed: bool,
    pub has_unseen_output: bool,
    pub left: usize,
    pub top: usize,
    pub width: usize,
    pub height: usize,
    pub pixel_width: usize,
    pub pixel_height: usize,
    pub title: String,
    pub user_vars: HashMap<String, String>,
    pub progress: Progress,
}

impl UserData for PaneInformation {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("pane_id", |_, this| Ok(this.pane_id));
        fields.add_field_method_get("pane_index", |_, this| Ok(this.pane_index));
        fields.add_field_method_get("is_active", |_, this| Ok(this.is_active));
        fields.add_field_method_get("is_zoomed", |_, this| Ok(this.is_zoomed));
        fields.add_field_method_get("has_unseen_output", |_, this| Ok(this.has_unseen_output));
        fields.add_field_method_get("left", |_, this| Ok(this.left));
        fields.add_field_method_get("top", |_, this| Ok(this.top));
        fields.add_field_method_get("width", |_, this| Ok(this.width));
        fields.add_field_method_get("height", |_, this| Ok(this.height));
        fields.add_field_method_get("pixel_width", |_, this| Ok(this.pixel_width));
        fields.add_field_method_get("pixel_height", |_, this| Ok(this.pixel_height));
        fields.add_field_method_get("progress", |lua, this| lua.to_value(&this.progress));
        fields.add_field_method_get("title", |_, this| Ok(this.title.clone()));
        fields.add_field_method_get("user_vars", |_, this| Ok(this.user_vars.clone()));
        fields.add_field_method_get("foreground_process_name", |_, this| {
            let mut name = None;
            if let Some(mux) = Mux::try_get() {
                if let Some(pane) = mux.get_pane(this.pane_id) {
                    name = pane.get_foreground_process_name(CachePolicy::AllowStale);
                }
            }
            match name {
                Some(name) => Ok(name),
                None => Ok("".to_string()),
            }
        });
        fields.add_field_method_get("tty_name", |_, this| {
            let mut name = None;
            if let Some(mux) = Mux::try_get() {
                if let Some(pane) = mux.get_pane(this.pane_id) {
                    name = pane.tty_name();
                }
            }
            Ok(name)
        });
        fields.add_field_method_get("current_working_dir", |_, this| {
            if let Some(mux) = Mux::try_get() {
                if let Some(pane) = mux.get_pane(this.pane_id) {
                    return Ok(pane
                        .get_current_working_dir(CachePolicy::AllowStale)
                        .map(|url| url_funcs::Url { url }));
                }
            }
            Ok(None)
        });
        fields.add_field_method_get("domain_name", |_, this| {
            let mut name = None;
            if let Some(mux) = Mux::try_get() {
                if let Some(pane) = mux.get_pane(this.pane_id) {
                    let domain_id = pane.domain_id();
                    name = mux
                        .get_domain(domain_id)
                        .map(|dom| dom.domain_name().to_string());
                }
            }
            match name {
                Some(name) => Ok(name),
                None => Ok("".to_string()),
            }
        });
    }
}

#[derive(Default)]
pub struct TabState {
    /// If is_some(), rather than display the actual tab
    /// contents, we're overlaying a little internal application
    /// tab.  We'll also route input to it.
    pub overlay: Option<OverlayState>,
}

/// Manages the state/queue of lua based event handlers.
/// We don't want to queue more than 1 event at a time,
/// so we use this enum to allow for at most 1 executing
/// and 1 pending event.
#[derive(Copy, Clone, Debug)]
enum EventState {
    /// The event is not running
    None,
    /// The event is running
    InProgress,
    /// The event is running, and we have another one ready to
    /// run once it completes
    InProgressWithQueued(Option<PaneId>),
}

pub struct TermWindow {
    pub window: Option<Window>,
    pub config: ConfigHandle,
    pub config_overrides: wezterm_dynamic::Value,
    os_parameters: Option<parameters::Parameters>,
    /// When we most recently received keyboard focus
    pub focused: Option<Instant>,
    fonts: Rc<FontConfiguration>,
    /// Window dimensions and dpi
    pub dimensions: Dimensions,
    pub window_state: WindowState,
    pub resizes_pending: usize,
    is_repaint_pending: bool,
    pending_scale_changes: LinkedList<resize::ScaleChange>,
    /// Terminal dimensions
    terminal_size: TerminalSize,
    pub mux_window_id: MuxWindowId,
    pub mux_window_id_for_subscriptions: Arc<Mutex<MuxWindowId>>,
    pub render_metrics: RenderMetrics,
    render_state: Option<RenderState>,
    input_map: InputMap,
    /// If is_some, the LEADER modifier is active until the specified instant.
    leader_is_down: Option<std::time::Instant>,
    dead_key_status: DeadKeyStatus,
    key_table_state: KeyTableState,
    show_tab_bar: bool,
    show_scroll_bar: bool,
    tab_bar: TabBarState,
    secondary_tab_bar: TabBarState,
    fancy_tab_bar: Option<box_model::ComputedElement>,
    fancy_secondary_tab_bar: Option<box_model::ComputedElement>,
    pub right_status: String,
    pub left_status: String,
    pub secondary_right_status: String,
    pub secondary_center_status: String,
    pub secondary_left_status: String,
    last_ui_item: Option<UIItem>,
    /// Tracks whether the current mouse-down event is part of click-focus.
    /// If so, we ignore mouse events until released
    is_click_to_focus_window: bool,
    last_mouse_coords: (usize, i64),
    window_drag_position: Option<MouseEvent>,
    current_mouse_event: Option<MouseEvent>,
    prev_cursor: PrevCursorPos,
    last_scroll_info: RenderableDimensions,

    tab_state: RefCell<HashMap<TabId, TabState>>,
    pane_state: RefCell<HashMap<PaneId, PaneState>>,
    semantic_zones: HashMap<PaneId, SemanticZoneCache>,

    window_background: Vec<LoadedBackgroundLayer>,

    current_modifier_and_leds: (Modifiers, KeyboardLedStatus),
    current_mouse_buttons: Vec<MousePress>,
    current_mouse_capture: Option<MouseCapture>,

    opengl_info: Option<String>,

    /// Keeps track of double and triple clicks
    last_mouse_click: Option<LastMouseClick>,

    /// The URL over which we are currently hovering
    current_highlight: Option<Arc<Hyperlink>>,

    quad_generation: usize,
    shape_generation: usize,
    shape_cache: RefCell<LfuCache<ShapeCacheKey, anyhow::Result<Rc<Vec<ShapedInfo>>>>>,
    line_to_ele_shape_cache: RefCell<LfuCache<LineToEleShapeCacheKey, LineToElementShapeItem>>,

    line_state_cache: RefCell<LfuCacheU64<Arc<CachedLineState>>>,
    next_line_state_id: u64,

    line_quad_cache: RefCell<LfuCache<LineQuadCacheKey, LineQuadCacheValue>>,

    last_status_call: Instant,
    cursor_blink_state: RefCell<ColorEase>,
    blink_state: RefCell<ColorEase>,
    rapid_blink_state: RefCell<ColorEase>,

    palette: Option<ColorPalette>,

    ui_items: Vec<UIItem>,
    dragging: Option<(UIItem, MouseEvent)>,

    modal: RefCell<Option<Rc<dyn Modal>>>,

    event_states: HashMap<String, EventState>,
    pub current_event: Option<Value>,
    has_animation: RefCell<Option<Instant>>,
    /// We use this to attempt to do something reasonable
    /// if we run out of texture space
    allow_images: AllowImage,
    scheduled_animation: RefCell<Option<Instant>>,

    created: Instant,

    pub last_frame_duration: Duration,
    last_fps_check_time: Instant,
    num_frames: usize,
    pub fps: f32,

    connection_name: String,

    gl: Option<Rc<glium::backend::Context>>,
    webgpu: Option<Rc<WebGpuState>>,
    config_subscription: Option<config::ConfigSubscription>,
}
