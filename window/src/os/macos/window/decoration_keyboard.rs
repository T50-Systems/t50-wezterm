fn effective_decorations(
    mut decorations: WindowDecorations,
    integrated_title_button_style: IntegratedTitleButtonStyle,
) -> WindowDecorations {
    if integrated_title_button_style != IntegratedTitleButtonStyle::MacOsNative {
        decorations.remove(WindowDecorations::INTEGRATED_BUTTONS);
    }
    decorations
}

fn apply_decorations_to_window(
    window: &StrongPtr,
    decorations: WindowDecorations,
    integrated_title_button_style: IntegratedTitleButtonStyle,
) {
    let mask = decoration_to_mask(decorations, integrated_title_button_style);
    let decorations = effective_decorations(decorations, integrated_title_button_style);
    unsafe {
        window.setStyleMask_(mask);

        let hidden = if decorations.contains(WindowDecorations::TITLE)
            || decorations.contains(WindowDecorations::INTEGRATED_BUTTONS)
        {
            NO
        } else {
            YES
        };

        for titlebar_button in &[
            appkit::NSWindowButton::NSWindowMiniaturizeButton,
            appkit::NSWindowButton::NSWindowCloseButton,
            appkit::NSWindowButton::NSWindowZoomButton,
        ] {
            let button = window.standardWindowButton_(*titlebar_button);
            let _: () = msg_send![button, setHidden: hidden];
        }

        window.setTitleVisibility_(if decorations.contains(WindowDecorations::TITLE) {
            appkit::NSWindowTitleVisibility::NSWindowTitleVisible
        } else {
            appkit::NSWindowTitleVisibility::NSWindowTitleHidden
        });

        if decorations.contains(WindowDecorations::INTEGRATED_BUTTONS)
            || decorations.contains(WindowDecorations::MACOS_USE_BACKGROUND_COLOR_AS_TITLEBAR_COLOR)
        {
            window.setTitlebarAppearsTransparent_(YES);
        } else {
            window.setTitlebarAppearsTransparent_(hidden);
        }
    }
}

fn decoration_to_mask(
    decorations: WindowDecorations,
    integrated_title_button_style: IntegratedTitleButtonStyle,
) -> NSWindowStyleMask {
    let decorations = effective_decorations(decorations, integrated_title_button_style);
    let decorations = decorations.difference(
        WindowDecorations::MACOS_FORCE_DISABLE_SHADOW
            | WindowDecorations::MACOS_FORCE_ENABLE_SHADOW,
    );
    if decorations == WindowDecorations::TITLE | WindowDecorations::RESIZE {
        NSWindowStyleMask::NSTitledWindowMask
            | NSWindowStyleMask::NSClosableWindowMask
            | NSWindowStyleMask::NSMiniaturizableWindowMask
            | NSWindowStyleMask::NSResizableWindowMask
    } else if decorations
        == WindowDecorations::MACOS_FORCE_SQUARE_CORNERS | WindowDecorations::RESIZE
    {
        NSWindowStyleMask::NSClosableWindowMask
            | NSWindowStyleMask::NSMiniaturizableWindowMask
            | NSWindowStyleMask::NSResizableWindowMask
            | NSWindowStyleMask::NSFullSizeContentViewWindowMask
    } else if decorations == WindowDecorations::RESIZE
        || decorations == WindowDecorations::INTEGRATED_BUTTONS
        || decorations == WindowDecorations::INTEGRATED_BUTTONS | WindowDecorations::RESIZE
    {
        NSWindowStyleMask::NSTitledWindowMask
            | NSWindowStyleMask::NSClosableWindowMask
            | NSWindowStyleMask::NSMiniaturizableWindowMask
            | NSWindowStyleMask::NSResizableWindowMask
            | NSWindowStyleMask::NSFullSizeContentViewWindowMask
    } else if decorations == WindowDecorations::NONE {
        NSWindowStyleMask::NSTitledWindowMask
            | NSWindowStyleMask::NSClosableWindowMask
            | NSWindowStyleMask::NSMiniaturizableWindowMask
            | NSWindowStyleMask::NSFullSizeContentViewWindowMask
    } else if decorations == WindowDecorations::TITLE {
        NSWindowStyleMask::NSTitledWindowMask
            | NSWindowStyleMask::NSClosableWindowMask
            | NSWindowStyleMask::NSMiniaturizableWindowMask
    } else if decorations == WindowDecorations::MACOS_FORCE_SQUARE_CORNERS {
        NSWindowStyleMask::NSClosableWindowMask
            | NSWindowStyleMask::NSMiniaturizableWindowMask
            | NSWindowStyleMask::NSFullSizeContentViewWindowMask
    } else {
        NSWindowStyleMask::NSTitledWindowMask
            | NSWindowStyleMask::NSClosableWindowMask
            | NSWindowStyleMask::NSMiniaturizableWindowMask
            | NSWindowStyleMask::NSResizableWindowMask
    }
}

