//! Presentation-only zoom and camera projection shared by portable backends.
use crate::{GpuTransform, Vec2};

pub const MIN_CAMERA_ZOOM: f32 = 0.6;
pub const MAX_CAMERA_ZOOM: f32 = 1.6;
pub const CAMERA_ZOOM_SECONDS: f32 = 0.15;

/// Retargetable 150ms smoothstep tween. This never changes simulation positions,
/// headings or input destinations; callers update extraction's world extent.
#[derive(Debug, Clone, Copy)]
pub struct CameraZoom {
    value: f32,
    target: f32,
    start: f32,
    elapsed: f32,
}

impl Default for CameraZoom {
    fn default() -> Self {
        Self {
            value: 1.0,
            target: 1.0,
            start: 1.0,
            elapsed: CAMERA_ZOOM_SECONDS,
        }
    }
}

impl CameraZoom {
    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    /// Nonfinite requests are ignored. Repeating the current target does not
    /// restart its animation; retargeting begins from the current visible zoom.
    pub fn set_target(&mut self, requested: f32) {
        if !requested.is_finite() {
            return;
        }
        let target = requested.clamp(MIN_CAMERA_ZOOM, MAX_CAMERA_ZOOM);
        if target == self.target {
            return;
        }
        self.target = target;
        self.start = self.value;
        self.elapsed = 0.0;
    }

    pub fn advance(&mut self, seconds: f32) {
        if !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        self.elapsed = (self.elapsed + seconds).min(CAMERA_ZOOM_SECONDS);
        let t = self.elapsed / CAMERA_ZOOM_SECONDS;
        let ease = t * t * (3.0 - 2.0 * t);
        self.value = self.start + (self.target - self.start) * ease;
        if self.elapsed == CAMERA_ZOOM_SECONDS {
            self.value = self.target;
        }
    }

    pub fn snap(&mut self) {
        self.value = self.target;
        self.start = self.target;
        self.elapsed = CAMERA_ZOOM_SECONDS;
    }

    pub fn world_extent(&self, logical_extent: Vec2) -> Vec2 {
        logical_extent / self.value
    }
}

/// Projection from a world center to a fixed logical pixel surface. A UI view
/// uses zoom=1 while the world view uses the current interpolated camera zoom.
/// Both retain their original affine geometry and material fields.
#[derive(Debug, Clone, Copy)]
pub struct CameraView {
    center: Vec2,
    logical_extent: Vec2,
    world_extent: Vec2,
}

impl CameraView {
    pub fn new(center: Vec2, logical_extent: Vec2, zoom: f32) -> Option<Self> {
        if !center.is_finite()
            || !logical_extent.is_finite()
            || logical_extent.min_element() <= 0.0
            || !zoom.is_finite()
        {
            return None;
        }
        Some(Self {
            center,
            logical_extent,
            world_extent: logical_extent / zoom.clamp(MIN_CAMERA_ZOOM, MAX_CAMERA_ZOOM),
        })
    }

    pub fn world_extent(&self) -> Vec2 {
        self.world_extent
    }

    pub fn world_to_screen(&self, world: Vec2) -> Vec2 {
        (world - self.center) / self.world_extent * self.logical_extent + self.logical_extent * 0.5
    }

    pub fn screen_to_world(&self, screen: Vec2) -> Vec2 {
        self.center + (screen / self.logical_extent - Vec2::splat(0.5)) * self.world_extent
    }

    /// Subtract before GPU upload to retain precision away from the origin.
    /// Lighting then receives camera-relative world cells, independent of zoom.
    pub fn relative_transform(&self, mut transform: GpuTransform) -> GpuTransform {
        transform.a[0] -= self.center.x;
        transform.a[1] -= self.center.y;
        transform
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FrameBuilder, PixelBody, PixelMaterial, Transform};

    fn near(a: Vec2, b: Vec2) {
        assert!(a.distance(b) < 0.0005, "{a:?} != {b:?}");
    }

