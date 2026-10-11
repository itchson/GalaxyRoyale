//! Gentle immutable belt orbits and swept, non-damaging sanctuary shields.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct Orbit {
    pub center: Vec2,
    pub radius: f32,
    pub spin: f32,
    pub speed: f32,
}
impl Orbit {
    pub fn at(seed: u32, position: Vec2, center: Vec2) -> Self {
        let h = hash3(
            seed,
            position.x.floor() as i32,
            position.y.floor() as i32,
            0x6f72_6269,
        );
        Self {
            center,
            radius: position.distance(center),
            spin: if h & 1 == 0 { -1.0 } else { 1.0 },
            speed: 6.0 + unit(mix(h)) * 6.0,
        }
    }
    pub fn tangent(self, position: Vec2) -> Vec2 {
        let radial = (position - self.center).normalize_or_zero();
        Vec2::new(-radial.y, radial.x) * self.spin
    }
}

const SLOP: f32 = 0.05;
const RESTITUTION: f32 = 0.7;

fn outward(delta: Vec2) -> Vec2 {
    if delta.length_squared() > 0.00001 {
        delta.normalize()
    } else {
        Vec2::X
    }
}

fn reflect(velocity: &mut Vec2, normal: Vec2) {
    *velocity -= normal * ((1.0 + RESTITUTION) * velocity.dot(normal).min(0.0));
}

// Existing segment_circle is a broadphase closest-point query. A physical
// shield needs the earliest surface contact, not that closest interior point.
fn first_contact(from: Vec2, to: Vec2, center: Vec2, radius: f32) -> Option<f32> {
    let d = to - from;
    let q = from - center;
    let a = f64::from(d.x).mul_add(f64::from(d.x), f64::from(d.y) * f64::from(d.y));
    if a <= 0.00000001 {
        return None;
    }
    let b = f64::from(q.x).mul_add(f64::from(d.x), f64::from(q.y) * f64::from(d.y));
    let c = f64::from(q.x).mul_add(f64::from(q.x), f64::from(q.y) * f64::from(q.y))
        - f64::from(radius) * f64::from(radius);
    let discriminant = b * b - a * c;
    if discriminant < 0.0 {
        return None;
    }
    let t = (-b - discriminant.sqrt()) / a;
    (0.0..=1.0).contains(&t).then_some(t as f32)
}

fn shield_circles(seed: u32, from: Vec2, to: Vec2, radius: f32) -> Vec<(Vec2, f32)> {
    let distance = from.distance(to);
    let steps = (distance / regions::REGION_SIZE).ceil().clamp(1.0, 512.0) as usize;
    let mut circles = vec![(Vec2::ZERO, SAFE_RADIUS + radius)];
    let mut grids = HashSet::new();
    for i in 0..steps {
        let point = from.lerp(to, (i as f32 + 0.5) / steps as f32);
        for site in regions::sites_near(seed, point) {
            if grids.insert(site.grid) {
                circles.push((Vec2::from_array(site.position), site.radius + radius));
            }
        }
    }
    circles
}

impl Game {
    pub(super) fn update_asteroid_orbits(&mut self, dt: f32) {
        for (_, (motion, actor, birth)) in self
            .world
            .query_mut::<(Entity, (&mut Motion, &Actor, Option<&AsteroidBirth>))>()
        {
            if actor.kind != Kind::Asteroid {
                continue;
            }
            if !motion.velocity.is_finite() {
                motion.velocity = Vec2::ZERO;
            }
            let Some(orbit) = birth.and_then(|b| b.orbit) else {
                continue;
            };
            let delta = motion.position - orbit.center;
            if !delta.is_finite() {
                continue;
            }
            let normal = outward(delta);
            let tangent = Vec2::new(-normal.y, normal.x) * orbit.spin;
            let error = delta.length() - orbit.radius;
            let acceleration = normal * (-error * 0.08 - motion.velocity.dot(normal) * 0.4)
                + tangent * ((orbit.speed - motion.velocity.dot(tangent)) * 0.25);
            motion.velocity += acceleration.clamp_length_max(12.0) * dt;
        }
    }

    pub(super) fn enforce_asteroid_shields(&mut self, swept: bool) {
        let rocks: Vec<_> = self
            .world
            .query::<(Entity, (&Motion, &Actor))>()
            .iter()
            .filter(|(_, (_, a))| a.kind == Kind::Asteroid)
            .map(|(e, (m, a))| (e, *m, a.radius))
            .collect();
        for (entity, mut motion, radius) in rocks {
            if !motion.previous.is_finite() {
                motion.previous = Vec2::new(SAFE_RADIUS + radius + SLOP, 0.0);
            }
            if !motion.position.is_finite() {
                motion.position = motion.previous;
            }
            if !motion.velocity.is_finite() {
                motion.velocity = Vec2::ZERO;
            }
            // Normal paths scan one neighborhood. Fast QA/impulse sweeps scan a
            // bounded line of neighborhoods, so endpoints cannot miss a refuge.
            if swept {
                let mut from = motion.previous;
                let mut to = motion.position;
                // Bound the solver to four contacts, refreshing the query for
                // each redirected corridor. Untested remaining travel is dropped.
                for contact_index in 0..4 {
                    let circles = shield_circles(self.profile.seed, from, to, radius);
                    for &(center, boundary) in &circles {
                        if from.distance_squared(center) < boundary * boundary {
                            let normal = outward(from - center);
                            let corrected = center + normal * (boundary + SLOP);
                            to += corrected - from;
                            from = corrected;
                            motion.previous = corrected;
                            reflect(&mut motion.velocity, normal);
                        }
                    }
                    let mut earliest: Option<(f32, Vec2, f32)> = None;
                    for &(center, boundary) in &circles {
                        if let Some(t) = first_contact(from, to, center, boundary) {
                            let normal = outward(from.lerp(to, t) - center);
                            if (to - from).dot(normal) < -0.00001
                                && earliest.is_none_or(|(old, _, _)| t < old)
                            {
                                earliest = Some((t, center, boundary));
                            }
                        }
                    }
                    let Some((t, center, boundary)) = earliest else {
                        break;
                    };
                    let contact = from.lerp(to, t);
                    let normal = outward(contact - center);
                    let mut remainder = (to - from) * (1.0 - t);
                    reflect(&mut remainder, normal);
                    reflect(&mut motion.velocity, normal);
                    from = center + normal * (boundary + SLOP);
                    motion.previous = from;
                    to = from + remainder;
                    if contact_index == 3 {
                        // Never advance an untested fifth corridor. Only extreme
                        // multi-reflection fixtures lose remaining travel this tick.
                        to = from;
                    }
                }
                motion.position = to;
            }
            // Pair separation/player impulses can penetrate after integration.
            // This second pass changes only penetration and inward velocity.
            let circles =
                shield_circles(self.profile.seed, motion.position, motion.position, radius);
            for &(center, boundary) in &circles {
                if motion.position.distance_squared(center) < (boundary + SLOP) * (boundary + SLOP)
                {
                    let normal = outward(motion.position - center);
                    motion.position = center + normal * (boundary + SLOP);
                    motion.previous = motion.position;
                    reflect(&mut motion.velocity, normal);
                }
            }
            *self.world.get::<&mut Motion>(entity).unwrap() = motion;
        }
    }
}

#[cfg(test)]
mod tests;
