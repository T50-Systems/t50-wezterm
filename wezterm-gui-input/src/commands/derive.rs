use super::*;

/// Given "1" return "1st", "2" -> "2nd" and so on
pub(super) fn english_ordinal(n: isize) -> String {
    let n = n.to_string();
    if n.ends_with('1') && !n.ends_with("11") {
        format!("{n}st")
    } else if n.ends_with('2') && !n.ends_with("12") {
        format!("{n}nd")
    } else if n.ends_with('3') && !n.ends_with("13") {
        format!("{n}rd")
    } else {
        format!("{n}th")
    }
}

pub(super) fn spawn_command_from_action(action: &KeyAssignment) -> Option<&SpawnCommand> {
    match action {
        SplitPane(config::keyassignment::SplitPane { command, .. }) => Some(command),
        SplitHorizontal(command)
        | SplitVertical(command)
        | SpawnCommandInNewWindow(command)
        | SpawnCommandInNewTab(command) => Some(command),
        _ => None,
    }
}

pub(super) fn label_string(action: &KeyAssignment, candidate: String) -> String {
    if let Some(label) = spawn_command_from_action(action).and_then(|cmd| cmd.label_for_palette()) {
        label
    } else {
        candidate
    }
}

/// Describes a key assignment action; returns a bunch
/// of metadata that is useful in the command palette/menubar context.
/// This function will be called for the result of compute_default_actions(),
/// but can also be used to describe user-provided commands
pub fn derive_command_from_key_assignment(action: &KeyAssignment) -> Option<CommandDef> {
    super::derive_part1::derive_part1(action)
        .or_else(|| super::derive_part2::derive_part2(action))
        .or_else(|| super::derive_part3::derive_part3(action))
        .or_else(|| super::derive_part4::derive_part4(action))
}
