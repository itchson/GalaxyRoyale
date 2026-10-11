//! Bounded camera-relative uniform data shared by both graphics backends.
use game_core::LightSource;
pub const MAX_LIGHTS: usize = 8;
pub const RASTER_OFFSET: usize = 8 + MAX_LIGHTS * 8;
pub const UNIFORM_FLOATS: usize = RASTER_OFFSET + 4;

/// Attachment sizes share one resize calculation. Only the UI keeps the old
/// integer pixel scale; world geometry rasterizes at the full backing size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RasterLayout {
    pub world: [u32; 2],
    pub logical: [u32; 2],
    pub ui_scale: u32,
}

impl RasterLayout {
    pub fn new(backing: [u32; 2]) -> Self {
        let world = backing.map(|size| size.max(1));
        let ui_scale = ((world[1] as f32 / 360.0).round() as u32).max(1);
        let logical = world.map(|size| (size / ui_scale).max(1));
        Self {
            world,
            logical,
            ui_scale,
        }
    }

    pub fn presentation(self) -> [f32; 4] {
        [
            self.world[0] as f32,
            self.world[1] as f32,
            (self.logical[0] * self.ui_scale).min(self.world[0]) as f32,
            (self.logical[1] * self.ui_scale).min(self.world[1]) as f32,
        ]
    }
}

/// Subtle world-material grading. Unlit UI and distant art retain their palettes.
pub fn ambient_for_biome(kind: u8) -> [f32; 4] {
    match kind {
        1 => [1.07, 0.96, 1.03, 1.0], // Amethyst
        2 => [0.96, 1.07, 1.02, 1.0], // Aurora
        3 => [1.04, 1.00, 0.96, 1.0], // Iron
        4 => [1.07, 1.03, 0.93, 1.0], // Radial
        5 => [0.95, 1.02, 1.07, 1.0], // Cryo
        _ => [1.0, 1.0, 1.0, 0.0],    // Haven / unknown
    }
}

pub fn uniforms(
    viewport: [f32; 2],
    camera: [f32; 2],
    lights: &[LightSource],
) -> [f32; UNIFORM_FLOATS] {
    uniforms_with_ambient(viewport, camera, lights, [1.0, 1.0, 1.0, 0.0])
}

pub fn uniforms_with_ambient(
    viewport: [f32; 2],
    camera: [f32; 2],
    lights: &[LightSource],
    ambient: [f32; 4],
) -> [f32; UNIFORM_FLOATS] {
    uniforms_with_raster(viewport, camera, lights, ambient, viewport)
}

