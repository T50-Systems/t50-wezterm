
impl Keyboard {
    pub fn new_default() -> anyhow::Result<Self> {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = default_keymap(&context)
            .ok_or_else(|| anyhow!("Failed to load system default keymap"))?;

        let state = xkb::State::new(&keymap);
        let locale = query_lc_ctype()?;

        let table =
            xkb::compose::Table::new_from_locale(&context, locale, xkb::compose::COMPILE_NO_FLAGS)
                .map_err(|_| anyhow!("Failed to acquire compose table from locale"))?;
        let compose_state = xkb::compose::State::new(&table, xkb::compose::STATE_NO_FLAGS);

        let phys_code_map = build_physkeycode_map(&keymap);
        let label = "fallback";

        Ok(Self {
            context,
            device_id: -1,
            keymap: RefCell::new(keymap),
            state: RefCell::new(state),
            compose_state: RefCell::new(Compose {
                state: compose_state,
                composition: String::new(),
                label,
            }),
            phys_code_map: RefCell::new(phys_code_map),
            mods_leds: RefCell::new(Default::default()),
            last_xcb_state: RefCell::new(Default::default()),
            label,
        })
    }

    pub fn new_from_string(s: String) -> anyhow::Result<Self> {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = xkb::Keymap::new_from_string(
            &context,
            s,
            xkbcommon::xkb::KEYMAP_FORMAT_TEXT_V1,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
        .ok_or_else(|| anyhow!("Failed to parse keymap state from file"))?;

        let state = xkb::State::new(&keymap);
        let locale = query_lc_ctype()?;

        let table =
            xkb::compose::Table::new_from_locale(&context, locale, xkb::compose::COMPILE_NO_FLAGS)
                .map_err(|_| anyhow!("Failed to acquire compose table from locale"))?;
        let compose_state = xkb::compose::State::new(&table, xkb::compose::STATE_NO_FLAGS);

        let phys_code_map = build_physkeycode_map(&keymap);
        let label = "selected";

        Ok(Self {
            context,
            device_id: -1,
            keymap: RefCell::new(keymap),
            state: RefCell::new(state),
            compose_state: RefCell::new(Compose {
                state: compose_state,
                composition: String::new(),
                label,
            }),
            phys_code_map: RefCell::new(phys_code_map),
            mods_leds: RefCell::new(Default::default()),
            last_xcb_state: RefCell::new(Default::default()),
            label,
        })
    }

    pub fn new(connection: &xcb::Connection) -> anyhow::Result<(Keyboard, u8)> {
        let first_ev = xcb::xkb::get_extension_data(connection)
            .ok_or_else(|| anyhow!("could not get xkb extension data"))?
            .first_event;

        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let device_id = xkb::x11::get_core_keyboard_device_id(&connection);
        ensure!(device_id != -1, "Couldn't find core keyboard device");

        let keymap = xkb::x11::keymap_new_from_device(
            &context,
            &connection,
            device_id,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        );

        let state = xkb::x11::state_new_from_device(&keymap, connection, device_id);

        let locale = query_lc_ctype()?;

        let table =
            xkb::compose::Table::new_from_locale(&context, locale, xkb::compose::COMPILE_NO_FLAGS)
                .map_err(|_| anyhow!("Failed to acquire compose table from locale"))?;
        let compose_state = xkb::compose::State::new(&table, xkb::compose::STATE_NO_FLAGS);

        {
            let map_parts = xcb::xkb::MapPart::KEY_TYPES
                | xcb::xkb::MapPart::KEY_SYMS
                | xcb::xkb::MapPart::MODIFIER_MAP
                | xcb::xkb::MapPart::EXPLICIT_COMPONENTS
                | xcb::xkb::MapPart::KEY_ACTIONS
                | xcb::xkb::MapPart::KEY_BEHAVIORS
                | xcb::xkb::MapPart::VIRTUAL_MODS
                | xcb::xkb::MapPart::VIRTUAL_MOD_MAP;

            let events = xcb::xkb::EventType::NEW_KEYBOARD_NOTIFY
                | xcb::xkb::EventType::MAP_NOTIFY
                | xcb::xkb::EventType::STATE_NOTIFY;

            connection.check_request(connection.send_request_checked(&xcb::xkb::SelectEvents {
                device_spec: device_id as u16,
                affect_which: events,
                clear: xcb::xkb::EventType::empty(),
                select_all: events,
                affect_map: map_parts,
                map: map_parts,
                details: &[],
            }))?;
        }

        let phys_code_map = build_physkeycode_map(&keymap);
        let label = "selected";

        let kbd = Keyboard {
            context,
            device_id,
            keymap: RefCell::new(keymap),
            state: RefCell::new(state),
            compose_state: RefCell::new(Compose {
                state: compose_state,
                composition: String::new(),
                label,
            }),
            phys_code_map: RefCell::new(phys_code_map),
            mods_leds: RefCell::new(Default::default()),
            last_xcb_state: RefCell::new(Default::default()),
            label,
        };

        Ok((kbd, first_ev))
    }

    /// Returns true if a given wayland keycode allows for automatic key repeats
    pub fn wayland_key_repeats(&self, code: u32) -> bool {
        self.keymap
            .borrow()
            .key_repeats(xkb::Keycode::new(code + 8))
    }

    pub fn get_device_id(&self) -> i32 {
        self.device_id
    }

    fn compose_feed(&self, xcode: xkb::Keycode, xsym: xkb::Keysym) -> FeedResult {
        self.compose_state
            .borrow_mut()
            .feed(xcode, xsym, &self.state)
    }

    pub fn compose_clear(&self) {
        self.compose_state.borrow_mut().reset();
    }

    pub fn update_modifier_state(
        &self,
        mods_depressed: u32,
        mods_latched: u32,
        mods_locked: u32,
        group: u32,
    ) {
        self.state.borrow_mut().update_mask(
            xkb::ModMask::from(mods_depressed),
            xkb::ModMask::from(mods_latched),
            xkb::ModMask::from(mods_locked),
            0,
            0,
            xkb::LayoutIndex::from(group),
        );
    }

    pub fn update_state(&self, ev: &xcb::xkb::StateNotifyEvent) {
        let state = StateFromXcbStateNotify {
            depressed_mods: xkb::ModMask::from(ev.base_mods().bits()),
            latched_mods: xkb::ModMask::from(ev.latched_mods().bits()),
            locked_mods: xkb::ModMask::from(ev.locked_mods().bits()),
            depressed_layout: ev.base_group() as xkb::LayoutIndex,
            latched_layout: ev.latched_group() as xkb::LayoutIndex,
            locked_layout: xkb::LayoutIndex::from(ev.locked_group() as u32),
        };
        log::trace!("update_state({}) with {state:?}", self.label);

        self.state.borrow_mut().update_mask(
            state.depressed_mods,
            state.latched_mods,
            state.locked_mods,
            state.depressed_layout,
            state.latched_layout,
            state.locked_layout,
        );

        *self.last_xcb_state.borrow_mut() = state;
    }

    pub fn merge_current_xcb_modifiers(&self, mods: ModMask) {
        let state = self.last_xcb_state.borrow().clone();
        log::trace!(
            "merge_current_xcb_modifiers({}); state before={state:?}, mods={mods:?}",
            self.label
        );
        self.state.borrow_mut().update_mask(
            mods,
            0,
            0,
            state.depressed_layout,
            state.latched_layout,
            state.locked_layout,
        );
    }

    pub fn reapply_last_xcb_state(&self) {
        let state = self.last_xcb_state.borrow().clone();
        self.state.borrow_mut().update_mask(
            state.depressed_mods,
            state.latched_mods,
            state.locked_mods,
            state.depressed_layout,
            state.latched_layout,
            state.locked_layout,
        );
    }

    pub fn update_keymap(&self, connection: &xcb::Connection) -> anyhow::Result<()> {
        log::debug!("update_keymap({}) was called", self.label);

        let new_keymap = xkb::x11::keymap_new_from_device(
            &self.context,
            &connection,
            self.get_device_id(),
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        );
        ensure!(
            !new_keymap.get_raw_ptr().is_null(),
            "problem with new keymap"
        );

        let new_state = xkb::x11::state_new_from_device(&new_keymap, connection, self.device_id);
        ensure!(!new_state.get_raw_ptr().is_null(), "problem with new state");
        let phys_code_map = build_physkeycode_map(&new_keymap);

        self.state.replace(new_state);
        self.keymap.replace(new_keymap);
        self.phys_code_map.replace(phys_code_map);
        Ok(())
    }
