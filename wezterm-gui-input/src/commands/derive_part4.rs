use super::derive::{english_ordinal, label_string};
use super::*;

pub(super) fn derive_part4(action: &KeyAssignment) -> Option<CommandDef> {
    Some(match action {
        SelectTextAtMouseCursor(mode) => CommandDef {
            brief: format!(
                "Selects text at the mouse cursor \
                           location using {mode:?}"
            )
            .into(),
            doc: format!(
                "Selects text at the mouse cursor \
                         location using {mode:?}"
            )
            .into(),
            keys: vec![],
            args: &[],
            menubar: &[],
            icon: None,
        },
        ExtendSelectionToMouseCursor(mode) => CommandDef {
            brief: format!(
                "Extends the selection text to the mouse \
                           cursor location using {mode:?}"
            )
            .into(),
            doc: format!(
                "Extends the selection text to the mouse \
                         cursor location using {mode:?}"
            )
            .into(),
            keys: vec![],
            args: &[],
            menubar: &[],
            icon: None,
        },
        ClearSelection => CommandDef {
            brief: "Clears the selection in the current pane".into(),
            doc: "Clears the selection in the current pane".into(),
            keys: vec![],
            args: &[],
            menubar: &[],
            icon: None,
        },
        CompleteSelection(destination) => CommandDef {
            brief: format!("Completes selection, and copy {destination:?}").into(),
            doc: format!(
                "Completes text selection using the mouse, and copies \
                to {destination:?}"
            )
            .into(),
            keys: vec![],
            args: &[],
            menubar: &[],
            icon: None,
        },
        CompleteSelectionOrOpenLinkAtMouseCursor(destination) => CommandDef {
            brief: format!(
                "Open a URL or Completes selection \
            by copying to {destination:?}"
            )
            .into(),
            doc: format!(
                "If the mouse is over a link, open it, otherwise, completes \
                text selection using the mouse, and copies to {destination:?}"
            )
            .into(),
            keys: vec![],
            args: &[],
            menubar: &[],
            icon: None,
        },
        StartWindowDrag => CommandDef {
            brief: "Requests a window drag operation from \
                the window environment"
                .into(),
            doc: "Requests a window drag operation from \
                the window environment"
                .into(),
            keys: vec![],
            args: &[],
            menubar: &[],
            icon: Some("md_drag"),
        },
        Multiple(actions) => {
            let mut brief = String::new();
            for act in actions {
                if !brief.is_empty() {
                    brief.push_str(", ");
                }
                match derive_command_from_key_assignment(act) {
                    Some(cmd) => {
                        brief.push_str(&cmd.brief);
                    }
                    None => {
                        brief.push_str(&format!("{act:?}"));
                    }
                }
            }
            CommandDef {
                brief: brief.into(),
                doc: "Performs multiple nested actions".into(),
                keys: vec![],
                args: &[ArgType::ActivePane],
                menubar: &[],
                icon: None,
            }
        }
        SwitchToWorkspace {
            name: None,
            spawn: None,
        } => CommandDef {
            brief: format!(
                "Spawn the default program into a new \
                           workspace and switch to it"
            )
            .into(),
            doc: format!(
                "Spawn the default program into a new \
                         workspace and switch to it"
            )
            .into(),
            keys: vec![],
            args: &[],
            menubar: &["Window", "Workspace"],
            icon: None,
        },
        SwitchToWorkspace {
            name: Some(name),
            spawn: None,
        } => CommandDef {
            brief: format!(
                "Switch to workspace `{name}`, spawn the \
                           default program if that workspace doesn't already exist"
            )
            .into(),
            doc: format!(
                "Switch to workspace `{name}`, spawn the \
                         default program if that workspace doesn't already exist"
            )
            .into(),
            keys: vec![],
            args: &[],
            menubar: &["Window", "Workspace"],
            icon: None,
        },
        SwitchToWorkspace {
            name: Some(name),
            spawn: Some(prog),
        } => CommandDef {
            brief: format!(
                "Switch to workspace `{name}`, spawn {prog:?} \
                           if that workspace doesn't already exist"
            )
            .into(),
            doc: format!(
                "Switch to workspace `{name}`, spawn {prog:?} \
                         if that workspace doesn't already exist"
            )
            .into(),
            keys: vec![],
            args: &[],
            menubar: &["Window", "Workspace"],
            icon: None,
        },
        SwitchToWorkspace {
            name: None,
            spawn: Some(prog),
        } => CommandDef {
            brief: format!("Spawn the {prog:?} into a new workspace and switch to it").into(),
            doc: format!("Spawn the {prog:?} into a new workspace and switch to it").into(),
            keys: vec![],
            args: &[],
            menubar: &["Window", "Workspace"],
            icon: None,
        },
        SwitchWorkspaceRelative(n) => {
            let (direction, amount) = if *n < 0 {
                ("previous", -n)
            } else {
                ("next", *n)
            };
            let ordinal = english_ordinal(amount);
            CommandDef {
                brief: format!("Switch to {ordinal} {direction} workspace").into(),
                doc: format!(
                    "Switch to the {ordinal} {direction} workspace, \
                             ordered lexicographically by workspace name"
                )
                .into(),
                keys: vec![],
                args: &[ArgType::ActivePane],
                menubar: &["Window", "Workspace"],
                icon: None,
            }
        }
        ActivateKeyTable { name, .. } => CommandDef {
            brief: format!("Activate key table `{name}`").into(),
            doc: format!("Activate key table `{name}`").into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &[],
            icon: None,
        },
        PopKeyTable => CommandDef {
            brief: "Pop the current key table".into(),
            doc: "Pop the current key table".into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &[],
            icon: None,
        },
        AttachDomain(name) => CommandDef {
            brief: format!("Attach domain `{name}`").into(),
            doc: format!("Attach domain `{name}`").into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &["Shell", "Attach"],
            icon: Some("md_pipe"),
        },
        CopyMode(copy_mode) => CommandDef {
            brief: format!("{copy_mode:?}").into(),
            doc: "".into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &["Edit", "Copy Mode"],
            icon: None,
        },
        RotatePanes(direction) => CommandDef {
            brief: format!("Rotate panes {direction:?}").into(),
            doc: format!("Rotate panes {direction:?}").into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &["Window", "Rotate Pane"],
            icon: Some(match direction {
                RotationDirection::Clockwise => "md_rotate_right",
                RotationDirection::CounterClockwise => "md_rotate_left",
            }),
        },
        SplitPane(split) => {
            let direction = split.direction;
            CommandDef {
                brief: label_string(action, format!("Split the current pane {direction:?}")).into(),
                doc: format!("Split the current pane {direction:?}").into(),
                keys: vec![],
                args: &[ArgType::ActivePane],
                menubar: &[],
                icon: match split.direction {
                    PaneDirection::Up | PaneDirection::Down => Some("cod_split_vertical"),
                    PaneDirection::Left | PaneDirection::Right => Some("cod_split_horizontal"),
                    PaneDirection::Next | PaneDirection::Prev => None,
                },
            }
        }
        ResetTerminal => CommandDef {
            brief: "Reset the terminal emulation state in the current pane".into(),
            doc: "Reset the terminal emulation state in the current pane".into(),
            keys: vec![],
            args: &[ArgType::ActivePane],
            menubar: &["Shell"],
            icon: None,
        },
        ActivateCommandPalette => CommandDef {
            brief: "Activate Command Palette".into(),
            doc: "Shows the command palette modal".into(),
            keys: vec![(Modifiers::CTRL.union(Modifiers::SHIFT), "p".into())],
            args: &[ArgType::ActivePane],
            menubar: &["Edit"],
            icon: None,
        },
        _ => return None,
    })
}
