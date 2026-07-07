impl crate::TermWindow {
    pub fn build_fancy_tab_bar(&self, palette: &ColorPalette) -> anyhow::Result<ComputedElement> {
        self.build_fancy_bar(palette, self.tab_bar.items(), self.config.tab_bar_at_bottom)
    }

    pub fn build_fancy_secondary_tab_bar(
        &self,
        palette: &ColorPalette,
    ) -> anyhow::Result<ComputedElement> {
        self.build_fancy_bar(
            palette,
            self.secondary_tab_bar.items(),
            !self.config.tab_bar_at_bottom,
        )
    }

    fn paint_fancy_bar(&self, computed: &ComputedElement) -> anyhow::Result<Vec<UIItem>> {
        let ui_items = computed.ui_items();

        let gl_state = self.render_state.as_ref().unwrap();
        self.render_element(computed, gl_state, None)?;

        Ok(ui_items)
    }

    pub fn paint_fancy_tab_bar(&self) -> anyhow::Result<Vec<UIItem>> {
        let computed = self.fancy_tab_bar.as_ref().ok_or_else(|| {
            anyhow::anyhow!("paint_fancy_tab_bar called but fancy_tab_bar is None")
        })?;
        self.paint_fancy_bar(computed)
    }

    pub fn paint_fancy_secondary_tab_bar(&self) -> anyhow::Result<Vec<UIItem>> {
        let computed = self.fancy_secondary_tab_bar.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "paint_fancy_secondary_tab_bar called but fancy_secondary_tab_bar is None"
            )
        })?;
        self.paint_fancy_bar(computed)
    }
}
