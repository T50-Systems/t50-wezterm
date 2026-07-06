enum Renderer {
    Terminfo(TerminfoRenderer),
    Windows(WindowsConsoleRenderer),
}

pub trait ConsoleInputHandle {
    fn set_input_mode(&mut self, mode: u32) -> Result<()>;
    fn get_input_mode(&mut self) -> Result<u32>;
    fn set_input_cp(&mut self, cp: u32) -> Result<()>;
    fn get_input_cp(&mut self) -> u32;
    fn get_number_of_input_events(&mut self) -> Result<usize>;
    fn read_console_input(&mut self, num_events: usize) -> Result<Vec<INPUT_RECORD>>;
}

pub trait ConsoleOutputHandle {
    fn set_output_mode(&mut self, mode: u32) -> Result<()>;
    fn get_output_mode(&mut self) -> Result<u32>;
    fn set_output_cp(&mut self, cp: u32) -> Result<()>;
    fn get_output_cp(&mut self) -> u32;
    fn fill_char(&mut self, text: char, x: i16, y: i16, len: u32) -> Result<u32>;
    fn fill_attr(&mut self, attr: u16, x: i16, y: i16, len: u32) -> Result<u32>;
    fn set_attr(&mut self, attr: u16) -> Result<()>;
    fn set_cursor_position(&mut self, x: i16, y: i16) -> Result<()>;
    fn get_buffer_info(&mut self) -> Result<CONSOLE_SCREEN_BUFFER_INFO>;
    fn get_buffer_contents(&mut self) -> Result<Vec<CHAR_INFO>>;
    fn set_buffer_contents(&mut self, buffer: &[CHAR_INFO]) -> Result<()>;
    fn set_viewport(&mut self, left: i16, top: i16, right: i16, bottom: i16) -> Result<()>;
    fn scroll_region(
        &mut self,
        left: i16,
        top: i16,
        right: i16,
        bottom: i16,
        dx: i16,
        dy: i16,
        attr: u16,
    ) -> Result<()>;
}
