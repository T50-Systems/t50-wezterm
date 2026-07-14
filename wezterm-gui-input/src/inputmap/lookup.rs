impl InputMap {
    /// Given an action, return the corresponding set of application-wide key assignments that are
    /// mapped to it.
    /// If any key_tables reference a given combination, then that combination
    /// is removed from the list.
    /// This is used to figure out whether an application-wide keyboard shortcut
    /// can be safely configured for this action, without interfering with any
    /// transient key_table mappings.
    #[allow(dead_code)]
    pub fn locate_app_wide_key_assignment(
        &self,
        action: &KeyAssignment,
    ) -> Vec<(KeyCode, Modifiers)> {
        let mut candidates = vec![];

        for ((key, mods), entry) in &self.keys.default {
            if mods.contains(Modifiers::LEADER) {
                continue;
            }
            if entry.action == *action {
                candidates.push((key.clone(), mods.clone()));
            }
        }

        // Now ensure that this combination is not part of a key table
        candidates.retain(|tuple| {
            for table in self.keys.by_name.values() {
                if table.contains_key(tuple) {
                    return false;
                }
            }
            true
        });

        candidates
    }

    pub fn is_leader(&self, key: &KeyCode, mods: Modifiers) -> Option<std::time::Duration> {
        if let Some((leader_key, leader_mods, timeout)) = self.leader.as_ref() {
            if *leader_key == *key && *leader_mods == mods.remove_positional_mods() {
                return Some(timeout.clone());
            }
        }
        None
    }

    pub fn has_table(&self, name: &str) -> bool {
        self.keys.by_name.contains_key(name)
    }

    pub fn lookup_key(
        &self,
        key: &KeyCode,
        mods: Modifiers,
        table_name: Option<&str>,
    ) -> Option<KeyTableEntry> {
        let table = match table_name {
            Some(name) => self.keys.by_name.get(name)?,
            None => &self.keys.default,
        };

        table
            .get(&key.normalize_shift(mods.remove_positional_mods()))
            .cloned()
    }

    pub fn lookup_mouse(
        &self,
        event: MouseEventTrigger,
        mut mods: MouseEventTriggerMods,
    ) -> Option<KeyAssignment> {
        mods.mods = mods.mods.remove_positional_mods();
        self.mouse.get(&(event, mods)).cloned()
    }

    pub fn dump_config(&self, key_table: Option<&str>) {
        println!("local wezterm = require 'wezterm'");
        println!("local act = wezterm.action");
        println!();
        println!("return {{");

        if key_table.is_none() {
            println!("  keys = {{");
            show_key_table_as_lua(&self.keys.default, 4);
            println!("  }},");
            println!();
        }

        let mut table_names = self.keys.by_name.keys().collect::<Vec<_>>();
        table_names.sort();
        println!("  key_tables = {{");
        for name in table_names {
            if let Some(wanted_table) = key_table {
                if name != wanted_table {
                    continue;
                }
            }
            if let Some(table) = self.keys.by_name.get(name) {
                println!("    {name} = {{");
                show_key_table_as_lua(table, 6);
                println!("    }},");
                println!();
            }
        }
        println!("  }}");

        println!("}}");
    }

    pub fn show_keys(&self) {
        if let Some((key, mods, duration)) = &self.leader {
            println!("Leader: {key:?} {mods:?} {duration:?}");
        }

        section_header("Default key table");
        show_key_table(&self.keys.default);
        println!();

        let mut table_names = self.keys.by_name.keys().collect::<Vec<_>>();
        table_names.sort();
        for name in table_names {
            if let Some(table) = self.keys.by_name.get(name) {
                section_header(&format!("Key Table: {name}"));
                show_key_table(table);
                println!();
            }
        }

        self.show_mouse();
    }

    fn show_mouse(&self) {
        for (label, alt_screen, mouse_reporting) in [
            ("Mouse", MouseEventAltScreen::False, false),
            ("Mouse: alt_screen", MouseEventAltScreen::True, false),
            ("Mouse: mouse_reporting", MouseEventAltScreen::False, true),
            (
                "Mouse: mouse_reporting + alt_screen",
                MouseEventAltScreen::True,
                true,
            ),
        ] {
            let ordered = self
                .mouse
                .iter()
                .filter(|((_, m), _)| {
                    m.alt_screen == alt_screen && m.mouse_reporting == mouse_reporting
                })
                .collect::<BTreeMap<_, _>>();

            if ordered.is_empty() {
                continue;
            }

            section_header(label);

            let mut trigger_width = 0;
            let mut mod_width = 0;
            for (trigger, mods) in ordered.keys() {
                mod_width = mod_width.max(format!("{:?}", mods.mods).len());
                trigger_width = trigger_width.max(format!("{trigger:?}").len());
            }

            for ((trigger, mods), action) in ordered {
                let mods = if mods.mods == Modifiers::NONE {
                    String::new()
                } else {
                    format!("{:?}", mods.mods)
                };
                let trigger = format!("{trigger:?}");
                println!("\t{mods:mod_width$}   {trigger:trigger_width$}   ->   {action:?}");
            }

            println!();
        }
    }
}
