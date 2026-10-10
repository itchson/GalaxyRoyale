//! Bounded camera-relative uniform data shared by both graphics backends.
use game_core::LightSource;
pub const MAX_LIGHTS: usize = 8;
pub const UNIFORM_FLOATS: usize = 8 + MAX_LIGHTS * 8;

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
    let mut result = [0.0; UNIFORM_FLOATS];
    result[2] = viewport[0].max(1.0);
    result[3] = viewport[1].max(1.0);
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
        assert_eq!(a.len() * 4, 288);
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
        assert!(result[8..].iter().all(|v| *v == 0.0));
    }

    #[test]
    fn biome_material_grade_is_bounded_and_invalid_grade_is_neutral() {
        for kind in 0..=6 {
            let grade = ambient_for_biome(kind);
            let packed = uniforms_with_ambient([640.0, 360.0], [0.0; 2], &[], grade);
            assert_eq!(&packed[4..8], &grade);
            assert!(packed[4..7].iter().all(|v| (0.9..=1.1).contains(v)));
            assert!(packed[8..].iter().all(|v| *v == 0.0));
        }
        assert_ne!(ambient_for_biome(1), ambient_for_biome(2));
        let invalid = uniforms_with_ambient([640.0, 360.0], [0.0; 2], &[], [f32::NAN; 4]);
        assert_eq!(&invalid[4..8], &[1.0, 1.0, 1.0, 0.0]);
    }
}
