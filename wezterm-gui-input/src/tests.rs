use super::*;
use config::ConfigHandle;
use config::keyassignment::{CopyModeAssignment, KeyAssignment, SpawnCommand, SpawnTabDomain};
use config::{DeferredKeyCode, Key, KeyNoAction};
use std::convert::TryFrom;
use wezterm_dynamic::ToDynamic;
use wezterm_input_types::{KeyCode, Modifiers};

#[test]
fn derives_activate_command_palette_metadata() {
    let cmd = derive_command_from_key_assignment(&KeyAssignment::ActivateCommandPalette)
        .expect("command palette metadata");

    assert_eq!(cmd.brief.as_ref(), "Activate Command Palette");
    assert_eq!(cmd.doc.as_ref(), "Shows the command palette modal");
    assert_eq!(
        cmd.keys,
        vec![(Modifiers::CTRL | Modifiers::SHIFT, "p".to_string())]
    );
    assert_eq!(cmd.menubar, &["Edit"]);
    assert_eq!(cmd.icon, None);
}

#[test]
fn default_command_order_starts_with_expected_actions() {
    let config = ConfigHandle::default_config();
    let actions: Vec<KeyAssignment> = CommandDef::expanded_commands(&config)
        .into_iter()
        .map(|cmd| cmd.action)
        .collect();

    #[cfg(target_os = "macos")]
    let expected = vec![
        KeyAssignment::ReloadConfiguration,
        KeyAssignment::HideApplication,
        KeyAssignment::QuitApplication,
        KeyAssignment::SpawnTab(SpawnTabDomain::CurrentPaneDomain),
        KeyAssignment::SpawnWindow,
    ];

    #[cfg(not(target_os = "macos"))]
    let expected = vec![
        KeyAssignment::ReloadConfiguration,
        KeyAssignment::SpawnTab(SpawnTabDomain::CurrentPaneDomain),
        KeyAssignment::SpawnWindow,
        KeyAssignment::SplitVertical(SpawnCommand {
            domain: SpawnTabDomain::CurrentPaneDomain,
            ..Default::default()
        }),
        KeyAssignment::SplitHorizontal(SpawnCommand {
            domain: SpawnTabDomain::CurrentPaneDomain,
            ..Default::default()
        }),
    ];

    assert_eq!(&actions[..expected.len()], expected.as_slice());
}

#[test]
fn default_input_map_injects_overlay_tables_and_default_entries() {
    let config = ConfigHandle::default_config();
    let overlays = default_overlay_key_tables();
    let input_map = InputMap::new(&config, &overlays);

    assert!(input_map.has_table("copy_mode"));
    assert!(input_map.has_table("search_mode"));

    assert_eq!(
        input_map
            .lookup_key(&KeyCode::Char('q'), Modifiers::NONE, Some("copy_mode"))
            .expect("copy_mode q binding")
            .action,
        KeyAssignment::Multiple(vec![
            KeyAssignment::ScrollToBottom,
            KeyAssignment::CopyMode(CopyModeAssignment::Close),
        ])
    );

    assert_eq!(
        input_map
            .lookup_key(&KeyCode::Char('p'), Modifiers::CTRL, Some("search_mode"))
            .expect("search_mode ctrl-p binding")
            .action,
        KeyAssignment::CopyMode(CopyModeAssignment::PriorMatch)
    );

    assert_eq!(
        input_map
            .lookup_key(
                &KeyCode::Char('p'),
                Modifiers::CTRL | Modifiers::SHIFT,
                None,
            )
            .expect("command palette binding")
            .action,
        KeyAssignment::ActivateCommandPalette
    );
}

#[test]
fn default_input_map_respects_user_disable_precedence_for_normalized_shift_bindings() {
    #[derive(ToDynamic)]
    struct Overrides {
        keys: Vec<Key>,
    }

    let overrides = Overrides {
        keys: vec![Key {
            key: KeyNoAction {
                key: DeferredKeyCode::try_from("p").expect("deferred key"),
                mods: Modifiers::CTRL | Modifiers::SHIFT,
            },
            action: KeyAssignment::DisableDefaultAssignment,
        }],
    }
    .to_dynamic();

    let config = config::overridden_config(&overrides).expect("config overrides");
    let overlays = default_overlay_key_tables();
    let input_map = InputMap::new(&config, &overlays);

    assert_eq!(
        input_map.lookup_key(
            &KeyCode::Char('p'),
            Modifiers::CTRL | Modifiers::SHIFT,
            None,
        ),
        None
    );
    assert!(
        input_map
            .locate_app_wide_key_assignment(&KeyAssignment::ActivateCommandPalette)
            .is_empty()
    );
}
