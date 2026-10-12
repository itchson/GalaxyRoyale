//! Presentation-only planar silhouettes. Occupancy, authority and saves stay in
//! PixelBody/gameplay; these fixed masks are never collider data.
use crate::{AffineTransform, PixelBody, Vec2};
use bytemuck::{Pod, Zeroable};

pub const MAX_SHADOW_CASTERS: usize = 8;
pub const SHADOW_SIDE: usize = 128;
pub const SHADOW_WORDS_PER_CASTER: usize = SHADOW_SIDE * SHADOW_SIDE / 32;
pub const SHADOW_WORDS: usize = MAX_SHADOW_CASTERS * SHADOW_WORDS_PER_CASTER;
pub const SHADOW_RECEIVER: u32 = 1;

/// Optional parallel transform metadata. Zero is exact legacy presentation.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct ObjectMeta {
    pub height: f32,
    pub flags: u32,
}
impl ObjectMeta {
    pub fn sanitized(self) -> Self {
        if !self.height.is_finite()
            || !(0.0..=64.0).contains(&self.height)
            || self.flags & !SHADOW_RECEIVER != 0
        {
            Self::default()
        } else {
            self
        }
    }
    pub const fn receiver(height: f32) -> Self {
        Self {
            height,
            flags: SHADOW_RECEIVER,
        }
    }
}

/// Identifies an actual native light before the renderer's finite filtering and
/// eight-light admission. It does not add or move a light.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowLight {
    pub position: [f32; 2],
    pub radius: f32,
    pub color: [f32; 3],
    pub intensity: f32,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowCaster {
    pub transform: AffineTransform,
    pub height: f32,
    pub width: u32,
    pub height_cells: u32,
}

