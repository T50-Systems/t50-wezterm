/// Manages the widgets on the display
#[derive(Default)]
pub struct Ui<'widget> {
    graph: Graph,
    render: FnvHashMap<WidgetId, RenderData<'widget>>,
    input_queue: VecDeque<WidgetEvent>,
    focused: Option<WidgetId>,
}

impl<'widget> Ui<'widget> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add<W: Widget + 'widget>(&mut self, parent: Option<WidgetId>, w: W) -> WidgetId {
        let id = self.graph.add(parent);

        self.render.insert(
            id,
            RenderData {
                surface: Surface::new(1, 1),
                cursor: Default::default(),
                coordinates: Default::default(),
                widget: Box::new(w),
            },
        );

        if parent.is_none() && self.focused.is_none() {
            self.focused = Some(id);
        }

        id
    }

    pub fn set_root<W: Widget + 'widget>(&mut self, w: W) -> WidgetId {
        self.add(None, w)
    }

    pub fn add_child<W: Widget + 'widget>(&mut self, parent: WidgetId, w: W) -> WidgetId {
        self.add(Some(parent), w)
    }

    fn do_deliver(&mut self, id: WidgetId, event: &WidgetEvent) -> bool {
        let render_data = self.render.get_mut(&id).unwrap();
        let mut args = UpdateArgs {
            id,
            cursor: &mut render_data.cursor,
        };

        render_data.widget.process_event(event, &mut args)
    }

    fn deliver_event(&mut self, mut id: WidgetId, event: &WidgetEvent) {
        loop {
            let handled = match event {
                WidgetEvent::Input(InputEvent::Resized { .. }) => true,
                WidgetEvent::Input(InputEvent::Mouse(m)) => {
                    let mut m = m.clone();
                    // convert from screen to widget coords
                    let coords = self.to_widget_coords(
                        id,
                        &ScreenRelativeCoords::new(m.x as usize, m.y as usize),
                    );
                    m.x = coords.x as u16;
                    m.y = coords.y as u16;
                    self.do_deliver(id, &WidgetEvent::Input(InputEvent::Mouse(m)))
                }
                WidgetEvent::Input(InputEvent::Paste(_))
                | WidgetEvent::Input(InputEvent::PixelMouse(_))
                | WidgetEvent::Input(InputEvent::Key(_))
                | WidgetEvent::Input(InputEvent::Wake) => self.do_deliver(id, event),
            };

            if handled {
                return;
            }

            id = match self.graph.parent.get(&id) {
                Some(parent) => *parent,
                None => return,
            };
        }
    }

    /// find the best matching widget that is under the mouse cursor.
    /// We're looking for the latest, deepest widget that contains the input
    /// coordinates.
    fn hovered_widget(&self, coords: &ScreenRelativeCoords) -> Option<WidgetId> {
        let root = match self.graph.root {
            Some(id) => id,
            _ => return None,
        };

        let depth = 0;
        let mut best = (depth, root);
        self.hovered_recursive(root, depth, coords.x, coords.y, &mut best);

        Some(best.1)
    }

    /// Recursive helper for hovered_widget().  The `best` tuple holds the
    /// best (depth, widget) pair.  Depth is incremented each time the function
    /// recurses.
    fn hovered_recursive(
        &self,
        widget: WidgetId,
        depth: usize,
        x: usize,
        y: usize,
        best: &mut (usize, WidgetId),
    ) {
        let render = &self.render[&widget];

        // only consider the dimensions if this node is at the same or a deeper
        // depth.  If so, then we check to see if the coords are within the bounds.
        if depth >= best.0 && x >= render.coordinates.x && y >= render.coordinates.y {
            let (width, height) = render.surface.dimensions();

            if (x - render.coordinates.x < width) && (y - render.coordinates.y < height) {
                *best = (depth, widget);
            }
        }

        for child in self.graph.children(widget) {
            self.hovered_recursive(
                *child,
                depth + 1,
                x + render.coordinates.x,
                y + render.coordinates.y,
                best,
            );
        }
    }

    pub fn process_event_queue(&mut self) -> Result<()> {
        while let Some(event) = self.input_queue.pop_front() {
            match event {
                WidgetEvent::Input(InputEvent::Resized { rows, cols }) => {
                    self.compute_layout(cols, rows)?;
                }
                WidgetEvent::Input(InputEvent::Mouse(ref m)) => {
                    if let Some(hover) =
                        self.hovered_widget(&ScreenRelativeCoords::new(m.x as usize, m.y as usize))
                    {
                        self.deliver_event(hover, &event);
                    }
                }
                WidgetEvent::Input(InputEvent::Key(_))
                | WidgetEvent::Input(InputEvent::Paste(_))
                | WidgetEvent::Input(InputEvent::PixelMouse(_))
                | WidgetEvent::Input(InputEvent::Wake) => {
                    if let Some(focus) = self.focused {
                        self.deliver_event(focus, &event);
                    }
                }
            }
        }
        Ok(())
    }

    /// Queue up an event.  Events are processed by the appropriate
    /// `Widget::update_state` method.  Events may be re-processed to
    /// simplify handling for widgets. eg: a TODO: is to synthesize double
    /// and triple click events.
    pub fn queue_event(&mut self, event: WidgetEvent) {
        self.input_queue.push_back(event);
    }

    /// Assign keyboard focus to the specified widget.
    pub fn set_focus(&mut self, id: WidgetId) {
        self.focused = Some(id);
    }
}
