//! Stable mux protocol ids and DTOs shared below the mux implementation.

pub mod client;
pub mod pane;
pub mod renderable;
pub mod tab;
pub mod window;

pub use client::{ClientId, ClientInfo};
pub use pane::{PaneId, Pattern, PatternType, SearchResult};
pub use renderable::{RenderableDimensions, StableCursorPosition};
pub use tab::{
    PaneEntry, PaneNode, SerdeUrl, SplitDirection, SplitDirectionAndSize, SplitRequest, SplitSize,
    TabId,
};
pub use window::WindowId;

#[cfg(test)]
mod tests;
