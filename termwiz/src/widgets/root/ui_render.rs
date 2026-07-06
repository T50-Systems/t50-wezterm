impl<'widget> Ui<'widget> {
    /// Helper for applying the surfaces from the widgets to the target
    /// screen in the correct order (from the root to the leaves)
    fn render_recursive(
        &mut self,
        id: WidgetId,
        screen: &mut Surface,
        abs_coords: &ScreenRelativeCoords,
    ) -> Result<()> {
        let coords = {
            let render_data = self.render.get_mut(&id).unwrap();
            let surface = &mut render_data.surface;
            {
                let mut args = RenderArgs {
                    id,
                    cursor: &mut render_data.cursor,
                    surface,
                    is_focused: self.focused.map(|f| f == id).unwrap_or(false),
                };
                render_data.widget.render(&mut args);
            }
            screen.draw_from_screen(
                surface,
                abs_coords.x + render_data.coordinates.x,
                abs_coords.y + render_data.coordinates.y,
            );
            surface.flush_changes_older_than(SequenceNo::max_value());
            render_data.coordinates
        };

        for child in self.graph.children(id).to_vec() {
            self.render_recursive(
                child,
                screen,
                &ScreenRelativeCoords::new(coords.x + abs_coords.x, coords.y + abs_coords.y),
            )?;
        }

        Ok(())
    }

    /// Reconsider the layout constraints and apply them.
    /// Returns true if the layout was changed, false if no changes were made.
    fn compute_layout(&mut self, width: usize, height: usize) -> Result<bool> {
        let mut layout = layout::LayoutState::new();

        let root = self.graph.root.unwrap();
        self.add_widget_to_layout(&mut layout, root)?;
        let mut changed = false;

        // Clippy is dead wrong about this iterator being an identity_conversion
        #[allow(clippy::identity_conversion)]
        for result in layout.compute_constraints(width, height, root)? {
            let render_data = self.render.get_mut(&result.widget).unwrap();
            let coords = ParentRelativeCoords::new(result.rect.x, result.rect.y);
            if coords != render_data.coordinates {
                render_data.coordinates = coords;
                changed = true;
            }

            if (result.rect.width, result.rect.height) != render_data.surface.dimensions() {
                render_data
                    .surface
                    .resize(result.rect.width, result.rect.height);
                changed = true;
            }
        }
        Ok(changed)
    }

    /// Recursive helper for building up the LayoutState
    fn add_widget_to_layout(
        &mut self,
        layout: &mut layout::LayoutState,
        widget: WidgetId,
    ) -> Result<()> {
        let constraints = self.render[&widget].widget.get_size_constraints();
        let children = self.graph.children(widget).to_vec();

        layout.add_widget(widget, &constraints, &children);

        for child in children {
            self.add_widget_to_layout(layout, child)?;
        }
        Ok(())
    }

    /// Apply the current state of the widgets to the screen.
    /// This has the side effect of clearing out any unconsumed input queue.
    /// Returns true if the Ui may need to be updated again; for example,
    /// if the most recent update operation changed layout.
    pub fn render_to_screen(&mut self, screen: &mut Surface) -> Result<bool> {
        if let Some(root) = self.graph.root {
            let (width, height) = screen.dimensions();
            // Render from scratch into a fresh screen buffer
            let mut alt_screen = Surface::new(width, height);
            self.render_recursive(root, &mut alt_screen, &ScreenRelativeCoords::new(0, 0))?;
            // Now compute a delta and apply it to the actual screen
            let diff = screen.diff_screens(&alt_screen);
            screen.add_changes(diff);
        }
        // TODO: garbage collect unreachable WidgetId's from self.state

        if let Some(id) = self.focused {
            let cursor = &self.render[&id].cursor;
            let coords = self.to_screen_coords(id, &cursor.coords);

            screen.add_changes(vec![
                Change::CursorShape(cursor.shape),
                Change::CursorColor(cursor.color),
                Change::CursorVisibility(cursor.visibility),
                Change::CursorPosition {
                    x: Position::Absolute(coords.x),
                    y: Position::Absolute(coords.y),
                },
            ]);
        }

        let (width, height) = screen.dimensions();
        self.compute_layout(width, height)
    }

    fn coord_walk<F: Fn(usize, usize) -> usize>(
        &self,
        widget: WidgetId,
        mut x: usize,
        mut y: usize,
        f: F,
    ) -> (usize, usize) {
        let mut widget = widget;
        loop {
            let render = &self.render[&widget];
            x = f(x, render.coordinates.x);
            y = f(y, render.coordinates.y);

            widget = match self.graph.parent.get(&widget) {
                Some(parent) => *parent,
                None => break,
            };
        }
        (x, y)
    }

    /// Convert coordinates that are relative to widget into coordinates
    /// that are relative to the screen origin (top left).
    pub fn to_screen_coords(
        &self,
        widget: WidgetId,
        coords: &ParentRelativeCoords,
    ) -> ScreenRelativeCoords {
        let (x, y) = self.coord_walk(widget, coords.x, coords.y, |a, b| a + b);
        ScreenRelativeCoords { x, y }
    }

    /// Convert coordinates that are relative to the screen origin (top left)
    /// into coordinates that are relative to the widget.
    pub fn to_widget_coords(
        &self,
        widget: WidgetId,
        coords: &ScreenRelativeCoords,
    ) -> ParentRelativeCoords {
        let (x, y) = self.coord_walk(widget, coords.x, coords.y, |a, b| a - b);
        ParentRelativeCoords { x, y }
    }
}
