use super::*;

impl CommandDef {
    pub fn actions_for_palette_and_menubar(
        config: &ConfigHandle,
        inputmap: &InputMap,
    ) -> Vec<ExpandedCommand> {
        let mut result = Self::expanded_commands(config);

        // Generate some stuff based on the config
        for cmd in &config.launch_menu {
            let label = match cmd.label.as_ref() {
                Some(label) => label.to_string(),
                None => match cmd.args.as_ref() {
                    Some(args) => args.join(" "),
                    None => "(default shell)".to_string(),
                },
            };
            result.push(ExpandedCommand {
                brief: format!("{label} (New Tab)").into(),
                doc: "".into(),
                keys: vec![],
                action: KeyAssignment::SpawnCommandInNewTab(cmd.clone()),
                menubar: &["Shell"],
                icon: Some("md_tab_plus".into()),
            });
        }

        // Generate some stuff based on the mux state
        if let Some(mux) = Mux::try_get() {
            let mut domains = mux.iter_domains();
            domains.sort_by(|a, b| {
                let a_state = a.state();
                let b_state = b.state();
                if a_state != b_state {
                    return if a_state == DomainState::Attached {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    };
                }
                a.domain_id().cmp(&b.domain_id())
            });
            for dom in &domains {
                let name = dom.domain_name();
                // FIXME: use domain_label here, but needs to be async
                let label = name;

                if dom.spawnable() {
                    if dom.state() == DomainState::Attached {
                        result.push(ExpandedCommand {
                            brief: format!("New Tab (Domain {label})").into(),
                            doc: "".into(),
                            keys: vec![],
                            action: KeyAssignment::SpawnCommandInNewTab(SpawnCommand {
                                domain: SpawnTabDomain::DomainName(name.to_string()),
                                ..SpawnCommand::default()
                            }),
                            menubar: &["Shell"],
                            icon: Some("md_tab_plus".into()),
                        });
                    } else {
                        result.push(ExpandedCommand {
                            brief: format!("Attach Domain {label}").into(),
                            doc: "".into(),
                            keys: vec![],
                            action: KeyAssignment::AttachDomain(name.to_string()),
                            menubar: &["Shell", "Attach"],
                            icon: Some("md_pipe".into()),
                        });
                    }
                }
            }
            for dom in &domains {
                let name = dom.domain_name();
                // FIXME: use domain_label here, but needs to be async
                let label = name;

                if dom.state() == DomainState::Attached {
                    if name == "local" {
                        continue;
                    }
                    result.push(ExpandedCommand {
                        brief: format!("Detach Domain {label}").into(),
                        doc: "".into(),
                        keys: vec![],
                        action: KeyAssignment::DetachDomain(SpawnTabDomain::DomainName(
                            name.to_string(),
                        )),
                        menubar: &["Shell", "Detach"],
                        icon: Some("md_pipe_disconnected".into()),
                    });
                }
            }

            let active_workspace = mux.active_workspace();
            for workspace in mux.iter_workspaces() {
                if workspace != active_workspace {
                    result.push(ExpandedCommand {
                        brief: format!("Switch to workspace {workspace}").into(),
                        doc: "".into(),
                        keys: vec![],
                        action: KeyAssignment::SwitchToWorkspace {
                            name: Some(workspace.clone()),
                            spawn: None,
                        },
                        menubar: &["Window", "Workspace"],
                        icon: None,
                    });
                }
            }
            result.push(ExpandedCommand {
                brief: "Create new Workspace".into(),
                doc: "".into(),
                keys: vec![],
                action: KeyAssignment::SwitchToWorkspace {
                    name: None,
                    spawn: None,
                },
                menubar: &["Window", "Workspace"],
                icon: None,
            });
        }

        // And sweep to pick up stuff from their key assignments
        for ((keycode, mods), entry) in inputmap.keys.default.iter() {
            if result
                .iter()
                .position(|cmd| cmd.action == entry.action)
                .is_some()
            {
                continue;
            }
            if let Some(cmd) = derive_command_from_key_assignment(&entry.action) {
                result.push(ExpandedCommand {
                    brief: cmd.brief.into(),
                    doc: cmd.doc.into(),
                    keys: vec![(*mods, keycode.clone())],
                    action: entry.action.clone(),
                    menubar: cmd.menubar,
                    icon: cmd.icon.map(Cow::Borrowed),
                });
            }
        }
        for table in inputmap.keys.by_name.values() {
            for entry in table.values() {
                if result
                    .iter()
                    .position(|cmd| cmd.action == entry.action)
                    .is_some()
                {
                    continue;
                }
                if let Some(cmd) = derive_command_from_key_assignment(&entry.action) {
                    result.push(ExpandedCommand {
                        brief: cmd.brief.into(),
                        doc: cmd.doc.into(),
                        keys: vec![],
                        action: entry.action.clone(),
                        menubar: cmd.menubar,
                        icon: cmd.icon.map(Cow::Borrowed),
                    });
                }
            }
        }

        result
    }

