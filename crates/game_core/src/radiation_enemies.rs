//! Radial Wastes' authored Dart variant, using live occupied cells and the
//! ordinary projectile, damage and reward paths.
use crate::{
    Actor, Brain, Bullet, COMBAT_RADIUS, ChampionFighter, Entity, Game, Kind, MAX_BULLETS, Motion,
    PixelBody, SAFE_RADIUS, Vec2, danger, pixel_assets, region_grid, regions,
};
use engine_core::FrameBuilder;
#[path = "mote_controller.rs"]
mod controller;
pub(super) use controller::WARNING_REACH;
use controller::{BIOME_KIND, BODY_SCALE, Birth, Controller, FAMILY, Input, Pose, Sanctuary};

#[derive(Clone, Copy)]
pub(super) struct Mote {
    pub(super) controller: Controller,
}

impl Game {
    /// Birth identity uses the actual spawn grid and immutable existing roll.
    /// Eligibility does not consume RNG or change ordinary family selection.
    pub(super) fn maybe_mote(&mut self, entity: Entity, biome: regions::Biome) -> bool {
        if self.arena.is_some() || self.boss_qa_scene || self.stress_mode {
            return false;
        }
        self.install_mote(entity, biome)
    }

    fn install_mote(&mut self, entity: Entity, biome: regions::Biome) -> bool {
        let birth = {
            let Ok(actor) = self.world.get::<&Actor>(entity) else {
                return false;
            };
            let Ok(brain) = self.world.get::<&Brain>(entity) else {
                return false;
            };
            let Ok(motion) = self.world.get::<&Motion>(entity) else {
                return false;
            };
            if actor.kind != Kind::Enemy(FAMILY)
                || brain.family != FAMILY
                || brain.boss
                || self.world.get::<&ChampionFighter>(entity).is_ok()
                || self.world.get::<&Mote>(entity).is_ok()
                || self
                    .world
                    .get::<&crate::biome_enemies::Riveter>(entity)
                    .is_ok()
                || !motion.position.is_finite()
                || motion.position.abs().max_element() > crate::WORLD_LIMIT
                || regions::at(self.profile.seed, motion.position) != biome
            {
                return false;
            }
            Birth {
                seed: self.profile.seed,
                grid: biome.grid,
                ticket: brain.phase.to_bits(),
                biome: biome.kind,
                family: brain.family,
                ordinary: !brain.boss,
            }
        };
        let Some(controller) = Controller::new(birth) else {
            return false;
        };
        let body = controller::concept_body();
        let radius = pixel_assets::radius(&body, BODY_SCALE);
        if self.world.insert_one(entity, body).is_err() {
            return false;
        }
        {
            let mut actor = self.world.get::<&mut Actor>(entity).unwrap();
            actor.scale = BODY_SCALE;
            actor.radius = radius;
        }
        self.world.insert_one(entity, Mote { controller }).is_ok()
    }

