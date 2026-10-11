//! Fifteen ordinary encounters, promotion accounting and signature evolution.
use crate::*;
use engine_core::PixelRigState;

#[derive(Clone, Debug, Serialize)]
pub struct FamilyBossHud {
    pub family: usize,
    pub defeats: [u32; 5],
    /// Includes an active fight: its credit is consumed only on actual defeat.
    pub queued: [u32; 5],
}

pub fn sanitize_progress(profile: &mut Profile) -> bool {
    let before = (profile.boss_generations, profile.boss_kills);
    for family in 0..3 {
        let defeats = &mut profile.boss_generations[family];
        if defeats.iter().all(|n| *n == 0) && profile.boss_kills[family] > 0 {
            defeats[0] = profile.boss_kills[family];
        }
        for generation in 1..5 {
            defeats[generation] = defeats[generation].min(defeats[generation - 1] / 2);
        }
        profile.boss_kills[family] = defeats
            .iter()
            .map(|n| *n as u64)
            .sum::<u64>()
            .min(u32::MAX as u64) as u32;
    }
    before != (profile.boss_generations, profile.boss_kills)
}
pub fn queues(profile: &Profile, family: usize) -> [u32; 5] {
    std::array::from_fn(|g| {
        if g == 0 {
            u32::from(profile.pending[family])
        } else {
            (profile.boss_generations[family][g - 1] / 2)
                .saturating_sub(profile.boss_generations[family][g])
        }
    })
}
pub fn evolution(profile: &Profile, family: usize) -> u8 {
    if family < 3 && ship_progression::equipped_rank(profile, (family + 14) as u8) > 0 {
        return 5;
    }
    profile.boss_generations[family]
        .iter()
        .rposition(|n| *n > 0)
        .map_or(1, |g| g as u8 + 1)
}
pub fn evolution_description(profile: &Profile, family: usize) -> &'static str {
    [
        [
            "Piercing fan",
            "Five-shot fan",
            "Fan and rear echoes",
            "Three-target pierce",
            "Echo every six shots",
        ],
        [
            "70-unit nova",
            "90-unit nova",
            "Nova clears hostile shots",
            "120-unit nova, faster cycle",
            "Nova reflects up to 16 shots",
        ],
        [
            "Two-second trail",
            "Longer ion trail",
            "Wider trail",
            "Dense extended trail",
            "Connected trail damage",
        ],
    ][family][evolution(profile, family) as usize - 1]
}
pub fn name(family: usize, generation: u8) -> &'static str {
    [
        [
            "Dart Matriarch",
            "Dart Harrier",
            "Dart Reaver",
            "Dart Tempest",
            "Dart Imperatrix",
        ],
        [
            "Bastion Prime",
            "Bastion Sentinel",
            "Bastion Gatekeeper",
            "Bastion Citadel",
            "Bastion Oblivion",
        ],
        [
            "Coil Sovereign",
            "Coil Sidewinder",
            "Coil Ouroboros",
            "Coil Leviathan",
            "Coil Worldeater",
        ],
    ][family][generation.saturating_sub(1).min(4) as usize]
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Phase {
    Tell,
    Attack,
    Recover,
}
#[derive(Clone, Copy)]
pub(super) struct BossEncounter {
    pub generation: u8,
    pub phase: Phase,
    pub phase_age: f32,
    pub shot_timer: f32,
    pub aim: Vec2,
    pub origin: Vec2,
    pub cycle: u32,
    pub diagnostic: bool,
}
impl BossEncounter {
    pub fn rig_state(self, time: f32, family: usize) -> PixelRigState {
        PixelRigState {
            time,
            charge: if self.phase == Phase::Tell {
                (self.phase_age / self.duration(family)).clamp(0.0, 1.0)
            } else {
                0.0
            },
            core_open: self.phase == Phase::Recover,
            hit_flash: 0.0,
        }
    }
    fn new(generation: u8, diagnostic: bool, position: Vec2) -> Self {
        Self {
            generation,
            phase: Phase::Tell,
            phase_age: 0.0,
            shot_timer: 0.0,
            aim: -Vec2::Y,
            origin: position,
            cycle: 0,
            diagnostic,
        }
    }
    fn duration(self, family: usize) -> f32 {
        match self.phase {
            Phase::Tell => [0.9, 1.1, 1.0][family] + 0.08 * self.generation as f32,
            Phase::Attack => match (family, self.generation) {
                (0, 2) => 0.75,
                (0, 3) => 1.7,
                (0, 4) => 2.7,
                (0, 5) => 1.5,
                (1, 3) => 2.8,
                (1, 4) => 3.0,
                (1, 5) => 2.5,
                (2, 2) => 1.5,
                (2, 3) => 3.2,
                (2, 4) => 2.5,
                (2, 5) => 2.9,
                _ => 2.1,
            },
            Phase::Recover => 1.25 + 0.13 * self.generation as f32,
        }
    }
    pub fn label(self, family: usize) -> &'static str {
        match self.phase {
            Phase::Tell => "Attack charging",
            Phase::Recover => "Core exposed",
            Phase::Attack => match (family, self.generation) {
                (0, 1) => "Radial weave",
                (0, 2) => "Locked dash",
                (0, 3) => "Scissor pass",
                (0, 4) => "Gap spiral",
                (0, _) => "Royal cross",
                (1, 1) => "Aimed fan",
                (1, 2) => "Siege broadside",
                (1, 3) => "Pincer gates",
                (1, 4) => "Quadrant wall",
                (1, _) => "Rotating rails",
                (2, 1) => "Serpent pursuit",
                (2, 2) => "Sidewinder rush",
                (2, 3) => "Ouroboros loop",
                (2, 4) => "Tail barrage",
                (2, _) => "Vortex lunge",
                _ => "Boss attack",
            },
        }
    }
}

