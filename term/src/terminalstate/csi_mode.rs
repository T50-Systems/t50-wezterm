use super::*;

impl TerminalState {
    pub(super) fn perform_csi_mode(&mut self, mode: Mode) {
        let mode = match self.perform_csi_mode_part1(mode) {
            Some(mode) => mode,
            None => return,
        };
        let mode = match self.perform_csi_mode_part2(mode) {
            Some(mode) => mode,
            None => return,
        };
        self.perform_csi_mode_part3(mode);
    }
}
