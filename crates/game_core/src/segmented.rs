//! Occupied-cell attachment joints and bounded articulated Coil follow.
use super::*;

#[derive(Clone, Copy)]
pub(super) struct ChainHead {
    rear: Vec2,
}

/// Select actual occupied cells along the central spine, not bounding circles.
/// These are bind-pose anchors: damage can erase them without restoring pixels.
fn anchors(body: &PixelBody, scale: f32) -> (Vec2, Vec2) {
    let nearest_x = body
        .cells()
        .map(|(p, _)| p.x.abs())
        .fold(f32::INFINITY, f32::min);
    let mut front = Vec2::ZERO;
    let mut rear = Vec2::ZERO;
    let mut found = false;
    for (p, _) in body.cells().filter(|(p, _)| p.x.abs() <= nearest_x + 0.001) {
        if !found || p.y < front.y || (p.y == front.y && p.x < front.x) {
            front = p;
        }
        if !found || p.y > rear.y || (p.y == rear.y && p.x < rear.x) {
            rear = p;
        }
        found = true;
    }
    (front * scale, rear * scale)
}

fn rotated(offset: Vec2, heading: f32) -> Vec2 {
    let (s, c) = (heading + FRAC_PI_2).sin_cos();
    Vec2::new(c * offset.x - s * offset.y, s * offset.x + c * offset.y)
}

const MAX_BEND: f32 = 0.7;

fn wrap(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}

impl Game {
    pub(super) fn rebind_chain_head(&mut self, owner: Entity) {
        if self.world.get::<&ChainHead>(owner).is_err() {
            return;
        }
        let head = *self.world.get::<&Motion>(owner).unwrap();
        let scale = self.world.get::<&Actor>(owner).unwrap().scale;
        let rear = anchors(&self.world.get::<&PixelBody>(owner).unwrap(), scale).1;
        self.world.insert_one(owner, ChainHead { rear }).unwrap();
        let mut parent = head.position;
        let mut parent_rear = rear;
        let mut children: Vec<_> = self
            .world
            .query::<(Entity, &Segment)>()
            .iter()
            .filter(|(_, s)| s.owner == owner)
            .map(|(e, s)| (e, *s))
            .collect();
        children.sort_by_key(|(_, s)| s.index);
        for (entity, s) in children {
            let position =
                parent + rotated(parent_rear, head.heading) - rotated(s.front, head.heading);
            *self.world.get::<&mut Motion>(entity).unwrap() =
                Motion::new(position, Vec2::ZERO, head.heading);
            self.world.get::<&mut Segment>(entity).unwrap().attached = true;
            parent = position;
            parent_rear = s.rear;
        }
    }

    pub(super) fn spawn_chain(
        &mut self,
        owner: Entity,
        body: PixelBody,
        scale: f32,
        count: u8,
        health: f32,
    ) {
        let head = *self.world.get::<&Motion>(owner).unwrap();
        let head_scale = self.world.get::<&Actor>(owner).unwrap().scale;
        let head_rear = anchors(&self.world.get::<&PixelBody>(owner).unwrap(), head_scale).1;
        let (front, rear) = anchors(&body, scale);
        let radius = pixel_assets::radius(&body, scale);
        let _ = self.world.insert_one(owner, ChainHead { rear: head_rear });
        let mut parent = head.position;
        let mut parent_rear = head_rear;
        for index in 1..=count {
            let position =
                parent + rotated(parent_rear, head.heading) - rotated(front, head.heading);
            self.world.spawn((
                Motion::new(position, Vec2::ZERO, head.heading),
                Segment {
                    owner,
                    index,
                    health,
                    scale,
                    radius,
                    front,
                    rear,
                    attached: true,
                },
                body.clone(),
            ));
            parent = position;
            parent_rear = rear;
        }
    }

