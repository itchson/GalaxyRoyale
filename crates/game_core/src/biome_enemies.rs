//! Iron Belt's authored Bastion variant. Birth identity never changes at biome
//! borders; the ordinary actor, projectile, damage and reward paths stay shared.
use crate::*;
use std::sync::LazyLock;

const IRON_BELT: u8 = 3;
const CHARGE: f32 = 0.9;
const LOCK_AT: f32 = 0.65;
const BURST: f32 = 0.36;
const RECOVER: f32 = 4.14;
const ROUND_INTERVAL: f32 = 0.12;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Phase {
    Waiting,
    Charge,
    Burst,
    Recover,
}

#[derive(Clone, Copy)]
pub(super) struct Riveter {
    phase: Phase,
    age: f32,
    rounds: u8,
    aim: Vec2,
    ready_delay: f32,
    birth_grid: [i32; 2],
}

impl Riveter {
    fn new(aim: Vec2, ready_delay: f32, birth_grid: [i32; 2]) -> Self {
        Self {
            phase: Phase::Waiting,
            age: 0.0,
            rounds: 0,
            aim,
            ready_delay,
            birth_grid,
        }
    }

    fn reset_warning(&mut self) {
        self.phase = Phase::Waiting;
        self.age = 0.0;
        self.rounds = 0;
    }

    fn advance(&mut self, towards: Vec2, visible: bool, dt: f32) -> Option<usize> {
        self.age += dt;
        match self.phase {
            Phase::Waiting => {
                self.aim = towards;
                if visible && self.age >= self.ready_delay {
                    self.phase = Phase::Charge;
                    self.age = 0.0;
                    self.rounds = 0;
                }
            }
            Phase::Charge => {
                if self.age < LOCK_AT {
                    self.aim = towards;
                }
                if self.age >= CHARGE {
                    self.phase = Phase::Burst;
                    self.age -= CHARGE;
                }
            }
            Phase::Burst => {}
            Phase::Recover => {
                if self.age >= RECOVER {
                    self.phase = Phase::Waiting;
                    self.age = 0.0;
                    self.ready_delay = 0.0;
                }
            }
        }
        let round = if self.phase == Phase::Burst
            && self.rounds < 3
            && self.age + 0.00001 >= self.rounds as f32 * ROUND_INTERVAL
        {
            let barrel = self.rounds as usize % 2;
            self.rounds += 1;
            Some(barrel)
        } else {
            None
        };
        if self.phase == Phase::Burst && self.age >= BURST {
            self.phase = Phase::Recover;
            self.age -= BURST;
        }
        round
    }

    fn charge(self) -> f32 {
        if self.phase == Phase::Charge {
            (self.age / CHARGE).clamp(0.0, 1.0)
        } else if self.phase == Phase::Burst {
            1.0
        } else {
            0.0
        }
    }
}

#[derive(Deserialize)]
struct Atlas {
    version: u32,
    palette: Vec<u32>,
    sprites: HashMap<String, Sprite>,
    muzzles: [Vec<[usize; 2]>; 2],
}

#[derive(Deserialize)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<u8>,
}

struct Art {
    body: PixelBody,
    muzzles: [Vec<[usize; 2]>; 2],
    palette: Vec<u32>,
}

static ART: LazyLock<Art> = LazyLock::new(|| {
    let atlas: Atlas =
        serde_json::from_str(include_str!("../../../assets/pixels/riveter-atlas.json"))
            .expect("authored Riveter atlas must parse");
    assert_eq!(atlas.version, 1);
    assert_eq!(atlas.palette.len(), 64);
    assert_eq!(atlas.palette[0], 0);
    assert!(atlas.palette[1..].iter().all(|color| color >> 24 == 255));
    let sprite = &atlas.sprites["riveter"];
    assert_eq!((sprite.width, sprite.height), (40, 40));
    assert_eq!(sprite.indices.len(), 1600);
    assert!(sprite.indices.iter().all(|&index| index < 64));
    let key = |index: usize| char::from_u32(0xe000 + index as u32).unwrap();
    let palette: Vec<_> = atlas
        .palette
        .iter()
        .enumerate()
        .skip(1)
        .map(|(index, &color)| (key(index), color))
        .collect();
    let rows: Vec<String> = sprite
        .indices
        .chunks_exact(40)
        .map(|row| {
            row.iter()
                .map(|&index| if index == 0 { ' ' } else { key(index as usize) })
                .collect()
        })
        .collect();
    let borrowed: Vec<_> = rows.iter().map(String::as_str).collect();
    let body = PixelBody::from_ascii(&borrowed, &palette);
    for mask in &atlas.muzzles {
        assert!(!mask.is_empty());
        let mut previous = None;
        for &[x, y] in mask {
            assert!(x < 40 && y < 40 && body.is_occupied(x, y));
            assert!(previous.is_none_or(|p| p < (y, x)));
            previous = Some((y, x));
        }
    }
    Art {
        body,
        muzzles: atlas.muzzles,
        palette: atlas.palette,
    }
});

