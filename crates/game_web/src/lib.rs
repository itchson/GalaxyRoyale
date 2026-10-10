//! Portable pixel HUD with a WASM-only browser runtime adapter.
#[cfg(target_arch = "wasm32")]
mod browser;
pub mod lighting;
pub mod pixel_ui;
#[cfg(target_arch = "wasm32")]
mod renderer;
pub mod ui_scene;
#[cfg(target_arch = "wasm32")]
pub use browser::*;
