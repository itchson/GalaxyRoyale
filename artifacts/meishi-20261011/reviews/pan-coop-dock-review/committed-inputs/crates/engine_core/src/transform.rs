use glam::Vec2;

/// Object-local translation, rotation in radians, and scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Transform {
    pub fn point(self, local: Vec2) -> Vec2 {
        self.affine().point(local)
    }

    /// Returns NaN coordinates for a singular or non-finite transform.
    /// Collision code can use `try_inverse_point` when it needs explicit failure.
    pub fn inverse_point(self, world: Vec2) -> Vec2 {
        self.affine().inverse_point(world)
    }

    pub fn try_inverse_point(self, world: Vec2) -> Option<Vec2> {
        self.affine().try_inverse_point(world)
    }

    pub fn affine(self) -> AffineTransform {
        let (sin, cos) = self.rotation.sin_cos();
        AffineTransform {
            position: self.position,
            basis_x: Vec2::new(cos, sin) * self.scale.x,
            basis_y: Vec2::new(-sin, cos) * self.scale.y,
        }
    }

    /// Composes TRS hierarchy semantics. This is exact for uniform parent scale,
    /// or an unrotated child. Nonuniform scale followed by child rotation can
    /// introduce shear, which TRS cannot represent: use `compose_affine` then.
    pub fn compose(self, child: Self) -> Self {
        Self {
            position: self.point(child.position),
            rotation: self.rotation + child.rotation,
            scale: self.scale * child.scale,
        }
    }

    /// Exact composition, including shear from nested nonuniform scales.
    pub fn compose_affine(self, child: Self) -> AffineTransform {
        self.affine().compose(child.affine())
    }
}

/// A complete 2D affine transform. Unlike TRS this preserves hierarchy shear.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AffineTransform {
    pub position: Vec2,
    pub basis_x: Vec2,
    pub basis_y: Vec2,
}

impl Default for AffineTransform {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            basis_x: Vec2::X,
            basis_y: Vec2::Y,
        }
    }
}

impl From<Transform> for AffineTransform {
    fn from(value: Transform) -> Self {
        value.affine()
    }
}

impl AffineTransform {
    pub fn vector(self, local: Vec2) -> Vec2 {
        self.basis_x * local.x + self.basis_y * local.y
    }

    pub fn point(self, local: Vec2) -> Vec2 {
        self.position + self.vector(local)
    }

    pub fn compose(self, child: Self) -> Self {
        Self {
            position: self.point(child.position),
            basis_x: self.vector(child.basis_x),
            basis_y: self.vector(child.basis_y),
        }
    }

    pub fn try_inverse_point(self, world: Vec2) -> Option<Vec2> {
        let determinant = self.basis_x.perp_dot(self.basis_y);
        if determinant == 0.0
            || !determinant.is_finite()
            || !world.is_finite()
            || !self.position.is_finite()
        {
            return None;
        }
        let relative = world - self.position;
        let local = Vec2::new(
            relative.perp_dot(self.basis_y) / determinant,
            self.basis_x.perp_dot(relative) / determinant,
        );
        local.is_finite().then_some(local)
    }

    pub fn inverse_point(self, world: Vec2) -> Vec2 {
        self.try_inverse_point(world)
            .unwrap_or(Vec2::splat(f32::NAN))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn near(a: Vec2, b: Vec2) {
        assert!((a - b).length() < 0.0001, "{a:?} != {b:?}");
    }

    #[test]
    fn transform_round_trip_rotation_nonuniform_scale_and_reflection() {
        let t = Transform {
            position: Vec2::new(19.0, -5.0),
            rotation: 1.2,
            scale: Vec2::new(-2.0, 0.4),
        };
        let p = Vec2::new(-3.5, 7.0);
        near(t.inverse_point(t.point(p)), p);
    }

    #[test]
    fn uniform_trs_hierarchy_matches_sequential_points() {
        let parent = Transform {
            position: Vec2::new(9.0, 2.0),
            rotation: 0.8,
            scale: Vec2::splat(3.0),
        };
        let child = Transform {
            position: Vec2::new(2.0, 3.0),
            rotation: -0.3,
            scale: Vec2::new(1.0, 2.0),
        };
        let p = Vec2::new(4.0, -1.0);
        near(parent.compose(child).point(p), parent.point(child.point(p)));
    }

    #[test]
    fn affine_preserves_shear_over_three_hierarchy_levels() {
        let a = Transform {
            position: Vec2::new(2.0, 4.0),
            rotation: 0.5,
            scale: Vec2::new(3.0, 0.5),
        };
        let b = Transform {
            position: Vec2::new(-1.0, 2.0),
            rotation: -0.9,
            scale: Vec2::new(0.7, 2.0),
        };
        let c = Transform {
            position: Vec2::new(1.0, -3.0),
            rotation: 0.6,
            scale: Vec2::new(2.0, 0.8),
        };
        let p = Vec2::new(3.0, -4.0);
        let composed = a.compose_affine(b).compose(c.affine());
        near(composed.point(p), a.point(b.point(c.point(p))));
        near(composed.inverse_point(composed.point(p)), p);
    }

    #[test]
    fn singular_transform_has_explicit_inverse_failure() {
        let t = Transform {
            scale: Vec2::new(0.0, 1.0),
            ..Transform::default()
        };
        assert!(t.try_inverse_point(Vec2::ZERO).is_none());
        assert!(!t.inverse_point(Vec2::ZERO).is_finite());
    }
}