/// Frontmost surviving row of a real barrel, in the current damaged body.
fn muzzle(body: &PixelBody, barrel: usize) -> Option<Vec2> {
    let mut front_row = None;
    let mut x_sum = 0.0;
    let mut count = 0;
    for &[x, y] in &ART.muzzles[barrel] {
        if front_row.is_some_and(|row| y > row) {
            break;
        }
        if body.is_occupied(x, y) {
            front_row = Some(y);
            x_sum += x as f32 + 0.5;
            count += 1;
        }
    }
    let y = front_row?;
    let center = x_sum / count as f32;
    let x = ART.muzzles[barrel]
        .iter()
        .filter(|&&[x, row]| row == y && body.is_occupied(x, row))
        .min_by(|a, b| {
            ((a[0] as f32 + 0.5) - center)
                .abs()
                .total_cmp(&((b[0] as f32 + 0.5) - center).abs())
                .then_with(|| a[0].cmp(&b[0]))
        })?[0];
    Some(Vec2::new(x as f32 + 0.5 - 20.0, y as f32 + 0.5 - 20.0))
}

impl Game {
    /// Called only after the normal family-weight roll and ordinary spawn.
    pub(super) fn maybe_riveter(&mut self, entity: Entity, biome: regions::Biome) -> bool {
        if biome.kind != IRON_BELT
            || self.arena.is_some()
            || self.boss_qa_scene
            || !self.riveter_eligible(entity)
            || self.random() >= 1.0 / 3.0
        {
            return false;
        }
        self.install_riveter(entity, biome.grid)
    }

    fn riveter_eligible(&self, entity: Entity) -> bool {
        self.world
            .get::<&Actor>(entity)
            .is_ok_and(|actor| actor.kind == Kind::Enemy(1))
            && self
                .world
                .get::<&Brain>(entity)
                .is_ok_and(|brain| brain.family == 1 && !brain.boss)
            && self.world.get::<&ChampionFighter>(entity).is_err()
            && self.world.get::<&Riveter>(entity).is_err()
    }

    fn install_riveter(&mut self, entity: Entity, birth_grid: [i32; 2]) -> bool {
        if !self.riveter_eligible(entity) {
            return false;
        }
        let (aim, ready_delay) = {
            let motion = self.world.get::<&Motion>(entity).unwrap();
            let brain = self.world.get::<&Brain>(entity).unwrap();
            (
                (self.position() - motion.position).normalize_or_zero(),
                brain.phase / TAU * 1.8,
            )
        };
        let aim = if aim.length_squared() > 0.0 {
            aim
        } else {
            Vec2::X
        };
        let body = ART.body.clone();
        if let Ok(mut actor) = self.world.get::<&mut Actor>(entity) {
            actor.radius = pixel_assets::radius(&body, actor.scale);
        }
        if let Ok(mut motion) = self.world.get::<&mut Motion>(entity) {
            motion.heading = aim.y.atan2(aim.x);
            motion.previous_heading = motion.heading;
        }
        self.world
            .insert(entity, (body, Riveter::new(aim, ready_delay, birth_grid)))
            .is_ok()
    }

