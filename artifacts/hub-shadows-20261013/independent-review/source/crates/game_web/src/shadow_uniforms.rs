//! Portable uniform packing, without devices or browser state.
use bytemuck::{Pod, Zeroable};
use engine_core::{PlanarShadows, Vec2, shadow::MAX_SHADOW_CASTERS};
use game_core::LightSource;

pub const OBJECT_TABLE_BYTES: u64 = 128 * 8;
pub const SHADOW_DESCRIPTOR_BYTES: u64 = 400;
pub const SHADOW_MASK_BYTES: u64 = 16384;
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CasterUniform {
    /// inverse rowX.xy, translation, presentation height.
    pub x: [f32; 4],
    /// inverse rowY.xy, translation, reserved.
    pub y: [f32; 4],
    /// width, height, bit offset, enabled.
    pub field: [u32; 4],
}
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct ShadowUniforms {
    /// count, admitted light bits, area taps, contact enabled.
    pub header: [u32; 4],
    pub casters: [CasterUniform; MAX_SHADOW_CASTERS],
}
pub fn uniforms(
    shadows: Option<&PlanarShadows>,
    camera: [f32; 2],
    lights: &[LightSource],
) -> ShadowUniforms {
    let mut out = ShadowUniforms::zeroed();
    let Some(shadows) = shadows else {
        return out;
    };
    let (Some(caster), Some(key)) = (shadows.caster, shadows.light) else {
        return out;
    };
    if caster.width == 0
        || caster.width > 128
        || caster.height_cells == 0
        || caster.height_cells > 128
        || !caster.height.is_finite()
        || !(0.0..=64.0).contains(&caster.height)
    {
        return out;
    }
    if !camera.iter().all(|v| v.is_finite()) {
        return out;
    }
    let mut admitted = 0u32;
    for light in lights {
        if admitted == crate::lighting::MAX_LIGHTS as u32 {
            break;
        }
        if crate::lighting::packed_light(light, camera).is_none() {
            continue;
        }
        if light.position == key.position
            && light.radius == key.radius
            && light.color == key.color
            && light.intensity == key.intensity
            && light.radius.clamp(1.0, 240.0) * 0.35 > caster.height
        {
            out.header[1] |= 1 << admitted;
        }
        admitted += 1;
    }
    if out.header[1] == 0 || !engine_core::shadow::valid_basis(caster.transform) {
        return out;
    }
    // Compute in f64, after rebasing, to avoid world-position cancellation.
    let t = caster.transform;
    let ax = f64::from(t.basis_x.x);
    let ay = f64::from(t.basis_x.y);
    let bx = f64::from(t.basis_y.x);
    let by = f64::from(t.basis_y.y);
    let det = ax * by - ay * bx;
    let ix = [by / det, -bx / det];
    let iy = [-ay / det, ax / det];
    let p = Vec2::from_array(camera);
    let px = f64::from(t.position.x) - f64::from(p.x);
    let py = f64::from(t.position.y) - f64::from(p.y);
    let x = [
        ix[0] as f32,
        ix[1] as f32,
        (-ix[0] * px - ix[1] * py) as f32,
        caster.height,
    ];
    let y = [
        iy[0] as f32,
        iy[1] as f32,
        (-iy[0] * px - iy[1] * py) as f32,
        0.0,
    ];
    if !x.iter().chain(y.iter()).all(|v| v.is_finite()) {
        out.header = [0; 4];
        return out;
    }
    out.casters[0] = CasterUniform {
        x,
        y,
        field: [caster.width, caster.height_cells, 0, 1],
    };
    out.header[0] = 1;
    out.header[2] = 4;
    out.header[3] = 1;
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::{PixelBody, ShadowLight, Transform};
    fn key() -> ShadowLight {
        ShadowLight {
            position: [0.0; 2],
            radius: 120.0,
            color: [0.2, 0.85, 0.92],
            intensity: 0.5,
        }
    }
    fn light(k: ShadowLight) -> LightSource {
        LightSource {
            position: k.position,
            radius: k.radius,
            color: k.color,
            intensity: k.intensity,
        }
    }
    #[test]
    fn split_std140_layout_matches_portable_uniform_limits() {
        assert_eq!(std::mem::size_of::<CasterUniform>(), 48);
        assert_eq!(std::mem::size_of::<ShadowUniforms>(), 400);
        assert_eq!(std::mem::offset_of!(ShadowUniforms, casters), 16);
        let limits = wgpu::Limits::downlevel_webgl2_defaults();
        for bytes in [
            OBJECT_TABLE_BYTES,
            SHADOW_DESCRIPTOR_BYTES,
            SHADOW_MASK_BYTES,
            4096,
            304,
        ] {
            assert!(bytes <= limits.max_uniform_buffer_binding_size);
        }
        assert!(limits.max_uniform_buffers_per_shader_stage >= 5);
        assert_eq!(
            SHADOW_MASK_BYTES,
            engine_core::shadow::SHADOW_WORDS as u64 * 4
        );
        assert_eq!(OBJECT_TABLE_BYTES % 256, 0);
    }
    #[test]
    fn light_selection_uses_actual_filtering_cap_and_camera_rebased_inverse() {
        let mut s = PlanarShadows::default();
        let k = key();
        let b = PixelBody::from_ascii(&["##"], &[('#', 0xff_ffffff)]);
        let t = Transform {
            position: Vec2::new(10000.25, -9000.5),
            rotation: 0.7,
            scale: Vec2::new(-0.2, 0.75),
        }
        .affine();
        s.set_body(&b, t, 6.0, k);
        let invalid = LightSource {
            position: [f32::NAN, 0.0],
            ..light(k)
        };
        let unrelated = LightSource {
            position: [1.0, 0.0],
            ..light(k)
        };
        let camera = [9990.0, -8990.0];
        let u = uniforms(Some(&s), camera, &[invalid, unrelated.clone(), light(k)]);
        assert_eq!(u.header, [1, 2, 4, 1]);
        let point = t.point(Vec2::new(-0.5, 0.0)) - Vec2::from_array(camera);
        let record = u.casters[0];
        let local = Vec2::new(
            record.x[0] * point.x + record.x[1] * point.y + record.x[2],
            record.y[0] * point.x + record.y[1] * point.y + record.y[2],
        );
        assert!((local - Vec2::new(-0.5, 0.0)).length() < 0.005);
        let mut full = vec![unrelated; 8];
        full.push(light(k));
        assert_eq!(uniforms(Some(&s), camera, &full).header, [0; 4]);
        s.clear();
        assert_eq!(uniforms(Some(&s), camera, &[light(k)]).header, [0; 4]);
        assert_eq!(uniforms(None, camera, &[light(k)]).header, [0; 4]);
    }
}