unsafe fn get_view_class_name(id: id) -> Option<String> {
    if id.is_null() {
        return None;
    }

    let class_name: id = msg_send![id, className];

    if class_name.is_null() {
        return None;
    }

    let cstr = CStr::from_ptr(class_name.UTF8String()).to_str();

    match cstr {
        Ok(s) => Some(s.to_string()),
        Err(_) => None,
    }
}

fn get_titlebar_view_container(window: &StrongPtr) -> Option<WeakPtr> {
    // The view container for the titlebar on macos is found next to the primary window view
    // so we need to traverse up to the super view to find it
    let super_view = get_view_superview(window)?;

    let sub_views = get_view_subviews(&super_view.load())?;

    let count = unsafe { sub_views.load().count() };

    for i in 0..count {
        let sub_view: id = unsafe { sub_views.load().objectAtIndex(i) };

        if sub_view.is_null() {
            continue;
        }

        let class_name = unsafe { get_view_class_name(sub_view)? };

        if class_name == TITLEBAR_VIEW_NAME {
            let titlebar_view = unsafe { WeakPtr::new(sub_view) };
            return Some(titlebar_view);
        }
    }

    None
}

fn get_view_superview(view: &StrongPtr) -> Option<WeakPtr> {
    let super_view_id: id = unsafe { msg_send![view.contentView(), superview] };

    if super_view_id.is_null() {
        return None;
    }

    let super_view = unsafe { WeakPtr::new(super_view_id) };

    Some(super_view)
}

fn get_view_subviews(view: &StrongPtr) -> Option<WeakPtr> {
    let sub_views_id: id = unsafe { msg_send![**view, subviews] };
    if sub_views_id.is_null() {
        return None;
    }

    let sub_views = unsafe { WeakPtr::new(sub_views_id) };
    Some(sub_views)
}

#[derive(Debug)]
struct DeadKeyState {
    /// The private dead key state preserved from UCKeyTranslate
    dead_state: u32,
}

struct Inner {
    events: WindowEventSender,
    view_id: Option<WeakPtr>,
    window: Option<WeakPtr>,
    screen_changed: bool,
    paint_throttled: bool,
    window_id: usize,
    invalidated: bool,
    gl_context_pair: Option<GlContextPair>,
    text_cursor_position: Rect,
    tracking_rect_tag: NSInteger,
    hscroll_remainder: f64,
    vscroll_remainder: f64,
    last_wheel: Instant,
    /// We use this to avoid double-emitting events when
    /// procesing key-up events.
    key_is_down: Option<bool>,

    /// First in a dead-key sequence
    dead_pending: Option<DeadKeyState>,

    /// When using simple fullscreen mode, this tracks
    /// the window dimensions that need to be restored
    fullscreen: Option<NSRect>,

    config: ConfigHandle,

