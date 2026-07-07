
fn subtract_borders(&self, width: i32, height: i32) -> (i32, i32) {
    if !self.showing_title_bar(&*self.inner.borrow()) {
        (width, height)
    } else {
        (width, height - HEADER_SIZE as i32)
    }
}

fn add_borders(&self, width: i32, height: i32) -> (i32, i32) {
    if !self.showing_title_bar(&*self.inner.borrow()) {
        (width, height)
    } else {
        (width, height + HEADER_SIZE as i32)
    }
}

fn location(&self) -> (i32, i32) {
    if !self.showing_title_bar(&*self.inner.borrow()) {
        (0, 0)
    } else {
        (0, -(HEADER_SIZE as i32))
    }
}

fn set_config(&mut self, config: ConceptConfig) {
    self.config = config;
    // Refresh parts to reflect window_decorations
    self.inner.borrow_mut().parts.clear();
    self.set_hidden(self.hidden);
    self.redraw();
}

fn set_title(&mut self, title: String) {
    self.title = Some(title);
}