    pub(super) fn update_biome_enemies(&mut self, player: Vec2, dt: f32) {
        let refuge = self.safe_zone_at(player);
        let pursuit_range = self.viewport.length().max(1050.0);
        let camera = self.render_camera(1.0);
        let half_view = self.viewport * 0.5;
        let mut shots = Vec::new();
        let mut remove = Vec::new();
        for (entity, (motion, actor, brain, body, riveter)) in self.world.query_mut::<(
            Entity,
            (&mut Motion, &Actor, &mut Brain, &PixelBody, &mut Riveter),
        )>() {
            brain.age += dt;
            if let Some(outward) =
                push_out_of_sanctuaries(&mut motion.position, actor.radius, &self.nearby_sites)
            {
                motion.velocity = outward * motion.velocity.length().max(50.0);
            }
            let delta = player - motion.position;
            let distance = delta.length().max(0.1);
            let towards = delta / distance;
            if let Some((center, _)) = refuge {
                riveter.reset_warning();
                let outward = (motion.position - center).normalize_or_zero();
                motion.velocity += (outward * 95.0 - motion.velocity) * (dt * 3.0);
                if distance > 550.0 {
                    remove.push(entity);
                }
                continue;
            }
            if distance > pursuit_range {
                remove.push(entity);
                continue;
            }
            let from_camera = (motion.position - Vec2::from_array(camera)).abs();
            // Full hull plus a 28-unit warning stays inside the actual camera.
            let visible = distance < 520.0
                && from_camera.x + actor.radius + 28.0 < half_view.x
                && from_camera.y + actor.radius + 28.0 < half_view.y;
            let round = riveter.advance(towards, visible, dt);
            let side = Vec2::new(-towards.y, towards.x);
            let approach = ((distance - 160.0) * 1.5).clamp(-72.0, 72.0);
            let strafe = brain.phase.sin().signum();
            let desired = match riveter.phase {
                Phase::Waiting | Phase::Recover => towards * approach + side * strafe * 48.0,
                Phase::Charge => towards * approach * 0.2 + side * strafe * 12.0,
                Phase::Burst => Vec2::ZERO,
            } * (1.0 + 0.035 * danger(player).saturating_sub(1) as f32).min(1.3);
            motion.velocity += (desired - motion.velocity) * (1.0 - (-3.0 * dt).exp());
            let facing = if matches!(riveter.phase, Phase::Charge | Phase::Burst) {
                riveter.aim
            } else {
                towards
            };
            motion.heading = turn_toward(motion.heading, facing.y.atan2(facing.x), 3.0 * PI * dt);
            if riveter.phase == Phase::Burst {
                motion.heading = facing.y.atan2(facing.x);
            }
            if let Some(barrel) = round
                && distance < 520.0
                && let Some(local) = muzzle(body, barrel).or_else(|| muzzle(body, 1 - barrel))
            {
                let origin = actor_transform(*motion, *actor).point(local)
                    + riveter.aim * (actor.scale * 0.75);
                shots.push((origin, riveter.aim * 125.0, actor.damage * 0.8));
            }
        }
        for (origin, velocity, damage) in shots {
            self.spawn_bullet(origin, velocity, damage, false, 1);
        }
        for entity in remove {
            self.remove_enemy(entity, false);
        }
    }

    pub(super) fn draw_riveter(
        &self,
        frame: &mut FrameBuilder,
        body: &PixelBody,
        motion: Motion,
        actor: Actor,
        riveter: Riveter,
    ) {
        let charge = riveter.charge();
        let decorative = (self.cosmetic_time() * 2.0
            + riveter.birth_grid[0] as f32 * 0.2
            + riveter.birth_grid[1] as f32 * 0.3)
            .sin()
            * 0.5
            + 0.5;
        frame.push_body_colored_material(
            body,
            actor_transform(motion, actor),
            ACTOR_MATERIAL,
            |_, color| {
                if color == ART.palette[8] || color == ART.palette[9] {
                    let [r, g, b, a] = color.to_le_bytes();
                    rgba(
                        (r as f32 + (255.0 - r as f32) * charge) as u8,
                        (g as f32 + (255.0 - g as f32) * charge) as u8,
                        (b as f32 + (255.0 - b as f32) * charge) as u8,
                        a,
                    )
                } else if color == ART.palette[12] {
                    rgba(100, (215.0 + decorative * 35.0) as u8, 224, 255)
                } else {
                    color
                }
            },
        );
        // These short world-space cells indicate the real locked shot direction.
        // They are a cue, not physics; muzzle loss removes that barrel's cue too.
        if riveter.phase == Phase::Charge {
            for barrel in 0..2 {
                if let Some(local) = muzzle(body, barrel) {
                    let origin = actor_transform(motion, actor).point(local);
                    for step in 1..=7 {
                        frame.push_pixel(
                            origin + riveter.aim * (step as f32 * 4.0),
                            0.7 + charge * 0.5,
                            rgba(255, 186, 95, (65.0 + charge * 140.0) as u8),
                        );
                    }
                }
            }
        }
    }

