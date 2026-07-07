impl crate::TermWindow {
    fn paint_pane_box_model(&mut self, pos: &PositionedPane) -> anyhow::Result<()> {
        let computed = self.build_pane(pos)?;
        let mut ui_items = computed.ui_items();
        self.ui_items.append(&mut ui_items);
        let gl_state = self.render_state.as_ref().unwrap();
        self.render_element(&computed, gl_state, None)
    }
}
