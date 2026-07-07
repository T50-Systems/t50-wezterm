use crate::os::xkeysyms::keysym_to_keycode;
use crate::{
    DeadKeyStatus, Handled, KeyCode, KeyEvent, Modifiers, RawKeyEvent, WindowEvent,
    WindowEventSender, WindowKeyEvent,
};
use anyhow::{anyhow, ensure};
use libc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, OsStr};
use std::os::unix::ffi::OsStrExt;
use wezterm_input_types::{KeyboardLedStatus, PhysKeyCode};
use xcb::x::KeyButMask;
use xkb::compose::Status as ComposeStatus;
use xkbcommon::xkb;
use xkbcommon::xkb::{LayoutIndex, ModMask};

pub struct Keyboard {
    context: xkb::Context,
    keymap: RefCell<xkb::Keymap>,
    device_id: i32,

    state: RefCell<xkb::State>,
    compose_state: RefCell<Compose>,
    phys_code_map: RefCell<HashMap<xkb::Keycode, PhysKeyCode>>,
    mods_leds: RefCell<(Modifiers, KeyboardLedStatus)>,
    last_xcb_state: RefCell<StateFromXcbStateNotify>,
    label: &'static str,
}

#[derive(Default, Debug, Clone, Copy)]
struct StateFromXcbStateNotify {
    depressed_mods: ModMask,
    latched_mods: ModMask,
    locked_mods: ModMask,
    depressed_layout: LayoutIndex,
    latched_layout: LayoutIndex,
    locked_layout: LayoutIndex,
}

pub struct KeyboardWithFallback {
    selected: Keyboard,
    fallback: Keyboard,
}

struct Compose {
    state: xkb::compose::State,
    composition: String,
    label: &'static str,
}

#[derive(Debug)]
enum FeedResult {
    Composing(String),
    Composed(String, xkb::Keysym),
    Nothing(String, xkb::Keysym),
    Cancelled,
}

impl Compose {
    fn reset(&mut self) {
        self.composition.clear();
        self.state.reset();
    }

    fn feed(
        &mut self,
        xcode: xkb::Keycode,
        xsym: xkb::Keysym,
        key_state: &RefCell<xkb::State>,
    ) -> FeedResult {
        if matches!(
            self.state.status(),
            ComposeStatus::Nothing | ComposeStatus::Cancelled | ComposeStatus::Composed
        ) {
            self.composition.clear();
        }

        let previously_composing = !self.composition.is_empty();
        let feed_result = self.state.feed(xsym);
        log::trace!(
            "Compose::feed({}) {xsym:?} -> result={feed_result:?} status={:?}",
            self.label,
            self.state.status()
        );

        match self.state.status() {
            ComposeStatus::Composing => {
                if !previously_composing {
                    // The common case for dead keys is a single combining sequence,
                    // and usually pressing the key a second time (or following it
                    // by a space) will output the key from the keycap.
                    // During composition we want to show that as the composition
                    // status, so we clock the state machine forwards to produce it,
                    // then reset and feed in the symbol again to get it ready
                    // for the next keypress

                    self.state.feed(xsym);
                    if self.state.status() == ComposeStatus::Composed {
                        if let Some(s) = self.state.utf8() {
                            self.composition = s;
                        }
                    }

                    self.state.reset();
                    self.state.feed(xsym);
                }

                if self.composition.is_empty() || previously_composing {
                    // If we didn't manage to resolve a string above,
                    // or if we're in a multi-key composition sequence,
                    // we don't have a fantastic way to indicate what is
                    // currently being composed, so we try to get something
                    // that might be meaningful by getting the utf8 for that
                    // key if known.
                    // We used to fall back to the name of the keysym, but
                    // feedback was that is was undesirable
                    // <https://github.com/wezterm/wezterm/issues/4511>
                    let key_state = key_state.borrow();
                    let utf8 = key_state.key_get_utf8(xcode);
                    if !utf8.is_empty() {
                        self.composition.push_str(&utf8);
                    }
                    if self.composition.is_empty() {
                        // Ensure that we have something in the composition
                        self.composition.push(' ');
                    }
                }
                FeedResult::Composing(self.composition.clone())
            }
            ComposeStatus::Composed => {
                let res = self.state.keysym();
                let composed = self.state.utf8().unwrap_or_default();
                self.state.reset();
                FeedResult::Composed(composed, res.unwrap_or(xsym))
            }
            ComposeStatus::Nothing => {
                let utf8 = key_state.borrow().key_get_utf8(xcode);
                FeedResult::Nothing(utf8, xsym)
            }
            ComposeStatus::Cancelled => {
                self.state.reset();
                FeedResult::Cancelled
            }
        }
    }
}

fn default_keymap(context: &xkb::Context) -> Option<xkb::Keymap> {
    // use $XKB_DEFAULT_RULES or system default
    let system_default_rules = "";
    // use $XKB_DEFAULT_MODEL or system default
    let system_default_model = "";
    // use $XKB_DEFAULT_LAYOUT or system default
    let system_default_layout = "";
    // use $XKB_DEFAULT_VARIANT or system default
    let system_default_variant = "";

    xkb::Keymap::new_from_names(
        context,
        system_default_rules,
        system_default_model,
        system_default_layout,
        system_default_variant,
        None,
        xkb::KEYMAP_COMPILE_NO_FLAGS,
    )
