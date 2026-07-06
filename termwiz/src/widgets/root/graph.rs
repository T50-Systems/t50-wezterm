struct RenderData<'widget> {
    surface: Surface,
    cursor: CursorShapeAndPosition,
    coordinates: ParentRelativeCoords,
    widget: Box<dyn Widget + 'widget>,
}

#[derive(Default)]
struct Graph {
    root: Option<WidgetId>,
    children: FnvHashMap<WidgetId, Vec<WidgetId>>,
    parent: FnvHashMap<WidgetId, WidgetId>,
}

impl Graph {
    fn add(&mut self, parent: Option<WidgetId>) -> WidgetId {
        let id = WidgetId::new();

        if self.root.is_none() {
            self.root = Some(id);
        }

        self.children.insert(id, Vec::new());

        if let Some(parent) = parent {
            self.parent.insert(id, parent);
            self.children.get_mut(&parent).unwrap().push(id);
        }

        id
    }

    fn children(&self, id: WidgetId) -> &[WidgetId] {
        self.children
            .get(&id)
            .map(|v| v.as_slice())
            .unwrap_or_else(|| &[])
    }
}
