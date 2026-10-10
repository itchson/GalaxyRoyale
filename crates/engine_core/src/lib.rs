//! Portable pixel geometry, transforms and batched render extraction.
//! No browser or graphics-device state lives in this crate.

mod frame;
mod pixels;
mod topology;
mod transform;

pub use frame::{
    FrameBuilder, GpuTransform, PixelInstance, PixelMaterial, RenderBatch, RenderFrame,
};
pub use glam::Vec2;
pub use pixels::PixelBody;
pub use topology::{CircularTopology, CircularWrap, CircularWrapError, MAX_CIRCULAR_CROSSINGS};
pub use transform::{AffineTransform, Transform};
