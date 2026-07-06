//! The launcher is a menu that presents a list of activities that can
//! be launched, such as spawning a new tab in various domains or attaching
//! ssh/tls domains.
//! The launcher is implemented here as an overlay, but could potentially
//! be rendered as a popup/context menu if the system supports it; at the
//! time of writing our window layer doesn't provide an API for context
//! menus.

include!("launcher/types.rs");
include!("launcher/build_entries.rs");
include!("launcher/render_loop.rs");
include!("launcher/launcher_fn.rs");