    pub(super) fn update_motes(&mut self, player: Vec2, dt: f32) {
        let refuge = self
            .safe_zone_at(player)
            .map(|(center, radius)| Sanctuary { center, radius });
        let player_radius = self.world.get::<&Actor>(self.player).unwrap().radius;
        let camera = Vec2::from_array(self.render_camera(1.0));
        let player_velocity = self.world.get::<&Motion>(self.player).unwrap().velocity;
        let predicted_player = (player + player_velocity * dt).clamp(
            Vec2::splat(-crate::WORLD_LIMIT),
            Vec2::splat(crate::WORLD_LIMIT),
        );
        let camera_next = camera + (predicted_player - camera) * (1.0 - (-5.0 * dt).exp());
        // Ordinary impacts/explosions can raise shake to5 later in this tick.
        // A larger already-active impulse also reserves its full displacement.
        let presentation_margin = if self.reduced_motion {
            0.0
        } else {
            self.shake.max(5.0)
        };
        let viewport = self.viewport;
        let seed = self.profile.seed;
        let cache_grid = self.site_grid;
        let mut slots = (if self.stress_mode { 300 } else { MAX_BULLETS })
            .saturating_sub(self.world.query::<&Bullet>().iter().count());
        let mut shots = Vec::new();
        let mut remove = Vec::new();
        for (entity, (motion, actor, brain, body, mote)) in self.world.query_mut::<(
            Entity,
            (&mut Motion, &Actor, &mut Brain, &PixelBody, &mut Mote),
        )>() {
            brain.age += dt;
            let grid = region_grid(motion.position);
            let distant =
                (grid[0] - cache_grid[0]).abs() > 1 || (grid[1] - cache_grid[1]).abs() > 1;
            let remote_sites = distant.then(|| regions::sites_near(seed, motion.position));
            let sites = remote_sites.as_deref().unwrap_or(&self.nearby_sites);
            let sanctuaries: Vec<_> = std::iter::once(Sanctuary {
                center: Vec2::ZERO,
                radius: SAFE_RADIUS,
            })
            .chain(sites.iter().map(|site| Sanctuary {
                center: Vec2::from_array(site.position),
                radius: site.radius,
            }))
            .collect();
            let input = Input {
                dt,
                pose: Pose {
                    position: motion.position,
                    velocity: motion.velocity,
                    heading: motion.heading,
                },
                player,
                player_radius,
                camera,
                camera_next,
                presentation_margin,
                viewport,
                radius: actor.radius,
                scale: actor.scale,
                contact_damage: actor.damage,
                tier: danger(player).max(1),
                sanctuaries: &sanctuaries,
                player_refuge: refuge,
                bullet_slots: slots,
            };
            match mote.controller.step(&input, body) {
                Ok(output) => {
                    motion.position = output.pose.position;
                    motion.velocity = output.pose.velocity;
                    motion.heading = output.pose.heading;
                    if output.retire {
                        remove.push(entity);
                    }
                    for shot in output.shots() {
                        // Reservation subtracts earlier entities' commands in this
                        // tick. Shared spawn_bullet remains the final hard cap.
                        slots = slots.saturating_sub(1);
                        shots.push(shot);
                    }
                }
                Err(_) => remove.push(entity), // Never leave an invalid frozen enemy.
            }
        }
        for shot in shots {
            self.spawn_bullet(shot.position, shot.velocity, shot.damage, false, 1);
        }
        for entity in remove {
            self.remove_enemy(entity, false);
        }
    }

    pub(super) fn draw_mote(
        &self,
        frame: &mut FrameBuilder,
        body: &PixelBody,
        motion: Motion,
        actor: Actor,
        mote: Mote,
    ) {
        controller::draw(
            frame,
            body,
            Pose {
                position: motion.position,
                velocity: motion.velocity,
                heading: motion.heading,
            },
            actor.scale,
            mote.controller,
        );
    }

    /// Explicit isolated combat QA; all ordinary rewards are suppressed by the
    /// existing scene flag, including incidental rocks. Reset clears this flag.
    pub(super) fn mote_diagnostic(&mut self, command: &str) -> bool {
        if command != "biome-mote" {
            return false;
        }
        self.reset_to_dock();
        self.boss_qa_scene = true;
        let mut candidates = Vec::new();
        for y in -4..=4 {
            for x in -4..=4 {
                for offset in [0.12, 0.5, 0.88] {
                    let p = Vec2::new(x as f32 + offset, y as f32 + offset) * regions::REGION_SIZE;
                    if regions::at(self.profile.seed, p).kind == BIOME_KIND
                        && p.length() > COMBAT_RADIUS + 300.0
                        && regions::protected(self.profile.seed, p).is_none()
                        && regions::protected(self.profile.seed, p - Vec2::Y * 100.0).is_none()
                    {
                        candidates.push(p);
                    }
                }
            }
        }
        candidates.sort_by(|a, b| a.length_squared().total_cmp(&b.length_squared()));
        if let Some(position) = candidates.first().copied() {
            self.move_player(position);
            self.update_chunks(position);
            let rocks: Vec<_> = self
                .world
                .query::<(Entity, (&Motion, &Actor))>()
                .iter()
                .filter_map(|(entity, (motion, actor))| {
                    (actor.kind == Kind::Asteroid && motion.position.distance(position) < 850.0)
                        .then_some(entity)
                })
                .collect();
            for rock in rocks {
                let _ = self.world.despawn(rock);
            }
            for _ in 0..128 {
                let entity = self.spawn_enemy(FAMILY, false, position - Vec2::Y * 100.0, 1);
                if self.install_mote(
                    entity,
                    regions::at(self.profile.seed, position - Vec2::Y * 100.0),
                ) {
                    break;
                }
                self.remove_enemy(entity, false);
            }
        }
        true
    }
}

#[cfg(test)]
#[path = "radiation_enemy_tests.rs"]
mod tests;
