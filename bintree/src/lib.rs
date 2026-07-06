//! This crate implements a binary tree with a Zipper based Cursor implementation.
//!
//! For more details on the Zipper concept, check out these resources:
//! * <https://www.st.cs.uni-saarland.de//edu/seminare/2005/advanced-fp/docs/huet-zipper.pdf>
//! * <https://donsbot.wordpress.com/2007/05/17/roll-your-own-window-manager-tracking-focus-with-a-zipper/>
//! * <https://stackoverflow.com/a/36168919/149111>

include!("root/types.rs");
include!("root/tree_impl.rs");
include!("root/cursor_core.rs");
include!("root/cursor_nav.rs");
include!("root/tests.rs");