/// World projection remains in logical/world units; raster is the actual
/// attachment resolution. The UI supplies its unchanged logical texture size.
/// Trailing raster data keeps the camera, ambient and light offsets unchanged.
pub fn uniforms_with_raster(
    viewport: [f32; 2],
    camera: [f32; 2],
    lights: &[LightSource],
    ambient: [f32; 4],
    raster: [f32; 2],
) -> [f32; UNIFORM_FLOATS] {
    let mut result = [0.0; UNIFORM_FLOATS];
    result[2] = viewport[0].max(1.0);
    result[3] = viewport[1].max(1.0);
    for i in 0..2 {
        result[RASTER_OFFSET + i] = if raster[i].is_finite() {
            raster[i].max(1.0)
        } else {
            1.0
        };
    }
    if ambient.iter().all(|value| value.is_finite()) {
        for i in 0..3 {
            result[4 + i] = ambient[i].clamp(0.9, 1.1);
        }
        result[7] = ambient[3].clamp(0.0, 1.0);
    } else {
        result[4..7].fill(1.0);
    }
    let mut count = 0;
    for light in lights {
        if count == MAX_LIGHTS {
            break;
        }
        if !light
            .position
            .iter()
            .chain(light.color.iter())
            .chain([&light.radius, &light.intensity])
            .all(|value| value.is_finite())
            || !camera.iter().all(|v| v.is_finite())
        {
            continue;
        }
        let offset = 8 + count * 8;
        result[offset] = light.position[0] - camera[0];
        result[offset + 1] = light.position[1] - camera[1];
        result[offset + 2] = light.radius.clamp(1.0, 240.0);
        result[offset + 3] = light.intensity.clamp(0.0, 1.8);
        for i in 0..3 {
            result[offset + 4 + i] = light.color[i].clamp(0.0, 1.0);
        }
        count += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn uniform_layout_is_bounded_and_lights_follow_camera() {
        let light = LightSource {
            position: [100.0, 200.0],
            radius: 80.0,
            color: [0.2, 0.6, 1.0],
            intensity: 1.2,
        };
        let lights = vec![light; 32];
        let a = uniforms([640.0, 360.0], [90.0, 180.0], &lights);
        assert_eq!(a.len() * 4, 304);
        assert_eq!(&a[..8], &[0.0, 0.0, 640.0, 360.0, 1.0, 1.0, 1.0, 0.0]);
        assert_eq!(&a[8..12], &[10.0, 20.0, 80.0, 1.2]);
        assert_eq!(&a[64..68], &[10.0, 20.0, 80.0, 1.2]);
        let b = uniforms([640.0, 360.0], [100.0, 200.0], &lights);
        assert_eq!(&b[8..10], &[0.0, 0.0]);
    }
    #[test]
    fn invalid_lights_are_ignored_and_unused_slots_are_dark() {
        let light = LightSource {
            position: [f32::NAN, 0.0],
            radius: 80.0,
            color: [1.0; 3],
            intensity: 1.0,
        };
        let result = uniforms([640.0, 360.0], [0.0; 2], &[light]);
        assert!(result[8..RASTER_OFFSET].iter().all(|v| *v == 0.0));
    }

    #[test]
    fn biome_material_grade_is_bounded_and_invalid_grade_is_neutral() {
        for kind in 0..=6 {
            let grade = ambient_for_biome(kind);
            let packed = uniforms_with_ambient([640.0, 360.0], [0.0; 2], &[], grade);
            assert_eq!(&packed[4..8], &grade);
            assert!(packed[4..7].iter().all(|v| (0.9..=1.1).contains(v)));
            assert!(packed[8..RASTER_OFFSET].iter().all(|v| *v == 0.0));
        }
        assert_ne!(ambient_for_biome(1), ambient_for_biome(2));
        let invalid = uniforms_with_ambient([640.0, 360.0], [0.0; 2], &[], [f32::NAN; 4]);
        assert_eq!(&invalid[4..8], &[1.0, 1.0, 1.0, 0.0]);
    }

    #[test]
    fn backing_raster_never_changes_world_projection_or_light_layout() {
        let a = uniforms_with_ambient([900.0, 500.0], [0.0; 2], &[], [1.0; 4]);
        let b = uniforms_with_raster([900.0, 500.0], [0.0; 2], &[], [1.0; 4], [1600.0, 900.0]);
        assert_eq!(&a[..RASTER_OFFSET], &b[..RASTER_OFFSET]);
        assert_eq!(&b[RASTER_OFFSET..], &[1600.0, 900.0, 0.0, 0.0]);
        let invalid = uniforms_with_raster(
            [533.0, 300.0],
            [0.0; 2],
            &[],
            [1.0; 4],
            [f32::INFINITY, f32::NAN],
        );
        assert_eq!(&invalid[RASTER_OFFSET..], &[1.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn resize_retains_ui_scale_and_world_uses_every_backing_pixel() {
        for (backing, logical, scale) in [
            ([1280, 720], [640, 360], 2),
            ([1600, 900], [533, 300], 3),
            ([853, 480], [853, 480], 1),
            ([320, 180], [320, 180], 1),
            ([180, 140], [180, 140], 1),
            ([1, 2160], [1, 360], 6),
            ([0, 0], [1, 1], 1),
        ] {
            let layout = RasterLayout::new(backing);
            assert_eq!(layout.world, backing.map(|x| x.max(1)));
            assert_eq!(layout.logical, logical);
            assert_eq!(layout.ui_scale, scale);
            let p = layout.presentation();
            assert_eq!(&p[..2], &layout.world.map(|x| x as f32));
            for axis in 0..2 {
                assert!(p[axis + 2] <= p[axis]);
                assert!(p[axis] - p[axis + 2] < scale as f32);
                // This is the exact nearest UI interval used by present.wgsl.
                let offset = ((p[axis] - p[axis + 2]) * 0.5).floor();
                for pixel in [0, logical[axis] - 1] {
                    let center = offset + (pixel as f32 + 0.5) * p[axis + 2] / logical[axis] as f32;
                    let sampled = ((center - offset) / p[axis + 2] * logical[axis] as f32).floor();
                    assert_eq!(sampled as u32, pixel);
                }
            }
        }
        assert_eq!(
            RasterLayout::new([1600, 900]),
            RasterLayout::new([1600, 900])
        );
    }
}
