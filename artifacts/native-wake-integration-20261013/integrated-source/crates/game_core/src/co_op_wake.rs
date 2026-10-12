//! Ordinary Ion Wake uses membership-local samples and the solo damage rules.
use super::*;

impl Game {
    pub(super) fn update_co_op_wakes(
        &mut self,
        origins: [Option<(Entity, Vec2)>; MAX_PARTICIPANTS],
        dt: f32,
    ) {
        // Slot order also defines attribution when overlapping trails kill.
        for (pilot_entity, origin) in origins.into_iter().flatten() {
            let Ok(motion) = self.world.get::<&Motion>(pilot_entity).map(|m| *m) else {
                continue;
            };
            let safe = self.safe_zone_at(motion.position).is_some();
            let (owner, rank, evolution, stamp) = {
                let Ok(mut pilot) = self.world.get::<&mut Pilot>(pilot_entity) else {
                    continue;
                };
                for (_, life) in &mut pilot.wake {
                    *life -= dt;
                }
                pilot.wake.retain(|(_, life)| *life > 0.0);
                if pilot.wake_rank == 0 || safe {
                    continue;
                }
                pilot.wake_timer -= dt;
                let evolution = pilot.wake_evolution;
                let stamp = (motion.velocity.length() > pilot.stats.speed * 0.5
                    && pilot.wake_timer <= 0.0)
                    .then(|| origin - forward(motion.heading) * 14.0);
                if let Some(point) = stamp {
                    pilot.wake_timer = if evolution >= 4 { 0.15 } else { 0.2 };
                    let (capacity, duration) = if evolution >= 4 {
                        (24, 3.4)
                    } else if evolution >= 2 {
                        (16, 2.6)
                    } else {
                        (12, 2.0)
                    };
                    if pilot.wake.len() >= capacity {
                        pilot.wake.remove(0);
                    }
                    pilot.wake.push((point, duration));
                }
                (
                    ShotOwner {
                        id: pilot.admission.id,
                        epoch: pilot.admission.epoch,
                    },
                    pilot.wake_rank,
                    evolution,
                    stamp,
                )
            };
            if let Some(point) = stamp {
                self.transient_light(point, 48.0, [0.23, 1.0, 0.62], 0.7, 0.3);
                self.event("ability-wake", origin, 0.45);
            }
            let reach = if evolution >= 3 { 28.0 } else { 20.0 };
            let targets =
                {
                    let pilot = self.world.get::<&Pilot>(pilot_entity).unwrap();
                    // At most 23 links. Query sanctuaries once per link, including
                    // remote neighborhoods outside the host's render-site cache.
                    let links: Vec<_> = if evolution >= 5 {
                        pilot
                            .wake
                            .windows(2)
                            .filter_map(|pair| {
                                let (a, b) = (pair[0].0, pair[1].0);
                                (a.distance_squared(b) <= 160.0 * 160.0
                                    && segment_circle(a, b, Vec2::ZERO, SAFE_RADIUS).is_none()
                                    && !regions::segment_protected(self.profile.seed, a, b))
                                .then_some((a, b))
                            })
                            .collect()
                    } else {
                        Vec::new()
                    };
                    self.world
                        .query::<(Entity, (&Motion, &Actor))>()
                        .iter()
                        .filter(|(_, (m, a))| {
                            matches!(a.kind, Kind::Enemy(_) | Kind::Boss(_))
                                && self.safe_zone_at(m.position).is_none()
                                && (pilot.wake.iter().any(|(point, _)| {
                                    m.position.distance(*point) < reach + a.radius
                                }) || links.iter().any(|&(from, to)| {
                                    segment_circle(from, to, m.position, reach + a.radius).is_some()
                                }))
                        })
                        .map(|(entity, (motion, actor))| (entity, actor.kind, motion.position))
                        .collect::<Vec<_>>()
                };
            let damage = 6.0 * (1.0 + 0.25 * rank.saturating_sub(1) as f32) * dt;
            let previous = self
                .co_op
                .as_mut()
                .map(|session| session.hit_owner.replace(owner));
            for (entity, kind, position) in targets {
                let multiplier = self.boss_damage_multiplier(entity, Vec2::splat(1000.0));
                let dead = if let Ok(mut actor) = self.world.get::<&mut Actor>(entity) {
                    actor.health -= damage * multiplier;
                    actor.health <= 0.0
                } else {
                    false
                };
                if dead {
                    self.kill_actor(entity, kind, position);
                }
            }
            if let Some(previous) = previous {
                self.co_op.as_mut().unwrap().hit_owner = previous;
            }
        }
    }
}
