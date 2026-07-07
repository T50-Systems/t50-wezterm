pub struct SearchOverlayPatternWriter {
    render: Arc<Mutex<CopyRenderable>>,
}

impl std::io::Write for SearchOverlayPatternWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut render = self.render.lock();
        let s = std::str::from_utf8(buf).map_err(|err| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("invalid UTF-8: {err:#}"))
        })?;
        render.search_line.insert_text(s);
        render.schedule_update_search();
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
