//! Opt-in arcade antipodal wrapping for a bounded circular arena.
//! World axes stay fixed across the seam; this is not a globally smooth manifold.
use crate::{AffineTransform, Transform, Vec2};
use glam::DVec2;

pub const MAX_CIRCULAR_CROSSINGS: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CircularTopology {
    center: Vec2,
    radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircularWrapError {
    InvalidCircle,
    NonFiniteStep,
    StartOutside,
    TangentialBoundary,
    CrossingLimit,
    UnrepresentableOutput,
}

impl std::fmt::Display for CircularWrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidCircle => "circle must have a finite center and positive finite radius",
            Self::NonFiniteStep => "step positions must be finite",
            Self::StartOutside => "step must start inside the closed circular arena",
            Self::TangentialBoundary => {
                "a boundary tangent cannot continue inward after antipodal wrapping"
            }
            Self::CrossingLimit => "step exceeded the bounded circular crossing count",
            Self::UnrepresentableOutput => {
                "wrapped coordinates cannot be represented reliably as f32"
            }
        };
        f.write_str(message)
    }
}
impl std::error::Error for CircularWrapError {}

/// The endpoint and chart translation for one integration step. Apply the same
/// delta to previous positions, camera history, and a click-movement target.
/// Velocities and headings are vectors/orientations and must remain unchanged.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CircularWrap {
    pub position: Vec2,
    pub rebase_delta: Vec2,
    pub crossings: u32,
}

impl CircularWrap {
    pub fn rebase_point(self, point: Vec2) -> Vec2 {
        point + self.rebase_delta
    }

    pub fn rebase_transform(self, mut transform: Transform) -> Transform {
        transform.position = self.rebase_point(transform.position);
        transform
    }

    pub fn rebase_affine(self, mut transform: AffineTransform) -> AffineTransform {
        transform.position = self.rebase_point(transform.position);
        transform
    }
}

impl CircularTopology {
    pub fn new(center: Vec2, radius: f32) -> Result<Self, CircularWrapError> {
        if !center.is_finite() || !radius.is_finite() || radius <= 0.0 {
            Err(CircularWrapError::InvalidCircle)
        } else {
            Ok(Self { center, radius })
        }
    }

    pub fn center(self) -> Vec2 {
        self.center
    }
    pub fn radius(self) -> f32 {
        self.radius
    }

    /// Sweeps an already integrated displacement. At an outward crossing q,
    /// continue from its antipode 2*center-q with the remaining displacement.
    /// An endpoint exactly on the boundary is retained until it moves outward.
    ///
    /// No overshoot is discarded. Failure is explicit and never mutates caller
    /// state. Collision integration needs the individual seam-split segments;
    /// the returned chart-rebased previous point alone is not a collision path.
    pub fn wrap_segment(
        self,
        previous: Vec2,
        current: Vec2,
    ) -> Result<CircularWrap, CircularWrapError> {
        if !previous.is_finite() || !current.is_finite() {
            return Err(CircularWrapError::NonFiniteStep);
        }
        // Solve in f64 so boundary intersections remain stable even for an
        // overshooting f32 step. This does not create extra f32 world precision.
        let center = self.center.as_dvec2();
        let radius = f64::from(self.radius);
        let radius_squared = radius * radius;
        let mut position = previous.as_dvec2() - center;
        if position.length_squared() > radius_squared {
            return Err(CircularWrapError::StartOutside);
        }
        let mut remaining = current.as_dvec2() - previous.as_dvec2();
        let mut crossings = 0;
        loop {
            let endpoint = position + remaining;
            if endpoint.length_squared() <= radius_squared {
                return self.finish(endpoint, current, crossings);
            }
            if crossings == MAX_CIRCULAR_CROSSINGS {
                return Err(CircularWrapError::CrossingLimit);
            }
            let a = remaining.length_squared();
            let b = position.dot(remaining);
            let c = position.length_squared() - radius_squared;
            let discriminant = (b * b - a * c).max(0.0).sqrt();
            // The alternate form avoids cancellation for a small outward step.
            let fraction = if b >= 0.0 && b + discriminant > 0.0 {
                -c / (b + discriminant)
            } else if a > 0.0 {
                (-b + discriminant) / a
            } else {
                return Err(CircularWrapError::TangentialBoundary);
            }
            .clamp(0.0, 1.0);
            let edge = position + remaining * fraction;
            if edge.dot(remaining) <= radius * a.sqrt() * f64::EPSILON * 16.0 {
                return Err(CircularWrapError::TangentialBoundary);
            }
            // Constrain solver roundoff to the circle before the next sweep.
            position = -edge.normalize() * radius;
            remaining *= 1.0 - fraction;
            crossings += 1;
        }
    }

