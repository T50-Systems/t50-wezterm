use super::derive::label_string;
use super::*;

pub(super) fn derive_part3(action: &KeyAssignment) -> Option<CommandDef> {
    Some(match action {
        ActivateCopyMode => CommandDef {
            brief: "Activate Copy Mode".into(),
            doc: "Enter mouse-less copy mode to select text using only \
            the keyboard"
                .into(),
            keys: vec![(Modifiers::CTRL.union(Modifiers::SHIFT), "x".into())],
            args: &[ArgType::ActivePane],
            menubar: &["Edit"],
            icon: Some("md_content_copy"),
        },
        SplitVertical(SpawnCommand {
            domain: SpawnTabDomain::CurrentPaneDomain,
            ..
        }) => CommandDef {
            brief: label_string(action, "Split Vertically (Top/Bottom)".to_string()).into(),
            doc: "Split the current pane vertically into two panes, by spawning \
            the default program into the bottom half"
                .into(),
            keys: vec![(
                Modifiers::CTRL
                    .union(Modifiers::ALT)
                    .union(Modifiers::SHIFT),
                "'".into(),
            )],
            args: &[ArgType::ActivePane],
            menubar: &["Shell"],
            icon: Some("cod_split_vertical"),
        },
        SplitHorizontal(SpawnCommand {
            domain: SpawnTabDomain::CurrentPaneDomain,
            ..
        }) => CommandDef {
            brief: label_string(action, "Split Horizontally (Left/Right)".to_string()).into(),
            doc: "Split the current pane horizontally into two panes, by spawning \
            the default program into the right hand side"
                .into(),
            keys: vec![(
                Modifiers::CTRL
                    .union(Modifiers::ALT)
                    .union(Modifiers::SHIFT),
                "5".into(),
            )],
            args: &[ArgType::ActivePane],
            menubar: &["Shell"],
            icon: Some("cod_split_horizontal"),
        },
        SplitHorizontal(_) => CommandDef {
            brief: label_string(action, "Split Horizontally (Left/Right)".to_string()).into(),
            doc: "Split the current pane horizontally into two panes, by spawning \
            the default program into the right hand side"
                .into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &[],
            icon: Some("cod_split_horizontal"),
        },
        SplitVertical(_) => CommandDef {
            brief: label_string(action, "Split Vertically (Top/Bottom)".to_string()).into(),
            doc: "Split the current pane veritically into two panes, by spawning \
            the default program into the bottom"
                .into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &[],
            icon: Some("cod_split_vertical"),
        },
        AdjustPaneSize(PaneDirection::Left, amount) => CommandDef {
            brief: format!("Resize Pane {amount} cell(s) to the Left").into(),
            doc: "Adjusts the closest split divider to the left".into(),
            keys: vec![(
                Modifiers::CTRL
                    .union(Modifiers::ALT)
                    .union(Modifiers::SHIFT),
                "LeftArrow".into(),
            )],
            args: &[ArgType::ActivePane],
            menubar: &["Window", "Resize Pane"],
            icon: None,
        },
        AdjustPaneSize(PaneDirection::Right, amount) => CommandDef {
            brief: format!("Resize Pane {amount} cell(s) to the Right").into(),
            doc: "Adjusts the closest split divider to the right".into(),
            keys: vec![(
                Modifiers::CTRL
                    .union(Modifiers::ALT)
                    .union(Modifiers::SHIFT),
                "RightArrow".into(),
            )],
            args: &[ArgType::ActivePane],
            menubar: &["Window", "Resize Pane"],
            icon: None,
        },
        AdjustPaneSize(PaneDirection::Up, amount) => CommandDef {
            brief: format!("Resize Pane {amount} cell(s) Upwards").into(),
            doc: "Adjusts the closest split divider towards the top".into(),
            keys: vec![(
                Modifiers::CTRL
                    .union(Modifiers::ALT)
                    .union(Modifiers::SHIFT),
                "UpArrow".into(),
            )],
            args: &[ArgType::ActivePane],
            menubar: &["Window", "Resize Pane"],
            icon: None,
        },
        AdjustPaneSize(PaneDirection::Down, amount) => CommandDef {
            brief: format!("Resize Pane {amount} cell(s) Downwards").into(),
            doc: "Adjusts the closest split divider towards the bottom".into(),
            keys: vec![(
                Modifiers::CTRL
                    .union(Modifiers::ALT)
                    .union(Modifiers::SHIFT),
                "DownArrow".into(),
            )],
            args: &[ArgType::ActivePane],
            menubar: &["Window", "Resize Pane"],
            icon: None,
        },
        AdjustPaneSize(PaneDirection::Next | PaneDirection::Prev, _) => return None,
        ActivatePaneDirection(PaneDirection::Next | PaneDirection::Prev) => return None,
        ActivatePaneDirection(PaneDirection::Left) => CommandDef {
            brief: "Activate Pane Left".into(),
            doc: "Activates the pane to the left of the current pane".into(),
            keys: vec![(Modifiers::CTRL.union(Modifiers::SHIFT), "LeftArrow".into())],
            args: &[ArgType::ActivePane],
            menubar: &["Window", "Select Pane"],
            icon: Some("fa_long_arrow_left"),
        },
        ActivatePaneDirection(PaneDirection::Right) => CommandDef {
            brief: "Activate Pane Right".into(),
            doc: "Activates the pane to the right of the current pane".into(),
            keys: vec![(Modifiers::CTRL.union(Modifiers::SHIFT), "RightArrow".into())],
            args: &[ArgType::ActivePane],
            menubar: &["Window", "Select Pane"],
            icon: Some("fa_long_arrow_right"),
        },
        ActivatePaneDirection(PaneDirection::Up) => CommandDef {
            brief: "Activate Pane Up".into(),
            doc: "Activates the pane to the top of the current pane".into(),
            keys: vec![(Modifiers::CTRL.union(Modifiers::SHIFT), "UpArrow".into())],
            args: &[ArgType::ActivePane],
            menubar: &["Window", "Select Pane"],
            icon: Some("fa_long_arrow_up"),
        },
        ActivatePaneDirection(PaneDirection::Down) => CommandDef {
            brief: "Activate Pane Down".into(),
            doc: "Activates the pane to the bottom of the current pane".into(),
            keys: vec![(Modifiers::CTRL.union(Modifiers::SHIFT), "DownArrow".into())],
            args: &[ArgType::ActivePane],
            menubar: &["Window", "Select Pane"],
            icon: Some("fa_long_arrow_down"),
        },
        TogglePaneZoomState => CommandDef {
            brief: "Toggle Pane Zoom".into(),
            doc: "Toggles the zoom state for the current pane".into(),
            keys: vec![(Modifiers::CTRL.union(Modifiers::SHIFT), "z".into())],
            args: &[ArgType::ActivePane],
            menubar: &["Window"],
            icon: Some("md_fullscreen"),
        },
        ActivateLastTab => CommandDef {
            brief: "Activate the last active tab".into(),
            doc: "If there was no prior active tab, has no effect.".into(),
            keys: vec![],
            args: &[ArgType::ActiveWindow],
            menubar: &["Window", "Select Tab"],
            icon: None,
        },
        ClearKeyTableStack => CommandDef {
            brief: "Clear the key table stack".into(),
            doc: "Removes all entries from the stack".into(),
            keys: vec![],
            args: &[ArgType::ActiveWindow],
            menubar: &["Edit"],
            icon: None,
        },
        OpenLinkAtMouseCursor => CommandDef {
            brief: "Open link at mouse cursor".into(),
            doc: "If there is no link under the mouse cursor, has no effect.".into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &["Shell"],
            icon: None,
        },
        ShowLauncherArgs(_) | ShowLauncher => CommandDef {
            brief: "Show the launcher".into(),
            doc: "Shows the launcher menu".into(),
            keys: vec![],
            args: &[ArgType::ActiveWindow],
            menubar: &["Shell"],
            icon: None,
        },
        ShowTabNavigator => CommandDef {
            brief: "Navigate tabs".into(),
            doc: "Shows the tab navigator".into(),
            keys: vec![],
            args: &[ArgType::ActiveWindow],
            menubar: &["Window", "Select Tab"],
            icon: Some("cod_list_flat"),
        },
        DetachDomain(SpawnTabDomain::CurrentPaneDomain) => CommandDef {
            brief: "Detach the domain of the active pane".into(),
            doc: "Detaches (disconnects from) the domain of the active pane".into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &["Shell", "Detach"],
            icon: Some("md_pipe_disconnected"),
        },
        DetachDomain(SpawnTabDomain::DefaultDomain) => CommandDef {
            brief: "Detach the default domain".into(),
            doc: "Detaches (disconnects from) the default domain".into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &["Shell", "Detach"],
            icon: Some("md_pipe_disconnected"),
        },
        DetachDomain(SpawnTabDomain::DomainName(name)) => CommandDef {
            brief: format!("Detach the `{name}` domain").into(),
            doc: format!("Detaches (disconnects from) the domain named `{name}`").into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &["Shell", "Detach"],
            icon: Some("md_pipe_disconnected"),
        },
        DetachDomain(SpawnTabDomain::DomainId(id)) => CommandDef {
            brief: format!("Detach the domain with id {id}").into(),
            doc: format!("Detaches (disconnects from) the domain with id {id}").into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &["Shell", "Detach"],
            icon: Some("md_pipe_disconnected"),
        },
        OpenUri(uri) => match uri.as_ref() {
            "https://wezterm.org/" => CommandDef {
                brief: "Documentation".into(),
                doc: "Visit the wezterm documentation website".into(),
                keys: vec![],
                args: &[],
                menubar: &["Help"],
                icon: Some("md_help"),
            },
            "https://github.com/wezterm/wezterm/discussions/" => CommandDef {
                brief: "Discuss on GitHub".into(),
                doc: "Visit wezterm's GitHub discussion".into(),
                keys: vec![],
                args: &[],
                menubar: &["Help"],
                icon: Some("oct_comment_discussion"),
            },
            "https://github.com/wezterm/wezterm/issues/" => CommandDef {
                brief: "Search or report issue on GitHub".into(),
                doc: "Visit wezterm's GitHub issues".into(),
                keys: vec![],
                args: &[],
                menubar: &["Help"],
                icon: Some("fa_ticket"),
            },
            _ => CommandDef {
                brief: format!("Open {uri} in your browser").into(),
                doc: format!("Open {uri} in your browser").into(),
                keys: vec![],
                args: &[],
                menubar: &[],
                icon: Some("oct_browser"),
            },
        },
        SendString(text) => CommandDef {
            brief: format!(
                "Sends `{text}` to the active pane, \
                           as though you typed it"
            )
            .into(),
            doc: format!(
                "Sends `{text}` to the active pane, as \
                         though you typed it"
            )
            .into(),
            keys: vec![],
            args: &[],
            menubar: &[],
            icon: Some("md_keyboard_variant"),
        },
        SendKey(key) => CommandDef {
            brief: format!(
                "Sends {key:?} to the active pane, \
                           as though you typed it"
            )
            .into(),
            doc: format!(
                "Sends {key:?} to the active pane, \
                         as though you typed it"
            )
            .into(),
            keys: vec![],
            args: &[],
            menubar: &[],
            icon: Some("md_keyboard_variant"),
        },
        Nop => CommandDef {
            brief: "Does nothing".into(),
            doc: "Has no effect".into(),
            keys: vec![],
            args: &[],
            menubar: &[],
            icon: None,
        },
        DisableDefaultAssignment => return None,
        _ => return None,
    })
}
