//! Portable pixel geometry, transforms and batched render extraction.
//! No browser or graphics-device state lives in this crate.

mod camera;
mod frame;
mod pixels;
mod pixel_rig;
mod topology;
mod transform;

pub use camera::{CAMERA_ZOOM_SECONDS, CameraView, CameraZoom, MAX_CAMERA_ZOOM, MIN_CAMERA_ZOOM};
pub use frame::{
    FrameBuilder, GpuTransform, PixelInstance, PixelMaterial, RenderBatch, RenderFrame,
};
pub use glam::Vec2;
pub use pixels::PixelBody;
pub use pixel_rig::{PixelRig, PixelRigError, PixelRigPart, PixelRigRole, PixelRigState, MAX_PIXEL_RIG_PARTS};
pub use topology::{CircularTopology, CircularWrap, CircularWrapError, MAX_CIRCULAR_CROSSINGS};
pub use transform::{AffineTransform, Transform};