    /// Used to signal when IME really just swallowed a key
    ime_state: ImeDisposition,
    /// Captures the last event that had ImeDisposition::Acted,
    /// so that we can use it to generate a repeat in the cases
    /// where the IME mysteriously swallows repeats but only
    /// for certain keys.
    ime_last_event: Option<KeyEvent>,

    /// Whether we're in live resize
    live_resizing: bool,

    ime_text: String,
}

#[repr(C)]
pub struct __InputSource {
    _dummy: i32,
}
pub type InputSourceRef = *const __InputSource;

declare_TCFType!(InputSource, InputSourceRef);
impl_TCFType!(InputSource, InputSourceRef, TISInputSourceGetTypeID);

#[repr(C)]
struct UCKeyboardLayout {
    _dummy: i32,
}

type UniCharCount = std::os::raw::c_ulong;

/// key is going down
#[allow(non_upper_case_globals)]
const kUCKeyActionDown: u16 = 0;
/// key is going up
#[allow(non_upper_case_globals, dead_code)]
const kUCKeyActionUp: u16 = 1;
/// auto-key down
#[allow(non_upper_case_globals, dead_code)]
const kUCKeyActionAutoKey: u16 = 2;
/// get information for key display (as in Key Caps)
#[allow(non_upper_case_globals)]
const kUCKeyActionDisplay: u16 = 3;

extern "C" {
    fn TISInputSourceGetTypeID() -> CFTypeID;
    fn TISCopyCurrentKeyboardInputSource() -> InputSourceRef;
    fn TISGetInputSourceProperty(source: InputSourceRef, propertyKey: CFStringRef) -> CFDataRef;

    static kTISPropertyUnicodeKeyLayoutData: CFStringRef;

    fn UCKeyTranslate(
        layout: *const UCKeyboardLayout,
        virtualKeyCode: u16,
        keyAction: u16,
        modifierKeyState: u32,
        keyboardType: u32,
        keyTranslateOptions: u32,
        deadKeyState: *mut u32,
        maxStringLength: UniCharCount,
        actualStringLength: *mut UniCharCount,
        unicodeString: *mut UniChar,
    ) -> u32;

    fn LMGetKbdType() -> u8;
}

#[derive(Debug)]
enum TranslateStatus {
    Composing(String),
    Composed(String),
    NotDead,
}

/// Represents the current keyboard layout.
/// Holds state needed to perform keymap translation.
struct Keyboard {
    _kbd: InputSource,
    layout_data: Option<CFData>,
}

/// Slightly more intelligible parameters for keymap translation
struct TranslateParams {
    virtual_key_code: u16,
    modifier_flags: NSEventModifierFlags,
    dead_state: u32,
    ignore_dead_keys: bool,
    display: bool,
}

/// The results of a keymap translation
#[derive(Debug)]
struct TranslateResults {
    dead_state: u32,
    text: String,
}

impl Keyboard {
    pub fn new() -> Self {
        let _kbd =
            unsafe { InputSource::wrap_under_create_rule(TISCopyCurrentKeyboardInputSource()) };

        let layout_data = unsafe {
            let data = TISGetInputSourceProperty(
                _kbd.as_concrete_TypeRef(),
                kTISPropertyUnicodeKeyLayoutData,
            );
            if data.is_null() {
                None
            } else {
                Some(CFData::wrap_under_get_rule(data))
            }
        };
        Self { _kbd, layout_data }
    }

