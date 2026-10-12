//! World scenery extends beyond the pilot travel square. Confine authoritative
//! surviving geometry, and admit the same finite domain at publication/replica.
use super::*;

pub(super) const GEOMETRY_LIMIT: f32 = 1_100_000.0;

pub(super) fn vector(value: [f32; 2], limit: f32) -> bool {
    value.into_iter().all(|v| v.is_finite() && v.abs() <= limit)
}

pub(super) fn pose(
    position: [f32; 2],
    previous: [f32; 2],
    velocity: [f32; 2],
    heading: f32,
    previous_heading: f32,
    position_limit: f32,
) -> bool {
    vector(position, position_limit)
        && vector(previous, position_limit)
        && vector(velocity, WORLD_LIMIT)
        && heading.is_finite()
        && heading.abs() <= WORLD_LIMIT
        && previous_heading.is_finite()
        && previous_heading.abs() <= WORLD_LIMIT
}

impl Game {
    pub(super) fn confine_world_geometry(&mut self) {
        for (entity, (motion, body, bullet, pilot)) in self.world.query_mut::<(
            Entity,
            (
                &mut Motion,
                Option<&PixelBody>,
                Option<&Bullet>,
                Option<&co_op::Pilot>,
            ),
        )>() {
            // Pilot confinement owns the stricter travel domain. This helper
            // must not make a malformed pilot appear valid as ordinary scenery.
            if entity == self.player || pilot.is_some() || (body.is_none() && bullet.is_none()) {
                continue;
            }
            if !motion.position.is_finite() || !motion.previous.is_finite() {
                continue; // Invalid native state remains a publication failure.
            }
            for axis in 0..2 {
                let position = motion.position[axis];
                if position > GEOMETRY_LIMIT {
                    motion.position[axis] = GEOMETRY_LIMIT;
                    if motion.velocity[axis] > 0.0 && motion.velocity[axis].is_finite() {
                        motion.velocity[axis] = 0.0;
                    }
                } else if position < -GEOMETRY_LIMIT {
                    motion.position[axis] = -GEOMETRY_LIMIT;
                    if motion.velocity[axis] < 0.0 && motion.velocity[axis].is_finite() {
                        motion.velocity[axis] = 0.0;
                    }
                }
                motion.previous[axis] =
                    motion.previous[axis].clamp(-GEOMETRY_LIMIT, GEOMETRY_LIMIT);
            }
        }
    }
}
