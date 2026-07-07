#[cfg(test)]
mod test {
    use super::*;
    use k9::snapshot;
    use parking_lot::{MappedMutexGuard, Mutex};
    use std::borrow::Cow;
    use termwiz::surface::SEQ_ZERO;

    include!("test/support.rs");
    include!("test/logical_lines.rs");
    include!("test/double_click.rs");
}
