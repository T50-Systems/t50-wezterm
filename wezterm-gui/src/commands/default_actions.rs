use super::*;

/// Returns a list of key assignment actions that should be
/// included in the default key assignments and command palette.
pub(super) fn compute_default_actions() -> Vec<KeyAssignment> {
    // These are ordered by their position within the various menus
    return vec![
        // ----------------- WezTerm
        ReloadConfiguration,
        #[cfg(target_os = "macos")]
        HideApplication,
        #[cfg(target_os = "macos")]
        QuitApplication,
        // ----------------- Shell
        SpawnTab(SpawnTabDomain::CurrentPaneDomain),
        SpawnWindow,
        SplitVertical(SpawnCommand {
            domain: SpawnTabDomain::CurrentPaneDomain,
            ..Default::default()
        }),
        SplitHorizontal(SpawnCommand {
            domain: SpawnTabDomain::CurrentPaneDomain,
            ..Default::default()
        }),
        CloseCurrentTab { confirm: true },
        CloseCurrentPane { confirm: true },
        DetachDomain(SpawnTabDomain::CurrentPaneDomain),
        ResetTerminal,
        // ----------------- Edit
        #[cfg(not(target_os = "macos"))]
        PasteFrom(ClipboardPasteSource::PrimarySelection),
        #[cfg(not(target_os = "macos"))]
        CopyTo(ClipboardCopyDestination::PrimarySelection),
        CopyTo(ClipboardCopyDestination::Clipboard),
        PasteFrom(ClipboardPasteSource::Clipboard),
        ClearScrollback(ScrollbackEraseMode::ScrollbackOnly),
        ClearScrollback(ScrollbackEraseMode::ScrollbackAndViewport),
        QuickSelect,
        CharSelect(CharSelectArguments::default()),
        ActivateCopyMode,
        ClearKeyTableStack,
        ActivateCommandPalette,
        // ----------------- View
        DecreaseFontSize,
        IncreaseFontSize,
        ResetFontSize,
        ResetFontAndWindowSize,
        ScrollByPage(NotNan::new(-1.0).unwrap()),
        ScrollByPage(NotNan::new(1.0).unwrap()),
        ScrollToTop,
        ScrollToBottom,
        // ----------------- Window
        ToggleFullScreen,
        ToggleAlwaysOnTop,
        ToggleAlwaysOnBottom,
        SetWindowLevel(WindowLevel::AlwaysOnBottom),
        SetWindowLevel(WindowLevel::Normal),
        SetWindowLevel(WindowLevel::AlwaysOnTop),
        Hide,
        Search(Pattern::CurrentSelectionOrEmptyString),
        PaneSelect(PaneSelectArguments {
            alphabet: String::new(),
            mode: PaneSelectMode::Activate,
            show_pane_ids: false,
        }),
        PaneSelect(PaneSelectArguments {
            alphabet: String::new(),
            mode: PaneSelectMode::SwapWithActive,
            show_pane_ids: false,
        }),
        PaneSelect(PaneSelectArguments {
            alphabet: String::new(),
            mode: PaneSelectMode::SwapWithActiveKeepFocus,
            show_pane_ids: false,
        }),
        PaneSelect(PaneSelectArguments {
            alphabet: String::new(),
            mode: PaneSelectMode::MoveToNewTab,
            show_pane_ids: false,
        }),
        PaneSelect(PaneSelectArguments {
            alphabet: String::new(),
            mode: PaneSelectMode::MoveToNewWindow,
            show_pane_ids: false,
        }),
        RotatePanes(RotationDirection::Clockwise),
        RotatePanes(RotationDirection::CounterClockwise),
        ActivateTab(0),
        ActivateTab(1),
        ActivateTab(2),
        ActivateTab(3),
        ActivateTab(4),
        ActivateTab(5),
        ActivateTab(6),
        ActivateTab(7),
        ActivateTab(-1),
        ActivateTabRelative(-1),
        ActivateTabRelative(1),
        ActivateWindow(0),
        ActivateWindow(1),
        ActivateWindow(2),
        ActivateWindow(3),
        ActivateWindow(4),
        ActivateWindow(5),
        ActivateWindow(6),
        ActivateWindow(7),
        ActivateWindow(8),
        ActivateWindow(9),
        ActivateWindowRelative(-1),
        ActivateWindowRelative(1),
        MoveTabRelative(-1),
        MoveTabRelative(1),
        AdjustPaneSize(PaneDirection::Left, 1),
        AdjustPaneSize(PaneDirection::Right, 1),
        AdjustPaneSize(PaneDirection::Up, 1),
        AdjustPaneSize(PaneDirection::Down, 1),
        ActivatePaneDirection(PaneDirection::Left),
        ActivatePaneDirection(PaneDirection::Right),
        ActivatePaneDirection(PaneDirection::Up),
        ActivatePaneDirection(PaneDirection::Down),
        TogglePaneZoomState,
        ActivateLastTab,
        ShowLauncher,
        ShowTabNavigator,
        // ----------------- Help
        OpenUri("https://wezterm.org/".to_string()),
        OpenUri("https://github.com/wezterm/wezterm/discussions/".to_string()),
        OpenUri("https://github.com/wezterm/wezterm/issues/".to_string()),
        ShowDebugOverlay,
        // ----------------- Misc
        OpenLinkAtMouseCursor,
    ];
}