    fn finish(
        self,
        local: DVec2,
        original: Vec2,
        crossings: u32,
    ) -> Result<CircularWrap, CircularWrapError> {
        let center = self.center.as_dvec2();
        let radius = f64::from(self.radius);
        let mut position = (center + local).as_vec2();
        // A rounded boundary point can lie one f32 ULP outside the circle.
        // Conservatively nudge only that final rounding case inward; otherwise
        // the next step would start outside despite a valid internal endpoint.
        for _ in 0..4 {
            let relative = position.as_dvec2() - center;
            if relative.length_squared() <= radius * radius {
                break;
            }
            if relative.length() > radius * (1.0 + f64::from(f32::EPSILON) * 32.0) {
                return Err(CircularWrapError::UnrepresentableOutput);
            }
            position = (center
                + relative.normalize() * radius * (1.0 - f64::from(f32::EPSILON) * 4.0))
                .as_vec2();
        }
        let rebase_delta = position - original;
        if !position.is_finite()
            || !rebase_delta.is_finite()
            || (position.as_dvec2() - center).length_squared() > radius * radius
        {
            return Err(CircularWrapError::UnrepresentableOutput);
        }
        Ok(CircularWrap {
            position,
            rebase_delta,
            crossings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn near(a: Vec2, b: Vec2) {
        assert!(a.distance(b) < 0.0001, "{a:?} != {b:?}");
    }

    #[test]
    fn boundary_is_closed_and_outward_overshoot_continues_inward() {
        let circle = CircularTopology::new(Vec2::ZERO, 10.0).unwrap();
        let edge = circle
            .wrap_segment(Vec2::ZERO, Vec2::new(10.0, 0.0))
            .unwrap();
        assert_eq!(edge.crossings, 0);
        assert_eq!(edge.rebase_delta, Vec2::ZERO);
        let outward = circle
            .wrap_segment(edge.position, Vec2::new(12.0, 0.0))
            .unwrap();
        assert_eq!(outward.crossings, 1);
        near(outward.position, Vec2::new(-8.0, 0.0));
        near(outward.rebase_delta, Vec2::new(-20.0, 0.0));
        let inward = circle
            .wrap_segment(Vec2::new(10.0, 0.0), Vec2::new(9.0, 1.0))
            .unwrap();
        assert_eq!(inward.crossings, 0);
        near(inward.position, Vec2::new(9.0, 1.0));
        let still = circle.wrap_segment(edge.position, edge.position).unwrap();
        assert_eq!(still.crossings, 0);
    }

    #[test]
    fn multiple_crossings_preserve_all_travel_and_diagonal_antipodes() {
        let circle = CircularTopology::new(Vec2::ZERO, 10.0).unwrap();
        let many = circle
            .wrap_segment(Vec2::ZERO, Vec2::new(65.0, 0.0))
            .unwrap();
        assert_eq!(many.crossings, 3);
        near(many.position, Vec2::new(5.0, 0.0));
        near(many.rebase_delta, Vec2::new(-60.0, 0.0));
        let center = Vec2::new(100.0, -50.0);
        let diagonal = CircularTopology::new(center, 10.0)
            .unwrap()
            .wrap_segment(center, center + Vec2::splat(12.0))
            .unwrap();
        assert_eq!(diagonal.crossings, 1);
        near(
            diagonal.position,
            center + Vec2::splat(12.0 - 10.0 * std::f32::consts::SQRT_2),
        );
        assert!(diagonal.position.distance(center) < 10.0);
    }

    #[test]
    fn shared_rebase_preserves_motion_heading_camera_and_destination() {
        let circle = CircularTopology::new(Vec2::ZERO, 10.0).unwrap();
        let velocity = Vec2::new(12.0, 0.0);
        let previous = Transform {
            position: Vec2::new(9.0, 0.0),
            rotation: 1.3,
            scale: Vec2::new(2.0, 0.5),
        };
        let current = Transform {
            position: previous.position + velocity * 0.25,
            ..previous
        };
        let wrap = circle
            .wrap_segment(previous.position, current.position)
            .unwrap();
        let rebased_previous = wrap.rebase_transform(previous);
        let rebased_current = wrap.rebase_transform(current);
        near(rebased_current.position, wrap.position);
        near(
            (rebased_current.position - rebased_previous.position) / 0.25,
            velocity,
        );
        assert_eq!(rebased_current.rotation, current.rotation);
        assert_eq!(rebased_current.scale, current.scale);
        let camera = Vec2::new(8.0, 1.0);
        let destination = Vec2::new(40.0, 3.0);
        near(
            wrap.position - wrap.rebase_point(camera),
            current.position - camera,
        );
        near(
            wrap.rebase_point(destination) - wrap.position,
            destination - current.position,
        );
        let affine = current.affine();
        let rebased_affine = wrap.rebase_affine(affine);
        assert_eq!(rebased_affine.basis_x, affine.basis_x);
        assert_eq!(rebased_affine.basis_y, affine.basis_y);
        let next = circle
            .wrap_segment(wrap.position, wrap.position + velocity * 0.25)
            .unwrap();
        assert_eq!(next.crossings, 0);
        near(next.position, Vec2::new(-5.0, 0.0));
    }

    #[test]
    fn invalid_and_tangent_steps_fail_explicitly_without_dropping_overshoot() {
        assert_eq!(
            CircularTopology::new(Vec2::ZERO, 0.0),
            Err(CircularWrapError::InvalidCircle)
        );
        assert_eq!(
            CircularTopology::new(Vec2::splat(f32::NAN), 10.0),
            Err(CircularWrapError::InvalidCircle)
        );
        let circle = CircularTopology::new(Vec2::ZERO, 10.0).unwrap();
        assert_eq!(
            circle.wrap_segment(Vec2::ZERO, Vec2::splat(f32::NAN)),
            Err(CircularWrapError::NonFiniteStep)
        );
        assert_eq!(
            circle.wrap_segment(Vec2::new(11.0, 0.0), Vec2::ZERO),
            Err(CircularWrapError::StartOutside)
        );
        assert_eq!(
            circle.wrap_segment(Vec2::new(10.0, 0.0), Vec2::new(10.0, 1.0)),
            Err(CircularWrapError::TangentialBoundary)
        );
        assert_eq!(
            circle.wrap_segment(Vec2::ZERO, Vec2::new(2000.0, 0.0)),
            Err(CircularWrapError::CrossingLimit)
        );
    }

    #[test]
    fn swept_result_matches_subdivided_motion_for_oblique_crossings() {
        let circle = CircularTopology::new(Vec2::new(100.0, -50.0), 10.0).unwrap();
        let mut seed = 0x5365_616d_u32;
        let mut random = || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (seed >> 8) as f32 / 16_777_216.0
        };
        for _ in 0..500 {
            let start_angle = random() * std::f32::consts::TAU;
            let start = circle.center()
                + Vec2::new(start_angle.cos(), start_angle.sin()) * (random() * 8.0);
            let angle = random() * std::f32::consts::TAU;
            let displacement = Vec2::new(angle.cos(), angle.sin()) * (random() * 100.0);
            let whole = circle.wrap_segment(start, start + displacement).unwrap();
            let mut position = start;
            let mut crossings = 0;
            for _ in 0..32 {
                let part = circle
                    .wrap_segment(position, position + displacement / 32.0)
                    .unwrap();
                position = part.position;
                crossings += part.crossings;
            }
            assert!(
                position.distance(whole.position) < 0.002,
                "{position:?} != {:?}",
                whole.position
            );
            assert_eq!(crossings, whole.crossings);
            assert!(position.distance(circle.center()) <= circle.radius());
        }
    }
}