    #[test]
    fn zoom_is_bounded_retargetable_and_finishes_in_150ms() {
        let mut zoom = CameraZoom::default();
        zoom.set_target(100.0);
        assert_eq!(zoom.target(), MAX_CAMERA_ZOOM);
        zoom.advance(0.075);
        assert!((zoom.value() - 1.3).abs() < 0.00001);
        zoom.set_target(MAX_CAMERA_ZOOM);
        zoom.advance(0.075);
        assert_eq!(zoom.value(), MAX_CAMERA_ZOOM);
        zoom.set_target(-100.0);
        zoom.advance(0.05);
        let before = zoom.value();
        zoom.set_target(1.0);
        assert_eq!(zoom.value(), before);
        zoom.advance(CAMERA_ZOOM_SECONDS);
        assert_eq!(zoom.value(), 1.0);
        zoom.set_target(f32::NAN);
        zoom.set_target(f32::INFINITY);
        zoom.advance(f32::NAN);
        zoom.advance(-1.0);
        assert_eq!(zoom.value(), 1.0);
        zoom.set_target(MIN_CAMERA_ZOOM);
        zoom.snap();
        assert_eq!(zoom.value(), MIN_CAMERA_ZOOM);
        near(
            zoom.world_extent(Vec2::new(640.0, 360.0)),
            Vec2::new(640.0, 360.0) / 0.6,
        );
    }

    #[test]
    fn world_projection_and_input_round_trip_use_current_extent() {
        let center = Vec2::new(1730.0, -880.0);
        let viewport = Vec2::new(640.0, 360.0);
        for zoom in [MIN_CAMERA_ZOOM, 1.0, MAX_CAMERA_ZOOM] {
            let view = CameraView::new(center, viewport, zoom).unwrap();
            for screen in [Vec2::ZERO, viewport * 0.5, viewport, Vec2::new(59.0, 291.0)] {
                near(view.world_to_screen(view.screen_to_world(screen)), screen);
            }
            near(
                view.world_to_screen(center + Vec2::X * 80.0),
                viewport * 0.5 + Vec2::X * (80.0 * zoom),
            );
        }
        assert!(CameraView::new(center, viewport, f32::NAN).is_none());
        assert!(CameraView::new(center, Vec2::ZERO, 1.0).is_none());
    }

    #[test]
    fn separate_ui_projection_preserves_glyph_positions_sizes_and_materials() {
        let viewport = Vec2::new(640.0, 360.0);
        let glyph = PixelBody::from_ascii(&["##", " #"], &[('#', 0xff_ccbb99)]);
        for zoom in [MIN_CAMERA_ZOOM, MAX_CAMERA_ZOOM] {
            // Actual shaken camera is shared with UI extraction and upload;
            // input continues using the caller's unshaken world view instead.
            for center in [Vec2::ZERO, Vec2::new(1701.25, -899.5)] {
                let world = CameraView::new(center, viewport, zoom).unwrap();
                let ui = CameraView::new(center, viewport, 1.0).unwrap();
                let parent = Transform {
                    position: center - viewport * 0.5 + Vec2::new(24.0, 18.0),
                    ..Transform::default()
                };
                let letter = Transform {
                    position: Vec2::new(13.0, 7.0),
                    scale: Vec2::splat(2.0),
                    ..Transform::default()
                };
                let mut frame = FrameBuilder::new();
                frame.push_body_affine_colored_material(
                    &glyph,
                    parent.compose_affine(letter),
                    PixelMaterial::default(),
                    |_, color| color,
                );
                let batch = &frame.frame().batches[0];
                let original = batch.transforms[0];
                let gpu = ui.relative_transform(original);
                assert_eq!(&gpu.b[2..], &[0.0, 0.0]);
                assert_eq!(&gpu.a[2..], &original.a[2..]);
                for pixel in &batch.pixels {
                    let local = Vec2::from_array(pixel.local);
                    let projected =
                        gpu.point(local) / ui.world_extent() * viewport + viewport * 0.5;
                    near(projected, Vec2::new(37.0, 25.0) + local * 2.0);
                    near(ui.world_to_screen(original.point(local)), projected);
                }
                // World pixels use the same cells but their size follows zoom.
                near(
                    world.world_to_screen(center + Vec2::new(2.0, 0.0)) - viewport * 0.5,
                    Vec2::X * (2.0 * zoom),
                );
            }
        }
    }

    #[test]
    fn zoom_projection_keeps_light_distance_in_world_units() {
        let center = Vec2::new(2100.0, -730.0);
        let transform = GpuTransform::from(Transform {
            position: center + Vec2::new(15.0, 11.0),
            rotation: 0.7,
            scale: Vec2::new(2.0, 0.5),
        })
        .with_material(PixelMaterial {
            light_response: 0.75,
            emission: 0.2,
        });
        for zoom in [MIN_CAMERA_ZOOM, MAX_CAMERA_ZOOM] {
            let view = CameraView::new(center, Vec2::new(640.0, 360.0), zoom).unwrap();
            let relative = view.relative_transform(transform);
            near(
                relative.point(Vec2::new(3.0, -1.0)),
                transform.point(Vec2::new(3.0, -1.0)) - center,
            );
            assert_eq!(&relative.b[2..], &[0.75, 0.2]);
        }
    }
}