    /// A wrapper around UCKeyTranslate
    pub fn translate(&self, params: TranslateParams) -> anyhow::Result<TranslateResults> {
        let layout_data = match &self.layout_data {
            Some(data) => unsafe {
                CFDataGetBytePtr(data.as_concrete_TypeRef()) as *const UCKeyboardLayout
            },
            None => std::ptr::null(),
        };

        let modifier_key_state: u32 = (params.modifier_flags.bits() >> 16) as u32 & 0xFF;

        let kbd_type = unsafe { LMGetKbdType() } as _;
        #[allow(non_upper_case_globals)]
        const kUCKeyTranslateNoDeadKeysBit: u32 = 0;

        let mut unicode_buffer = [0u16; 32];
        let mut length = 0;
        let mut dead_state = params.dead_state;
        unsafe {
            UCKeyTranslate(
                layout_data,
                params.virtual_key_code,
                if params.display {
                    kUCKeyActionDisplay
                } else {
                    kUCKeyActionDown
                },
                modifier_key_state,
                kbd_type,
                if params.ignore_dead_keys {
                    1 << kUCKeyTranslateNoDeadKeysBit
                } else {
                    0
                },
                &mut dead_state,
                unicode_buffer.len() as _,
                &mut length,
                unicode_buffer.as_mut_ptr(),
            )
        };

        let text = String::from_utf16(unsafe {
            std::slice::from_raw_parts(unicode_buffer.as_mut_ptr(), length as _)
        })?;

        Ok(TranslateResults { text, dead_state })
    }
}

impl Inner {
    fn enable_opengl(&mut self) -> anyhow::Result<Rc<glium::backend::Context>> {
        let view = self.view_id.as_ref().unwrap().load();
        let glium_context = GlContextPair::create(*view)?;

        self.gl_context_pair.replace(glium_context.clone());

        Ok(glium_context.context)
    }

    /// <https://stackoverflow.com/a/22677690>
    /// <https://stackoverflow.com/a/12548163>
    /// <https://stackoverflow.com/a/8263841>
    /// <https://developer.apple.com/documentation/coreservices/1390584-uckeytranslate?language=objc>
    fn translate_key_event(
        &mut self,
        virtual_key_code: u16,
        modifier_flags: NSEventModifierFlags,
    ) -> anyhow::Result<TranslateStatus> {
        let keyboard = Keyboard::new();

        let mods = key_modifiers(modifier_flags);

        let config = &self.config;

        let use_dead_keys = if !config.use_dead_keys {
            false
        } else if mods.contains(Modifiers::LEFT_ALT) {
            config.send_composed_key_when_left_alt_is_pressed
        } else if mods.contains(Modifiers::RIGHT_ALT) {
            config.send_composed_key_when_right_alt_is_pressed
        } else {
            true
        };

        if let Some(DeadKeyState { dead_state }) = self.dead_pending.take() {
            let result = keyboard.translate(TranslateParams {
                virtual_key_code,
                modifier_flags,
                dead_state,
                ignore_dead_keys: false,
                display: true,
            })?;

            // If length == 0 it means that they double-pressed the dead key.
            // We treat that the same as the dead key disabled state:
            // we want to clock through a space keypress so that we clear
            // the state and output the original keypress.
            let generate_space = !use_dead_keys || result.text.len() == 0;

            if generate_space {
                // synthesize a SPACE press to
                // elicit the underlying key code and get out
                // of the dead key state
                let result = keyboard.translate(TranslateParams {
                    virtual_key_code,
                    modifier_flags,
                    dead_state: result.dead_state,
                    ignore_dead_keys: false,
                    display: false,
                })?;
                Ok(TranslateStatus::Composed(result.text))
            } else {
                Ok(TranslateStatus::Composed(result.text))
            }
        } else if use_dead_keys {
            let result = keyboard.translate(TranslateParams {
                virtual_key_code,
                modifier_flags,
                dead_state: 0,
                ignore_dead_keys: false,
                display: false,
            })?;

            self.dead_pending.replace(DeadKeyState {
                dead_state: result.dead_state,
            });

            // Get the non-dead-key rendition to show as the composing state
            let composing = keyboard.translate(TranslateParams {
                virtual_key_code,
                modifier_flags,
                dead_state: 0,
                ignore_dead_keys: true,
                display: true,
            })?;

            Ok(TranslateStatus::Composing(composing.text))
        } else {
            Ok(TranslateStatus::NotDead)
        }
    }
}
