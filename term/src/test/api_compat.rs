use std::any::TypeId;
use std::sync::Arc;

fn takes_old_palette(_: crate::color::ColorPalette) {}
fn takes_new_palette(_: wezterm_term_api::color::ColorPalette) {}
fn takes_old_mouse(_: crate::input::MouseEvent) {}
fn takes_new_mouse(_: wezterm_term_api::input::MouseEvent) {}
fn takes_old_config(_: Arc<dyn crate::config::TerminalConfiguration>) {}
fn takes_new_config(_: Arc<dyn wezterm_term_api::config::TerminalConfiguration>) {}
fn takes_old_size(_: crate::terminal::TerminalSize) {}
fn takes_new_size(_: wezterm_term_api::terminal::TerminalSize) {}

#[test]
fn legacy_paths_alias_api_types() {
    let _: fn(wezterm_term_api::color::ColorPalette) = takes_old_palette;
    let _: fn(crate::color::ColorPalette) = takes_new_palette;

    let _: fn(wezterm_term_api::input::MouseEvent) = takes_old_mouse;
    let _: fn(crate::input::MouseEvent) = takes_new_mouse;

    let _: fn(Arc<dyn wezterm_term_api::config::TerminalConfiguration>) = takes_old_config;
    let _: fn(Arc<dyn crate::config::TerminalConfiguration>) = takes_new_config;

    let _: fn(wezterm_term_api::terminal::TerminalSize) = takes_old_size;
    let _: fn(crate::terminal::TerminalSize) = takes_new_size;

    assert_eq!(
        TypeId::of::<crate::CursorPosition>(),
        TypeId::of::<wezterm_term_api::CursorPosition>()
    );
    assert_eq!(
        TypeId::of::<crate::SemanticZone>(),
        TypeId::of::<wezterm_term_api::SemanticZone>()
    );
    assert_eq!(
        TypeId::of::<crate::terminal::Alert>(),
        TypeId::of::<wezterm_term_api::terminal::Alert>()
    );
    assert_eq!(
        TypeId::of::<crate::terminal::Progress>(),
        TypeId::of::<wezterm_term_api::terminal::Progress>()
    );
}