impl Game {
    pub(super) fn next_boss(&self) -> Option<(usize, u8)> {
        for generation in (0..5).rev() {
            for family in 0..3 {
                if queues(&self.profile, family)[generation] > 0 {
                    return Some((family, generation as u8 + 1));
                }
            }
        }
        None
    }
    pub(super) fn configure_boss(
        &mut self,
        entity: Entity,
        family: usize,
        generation: u8,
        diagnostic: bool,
        tier: u32,
    ) {
        let art = boss_assets::art(family, generation);
        let scale = [1.0, 1.1, 1.0][family] + 0.06 * generation.saturating_sub(1) as f32;
        let health = [270.0, 432.0, 540.0][family]
            * [1.0, 1.3, 1.65, 2.0, 2.4][generation as usize - 1]
            * (1.0 + tier.saturating_sub(1) as f32 * 0.2);
        let position = self.world.get::<&Motion>(entity).unwrap().position;
        if let Ok(mut actor) = self.world.get::<&mut Actor>(entity) {
            actor.scale = scale;
            actor.radius = pixel_assets::radius(&art.body, scale);
            actor.health = health;
            actor.max_health = health;
            actor.damage = 20.0
                * (1.0 + 0.15 * generation.saturating_sub(1) as f32)
                * (1.0 + tier.saturating_sub(1) as f32 * 0.12);
        }
        let _ = self.world.insert_one(entity, art.body.clone());
        let _ = self
            .world
            .insert_one(entity, BossEncounter::new(generation, diagnostic, position));
        if family == 2 {
            let direction = initial_chain_direction(position, self.position());
            let heading = (-direction).y.atan2((-direction).x);
            if let Ok(mut motion) = self.world.get::<&mut Motion>(entity) {
                motion.heading = heading;
                motion.previous_heading = heading;
            }
            if let Ok(mut encounter) = self.world.get::<&mut BossEncounter>(entity) {
                encounter.aim = -direction;
            }
            let children: Vec<_> = self
                .world
                .query::<(Entity, &Segment)>()
                .iter()
                .filter(|(_, s)| s.owner == entity)
                .map(|(e, _)| e)
                .collect();
            for child in children {
                let _ = self.world.despawn(child);
            }
            let body = boss_assets::segment(generation);
            let segment_scale = 1.25;
            self.spawn_chain(
                entity,
                body,
                segment_scale,
                9 + 2 * generation,
                42.0 + 8.0 * generation.saturating_sub(1) as f32,
            );
        }
    }
    pub(super) fn diagnostic_boss_active(&self) -> bool {
        self.boss_qa_scene
    }
    pub(super) fn boss_damage_multiplier(&self, entity: Entity, local: Vec2) -> f32 {
        let Ok(boss) = self.world.get::<&BossEncounter>(entity) else {
            return 1.0;
        };
        let family = self.world.get::<&Brain>(entity).unwrap().family;
        let art = boss_assets::art(family, boss.generation);
        if boss.phase != Phase::Recover {
            0.55
        } else if local.distance(art.core_position) <= art.core_radius {
            1.65
        } else {
            1.0
        }
    }
    pub(super) fn update_bosses(&mut self, player: Vec2, dt: f32) {
        let sanctuary = self.safe_zone_at(player);
        let player_velocity = self.world.get::<&Motion>(self.player).unwrap().velocity;
        let range = self.viewport.length().max(1050.0);
        let mut shots = Vec::new();
        let mut tails = Vec::new();
        let mut remove = Vec::new();
        for (entity, (motion, actor, brain, boss)) in self.world.query_mut::<(
            Entity,
            (&mut Motion, &Actor, &mut Brain, &mut BossEncounter),
        )>() {
            if sanctuary.is_some() || motion.position.distance(player) > range {
                remove.push(entity);
                continue;
            }
            brain.age += dt;
            boss.phase_age += dt;
            boss.shot_timer -= dt;
            let delta = player - motion.position;
            let towards = delta.normalize_or_zero();
            let side = Vec2::new(-towards.y, towards.x);
            if boss.phase == Phase::Tell && boss.phase_age < boss.duration(brain.family) * 0.75 {
                boss.aim = (player + player_velocity * 0.35 - motion.position).normalize_or_zero();
                boss.origin = player;
            }
            if boss.phase_age >= boss.duration(brain.family) {
                boss.phase_age -= boss.duration(brain.family);
                boss.phase = match boss.phase {
                    Phase::Tell => Phase::Attack,
                    Phase::Attack => Phase::Recover,
                    Phase::Recover => Phase::Tell,
                };
                boss.shot_timer = 0.0;
                if boss.phase == Phase::Tell {
                    boss.cycle = boss.cycle.wrapping_add(1);
                }
            }
            let family = brain.family;
            let generation = boss.generation;
            let age = brain.age;
            let desired = match boss.phase {
                Phase::Tell => {
                    side * (age * 1.8).sin() * 22.0
                        + towards * ((delta.length() - 210.0) * 0.2).clamp(-24.0, 24.0)
                }
                Phase::Recover => -towards * 42.0 + side * (age * 2.0).sin() * 34.0,
                Phase::Attack => match (family, generation) {
                    (0, 1) => {
                        towards * (if delta.length() > 160.0 { 80.0 } else { -35.0 })
                            + side * (age * 2.2).sin() * 92.0
                    }
                    (0, 2) => boss.aim * 225.0,
                    (0, 3) => {
                        Vec2::new(-boss.aim.y, boss.aim.x)
                            * (if boss.cycle.is_multiple_of(2) {
                                190.0
                            } else {
                                -190.0
                            })
                            + boss.aim * 65.0
                    }
                    (0, 4) => {
                        side * 165.0 + towards * ((delta.length() - 210.0) * 0.6).clamp(-60.0, 60.0)
                    }
                    (0, _) => {
                        boss.aim * (if boss.phase_age < 0.55 { 240.0 } else { -90.0 }) + side * 70.0
                    }
                    (1, 1) => {
                        towards * ((delta.length() - 190.0) * 1.5).clamp(-60.0, 60.0) + side * 45.0
                    }
                    (1, 2) => {
                        side * (if boss.cycle.is_multiple_of(2) {
                            35.0
                        } else {
                            -35.0
                        })
                    }
                    (1, 3) => -towards * 32.0 + side * (boss.phase_age * 3.0).sin() * 75.0,
                    (1, 4) => Vec2::ZERO,
                    (1, _) => -towards * 22.0,
                    (2, 1) => towards * 100.0 + side * (age * 2.2).sin() * 85.0,
                    (2, 2) => {
                        boss.aim * 165.0
                            + Vec2::new(-boss.aim.y, boss.aim.x)
                                * (boss.phase_age * 6.0).sin()
                                * 125.0
                    }
                    (2, 3) => {
                        let radial = (motion.position - boss.origin).normalize_or_zero();
                        Vec2::new(-radial.y, radial.x) * 160.0
                            + radial
                                * ((145.0 - motion.position.distance(boss.origin)) * 1.6)
                                    .clamp(-75.0, 75.0)
                    }
                    (2, 4) => towards * 65.0 + side * (age * 1.5).sin() * 175.0,
                    (2, _) => {
                        if boss.cycle.is_multiple_of(2) {
                            side * 185.0 + towards * 28.0
                        } else {
                            boss.aim * 210.0 + side * (boss.phase_age * 7.0).sin() * 55.0
                        }
                    }
                    _ => Vec2::ZERO,
                },
            };
            motion.velocity += (desired - motion.velocity) * (1.0 - (-5.0 * dt).exp());
            let facing = if boss.phase == Phase::Recover {
                towards
            } else {
                boss.aim
            };
            motion.heading = turn_toward(motion.heading, facing.y.atan2(facing.x), 3.0 * PI * dt);
            if boss.phase != Phase::Attack || boss.shot_timer > 0.0 || delta.length() > 650.0 {
                continue;
            }
            let locked = boss.aim.y.atan2(boss.aim.x);
            let aimed = towards.y.atan2(towards.x);
            let damage = actor.damage * 0.62;
            let mut emit = |angle: f32, speed: f32| {
                let dir = forward(angle);
                shots.push((
                    motion.position + dir * (actor.radius + 3.0),
                    dir * speed,
                    damage,
                ));
            };
            let delay = match (family, generation) {
                (0, 1) => {
                    for n in 0..8 {
                        emit(age * 0.5 + n as f32 * TAU / 8.0, 95.0);
                    }
                    1.05
                }
                (0, 2) => {
                    for n in -1..=1 {
                        emit(locked + n as f32 * 0.10, 210.0);
                    }
                    0.26
                }
                (0, 3) => {
                    for offset in [-0.42, -0.22, 0.22, 0.42] {
                        emit(aimed + offset, 180.0);
                    }
                    0.38
                }
                (0, 4) => {
                    for n in 2..14 {
                        emit(locked + age * 0.28 + n as f32 * TAU / 14.0, 120.0);
                    }
                    0.68
                }
                (0, _) => {
                    for n in [-3, -2, -1, 1, 2, 3] {
                        emit(locked + n as f32 * 0.16, 230.0);
                    }
                    emit(locked + PI - 0.22, 145.0);
                    emit(locked + PI + 0.22, 145.0);
                    0.42
                }
                (1, 1) => {
                    for n in -2..=2 {
                        emit(aimed + n as f32 * 0.18, 145.0);
                    }
                    1.0
                }
                (1, 2) => {
                    for sign in [-1.0, 1.0] {
                        for n in -1..=1 {
                            emit(locked + sign * FRAC_PI_2 + n as f32 * 0.16, 170.0);
                        }
                    }
                    0.65
                }
                (1, 3) => {
                    let sign = if ((boss.phase_age / 0.5).floor() as u32).is_multiple_of(2) {
                        1.0
                    } else {
                        -1.0
                    };
                    for n in 0..5 {
                        emit(
                            locked + sign * (0.25 + n as f32 * 0.13),
                            130.0 + 25.0 * n as f32,
                        );
                    }
                    0.5
                }
                (1, 4) => {
                    let gap = (boss.cycle % 4) as usize;
                    for n in 0..20 {
                        if n / 5 != gap {
                            emit(n as f32 * TAU / 20.0 + age * 0.06, 115.0);
                        }
                    }
                    0.85
                }
                (1, _) => {
                    for n in 0..4 {
                        emit(locked + boss.phase_age * 0.75 + n as f32 * FRAC_PI_2, 280.0);
                    }
                    0.18
                }
                (2, 1) => {
                    for offset in [-0.38, 0.0, 0.38] {
                        emit(aimed + offset, 110.0);
                    }
                    1.05
                }
                (2, 2) => {
                    for offset in [-0.65, -0.2, 0.2, 0.65] {
                        emit(locked + offset, 165.0);
                    }
                    0.55
                }
                (2, 3) => {
                    for offset in [-0.2, 0.0, 0.2] {
                        emit(aimed + offset, 125.0);
                    }
                    0.48
                }
                (2, 4) => {
                    emit(aimed, 170.0);
                    tails.push((entity, 160.0, damage, false));
                    0.78
                }
                (2, _) => {
                    for n in 0..9 {
                        emit(locked + age * 0.8 + n as f32 * TAU / 9.0, 140.0);
                    }
                    tails.push((entity, 190.0, damage, true));
                    0.68
                }
                _ => 1.0,
            };
            boss.shot_timer = delay
                * if actor.health < actor.max_health * 0.45 {
                    0.85
                } else {
                    1.0
                };
        }
        for (owner, speed, damage, alternating) in tails {
            for (motion, segment) in self.world.query::<(&Motion, &Segment)>().iter() {
                if segment.owner == owner
                    && segment
                        .index
                        .is_multiple_of(if alternating { 4 } else { 3 })
                {
                    let aim = (player - motion.position).normalize_or_zero();
                    let angle = aim.y.atan2(aim.x);
                    for offset in if alternating {
                        [-0.3, 0.3]
                    } else {
                        [-0.15, 0.15]
                    } {
                        shots.push((motion.position, forward(angle + offset) * speed, damage));
                    }
                }
            }
        }
        for (position, velocity, damage) in shots {
            self.spawn_bullet(position, velocity, damage, false, 1);
        }
        for entity in remove {
            self.remove_enemy(entity, false);
        }
    }
    pub(super) fn draw_boss(
        &self,
        frame: &mut FrameBuilder,
        body: &PixelBody,
        motion: Motion,
        actor: Actor,
        boss: BossEncounter,
        family: usize,
    ) {
        let charge = if boss.phase == Phase::Tell {
            (boss.phase_age / boss.duration(family)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        boss_assets::art(family, boss.generation).rig.draw(
            frame,
            body,
            actor_transform(motion, actor),
            boss.rig_state(self.cosmetic_time(), family),
        );
        if boss.phase == Phase::Tell {
            let color = with_alpha(currency_color(family), 110 + (charge * 100.0) as u8);
            for n in 0_usize..12 {
                let position = motion.position + boss.aim * (actor.radius + 8.0 + n as f32 * 9.0);
                if self.visible(position, 3.0) {
                    frame.push_pixel(position, if n.is_multiple_of(3) { 2.0 } else { 1.0 }, color);
                }
            }
        }
    }
    pub(super) fn boss_diagnostic(&mut self, command: &str) -> bool {
        let Some(value) = command.strip_prefix("boss-") else {
            return false;
        };
        let mut tokens = value.split('-');
        let family = match tokens.next() {
            Some("dart") => 0,
            Some("bastion") => 1,
            Some("coil") => 2,
            _ => return false,
        };
        let generation = match tokens.next() {
            Some(v) => match v.parse::<u8>() {
                Ok(g) if (1..=5).contains(&g) => g,
                _ => return false,
            },
            None => 1,
        };
        if tokens.next().is_some() {
            return false;
        }
        self.reset_to_dock();
        self.boss_qa_scene = true;
        self.move_player(Vec2::new(1800.0, 0.0));
        self.update_chunks(self.position());
        let rocks: Vec<_> = self
            .world
            .query::<(Entity, (&Motion, &Actor))>()
            .iter()
            .filter(|(_, (m, a))| {
                a.kind == Kind::Asteroid && m.position.distance(self.position()) < 850.0
            })
            .map(|(e, _)| e)
            .collect();
        for rock in rocks {
            let _ = self.world.despawn(rock);
        }
        let entity = self.spawn_enemy(family, true, self.position() + Vec2::new(0.0, -210.0), 1);
        self.configure_boss(entity, family, generation, true, 1);
        self.active_boss = Some(entity);
        self.event("boss", self.position(), 1.0);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn spawn_ready(game: &mut Game) -> Entity {
        game.move_player(Vec2::new(1800.0, 0.0));
        game.spawn_timer = 100.0;
        game.boss_timer = 0.0;
        for _ in 0..8 {
            game.update_spawns(game.position(), 1.0 / 60.0);
            if let Some(entity) = game.active_boss {
                return entity;
            }
        }
        panic!("queued boss must spawn in combat space");
    }
    fn defeat(game: &mut Game, entity: Entity) {
        game.damage_actor(entity, 100_000.0, Vec2::ZERO, Vec2::ZERO);
        assert!(!game.world.contains(entity));
    }
    fn sovereign() -> Profile {
        let mut profile = Profile {
            schema_version: 3,
            ..Profile::default()
        }; // declared pre-slot reward migration fixture
        profile.campaign.patches = [true; 8];
        profile.campaign.blueprints = [true; 8];
        profile.campaign.royale_results = [1; 8];
        profile.campaign.victory_claimed = true;
        profile.campaign.selected_ship = 9;
        profile.campaign.ship_xp[9] = 1320;
        profile
    }
    #[test]
    fn legacy_and_adversarial_generation_counts_sanitize_without_inventing_base_kills() {
        let legacy:Profile=serde_json::from_str(r#"{"schema_version":3,"kills":[101,0,0],"boss_kills":[1,7,0],"pending":[true,false,false]}"#).unwrap();
        let game = Game::new(legacy);
        assert_eq!(
            game.profile.boss_generations,
            [[1, 0, 0, 0, 0], [7, 0, 0, 0, 0], [0; 5]]
        );
        assert_eq!(game.kill_credit, [53, 0, 0]);
        assert_eq!(game.profile.kills, [101, 0, 0]);
        let profile = Profile {
            boss_generations: [
                [3, 9, 9, 9, 9],
                [u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX],
                [0; 5],
            ],
            boss_kills: [99, 1, 0],
            ..Profile::default()
        };
        let game = Game::new(profile);
        assert_eq!(game.profile.boss_generations[0], [3, 1, 0, 0, 0]);
        assert_eq!(game.profile.boss_kills[0], 4);
        assert_eq!(game.profile.boss_kills[1], u32::MAX);
        assert_eq!(
            game.profile.boss_generations[1],
            [
                u32::MAX,
                u32::MAX / 2,
                u32::MAX / 4,
                u32::MAX / 8,
                u32::MAX / 16
            ]
        );
    }
    #[test]
    fn complete_binary_promotion_tree_uses_384_base_kills_and_31_real_boss_defeats() {
        let mut game = Game::new(Profile::default());
        for _ in 0..16 {
            for _ in 0..24 {
                let enemy = game.spawn_enemy(0, false, Vec2::new(1800.0, 90.0), 1);
                defeat(&mut game, enemy);
            }
            while let Some((family, generation)) = game.next_boss() {
                assert_eq!(family, 0);
                let boss = spawn_ready(&mut game);
                assert_eq!(
                    game.world.get::<&BossEncounter>(boss).unwrap().generation,
                    generation
                );
                assert_eq!(
                    game.world.get::<&PixelBody>(boss).unwrap().width,
                    64 + 8 * (generation as usize - 1)
                );
                defeat(&mut game, boss);
            }
        }
        assert_eq!(game.profile.kills, [384, 0, 0]);
        assert_eq!(game.profile.boss_generations[0], [16, 8, 4, 2, 1]);
        assert_eq!(game.profile.boss_kills, [31, 0, 0]);
        assert_eq!(queues(&game.profile, 0), [0; 5]);
        assert_eq!(game.kill_credit, [0; 3]);
    }
    #[test]
    fn highest_queue_retries_across_retreat_death_reload_and_consumes_exactly_once() {
        let profile = Profile {
            kills: [453, 48, 24],
            boss_kills: [30, 2, 1],
            boss_generations: [[16, 8, 4, 2, 0], [2, 0, 0, 0, 0], [1, 0, 0, 0, 0]],
            pending: [true, true, true],
            currency: [7, 8, 9],
            ..Profile::default()
        };
        let mut game = Game::new(profile);
        assert_eq!(game.kill_credit[0], 45);
        let boss = spawn_ready(&mut game);
        assert_eq!(
            game.world.get::<&BossEncounter>(boss).unwrap().generation,
            5
        );
        game.update_spawns(game.position(), 0.05);
        assert_eq!(game.world.query::<&BossEncounter>().iter().count(), 1);
        let before = game.profile.boss_generations;
        game.move_player(Vec2::ZERO);
        game.update_bosses(game.position(), 1.0 / 60.0);
        assert!(game.active_boss.is_none());
        assert_eq!(game.profile.boss_generations, before);
        let mut game = Game::new(
            serde_json::from_str(&serde_json::to_string(game.profile()).unwrap()).unwrap(),
        );
        let boss = spawn_ready(&mut game);
        game.world.get::<&mut Actor>(game.player).unwrap().health = 0.0;
        game.tick(&InputFrame::default(), 1.0 / 60.0);
        assert_eq!(game.position(), Vec2::ZERO);
        assert_eq!(queues(&game.profile, 0), [1, 0, 0, 0, 1]);
        assert_eq!(game.kill_credit[0], 45);
        assert!(!game.world.contains(boss));
        let boss = spawn_ready(&mut game);
        let position = game.world.get::<&Motion>(boss).unwrap().position;
        defeat(&mut game, boss);
        let expected = [[16, 8, 4, 2, 1], [2, 0, 0, 0, 0], [1, 0, 0, 0, 0]];
        assert_eq!(game.profile.boss_generations, expected);
        assert_eq!(game.profile.boss_kills, [31, 2, 1]);
        assert_eq!(queues(&game.profile, 0), [1, 0, 0, 0, 0]);
        assert_eq!(queues(&game.profile, 1), [1, 1, 0, 0, 0]);
        assert_eq!(queues(&game.profile, 2), [1, 0, 0, 0, 0]);
        assert_eq!(game.kill_credit[0], 45);
        game.kill_actor(boss, Kind::Boss(0), position);
        assert_eq!(game.profile.boss_generations, expected);
    }
    #[test]
    fn fifteen_diagnostic_fights_have_distinct_live_patterns_and_no_incidental_rewards() {
        let mut signatures = HashSet::new();
        for family in 0..3 {
            for generation in 1..=5 {
                let mut game = Game::new(Profile::default());
                game.diagnostic(&format!(
                    "boss-{}-{generation}",
                    ["dart", "bastion", "coil"][family]
                ));
                let before = serde_json::to_string(game.profile()).unwrap();
                let boss = game.active_boss.unwrap();
                game.invulnerability = 100.0;
                let mut phases = HashSet::new();
                let mut signature = Vec::new();
                let mut maximum_shots = 0;
                for tick in 0..540 {
                    game.tick(&InputFrame::default(), 1.0 / 60.0);
                    let hud = game.hud().boss.unwrap();
                    phases.insert(hud.phase.clone());
                    assert!(hud.phase.is_ascii() && hud.phase.len() <= 22);
                    let shots = game.world.query::<&Bullet>().iter().count();
                    maximum_shots = maximum_shots.max(shots);
                    if tick % 60 == 0 {
                        let velocity = game.world.get::<&Motion>(boss).unwrap().velocity;
                        signature.push((
                            (velocity.x * 10.0) as i32,
                            (velocity.y * 10.0) as i32,
                            shots,
                        ));
                    }
                }
                assert!(maximum_shots > 0 && maximum_shots <= MAX_BULLETS);
                assert!(phases.contains("Attack charging") && phases.contains("Core exposed"));
                assert!(
                    signatures.insert(signature),
                    "duplicate family {family} generation {generation}"
                );
                let rock = game.world.spawn((
                    Motion::new(game.position() + Vec2::X * 1000.0, Vec2::ZERO, 0.0),
                    Actor {
                        kind: Kind::Asteroid,
                        health: 10.0,
                        max_health: 10.0,
                        radius: 20.0,
                        scale: 1.0,
                        damage: 8.0,
                    },
                    asteroid_body(17),
                ));
                defeat(&mut game, rock);
                defeat(&mut game, boss);
                // Lingering piercing bullets still cannot earn asteroid rewards after victory.
                let rock = game.world.spawn((
                    Motion::new(game.position(), Vec2::ZERO, 0.0),
                    Actor {
                        kind: Kind::Asteroid,
                        health: 10.0,
                        max_health: 10.0,
                        radius: 20.0,
                        scale: 1.0,
                        damage: 8.0,
                    },
                    asteroid_body(17),
                ));
                defeat(&mut game, rock);
                assert_eq!(serde_json::to_string(game.profile()).unwrap(), before);
                assert_eq!(game.world.query::<&Loot>().iter().count(), 0);
            }
        }
    }
    #[test]
    fn localized_core_phase_modifies_real_damage_and_rigs_keep_destroyed_holes() {
        let mut outlines = HashSet::new();
        for family in 0..3 {
            for generation in 1..=5 {
                let art = boss_assets::art(family, generation);
                assert!(
                    art.body
                        .hit_segment(art.core_position, art.core_position)
                        .is_some()
                );
                let outline: Vec<_> = art
                    .body
                    .cells()
                    .map(|(p, _)| ((p.x * 2.0) as i32, (p.y * 2.0) as i32))
                    .collect();
                assert!(outlines.insert((art.body.width, outline)));
                let mut body = art.body.clone();
                let original = body.occupied_count();
                assert!(!body.damage(art.core_position, 2.0).is_empty());
                let mut frame = FrameBuilder::new();
                art.rig.draw(
                    &mut frame,
                    &body,
                    Transform::default(),
                    PixelRigState {
                        time: 1.0,
                        charge: 1.0,
                        core_open: false,
                        hit_flash: 0.0,
                    },
                );
                assert_eq!(frame.frame().pixel_count, body.occupied_count());
                let closed: Vec<_> = frame
                    .frame()
                    .batches
                    .iter()
                    .flat_map(|b| b.pixels.iter().map(|p| p.color))
                    .collect();
                frame.clear();
                art.rig.draw(
                    &mut frame,
                    &body,
                    Transform::default(),
                    PixelRigState {
                        time: 1.0,
                        charge: 0.0,
                        core_open: true,
                        hit_flash: 0.0,
                    },
                );
                let open: Vec<_> = frame
                    .frame()
                    .batches
                    .iter()
                    .flat_map(|b| b.pixels.iter().map(|p| p.color))
                    .collect();
                assert_eq!(frame.frame().pixel_count, body.occupied_count());
                assert!(body.occupied_count() < original);
                assert_ne!(open, closed);
            }
        }
        let mut game = Game::new(Profile::default());
        game.diagnostic("boss-bastion-5");
        let entity = game.active_boss.unwrap();
        let core = boss_assets::art(1, 5).core_position;
        let before = game.world.get::<&Actor>(entity).unwrap().health;
        game.damage_actor(entity, 10.0, core, Vec2::ZERO);
        let closed = game.world.get::<&Actor>(entity).unwrap().health;
        assert!((before - closed - 5.5).abs() < 0.001);
        game.world.get::<&mut BossEncounter>(entity).unwrap().phase = Phase::Recover;
        game.damage_actor(entity, 10.0, core, Vec2::ZERO);
        assert!((closed - game.world.get::<&Actor>(entity).unwrap().health - 16.5).abs() < 0.001);
        assert!(
            !game
                .world
                .get::<&PixelBody>(entity)
                .unwrap()
                .is_occupied(48, 48)
        );
    }
    #[test]
    fn sovereign_intrinsic_evolution_preserves_global_ranks_and_changes_real_echo_behavior() {
        let mut game = Game::new(sovereign());
        assert_eq!(
            (0..3)
                .map(|f| evolution(&game.profile, f))
                .collect::<Vec<_>>(),
            vec![5; 3]
        );
        assert_eq!(game.profile.boss_generations, [[0; 5]; 3]);
        game.move_player(Vec2::new(1800.0, 0.0));
        let stats = Stats::from_profile(&game.profile);
        for _ in 0..6 {
            game.fire(stats);
        }
        let piercing: Vec<_> = game
            .world
            .query::<(&Motion, &Bullet)>()
            .iter()
            .filter(|(_, b)| b.remaining_hits == 3)
            .map(|(m, b)| (m.velocity, b.damage))
            .collect();
        assert_eq!(piercing.len(), 7);
        assert!(piercing.iter().any(|(v, _)| v.y > 0.0));
        assert_eq!(game.profile.ranks, [0; 9]);
        assert_eq!(game.profile.unlocked, [false; 3]);
        let mut profile = Profile::default();
        profile.unlocked[0] = true;
        profile.schema_version = 3;
        profile.ranks[6] = 5;
        profile.boss_generations[0] = [16, 8, 4, 2, 1];
        profile.boss_kills[0] = 31;
        let mut game = Game::new(profile);
        game.move_player(Vec2::new(1800.0, 0.0));
        game.shot_count = 5;
        game.fire(Stats::from_profile(&game.profile));
        let damages: Vec<_> = game
            .world
            .query::<&Bullet>()
            .iter()
            .filter(|b| b.remaining_hits == 3)
            .map(|b| b.damage)
            .collect();
        assert_eq!(damages.len(), 7);
        assert!(
            damages.iter().all(|d| (*d - 6.0).abs() < 0.001),
            "level1 caps rank damage despite evolution5"
        );
    }
    #[test]
    fn nova_reflection_and_connected_wake_are_bounded_and_respect_sanctuaries() {
        let mut game = Game::new(sovereign());
        game.move_player(Vec2::new(1800.0, 0.0));
        game.invulnerability = 0.0;
        for _ in 0..24 {
            game.spawn_bullet(
                game.position() + Vec2::X * 60.0,
                -Vec2::X * 150.0,
                9.0,
                false,
                1,
            );
        }
        game.hurt_player(1.0, None);
        let bullets: Vec<_> = game
            .world
            .query::<(&Motion, &Bullet)>()
            .iter()
            .map(|(m, b)| (*m, *b))
            .collect();
        assert_eq!(bullets.len(), 16);
        assert!(
            bullets
                .iter()
                .all(|(m, b)| b.player && b.remaining_hits == 2 && m.velocity.x > 0.0)
        );
        assert!((game.repulsor_timer - 8.5).abs() < 0.001);
        game.wake = vec![
            (game.position() - Vec2::X * 80.0, 3.0),
            (game.position() + Vec2::X * 80.0, 3.0),
        ];
        let enemy = game.spawn_enemy(0, false, game.position(), 1);
        let health = game.world.get::<&Actor>(enemy).unwrap().health;
        game.update_wake(game.position(), 0.1);
        assert!(
            game.world.get::<&Actor>(enemy).unwrap().health < health,
            "connected segment hits between separated wake samples"
        );
        game.world.get::<&mut Motion>(game.player).unwrap().velocity = Vec2::X * 200.0;
        for _ in 0..100 {
            game.update_wake(game.position(), 0.15);
        }
        assert!(game.wake.len() <= 24);
        assert!(game.wake.iter().all(|(_, life)| *life <= 3.4));
        game.move_player(Vec2::ZERO);
        game.invulnerability = 0.0;
        game.repulsor_timer = 0.0;
        let hull = game.hud().hull;
        game.hurt_player(50.0, None);
        game.update_wake(game.position(), 0.2);
        assert_eq!(game.hud().hull, hull);
        assert_eq!(game.repulsor_timer, 0.0);
        game.tick(
            &InputFrame {
                fire: true,
                ..Default::default()
            },
            1.0 / 60.0,
        );
        assert_eq!(game.world.query::<&Bullet>().iter().count(), 0);
    }
    #[test]
    fn reduced_motion_freezes_reactor_shimmer_but_preserves_live_phase_cues() {
        let mut game = Game::new(Profile::default());
        game.diagnostic("boss-bastion-1");
        game.set_reduced_motion(true);
        let entity = game.active_boss.unwrap();
        let colors = |game: &Game| {
            let boss = *game.world.get::<&BossEncounter>(entity).unwrap();
            let body = game.world.get::<&PixelBody>(entity).unwrap();
            let mut frame = FrameBuilder::new();
            boss_assets::art(1, 1).rig.draw(
                &mut frame,
                &body,
                Transform::default(),
                boss.rig_state(game.cosmetic_time(), 1),
            );
            frame
                .frame()
                .batches
                .iter()
                .flat_map(|b| b.pixels.iter().map(|p| p.color))
                .collect::<Vec<_>>()
        };
        game.world.get::<&mut BossEncounter>(entity).unwrap().phase = Phase::Recover;
        game.time = 1.0;
        let first = colors(&game);
        game.time = 2.0;
        assert_eq!(colors(&game), first);
        game.set_reduced_motion(false);
        assert_ne!(colors(&game), first);
        game.set_reduced_motion(true);
        {
            let mut boss = game.world.get::<&mut BossEncounter>(entity).unwrap();
            boss.phase = Phase::Tell;
            boss.phase_age = 0.1;
        }
        let early = colors(&game);
        game.world
            .get::<&mut BossEncounter>(entity)
            .unwrap()
            .phase_age = 0.6;
        assert_ne!(
            colors(&game),
            early,
            "charge intensity is gameplay information"
        );
        for _ in 0..60 {
            game.tick(&InputFrame::default(), 1.0 / 60.0);
        }
        assert!(game.time > 2.9);
        assert_eq!(
            game.world.get::<&BossEncounter>(entity).unwrap().phase,
            Phase::Attack
        );
    }
    #[test]
    fn coil_segment_rig_extraction_preserves_actual_damage_and_exact_world_radii() {
        let mut game = Game::new(Profile::default());
        game.set_viewport(4096.0, 2048.0);
        game.diagnostic("boss-coil-5");
        let owner = game.active_boss.unwrap();
        let segments: Vec<_> = game
            .world
            .query::<(Entity, &Segment)>()
            .iter()
            .filter(|(_, s)| s.owner == owner)
            .map(|(e, _)| e)
            .collect();
        assert_eq!(segments.len(), 19);
        for entity in &segments {
            let segment = game.world.get::<&Segment>(*entity).unwrap();
            let body = game.world.get::<&PixelBody>(*entity).unwrap();
            assert!((segment.radius - pixel_assets::radius(&body, segment.scale)).abs() < 0.001);
        }
        let mut frame = FrameBuilder::new();
        game.render_into(1.0, &mut frame);
        let before = frame.frame().pixel_count;
        let core = boss_assets::segment_art(5).core_position;
        let removed = game
            .world
            .get::<&mut PixelBody>(segments[0])
            .unwrap()
            .damage(core, 1.5)
            .len();
        assert!(removed > 0);
        game.render_into(1.0, &mut frame);
        assert_eq!(frame.frame().pixel_count, before - removed);
        assert!(
            frame
                .frame()
                .batches
                .iter()
                .all(|batch| batch.transforms.len() <= 128)
        );
    }

    fn first_chain_tick(game: &mut Game) {
        let dt = 1.0 / 60.0;
        let player = game.position();
        game.update_bosses(player, dt);
        game.update_enemies(player, dt);
        game.update_segments(dt);
        game.integrate(dt);
        game.enforce_actor_sanctuaries();
        game.resolve_collisions(dt);
    }

    #[test]
    fn articulated_solver_repairs_old_vertical_layout_before_spawn_contact() {
        let mut game = Game::new(Profile::default());
        game.move_player(Vec2::new(1800.0, 0.0));
        game.set_viewport(533.0, 300.0);
        let distance = game.viewport.length() * 0.5 + 45.0;
        let spawn = game.position() - Vec2::Y * distance;
        let owner = game.spawn_enemy(2, true, spawn, 1);
        game.configure_boss(owner, 2, 3, false, 1);
        let mut segments: Vec<_> = game
            .world
            .query::<(Entity, &Segment)>()
            .iter()
            .filter(|(_, segment)| segment.owner == owner)
            .map(|(entity, segment)| {
                (
                    entity,
                    segment.index,
                    if segment.index == 1 {
                        game.world.get::<&Actor>(owner).unwrap().radius + segment.radius * 0.8
                    } else {
                        segment.radius * 1.7
                    },
                )
            })
            .collect();
        segments.sort_by_key(|(_, index, _)| *index);
        let mut length = 0.0;
        // Reconstruct the old initialization through actual physics entities.
        // The old orientation would cross the player; articulation repairs it
        // before the first contact pass without hiding or invulnerabling enemies.
        for (entity, _, spacing) in segments {
            length += spacing;
            *game.world.get::<&mut Motion>(entity).unwrap() =
                Motion::new(spawn + Vec2::Y * length, Vec2::ZERO, -FRAC_PI_2);
        }
        let hull = game.hud().hull;
        game.take_events();
        first_chain_tick(&mut game);
        assert_eq!(game.hud().hull, hull);
        assert!(game.take_events().iter().all(|event| event.kind != "hit"));
        let boss = game.world.get::<&BossEncounter>(owner).unwrap();
        assert_eq!(boss.phase, Phase::Tell);
        assert!(boss.phase_age < 0.02);
    }

    #[test]
    fn streamed_coil_chains_face_inward_and_cannot_contact_the_player_on_the_first_tick() {
        assert_eq!(initial_chain_direction(Vec2::ZERO, Vec2::ZERO), Vec2::Y);
        for viewport in [Vec2::new(533.0, 300.0), Vec2::new(320.0, 180.0)] {
            for generation in 0..=5 {
                for angle in 0..16 {
                    let mut game = Game::new(Profile::default());
                    game.move_player(Vec2::new(1800.0, 0.0));
                    game.set_viewport(viewport.x, viewport.y);
                    let outward = forward(angle as f32 * TAU / 16.0);
                    let distance =
                        viewport.length() * 0.5 + if generation == 0 { 60.0 } else { 45.0 };
                    let spawn = game.position() + outward * distance;
                    let owner = game.spawn_enemy(2, generation != 0, spawn, 1);
                    if generation > 0 {
                        game.configure_boss(owner, 2, generation, false, 1);
                    }
                    let heading = game.world.get::<&Motion>(owner).unwrap().heading;
                    assert!(forward(heading).distance(-outward) < 0.001);
                    for (motion, segment) in game.world.query::<(&Motion, &Segment)>().iter() {
                        if segment.owner == owner {
                            assert!((motion.position - spawn).dot(outward) > 0.0);
                            assert!(motion.position.distance(game.position()) > distance);
                            assert!(forward(motion.heading).distance(-outward) < 0.001);
                            assert_eq!(motion.heading, motion.previous_heading);
                        }
                    }
                    let hull = game.hud().hull;
                    game.take_events();
                    first_chain_tick(&mut game);
                    assert_eq!(game.hud().hull, hull, "G{generation}, angle {angle}");
                    assert!(game.take_events().iter().all(|event| event.kind != "hit"));
                    if generation > 0 {
                        assert_eq!(
                            game.world.get::<&BossEncounter>(owner).unwrap().phase,
                            Phase::Tell
                        );
                    }
                }
            }
        }
    }
}
