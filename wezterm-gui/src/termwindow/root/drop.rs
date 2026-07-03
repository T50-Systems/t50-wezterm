impl Drop for TermWindow {
    fn drop(&mut self) {
        self.clear_all_overlays();
        if let Some(window) = self.window.take() {
            if let Some(fe) = try_front_end() {
                fe.forget_known_window(&window);
            }
        }
    }
}