    /// Explicit QA setup, never a production reward or progression shortcut.
    pub(super) fn biome_enemy_diagnostic(&mut self, command: &str) -> bool {
        if command != "biome-riveter" {
            return false;
        }
        self.reset_to_dock();
        self.boss_qa_scene = true;
        let mut candidates = Vec::new();
        for y in -4..=4 {
            for x in -4..=4 {
                for offset in [0.12, 0.5, 0.88] {
                    let p = Vec2::new(x as f32 + offset, y as f32 + offset) * regions::REGION_SIZE;
                    if regions::at(self.profile.seed, p).kind == IRON_BELT
                        && p.length() > COMBAT_RADIUS + 300.0
                        && regions::protected(self.profile.seed, p).is_none()
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
            let entity = self.spawn_enemy(1, false, position - Vec2::Y * 100.0, 1);
            self.install_riveter(entity, regions::at(self.profile.seed, position).grid);
            self.world.get::<&mut Riveter>(entity).unwrap().ready_delay = 0.0;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iron(seed: u32) -> (Vec2, regions::Biome) {
        for y in -4..=4 {
            for x in -4..=4 {
                let position = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) * regions::REGION_SIZE;
                let biome = regions::at(seed, position);
                if biome.kind == IRON_BELT && regions::protected(seed, position).is_none() {
                    return (position, biome);
                }
            }
        }
        panic!("bounded fixture must find the actual deterministic Iron Belt");
    }

    fn fixture() -> (Game, Entity) {
        let mut game = Game::new(Profile::default());
        game.set_viewport(640.0, 360.0);
        let (position, biome) = iron(game.profile.seed);
        game.move_player(position);
        game.update_chunks(position);
        let rocks: Vec<_> = game
            .world
            .query::<(Entity, &Actor)>()
            .iter()
            .filter_map(|(entity, actor)| (actor.kind == Kind::Asteroid).then_some(entity))
            .collect();
        for entity in rocks {
            let _ = game.world.despawn(entity);
        }
        game.spawn_timer = 10_000.0;
        game.boss_timer = 10_000.0;
        game.invulnerability = 0.0;
        let entity = game.spawn_enemy(1, false, position - Vec2::Y * 100.0, 1);
        assert!(game.install_riveter(entity, biome.grid));
        game.world.get::<&mut Riveter>(entity).unwrap().ready_delay = 0.0;
        (game, entity)
    }

    fn hostile_count(game: &Game) -> usize {
        game.world
            .query::<(Entity, &Bullet)>()
            .iter()
            .filter(|(_, bullet)| !bullet.player)
            .count()
    }

    fn erase_barrel(body: &mut PixelBody, barrel: usize) {
        for &[x, y] in &ART.muzzles[barrel] {
            body.damage(
                Vec2::new(x as f32 + 0.5 - 20.0, y as f32 + 0.5 - 20.0),
                0.01,
            );
        }
    }

    #[test]
    fn authored_cells_and_muzzle_origins_match_the_live_damage_mask() {
        assert_eq!((ART.body.width, ART.body.height), (40, 40));
        assert_eq!(ART.body.occupied_count(), 856);
        assert_ne!(
            ART.body.occupied_count(),
            pixel_assets::body("bastion").occupied_count()
        );
        assert_eq!(ART.muzzles.each_ref().map(Vec::len), [100, 100]);
        assert!(pixel_assets::radius(&ART.body, 0.7) < 24.0);
        let mut body = ART.body.clone();
        let front = muzzle(&body, 0).unwrap();
        assert_eq!(front, Vec2::new(9.5 - 20.0, 2.5 - 20.0));
        body.damage(front, 0.01);
        assert!(!body.is_occupied(9, 2));
        assert!(body.is_occupied(8, 2) && body.is_occupied(10, 2));
        let moved = muzzle(&body, 0).unwrap();
        assert_ne!(front, moved);
        assert_eq!(
            moved,
            Vec2::new(8.5 - 20.0, 2.5 - 20.0),
            "symmetric hole uses the left surviving cell center on ties"
        );
        assert!(body.is_occupied(
            (moved.x + 20.0).floor() as usize,
            (moved.y + 20.0).floor() as usize
        ));
        erase_barrel(&mut body, 0);
        assert!(muzzle(&body, 0).is_none());
        assert!(muzzle(&body, 1).is_some());
        erase_barrel(&mut body, 1);
        assert!(muzzle(&body, 1).is_none());
        assert!(body.occupied_count() > 500);
    }

    #[test]
    fn birth_selection_is_seeded_bounded_and_does_not_migrate_at_borders() {
        let mut a = Game::new(Profile::default());
        let mut b = Game::new(Profile::default());
        let (position, biome) = iron(a.profile.seed);
        a.move_player(position);
        b.move_player(position);
        let before = serde_json::to_value(&a.profile).unwrap();
        let mut selected = 0;
        for _ in 0..96 {
            let ea = a.spawn_enemy(1, false, position + Vec2::X * 200.0, 1);
            let eb = b.spawn_enemy(1, false, position + Vec2::X * 200.0, 1);
            let va = a.maybe_riveter(ea, biome);
            assert_eq!(va, b.maybe_riveter(eb, biome));
            selected += va as usize;
            if va {
                assert_eq!(a.world.get::<&Riveter>(ea).unwrap().birth_grid, biome.grid);
                let mut foreign = biome;
                foreign.kind = 0;
                assert!(!a.maybe_riveter(ea, foreign));
                assert!(a.world.get::<&Riveter>(ea).is_ok());
                assert_eq!(a.world.get::<&Actor>(ea).unwrap().kind, Kind::Enemy(1));
            }
            a.remove_enemy(ea, false);
            b.remove_enemy(eb, false);
        }
        assert!(
            (20..45).contains(&selected),
            "seeded selection count {selected}"
        );
        for family in 0..3 {
            for boss in [false, true] {
                if family == 1 && !boss {
                    continue;
                }
                let entity = a.spawn_enemy(family, boss, position, 1);
                assert!(!a.maybe_riveter(entity, biome));
                a.remove_enemy(entity, false);
            }
        }
        let entity = a.spawn_enemy(1, false, position, 1);
        for kind in [0, 1, 2, 4, 5] {
            let mut foreign = biome;
            foreign.kind = kind;
            assert!(!a.maybe_riveter(entity, foreign));
        }
        assert_eq!(serde_json::to_value(&a.profile).unwrap(), before);
    }

    #[test]
    fn warning_waits_on_screen_and_the_final_quarter_second_locks_three_rounds() {
        let mut riveter = Riveter::new(Vec2::X, 0.0, [2, -1]);
        for _ in 0..900 {
            assert!(riveter.advance(Vec2::X, false, 1.0 / 60.0).is_none());
            assert_eq!(riveter.phase, Phase::Waiting);
        }
        assert!(riveter.advance(Vec2::X, true, 1.0 / 60.0).is_none());
        assert_eq!(riveter.phase, Phase::Charge);
        let mut shots = Vec::new();
        for _ in 0..330 {
            let towards = if riveter.phase == Phase::Charge && riveter.age > LOCK_AT {
                Vec2::Y
            } else {
                Vec2::X
            };
            if let Some(barrel) = riveter.advance(towards, true, 1.0 / 60.0) {
                shots.push((barrel, riveter.aim));
            }
        }
        assert_eq!(shots.len(), 3);
        assert_eq!(shots.iter().map(|s| s.0).collect::<Vec<_>>(), [0, 1, 0]);
        assert!(shots.iter().all(|s| s.1 == Vec2::X));
    }

    #[test]
    fn installed_controller_emits_real_locked_projectiles_and_no_generic_extra_shot() {
        let (mut game, entity) = fixture();
        game.shield = 0.0; // Declared depleted shield isolates real round-to-hull collisions.
        let player = game.position();
        let mut seen = HashSet::new();
        let mut directions = Vec::new();
        let mut phases = HashSet::new();
        for _ in 0..210 {
            game.tick(&InputFrame::default(), 1.0 / 60.0);
            phases.insert(format!(
                "{:?}",
                game.world.get::<&Riveter>(entity).unwrap().phase
            ));
            for (bullet_entity, (motion, bullet)) in
                game.world.query::<(Entity, (&Motion, &Bullet))>().iter()
            {
                if !bullet.player && seen.insert(bullet_entity) {
                    assert!((motion.velocity.length() - 125.0).abs() < 0.001);
                    assert!((bullet.damage - 7.2).abs() < 0.001);
                    directions.push(motion.velocity.normalize_or_zero());
                }
            }
        }
        assert_eq!(
            directions.len(),
            3,
            "generic Bastion firing must be skipped"
        );
        assert!(
            directions
                .iter()
                .all(|direction| direction.distance(directions[0]) < 0.0001)
        );
        assert!(
            phases.contains("Charge") && phases.contains("Burst") && phases.contains("Recover")
        );
        assert!(
            game.hud().hull < 100.0,
            "actual Riveter rounds must reach the player's occupied hull"
        );
        assert!(game.position().distance(player) < 0.1);
    }

    #[test]
    fn integrated_camera_gate_and_range_retreat_preserve_family_progression() {
        let (mut game, entity) = fixture();
        let player = game.position();
        {
            let mut motion = game.world.get::<&mut Motion>(entity).unwrap();
            motion.position = player + Vec2::X * 630.0;
            motion.previous = motion.position;
        }
        let before = serde_json::to_value(&game.profile).unwrap();
        for _ in 0..30 {
            game.tick(&InputFrame::default(), 1.0 / 60.0);
            assert_eq!(
                game.world.get::<&Riveter>(entity).unwrap().phase,
                Phase::Waiting
            );
            assert_eq!(hostile_count(&game), 0);
        }
        {
            let mut motion = game.world.get::<&mut Motion>(entity).unwrap();
            motion.position = player - Vec2::Y * 100.0;
            motion.previous = motion.position;
            motion.velocity = Vec2::ZERO;
        }
        game.tick(&InputFrame::default(), 1.0 / 60.0);
        assert_eq!(
            game.world.get::<&Riveter>(entity).unwrap().phase,
            Phase::Charge
        );
        {
            let mut motion = game.world.get::<&mut Motion>(entity).unwrap();
            motion.position = player + Vec2::X * 1500.0;
            motion.previous = motion.position;
        }
        game.tick(&InputFrame::default(), 1.0 / 60.0);
        assert!(!game.world.contains(entity));
        assert_eq!(serde_json::to_value(&game.profile).unwrap(), before);
    }

    #[test]
    fn destroyed_barrels_and_the_shared_bullet_cap_prevent_phantom_fire() {
        let (mut single_barrel, entity) = fixture();
        let origin = {
            let mut body = single_barrel.world.get::<&mut PixelBody>(entity).unwrap();
            erase_barrel(&mut body, 0);
            let surviving = muzzle(&body, 1).unwrap();
            actor_transform(
                *single_barrel.world.get::<&Motion>(entity).unwrap(),
                *single_barrel.world.get::<&Actor>(entity).unwrap(),
            )
            .point(surviving)
        };
        single_barrel
            .world
            .get::<&mut Riveter>(entity)
            .unwrap()
            .phase = Phase::Burst;
        single_barrel.update_biome_enemies(single_barrel.position(), 1.0 / 60.0);
        let (motion, _) = single_barrel
            .world
            .query::<(&Motion, &Bullet)>()
            .iter()
            .next()
            .map(|(motion, bullet)| (*motion, *bullet))
            .unwrap();
        assert!(
            motion.position.distance(origin) < 1.0,
            "shot must use the surviving actual barrel"
        );
        let (mut game, entity) = fixture();
        {
            let mut body = game.world.get::<&mut PixelBody>(entity).unwrap();
            erase_barrel(&mut body, 0);
            erase_barrel(&mut body, 1);
        }
        for _ in 0..100 {
            game.update_biome_enemies(game.position(), 1.0 / 60.0);
        }
        assert_eq!(hostile_count(&game), 0);
        assert!(game.world.get::<&Actor>(entity).unwrap().health > 0.0);
        let (mut game, entity) = fixture();
        let position = game.position();
        for _ in 0..MAX_BULLETS {
            game.spawn_bullet(position, Vec2::X * 10.0, 1.0, false, 1);
        }
        game.world.get::<&mut Riveter>(entity).unwrap().phase = Phase::Burst;
        game.update_biome_enemies(position, 1.0 / 60.0);
        assert_eq!(hostile_count(&game), MAX_BULLETS);
        assert_eq!(game.world.get::<&Riveter>(entity).unwrap().rounds, 1);
    }

    #[test]
    fn haven_and_remote_sanctuaries_reset_bursts_and_keep_real_shots_out() {
        let (game, _) = fixture();
        let site = regions::sites_near(game.profile.seed, game.position()).remove(0);
        for (center, radius) in [
            (Vec2::ZERO, SAFE_RADIUS),
            (Vec2::from_array(site.position), site.radius),
        ] {
            let (mut game, entity) = fixture();
            game.move_player(center);
            {
                let mut motion = game.world.get::<&mut Motion>(entity).unwrap();
                motion.position = center + Vec2::X * (radius - 3.0);
                motion.previous = motion.position;
            }
            game.world.get::<&mut Riveter>(entity).unwrap().phase = Phase::Burst;
            let before = game.hud().hull;
            game.tick(&InputFrame::default(), 0.05);
            if center == Vec2::ZERO {
                assert!(
                    !game.world.contains(entity),
                    "Haven arrival clears ordinary enemies"
                );
            } else {
                assert_eq!(
                    game.world.get::<&Riveter>(entity).unwrap().phase,
                    Phase::Waiting
                );
                let motion = *game.world.get::<&Motion>(entity).unwrap();
                let actor = *game.world.get::<&Actor>(entity).unwrap();
                assert!(motion.position.distance(center) > radius + actor.radius);
            }
            assert_eq!(hostile_count(&game), 0);
            assert_eq!(game.hud().hull, before);
            game.spawn_bullet(
                center - Vec2::X * (radius + 30.0),
                Vec2::X * 20_000.0,
                99.0,
                false,
                1,
            );
            game.integrate(0.05);
            game.resolve_collisions(0.05);
            assert_eq!(hostile_count(&game), 0);
            assert_eq!(game.hud().hull, before);
        }
    }

    #[test]
    fn actual_pixel_projectile_defeat_counts_once_as_one_ordinary_bastion() {
        let (mut game, entity) = fixture();
        game.profile.kills[1] = 23;
        game.kill_credit[1] = 23;
        game.profile.boss_generations[0] = [2, 1, 0, 0, 0];
        game.profile.boss_kills[0] = 3;
        game.profile.kills[0] = 48;
        let source = (-3..=3)
            .flat_map(|y| {
                (-3..=3).flat_map(move |x| {
                    regions::sites_near(
                        Profile::default().seed,
                        Vec2::new(x as f32, y as f32) * regions::REGION_SIZE,
                    )
                })
            })
            .find(|site| site.family == 1 && site.event == "bounty")
            .unwrap();
        game.profile.quests.push(QuestProgress {
            site_id: source.id,
            event: "bounty".into(),
            family: 1,
            target: source.target_kills,
            progress: 0,
            completed: false,
        });
        let before = game.profile.clone();
        let position = game.world.get::<&Motion>(entity).unwrap().position;
        for _ in 0..4 {
            let local = game
                .world
                .get::<&PixelBody>(entity)
                .unwrap()
                .cells()
                .next()
                .unwrap()
                .0;
            let target = actor_transform(
                *game.world.get::<&Motion>(entity).unwrap(),
                *game.world.get::<&Actor>(entity).unwrap(),
            )
            .point(local);
            game.spawn_bullet(target - Vec2::X * 20.0, Vec2::X * 2400.0, 12.0, true, 1);
            game.integrate(1.0 / 60.0);
            game.resolve_collisions(1.0 / 60.0);
        }
        assert!(!game.world.contains(entity));
        assert_eq!(game.profile.kills, [before.kills[0], 24, before.kills[2]]);
        assert!(game.profile.pending[1]);
        assert_eq!(game.kill_credit[1], 0);
        assert_eq!(
            game.profile.campaign.ship_xp[0],
            before.campaign.ship_xp[0] + 5
        );
        assert_eq!(game.profile.currency, before.currency);
        assert_eq!(game.profile.boss_generations, before.boss_generations);
        assert_eq!(game.profile.boss_kills, before.boss_kills);
        assert_eq!(game.profile.unlocked, before.unlocked);
        assert_eq!(game.profile.quests[0].progress, 1);
        assert!(!game.profile.quests[0].completed);
        assert!(game.world.query::<(Entity, &Loot)>().iter().count() <= 1);
        let once = serde_json::to_value(&game.profile).unwrap();
        game.kill_actor(entity, Kind::Enemy(1), position);
        assert_eq!(serde_json::to_value(&game.profile).unwrap(), once);
    }

    #[test]
    fn damaged_render_keeps_holes_and_live_charge_under_reduced_motion() {
        let (mut game, entity) = fixture();
        game.set_reduced_motion(true);
        let body = {
            let mut body = game.world.get::<&mut PixelBody>(entity).unwrap();
            body.damage(Vec2::new(0.5, 5.5), 3.0);
            body.clone()
        };
        let motion = *game.world.get::<&Motion>(entity).unwrap();
        let actor = *game.world.get::<&Actor>(entity).unwrap();
        let mut riveter = *game.world.get::<&Riveter>(entity).unwrap();
        riveter.phase = Phase::Recover;
        let mut frame = FrameBuilder::new();
        game.draw_riveter(&mut frame, &body, motion, actor, riveter);
        assert_eq!(frame.frame().pixel_count, body.occupied_count());
        let resting = frame.finish();
        riveter.phase = Phase::Charge;
        riveter.age = 0.75;
        let mut frame = FrameBuilder::new();
        game.draw_riveter(&mut frame, &body, motion, actor, riveter);
        assert_eq!(frame.frame().pixel_count, body.occupied_count() + 14);
        let resting_colors: Vec<_> = resting
            .batches
            .iter()
            .flat_map(|batch| batch.pixels.iter().map(|pixel| pixel.color))
            .collect();
        let charged_colors: Vec<_> = frame
            .frame()
            .batches
            .iter()
            .flat_map(|batch| {
                batch
                    .pixels
                    .iter()
                    .take(body.occupied_count())
                    .map(|pixel| pixel.color)
            })
            .collect();
        assert_ne!(resting_colors, charged_colors);
        assert!(body.occupied_count() < ART.body.occupied_count());
    }

    #[test]
    fn explicit_iron_belt_diagnostic_does_not_grant_progress_after_real_defeat() {
        let mut game = Game::new(Profile::default());
        let before = serde_json::to_value(&game.profile).unwrap();
        game.diagnostic("biome-riveter");
        assert_eq!(
            regions::at(game.profile.seed, game.position()).kind,
            IRON_BELT
        );
        let entity = game
            .world
            .query::<(Entity, &Riveter)>()
            .iter()
            .next()
            .unwrap()
            .0;
        for _ in 0..4 {
            let local = game
                .world
                .get::<&PixelBody>(entity)
                .unwrap()
                .cells()
                .next()
                .unwrap()
                .0;
            let target = actor_transform(
                *game.world.get::<&Motion>(entity).unwrap(),
                *game.world.get::<&Actor>(entity).unwrap(),
            )
            .point(local);
            game.spawn_bullet(target - Vec2::X * 20.0, Vec2::X * 2400.0, 12.0, true, 1);
            game.integrate(1.0 / 60.0);
            game.resolve_collisions(1.0 / 60.0);
        }
        assert!(!game.world.contains(entity));
        assert_eq!(serde_json::to_value(&game.profile).unwrap(), before);
        assert_eq!(game.world.query::<(Entity, &Loot)>().iter().count(), 0);
    }
}
