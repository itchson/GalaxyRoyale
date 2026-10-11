//! Portable pixel HUD with a WASM-only browser runtime adapter.
pub mod boss_layout;
pub mod boss_overlay;
#[cfg(target_arch = "wasm32")]
mod browser;
mod champion_patches;
pub mod dock_art;
pub mod flight_icons;
pub mod kikai_art;
pub mod lighting;
pub mod pixel_ui;
#[cfg(target_arch = "wasm32")]
mod renderer;
pub mod ui_scene;
#[cfg(target_arch = "wasm32")]
pub use browser::*;
