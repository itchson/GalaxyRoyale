//! Optional object-local normal/emissive metadata in spare cell-instance bits.
use crate::AffineTransform;

pub const PIXEL_TRANSFORM_SLOT_MASK: u32 = 0x7f;
pub const PIXEL_SURFACE_ENABLED: u32 = 1 << 31;

/// Four-byte validated full-octahedral normal plus a linear emission multiplier.
/// Local axes are +X right, +Y down, +Z toward the viewer. These fields never
/// change albedo, alpha, occupancy or physics. Missing maps use no payload bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Surface(u32);

impl Surface {
    /// Normalizes finite nonzero triples without overflow/underflow from their
    /// magnitude. Quantized normals have eight bits per octahedral coordinate.
    /// Mask255 preserves the existing object's material emission; mask0 blocks
    /// self-illumination. This is not independent energy or a new light source.
    pub fn new(normal: [f32; 3], emission_mask: u8) -> Result<Self, &'static str> {
        if !normal.iter().all(|v| v.is_finite()) {
            return Err("surface normal must be finite");
        }
        let scale = normal.iter().map(|v| v.abs()).fold(0.0_f32, f32::max);
        if scale == 0.0 {
            return Err("surface normal must be nonzero");
        }
        let mut n = normal.map(|v| v / scale);
        let length = n.iter().map(|v| v.abs()).sum::<f32>();
        n = n.map(|v| v / length);
        let mut oct = [n[0], n[1]];
        if n[2] < 0.0 {
            oct = [
                (1.0 - n[1].abs()) * sign_not_zero(n[0]),
                (1.0 - n[0].abs()) * sign_not_zero(n[1]),
            ];
        }
        let [x, y] = oct.map(|v| ((v * 0.5 + 0.5) * 255.0).round() as u32);
        Ok(Self(
            PIXEL_SURFACE_ENABLED | (x << 7) | (y << 15) | (u32::from(emission_mask) << 23),
        ))
    }

    /// Cell payload with low seven transform-slot bits always zero.
    pub fn packed(self) -> u32 {
        self.0
    }
    pub fn from_cell_word(word: u32) -> Option<Self> {
        (word & PIXEL_SURFACE_ENABLED != 0).then_some(Self(word & !PIXEL_TRANSFORM_SLOT_MASK))
    }
    pub fn emission_mask(self) -> u8 {
        ((self.0 >> 23) & 255) as u8
    }
    pub fn normal(self) -> [f32; 3] {
        let oct =
            [((self.0 >> 7) & 255), ((self.0 >> 15) & 255)].map(|v| v as f32 / 255.0 * 2.0 - 1.0);
        let mut n = [oct[0], oct[1], 1.0 - oct[0].abs() - oct[1].abs()];
        if n[2] < 0.0 {
            n[0] = (1.0 - oct[1].abs()) * sign_not_zero(oct[0]);
            n[1] = (1.0 - oct[0].abs()) * sign_not_zero(oct[1]);
        }
        let length = n.iter().map(|v| v * v).sum::<f32>().sqrt();
        n.map(|v| v / length)
    }

    /// Portable lighting oracle for the full composed basis: inverse transpose
    /// of diag(A, sqrt(abs(det A))). Uniform scale preserves normal appearance;
    /// shear, reflection and nested rotation remain exact. Translation is
    /// irrelevant. Singular/nonfinite bases have no defined transformed normal.
    /// f64 intermediates admit finite extreme f32 bases without determinant
    /// overflow/underflow; a renderer still needs its own representable limits.
    pub fn world_normal(self, transform: AffineTransform) -> Option<[f32; 3]> {
        if !transform.basis_x.is_finite() || !transform.basis_y.is_finite() {
            return None;
        }
        let [a, c] = transform.basis_x.to_array().map(f64::from);
        let [b, d] = transform.basis_y.to_array().map(f64::from);
        let determinant = a * d - b * c;
        if determinant == 0.0 {
            return None;
        }
        let n = self.normal().map(f64::from);
        let world = [
            (d * n[0] - c * n[1]) / determinant,
            (-b * n[0] + a * n[1]) / determinant,
            n[2] / determinant.abs().sqrt(),
        ];
        let scale = world.iter().map(|v| v.abs()).fold(0.0_f64, f64::max);
        if !scale.is_finite() || scale == 0.0 {
            return None;
        }
        let world = world.map(|v| v / scale);
        let length = world.iter().map(|v| v * v).sum::<f64>().sqrt();
        Some(world.map(|v| (v / length) as f32))
    }
}
fn sign_not_zero(value: f32) -> f32 {
    if value < 0.0 { -1.0 } else { 1.0 }
}

#[cfg(test)]
#[path = "surface_tests.rs"]
mod tests;
