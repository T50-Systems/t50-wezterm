#[cfg(test)]
mod test {
    use super::*;

    struct CursorHider {}

    impl Widget for CursorHider {
        fn render(&mut self, args: &mut RenderArgs) {
            args.cursor.visibility = CursorVisibility::Hidden;
        }
    }

    #[test]
    fn hide_cursor() {
        let mut ui = Ui::new();

        ui.set_root(CursorHider {});

        let mut surface = Surface::new(10, 10);
        assert_eq!(CursorVisibility::Visible, surface.cursor_visibility());
        ui.render_to_screen(&mut surface).unwrap();
        assert_eq!(CursorVisibility::Hidden, surface.cursor_visibility());
    }
}
