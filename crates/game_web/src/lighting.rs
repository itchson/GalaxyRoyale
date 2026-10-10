//! Bounded camera-relative uniform data shared by both graphics backends.
use game_core::LightSource;
pub const MAX_LIGHTS: usize = 8;
pub const UNIFORM_FLOATS: usize = 4 + MAX_LIGHTS * 8;

pub fn uniforms(
    viewport: [f32; 2],
    camera: [f32; 2],
    lights: &[LightSource],
) -> [f32; UNIFORM_FLOATS] {
    let mut result = [0.0; UNIFORM_FLOATS];
    result[2] = viewport[0].max(1.0);
    result[3] = viewport[1].max(1.0);
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
        let offset = 4 + count * 8;
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
        assert_eq!(a.len() * 4, 272);
        assert_eq!(&a[..8], &[0.0, 0.0, 640.0, 360.0, 10.0, 20.0, 80.0, 1.2]);
        assert_eq!(&a[60..64], &[10.0, 20.0, 80.0, 1.2]);
        let b = uniforms([640.0, 360.0], [100.0, 200.0], &lights);
        assert_eq!(&b[4..6], &[0.0, 0.0]);
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
        assert!(result[4..].iter().all(|v| *v == 0.0));
    }
}