/// The mask block is exactly 16KiB (the WebGL2 uniform-block floor). Descriptor
/// and object metadata live in separate blocks. Slot0 alone is populated by the
/// first runtime increment; the capacity is bounded, not a performance claim.
#[derive(Debug, Clone)]
pub struct PlanarShadows {
    pub caster: Option<ShadowCaster>,
    pub light: Option<ShadowLight>,
    masks: Box<[u32; SHADOW_WORDS]>,
}
impl Default for PlanarShadows {
    fn default() -> Self {
        Self {
            caster: None,
            light: None,
            masks: Box::new([0; SHADOW_WORDS]),
        }
    }
}
impl PlanarShadows {
    pub fn clear(&mut self) {
        self.caster = None;
        self.light = None;
    }
    pub fn masks(&self) -> &[u32; SHADOW_WORDS] {
        &self.masks
    }
    /// Reject before modifying the mask; on rejection no stale caster is active.
    /// Alpha>=128 is the explicit opaque silhouette rule. Actual missing body
    /// cells never enter this journal, including after localized destruction.
    pub fn set_body(
        &mut self,
        body: &PixelBody,
        transform: AffineTransform,
        height: f32,
        light: ShadowLight,
    ) -> bool {
        self.clear();
        if body.width == 0
            || body.height == 0
            || body.width > SHADOW_SIDE
            || body.height > SHADOW_SIDE
            || !(0.0..=64.0).contains(&height)
            || !height.is_finite()
            || !light
                .position
                .iter()
                .chain(light.color.iter())
                .chain([&light.radius, &light.intensity])
                .all(|v| v.is_finite())
            || !valid_basis(transform)
        {
            return false;
        }
        self.masks[..SHADOW_WORDS_PER_CASTER].fill(0);
        let mut opaque = 0;
        for (index, _, color) in body.cells_indexed() {
            if color >> 24 < 128 {
                continue;
            }
            let bit = (index / body.width) * SHADOW_SIDE + index % body.width;
            self.masks[bit / 32] |= 1 << (bit % 32);
            opaque += 1;
        }
        if opaque == 0 {
            return false;
        }
        self.caster = Some(ShadowCaster {
            transform,
            height,
            width: body.width as u32,
            height_cells: body.height as u32,
        });
        self.light = Some(light);
        true
    }
    pub fn occupied(&self, local: Vec2) -> bool {
        let Some(caster) = self.caster else {
            return false;
        };
        let cell =
            (local + Vec2::new(caster.width as f32, caster.height_cells as f32) * 0.5).floor();
        if !cell.is_finite()
            || cell.min_element() < 0.0
            || cell.x >= caster.width as f32
            || cell.y >= caster.height_cells as f32
        {
            return false;
        }
        let bit = cell.y as usize * SHADOW_SIDE + cell.x as usize;
        self.masks[bit / 32] & (1 << (bit % 32)) != 0
    }
    /// Pure center-ray oracle for native checks. GPU uses four bounded source
    /// taps for a small penumbra, all with the same ray and bitmask contract.
    pub fn blocks_ray(
        &self,
        receiver: Vec2,
        receiver_height: f32,
        source: Vec2,
        source_height: f32,
    ) -> bool {
        let Some(caster) = self.caster else {
            return false;
        };
        if !receiver.is_finite()
            || !source.is_finite()
            || !receiver_height.is_finite()
            || !source_height.is_finite()
            || receiver_height >= caster.height
            || caster.height >= source_height
        {
            return false;
        }
        let t = (caster.height - receiver_height) / (source_height - receiver_height);
        let point = receiver.lerp(source, t);
        caster
            .transform
            .try_inverse_point(point)
            .is_some_and(|local| self.occupied(local))
    }
}
pub fn valid_basis(t: AffineTransform) -> bool {
    if !t.position.is_finite() || !t.basis_x.is_finite() || !t.basis_y.is_finite() {
        return false;
    }
    let magnitude = t
        .basis_x
        .abs()
        .max_element()
        .max(t.basis_y.abs().max_element());
    if magnitude == 0.0 {
        return false;
    }
    let x = t.basis_x / magnitude;
    let y = t.basis_y / magnitude;
    x.perp_dot(y).abs() >= 1e-8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Transform;
    fn light() -> ShadowLight {
        ShadowLight {
            position: [0.0; 2],
            radius: 120.0,
            color: [1.0; 3],
            intensity: 0.5,
        }
    }
    #[test]
    fn ray_uses_actual_opaque_cells_and_damage_holes() {
        let mut body = PixelBody::from_ascii(&["###", "#.#", "###"], &[('#', 0xff_ffffff)]);
        let mut s = PlanarShadows::default();
        assert!(s.set_body(&body, AffineTransform::default(), 6.0, light()));
        assert!(!s.blocks_ray(Vec2::ZERO, 0.0, Vec2::ZERO, 42.0));
        let receiver = Vec2::new(-1.0, 0.0) * (42.0 / 36.0);
        assert!(s.blocks_ray(receiver, 0.0, Vec2::ZERO, 42.0));
        body.damage(Vec2::new(-1.0, 0.0), 0.0);
        s.set_body(&body, AffineTransform::default(), 6.0, light());
        assert!(!s.blocks_ray(receiver, 0.0, Vec2::ZERO, 42.0));
        assert!(!s.blocks_ray(Vec2::ZERO, 6.0, Vec2::ZERO, 42.0));
        assert!(!s.blocks_ray(Vec2::ZERO, 0.0, Vec2::ZERO, 5.0));
    }
    #[test]
    fn affine_projection_handles_nested_shear_reflection_fractional_scale_and_rebase() {
        let b = PixelBody::from_ascii(&["#."], &[('#', 0xff_ffffff)]);
        for scale in [Vec2::new(0.125, 0.7), Vec2::new(-2.0, 0.5)] {
            let t = Transform {
                position: Vec2::new(10000.25, -9000.5),
                rotation: 0.7,
                scale,
            }
            .compose_affine(Transform {
                rotation: -0.2,
                scale: Vec2::new(1.2, 0.8),
                ..Default::default()
            });
            let mut s = PlanarShadows::default();
            s.set_body(&b, t, 6.0, light());
            let source = t.position + Vec2::new(-12.0, 8.0);
            let opaque = t.point(Vec2::new(-0.5, 0.0));
            let receiver = (opaque - source * (6.0 / 42.0)) / (1.0 - 6.0 / 42.0);
            assert!(s.blocks_ray(receiver, 0.0, source, 42.0));
            let offset = Vec2::new(9990.0, -8990.0);
            let rebased = AffineTransform {
                position: t.position - offset,
                ..t
            };
            s.set_body(&b, rebased, 6.0, light());
            assert!(s.blocks_ray(receiver - offset, 0.0, source - offset, 42.0));
            assert!(!s.occupied(Vec2::new(0.5, 0.0)));
        }
    }
    #[test]
    fn bounded_mask_transparency_invalid_and_empty_do_not_keep_stale_caster() {
        assert_eq!(SHADOW_WORDS * 4, 16384);
        assert_eq!(std::mem::size_of::<ObjectMeta>(), 8);
        let mut s = PlanarShadows::default();
        let b = PixelBody::from_ascii(&["#"], &[('#', 0x7f_ffffff)]);
        assert!(!s.set_body(&b, AffineTransform::default(), 6.0, light()));
        let opaque = PixelBody::from_ascii(&["#"], &[('#', 0x80_ffffff)]);
        assert!(s.set_body(&opaque, AffineTransform::default(), 6.0, light()));
        let invalid = AffineTransform {
            basis_y: Vec2::X,
            ..Default::default()
        };
        assert!(!s.set_body(&opaque, invalid, 6.0, light()));
        assert!(s.caster.is_none());
        let oversize = PixelBody::from_ascii(&[&"#".repeat(129)], &[('#', 0xff_ffffff)]);
        assert!(!s.set_body(&oversize, AffineTransform::default(), 6.0, light()));
        assert_eq!(
            ObjectMeta::receiver(f32::NAN).sanitized(),
            ObjectMeta::default()
        );
    }
}
