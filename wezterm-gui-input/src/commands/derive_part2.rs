use super::derive::english_ordinal;
use super::*;

pub(super) fn derive_part2(action: &KeyAssignment) -> Option<CommandDef> {
    Some(match action {
        ActivateTab(-1) => CommandDef {
            brief: "Activate right-most tab".into(),
            doc: "Activates the tab on the far right".into(),
            keys: vec![(Modifiers::SUPER, "9".into())],
            args: &[ArgType::ActiveWindow],
            menubar: &["Window", "Select Tab"],
            icon: None,
        },
        ActivateTab(n) => {
            let n = *n;
            let ordinal = english_ordinal(n + 1);
            let keys = if n >= 0 && n <= 7 {
                vec![(Modifiers::SUPER, (n + 1).to_string())]
            } else {
                vec![]
            };
            CommandDef {
                brief: format!("Activate {ordinal} Tab").into(),
                doc: format!("Activates the {ordinal} tab").into(),
                keys,
                args: &[ArgType::ActiveWindow],
                menubar: &["Window", "Select Tab"],
                icon: None,
            }
        }
        ActivatePaneByIndex(n) => {
            let n = *n;
            let ordinal = english_ordinal(n as isize);
            CommandDef {
                brief: format!("Activate {ordinal} Pane").into(),
                doc: format!("Activates the {ordinal} Pane").into(),
                keys: vec![],
                args: &[ArgType::ActiveWindow],
                menubar: &[],
                icon: None,
            }
        }
        SetPaneZoomState(true) => CommandDef {
            brief: format!("Zooms the current Pane").into(),
            doc: format!(
                "Places the current pane into the zoomed state, \
                             filling all of the space in the tab"
            )
            .into(),
            keys: vec![],
            args: &[ArgType::ActiveWindow],
            menubar: &[],
            icon: Some("md_fullscreen"),
        },
        SetPaneZoomState(false) => CommandDef {
            brief: format!("Un-Zooms the current Pane").into(),
            doc: format!("Takes the current pane out of the zoomed state").into(),
            keys: vec![],
            args: &[ArgType::ActiveWindow],
            menubar: &[],
            icon: Some("md_fullscreen"),
        },
        EmitEvent(name) => CommandDef {
            brief: format!("Emit event `{name}`").into(),
            doc: format!(
                "Emits the named event, causing any \
                             associated event handler(s) to trigger"
            )
            .into(),
            keys: vec![],
            args: &[ArgType::ActiveWindow],
            menubar: &[],
            icon: None,
        },
        CloseCurrentTab { confirm: true } => CommandDef {
            brief: "Close current Tab".into(),
            doc: "Closes the current tab, terminating all the \
            processes that are running in its panes."
                .into(),
            keys: vec![(Modifiers::SUPER, "w".into())],
            args: &[ArgType::ActiveTab],
            menubar: &["Shell"],
            icon: Some("md_close_box_outline"),
        },
        CloseCurrentTab { confirm: false } => CommandDef {
            brief: "Close current Tab".into(),
            doc: "Closes the current tab, terminating all the \
            processes that are running in its panes."
                .into(),
            keys: vec![],
            args: &[ArgType::ActiveTab],
            menubar: &[],
            icon: Some("md_close_box_outline"),
        },
        CloseCurrentPane { confirm: true } => CommandDef {
            brief: "Close current Pane".into(),
            doc: "Closes the current pane, terminating the \
            processes that are running inside it."
                .into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &["Shell"],
            icon: Some("md_close_box_outline"),
        },
        CloseCurrentPane { confirm: false } => CommandDef {
            brief: "Close current Pane".into(),
            doc: "Closes the current pane, terminating the \
            processes that are running inside it."
                .into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &[],
            icon: Some("md_close_box_outline"),
        },
        ActivateWindow(n) => {
            let n = *n;
            let ordinal = english_ordinal(n as isize + 1);
            CommandDef {
                brief: format!("Activate {ordinal} Window").into(),
                doc: format!("Activates the {ordinal} window").into(),
                keys: vec![],
                args: &[ArgType::ActiveWindow],
                menubar: &["Window", "Select Window"],
                icon: None,
            }
        }
        ActivateWindowRelative(-1) => CommandDef {
            brief: "Activate the preceeding window".into(),
            doc: "Activates the preceeding window. If this is the first \
            window then cycles around and activates last window"
                .into(),
            keys: vec![],
            args: &[ArgType::ActiveWindow],
            menubar: &["Window", "Select Window"],
            icon: None,
        },
        ActivateWindowRelative(1) => CommandDef {
            brief: "Activate the next window".into(),
            doc: "Activates the next window. If this is the last \
            window then cycles around and activates first window"
                .into(),
            keys: vec![],
            args: &[ArgType::ActiveWindow],
            menubar: &["Window", "Select Window"],
            icon: None,
        },
        ActivateWindowRelative(n) => {
            let (direction, amount) = if *n < 0 {
                ("backwards", -n)
            } else {
                ("forwards", *n)
            };
            let ordinal = english_ordinal(amount + 1);
            CommandDef {
                brief: format!("Activate the {ordinal} window {direction}").into(),
                doc: format!(
                    "Activates the {ordinal} window, moving {direction}. \
                         Wraps around to the other end"
                )
                .into(),
                keys: vec![],
                args: &[ArgType::ActiveWindow],
                menubar: &[],
                icon: None,
            }
        }
        ActivateWindowRelativeNoWrap(-1) => CommandDef {
            brief: "Activate the preceeding window".into(),
            doc: "Activates the preceeding window, stopping at the first \
            window"
                .into(),
            keys: vec![],
            args: &[ArgType::ActiveWindow],
            menubar: &["Window", "Select Window"],
            icon: None,
        },
        ActivateWindowRelativeNoWrap(1) => CommandDef {
            brief: "Activate the next window".into(),
            doc: "Activates the next window, stopping at the last \
            window"
                .into(),
            keys: vec![],
            args: &[ArgType::ActiveWindow],
            menubar: &["Window", "Select Window"],
            icon: None,
        },
        ActivateWindowRelativeNoWrap(n) => {
            let (direction, amount) = if *n < 0 {
                ("backwards", -n)
            } else {
                ("forwards", *n)
            };
            let ordinal = english_ordinal(amount + 1);
            CommandDef {
                brief: format!("Activate the {ordinal} window {direction}").into(),
                doc: format!("Activates the {ordinal} window, moving {direction}.").into(),
                keys: vec![],
                args: &[ArgType::ActiveWindow],
                menubar: &[],
                icon: None,
            }
        }
        ActivateTabRelative(-1) => CommandDef {
            brief: "Activate the tab to the left".into(),
            doc: "Activates the tab to the left. If this is the left-most \
            tab then cycles around and activates the right-most tab"
                .into(),
            keys: vec![
                (Modifiers::SUPER.union(Modifiers::SHIFT), "[".into()),
                (Modifiers::CTRL.union(Modifiers::SHIFT), "Tab".into()),
                (Modifiers::CTRL, "PageUp".into()),
            ],
            args: &[ArgType::ActiveWindow],
            menubar: &["Window", "Select Tab"],
            icon: None,
        },
        ActivateTabRelative(1) => CommandDef {
            brief: "Activate the tab to the right".into(),
            doc: "Activates the tab to the right. If this is the right-most \
            tab then cycles around and activates the left-most tab"
                .into(),
            keys: vec![
                (Modifiers::SUPER.union(Modifiers::SHIFT), "]".into()),
                (Modifiers::CTRL, "Tab".into()),
                (Modifiers::CTRL, "PageDown".into()),
            ],
            args: &[ArgType::ActiveWindow],
            menubar: &["Window", "Select Tab"],
            icon: None,
        },
        ActivateTabRelative(n) => {
            let (direction, amount) = if *n < 0 { ("left", -n) } else { ("right", *n) };
            let ordinal = english_ordinal(amount + 1);
            CommandDef {
                brief: format!("Activate the {ordinal} tab to the {direction}").into(),
                doc: format!(
                    "Activates the {ordinal} tab to the {direction}. \
                         Wraps around to the other end"
                )
                .into(),
                keys: vec![],
                args: &[ArgType::ActiveWindow],
                menubar: &[],
                icon: None,
            }
        }
        ActivateTabRelativeNoWrap(-1) => CommandDef {
            brief: "Activate the tab to the left (no wrapping)".into(),
            doc: "Activates the tab to the left. Stopping at the left-most tab".into(),
            keys: vec![],
            args: &[ArgType::ActiveWindow],
            menubar: &[],
            icon: None,
        },
        ActivateTabRelativeNoWrap(1) => CommandDef {
            brief: "Activate the tab to the right (no wrapping)".into(),
            doc: "Activates the tab to the right. Stopping at the right-most tab".into(),
            keys: vec![],
            args: &[ArgType::ActiveWindow],
            menubar: &[],
            icon: None,
        },
        ActivateTabRelativeNoWrap(n) => {
            let (direction, amount) = if *n < 0 { ("left", -n) } else { ("right", *n) };
            let ordinal = english_ordinal(amount + 1);
            CommandDef {
                brief: format!("Activate the {ordinal} tab to the {direction}").into(),
                doc: format!("Activates the {ordinal} tab to the {direction}").into(),
                keys: vec![],
                args: &[ArgType::ActiveWindow],
                menubar: &[],
                icon: None,
            }
        }
        ReloadConfiguration => CommandDef {
            brief: "Reload configuration".into(),
            doc: "Reloads the configuration file".into(),
            keys: vec![(Modifiers::SUPER, "r".into())],
            args: &[],
            menubar: &["WezTerm"],
            icon: Some("md_reload"),
        },
        QuitApplication => CommandDef {
            brief: "Quit WezTerm".into(),
            doc: "Quits WezTerm".into(),
            keys: vec![(Modifiers::SUPER, "q".into())],
            args: &[],
            menubar: &["WezTerm"],
            icon: Some("oct_stop"),
        },
        MoveTabRelative(-1) => CommandDef {
            brief: "Move tab one place to the left".into(),
            doc: "Rearranges the tabs so that the current tab moves \
            one place to the left"
                .into(),
            keys: vec![(Modifiers::CTRL.union(Modifiers::SHIFT), "PageUp".into())],
            args: &[ArgType::ActiveTab],
            menubar: &["Window", "Move Tab"],
            icon: Some("fa_long_arrow_left"),
        },
        MoveTabRelative(1) => CommandDef {
            brief: "Move tab one place to the right".into(),
            doc: "Rearranges the tabs so that the current tab moves \
            one place to the right"
                .into(),
            keys: vec![(Modifiers::CTRL.union(Modifiers::SHIFT), "PageDown".into())],
            args: &[ArgType::ActiveTab],
            menubar: &["Window", "Move Tab"],
            icon: Some("fa_long_arrow_right"),
        },
        MoveTabRelative(n) => {
            let (direction, amount, icon) = if *n < 0 {
                ("left", (-n).to_string(), "md_chevron_double_left")
            } else {
                ("right", n.to_string(), "md_chevron_double_right")
            };

            CommandDef {
                brief: format!("Move tab {amount} place(s) to the {direction}").into(),
                doc: format!(
                    "Rearranges the tabs so that the current tab moves \
            {amount} place(s) to the {direction}"
                )
                .into(),
                keys: vec![],
                args: &[ArgType::ActiveTab],
                menubar: &[],
                icon: Some(icon),
            }
        }
        MoveTab(n) => {
            let n = (*n) + 1;
            CommandDef {
                brief: format!("Move tab to index {n}").into(),
                doc: format!(
                    "Rearranges the tabs so that the current tab \
                             moves to position {n}"
                )
                .into(),
                keys: vec![],
                args: &[ArgType::ActiveTab],
                menubar: &[],
                icon: None,
            }
        }
        ScrollByPage(amount) => {
            let amount = amount.into_inner();
            if amount == -1.0 {
                CommandDef {
                    brief: "Scroll Up One Page".into(),
                    doc: "Scrolls the viewport up by 1 page".into(),
                    keys: vec![(Modifiers::SHIFT, "PageUp".into())],
                    args: &[ArgType::ActivePane],
                    menubar: &["View"],
                    icon: None,
                }
            } else if amount == 1.0 {
                CommandDef {
                    brief: "Scroll Down One Page".into(),
                    doc: "Scrolls the viewport down by 1 page".into(),
                    keys: vec![(Modifiers::SHIFT, "PageDown".into())],
                    args: &[ArgType::ActivePane],
                    menubar: &["View"],
                    icon: None,
                }
            } else if amount < 0.0 {
                let amount = -amount;
                CommandDef {
                    brief: format!("Scroll Up {amount} Page(s)").into(),
                    doc: format!("Scrolls the viewport up by {amount} pages").into(),
                    keys: vec![],
                    args: &[ArgType::ActivePane],
                    menubar: &["View"],
                    icon: None,
                }
            } else {
                CommandDef {
                    brief: format!("Scroll Down {amount} Page(s)").into(),
                    doc: format!("Scrolls the viewport down by {amount} pages").into(),
                    keys: vec![],
                    args: &[ArgType::ActivePane],
                    menubar: &["View"],
                    icon: None,
                }
            }
        }
        ScrollByLine(n) => {
            let (direction, amount) = if *n < 0 {
                ("up", (-n).to_string())
            } else {
                ("down", n.to_string())
            };
            CommandDef {
                brief: format!("Scroll {direction} {amount} line(s)").into(),
                doc: format!(
                    "Scrolls the viewport {direction} by \
                             {amount} line(s)"
                )
                .into(),
                keys: vec![],
                args: &[ArgType::ActivePane],
                menubar: &[],
                icon: None,
            }
        }
        ScrollToPrompt(n) => {
            let (direction, amount) = if *n < 0 { ("up", -n) } else { ("down", *n) };
            let ordinal = english_ordinal(amount);
            CommandDef {
                brief: format!("Scroll {direction} {amount} prompt(s)").into(),
                doc: format!(
                    "Scrolls the viewport {direction} to the \
                             {ordinal} semantic prompt zone in that direction"
                )
                .into(),
                keys: vec![],
                args: &[ArgType::ActivePane],
                menubar: &[],
                icon: Some("oct_terminal"),
            }
        }
        ScrollByCurrentEventWheelDelta => CommandDef {
            brief: "Scrolls based on the mouse wheel position \
                in the current mouse event"
                .into(),
            doc: "Scrolls based on the mouse wheel position \
                in the current mouse event"
                .into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &[],
            icon: None,
        },
        ScrollToBottom => CommandDef {
            brief: "Scroll to the bottom".into(),
            doc: "Scrolls to the bottom of the viewport".into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &["View"],
            icon: Some("md_format_align_bottom"),
        },
        ScrollToTop => CommandDef {
            brief: "Scroll to the top".into(),
            doc: "Scrolls to the top of the viewport".into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &["View"],
            icon: Some("md_format_align_top"),
        },
        _ => return None,
    })
}