    #[cfg(not(target_os = "macos"))]
    pub fn recreate_menubar(_config: &ConfigHandle, _inputmap: &InputMap) {}

    /// Update the menubar to reflect the current config state.
    /// We cannot simply build a completely new one and replace it at runtime,
    /// because something in cocoa get's unhappy and crashes shortly after.
    /// The strategy we have is to try to find the existing item with the
    /// same action and update it.
    /// We use the macos menu item tag to do a mark-sweep style garbage
    /// collection to figure out which items were not reused/updated
    /// and remove them at the end.
    #[cfg(target_os = "macos")]
    pub fn recreate_menubar(config: &ConfigHandle, inputmap: &InputMap) {
        use window::os::macos::menu::*;

        let mut candidates_for_removal = vec![];
        #[allow(unexpected_cfgs)] // <https://github.com/SSheldon/rust-objc/issues/125>
        let wezterm_perform_key_assignment_sel = sel!(weztermPerformKeyAssignment:);

        /// Mark menu items as candidates for removal
        fn mark_candidates(menu: &Menu, candidates: &mut Vec<MenuItem>, action: SEL) {
            for item in menu.items() {
                if let Some(submenu) = item.get_sub_menu() {
                    mark_candidates(&submenu, candidates, action);
                }
                if item.get_action() == Some(action) {
                    item.set_tag(0);
                    candidates.push(item);
                }
            }
        }

        let main_menu = match Menu::get_main_menu() {
            Some(existing) => {
                mark_candidates(
                    &existing,
                    &mut candidates_for_removal,
                    wezterm_perform_key_assignment_sel,
                );

                existing
            }
            None => {
                let menu = Menu::new_with_title("MainMenu");
                menu.assign_as_main_menu();
                menu
            }
        };

        let mut commands = Self::actions_for_palette_and_menubar(config, inputmap);
        commands.retain(|cmd| !cmd.menubar.is_empty());

        // Prefer to put the menus in this order
        let mut order: Vec<&'static str> = vec!["WezTerm", "Shell", "Edit", "View", "Window"];
        // Add any other menus on the end
        for cmd in &commands {
            if !order.contains(&cmd.menubar[0]) {
                order.push(cmd.menubar[0]);
            }
        }

        for &title in &order {
            for cmd in &commands {
                if cmd.menubar[0] != title {
                    continue;
                }

                let mut submenu = main_menu.get_or_create_sub_menu(&cmd.menubar[0], |menu| {
                    if cmd.menubar[0] == "Window" {
                        menu.assign_as_windows_menu();
                        // macOS will insert stuff at the top and bottom, so we add
                        // a separator to tidy things up a bit
                        menu.add_item(&MenuItem::new_separator());
                    } else if cmd.menubar[0] == "WezTerm" {
                        menu.assign_as_app_menu();

                        let about_item = MenuItem::new_with(
                            &format!("WezTerm {}", config::wezterm_version()),
                            Some(wezterm_perform_key_assignment_sel),
                            "",
                        );
                        about_item.set_tool_tip("Click to copy version number");
                        about_item.set_represented_item(RepresentedItem::KeyAssignment(
                            KeyAssignment::CopyTextTo {
                                text: config::wezterm_version().to_string(),
                                destination: ClipboardCopyDestination::ClipboardAndPrimarySelection,
                            },
                        ));

                        menu.add_item(&about_item);
                        menu.add_item(&MenuItem::new_separator());

                        let services_menu = Menu::new_with_title("Services");
                        services_menu.assign_as_services_menu();
                        let services_item = MenuItem::new_with("Services", None, "");
                        menu.add_item(&services_item);
                        services_item.set_sub_menu(&services_menu);

                        menu.add_item(&MenuItem::new_separator());
                    } else if cmd.menubar[0] == "Help" {
                        menu.assign_as_help_menu();
                    }
                });

                // Fill out any submenu hierarchy
                for sub_title in cmd.menubar.iter().skip(1) {
                    submenu = submenu.get_or_create_sub_menu(sub_title, |_menu| {});
                }

                let mut candidate = inputmap.locate_app_wide_key_assignment(&cmd.action);
                candidate.sort_by(|(a_key, a_mods), (b_key, b_mods)| {
                    fn score_mods(mods: &Modifiers) -> usize {
                        let mut score: usize = mods.bits() as usize;
                        // Prefer keys with CMD on macOS
                        if mods.contains(Modifiers::SUPER) {
                            score += 1000;
                        }
                        score
                    }

                    let a_mods = score_mods(a_mods);
                    let b_mods = score_mods(b_mods);

                    match b_mods.cmp(&a_mods) {
                        Ordering::Equal => {}
                        ordering => return ordering,
                    }

                    a_key.cmp(&b_key)
                });

                fn key_code_to_equivalent(key: &KeyCode) -> String {
                    match key {
                        KeyCode::Hyper
                        | KeyCode::Super
                        | KeyCode::Meta
                        | KeyCode::Cancel
                        | KeyCode::Composed(_)
                        | KeyCode::RawCode(_) => "".to_string(),
                        KeyCode::Char(c) => c.to_string(),
                        KeyCode::Physical(phys) => key_code_to_equivalent(&phys.to_key_code()),
                        _ => "".to_string(),
                    }
                }

                let short_cut = candidate
                    .get(0)
                    .map(|(key, _)| key_code_to_equivalent(key))
                    .unwrap_or_else(String::new);

                let represented_item = RepresentedItem::KeyAssignment(cmd.action.clone());
                let item = match submenu.get_item_with_represented_item(&represented_item) {
                    Some(existing) => {
                        existing.set_title(&cmd.brief);
                        existing.set_key_equivalent(&short_cut);
                        existing
                    }
                    None => {
                        let item = MenuItem::new_with(
                            &cmd.brief,
                            Some(wezterm_perform_key_assignment_sel),
                            &short_cut,
                        );
                        submenu.add_item(&item);
                        item
                    }
                };

                if !short_cut.is_empty() {
                    let mods: Modifiers = candidate[0].1;
                    let mut equiv_mods = NSEventModifierFlags::empty();

                    equiv_mods.set(
                        NSEventModifierFlags::NSShiftKeyMask,
                        mods.contains(Modifiers::SHIFT),
                    );
                    equiv_mods.set(
                        NSEventModifierFlags::NSAlternateKeyMask,
                        mods.contains(Modifiers::ALT),
                    );
                    equiv_mods.set(
                        NSEventModifierFlags::NSControlKeyMask,
                        mods.contains(Modifiers::CTRL),
                    );
                    equiv_mods.set(
                        NSEventModifierFlags::NSCommandKeyMask,
                        mods.contains(Modifiers::SUPER),
                    );

                    item.set_key_equiv_modifier_mask(equiv_mods);
                }

                item.set_represented_item(represented_item);
                item.set_tool_tip(&cmd.doc);
                // Update the tag to indicate that this item should
                // not be removed by the sweep below
                item.set_tag(1);
            }
        }

        // Now sweep away any items that were not updated
        for item in candidates_for_removal {
            if item.get_tag() == 0 {
                item.get_menu().map(|menu| menu.remove_item(&item));
            }
        }
    }
}
