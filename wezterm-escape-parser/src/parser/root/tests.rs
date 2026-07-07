#[cfg(all(test, feature = "std"))]
mod test {
    use super::*;
    use crate::color::ColorSpec;
    use crate::csi::*;
    use crate::osc::{DynamicColorNumber, OperatingSystemCommand};
    use crate::parser::Parser;
    use crate::{Action, Esc, EscCode};

    include!("../test/support_and_osc.rs");
    include!("../test/basic_and_controls.rs");
    include!("../test/graphics_and_modes.rs");
    include!("../test/dynamic_colors.rs");
}