    pub(super) fn update_segments(&mut self, dt: f32) {
        let mut parents: HashMap<_, _> = self
            .world
            .query::<(Entity, (&Motion, &ChainHead))>()
            .iter()
            .map(|(e, (m, h))| (e, (m.position, m.heading, h.rear)))
            .collect();
        let mut segments: Vec<_> = self
            .world
            .query::<(Entity, (&Motion, &Segment))>()
            .iter()
            .map(|(e, (m, s))| (e, *m, *s))
            .collect();
        segments.sort_by_key(|(_, _, s)| s.index);
        let mut remove = Vec::new();
        for (entity, mut motion, segment) in segments {
            let Some(&(parent, parent_heading, parent_rear)) = parents.get(&segment.owner) else {
                remove.push(entity);
                continue;
            };
            let joint = parent + rotated(parent_rear, parent_heading);
            let delta = joint - motion.position;
            let target = if delta.length_squared() > 0.001 {
                delta.y.atan2(delta.x)
            } else {
                parent_heading
            };
            let target = parent_heading + wrap(target - parent_heading).clamp(-MAX_BEND, MAX_BEND);
            let heading = turn_toward(motion.heading, target, 6.0 * PI * dt);
            let mut heading =
                parent_heading + wrap(heading - parent_heading).clamp(-MAX_BEND, MAX_BEND);
            let arena = self.arena.filter(|a| a.opponent == segment.owner);
            let distant_sites = self
                .outside_site_cache(joint)
                .then(|| regions::sites_near(self.profile.seed, joint));
            let sites = distant_sites.as_deref().unwrap_or(&self.nearby_sites);
            let allowed = |angle| {
                let p = joint - rotated(segment.front, angle);
                let outside_home = p.length() >= SAFE_RADIUS + segment.radius + 6.0;
                let outside_sites = arena.is_some()
                    || sites.iter().all(|s| {
                        p.distance(Vec2::from_array(s.position)) >= s.radius + segment.radius + 6.0
                    });
                let inside_arena = arena.is_none_or(|a| {
                    p.distance(a.center) <= (a.radius - segment.radius - 4.0).max(1.0)
                });
                outside_home && outside_sites && inside_arena
            };
            // At a sanctuary or arena wall, bend the joint before separating it.
            if !allowed(heading) {
                let mut best = f32::INFINITY;
                for sample in 0..64 {
                    let angle = parent_heading + sample as f32 * TAU / 64.0;
                    let score = wrap(angle - heading).abs();
                    if score < best && allowed(angle) {
                        heading = angle;
                        best = score;
                    }
                }
            }
            let attached = allowed(heading);
            motion.heading = heading;
            motion.position = joint - rotated(segment.front, heading);
            if !attached && arena.is_none() {
                // Safety wins in impossible geometry (e.g. externally teleported
                // head inside protection). Extraction uses this actual detached pose.
                push_out_of_sanctuaries(&mut motion.position, segment.radius, sites);
            }
            motion.velocity = (motion.position - motion.previous) / dt;
            parents.insert(
                segment.owner,
                (motion.position, motion.heading, segment.rear),
            );
            *self.world.get::<&mut Motion>(entity).unwrap() = motion;
            self.world.get::<&mut Segment>(entity).unwrap().attached = attached;
        }
        for entity in remove {
            let _ = self.world.despawn(entity);
        }
    }

    /// Re-solve joints at presentation alpha, so independent angle interpolation
    /// cannot pull adjacent occupied cells apart between simulation ticks.
    pub(super) fn segment_render_poses(&self, alpha: f32) -> HashMap<Entity, (Vec2, f32)> {
        let mut parents: HashMap<_, _> = self
            .world
            .query::<(Entity, (&Motion, &ChainHead))>()
            .iter()
            .map(|(e, (m, h))| {
                (
                    e,
                    (
                        m.previous.lerp(m.position, alpha),
                        interpolate_heading(m.previous_heading, m.heading, alpha),
                        h.rear,
                    ),
                )
            })
            .collect();
        let mut segments: Vec<_> = self
            .world
            .query::<(Entity, (&Motion, &Segment))>()
            .iter()
            .map(|(e, (m, s))| (e, *m, *s))
            .collect();
        segments.sort_by_key(|(_, _, s)| s.index);
        let mut poses = HashMap::with_capacity(segments.len());
        for (entity, motion, segment) in segments {
            let Some(&(parent, parent_heading, parent_rear)) = parents.get(&segment.owner) else {
                continue;
            };
            let heading = interpolate_heading(motion.previous_heading, motion.heading, alpha);
            let position = if segment.attached {
                parent + rotated(parent_rear, parent_heading) - rotated(segment.front, heading)
            } else {
                motion.previous.lerp(motion.position, alpha)
            };
            parents.insert(segment.owner, (position, heading, segment.rear));
            poses.insert(entity, (position, heading));
        }
        poses
    }
}

#[cfg(test)]
mod tests;
