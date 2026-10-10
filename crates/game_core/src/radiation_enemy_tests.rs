//! Focused actual Game/ECS candidate acceptance. Fixtures explicitly suppress
//! unrelated natural spawns/rocks; combat rewards are never granted directly.
use super::*;
use crate::{InputFrame, Profile, actor_transform};
use controller::Phase;
const DT: f32 = 1.0 / 60.0;

fn radial(seed: u32) -> Vec2 {
    for y in -6..=6 {
        for x in -6..=6 {
            let p = (Vec2::new(x as f32, y as f32) + Vec2::splat(0.5)) * regions::REGION_SIZE;
            if regions::at(seed, p).kind == BIOME_KIND
                && regions::sites_near(seed, p)
                    .iter()
                    .all(|s| Vec2::from_array(s.position).distance(p) > s.radius + 700.0)
                && p.length() > COMBAT_RADIUS + 500.0
            {
                return p;
            }
        }
    }
    panic!("bounded deterministic Radial Wastes fixture not found")
}

fn clear_rocks(game: &mut Game) {
    let rocks: Vec<_> = game
        .world
        .query::<(Entity, &Actor)>()
        .iter()
        .filter_map(|(e, a)| (a.kind == Kind::Asteroid).then_some(e))
        .collect();
    for e in rocks {
        let _ = game.world.despawn(e);
    }
}
fn setup(mut game: Game, viewport: Vec2, offset: Vec2) -> (Game, Entity) {
    let p = radial(game.profile.seed);
    game.set_viewport(viewport.x, viewport.y);
    game.move_player(p);
    game.update_chunks(p);
    clear_rocks(&mut game);
    game.spawn_timer = 10_000.0;
    game.boss_timer = 10_000.0;
    game.invulnerability = 0.0;
    let spawn = p + offset;
    for _ in 0..128 {
        let e = game.spawn_enemy(FAMILY, false, spawn, 1);
        if game.install_mote(e, regions::at(game.profile.seed, spawn)) {
            game.take_events();
            return (game, e);
        }
        game.remove_enemy(e, false);
    }
    panic!("deterministic birth failed to select Mote within bounded fixture")
}
fn fixture(viewport: Vec2, offset: Vec2) -> (Game, Entity) {
    setup(Game::new(Profile::default()), viewport, offset)
}
fn hostile(game: &Game) -> Vec<Entity> {
    game.world
        .query::<(Entity, &Bullet)>()
        .iter()
        .filter_map(|(e, b)| (!b.player).then_some(e))
        .collect()
}
fn tick(game: &mut Game) {
    game.tick(&InputFrame::default(), DT);
}
fn phase(game: &Game, e: Entity) -> Phase {
    game.world.get::<&Mote>(e).unwrap().controller.phase()
}
fn to_charge(game: &mut Game, e: Entity) {
    for _ in 0..1200 {
        tick(game);
        if phase(game, e) == Phase::Charge {
            return;
        }
    }
    panic!("no real charge after20s at viewport {:?}", game.viewport)
}
fn rewards(game: &Game) -> serde_json::Value {
    serde_json::json!({"currency":game.profile.currency,"ranks":game.profile.ranks,"unlocked":game.profile.unlocked,
        "kills":game.profile.kills,"boss_kills":game.profile.boss_kills,"generations":game.profile.boss_generations,
        "pity":game.profile.pity,"pending":game.profile.pending,"xp":game.profile.campaign.ship_xp,
        "quests":game.profile.quests,"materials":[game.profile.campaign.cargo_materials,game.profile.campaign.stored_materials]})
}

#[test]
fn birth_uses_actual_spawn_grid_without_rng_or_reward_mutation() {
    let mut game = Game::new(Profile::default());
    let p = radial(game.profile.seed);
    let biome = regions::at(game.profile.seed, p);
    let before = rewards(&game);
    let mut selected = 0;
    for _ in 0..128 {
        let e = game.spawn_enemy(FAMILY, false, p, 1);
        let rng = game.rng;
        let old = *game.world.get::<&Actor>(e).unwrap();
        let ticket = game.world.get::<&Brain>(e).unwrap().phase.to_bits();
        let expected = Birth {
            seed: game.profile.seed,
            grid: biome.grid,
            ticket,
            biome: biome.kind,
            family: FAMILY,
            ordinary: true,
        }
        .eligible();
        assert_eq!(game.maybe_mote(e, biome), expected);
        assert_eq!(
            game.rng, rng,
            "Mote selection must not perturb ordinary loot/spawn RNG"
        );
        if expected {
            selected += 1;
            let a = *game.world.get::<&Actor>(e).unwrap();
            assert_eq!(
                (a.kind, a.health, a.max_health, a.damage),
                (old.kind, old.health, old.max_health, old.damage)
            );
            assert_eq!(a.scale, BODY_SCALE);
            assert!((a.radius - 13.82055).abs() < 0.001);
            let birth = game.world.get::<&Mote>(e).unwrap().controller.birth();
            assert_eq!((birth.grid, birth.ticket), (biome.grid, ticket));
            game.world.get::<&mut Motion>(e).unwrap().position = Vec2::new(1600.0, 0.0);
            assert!(!game.maybe_mote(e, regions::at(game.profile.seed, Vec2::new(1600.0, 0.0))));
            assert_eq!(
                game.world.get::<&Mote>(e).unwrap().controller.birth(),
                birth
            );
        }
        game.remove_enemy(e, false);
    }
    assert!((16..49).contains(&selected), "selected {selected}/128");
    let e = game.spawn_enemy(FAMILY, false, p, 1);
    assert!(
        !game.maybe_mote(e, regions::at(game.profile.seed, Vec2::ZERO)),
        "player biome is not a birth biome"
    );
    for family in 0..3 {
        for boss in [false, true] {
            let e = game.spawn_enemy(family, boss, p, 1);
            if family != FAMILY || boss {
                assert!(!game.maybe_mote(e, biome));
            }
            game.remove_enemy(e, false);
        }
    }
    assert_eq!(rewards(&game), before);
}

#[test]
fn real_ticks_admit_full_warning_and_real_projectiles_at_all_supported_zoom_extents() {
    let mut proof = Vec::new();
    for ui in [
        Vec2::new(533.0, 300.0),
        Vec2::new(720.0, 360.0),
        Vec2::new(400.0, 240.0),
        Vec2::new(320.0, 180.0),
    ] {
        for zoom in [0.6, 1.0, 1.6] {
            let (mut game, e) = fixture(ui / zoom, -Vec2::Y * 210.0);
            let mut charged = false;
            let mut attack = false;
            let mut first = None;
            let mut minimum_clearance = f32::INFINITY;
            for step in 0..1200 {
                let old = game.world.get::<&Motion>(e).unwrap().position;
                tick(&mut game);
                let motion = *game.world.get::<&Motion>(e).unwrap();
                let actor = *game.world.get::<&Actor>(e).unwrap();
                let player_radius = game.world.get::<&Actor>(game.player).unwrap().radius;
                minimum_clearance = minimum_clearance
                    .min(motion.position.distance(game.position()) - actor.radius - player_radius);
                assert!(
                    motion.position.distance(old) < 3.0,
                    "no teleport or sudden position snap"
                );
                assert!(
                    minimum_clearance > 7.99,
                    "firing approach crossed physical player clearance"
                );
                let phase = phase(&game, e);
                if matches!(phase, Phase::Charge | Phase::Attack) {
                    charged |= phase == Phase::Charge;
                    attack |= phase == Phase::Attack;
                    let delta = (motion.position - Vec2::from_array(game.render_camera(1.0))).abs();
                    assert!(delta.x + actor.radius + WARNING_REACH + 5.0 < game.viewport.x * 0.5);
                    assert!(delta.y + actor.radius + WARNING_REACH + 5.0 < game.viewport.y * 0.5);
                    assert!(
                        motion.velocity.length() < 8.5,
                        "enemy must settle before warning"
                    );
                    assert!(
                        game.world
                            .get::<&Mote>(e)
                            .unwrap()
                            .controller
                            .target()
                            .is_some()
                    );
                }
                if !hostile(&game).is_empty() {
                    first = Some(step + 1);
                    break;
                }
            }
            assert!(
                charged && attack && first.is_some(),
                "UI{ui:?}/zoom{zoom} failed"
            );
            assert!((1..=6).contains(&hostile(&game).len()));
            assert_eq!(game.profile.kills, [0; 3]);
            proof.push(serde_json::json!({"ui":ui.to_array(),"zoom":zoom,"first_shot_tick":first,"clearance":minimum_clearance}));
        }
    }
    if let Ok(path) = std::env::var("GALAXY_MOTE_VIEW_PROOF") {
        std::fs::write(path, serde_json::to_string_pretty(&proof).unwrap()).unwrap();
    }
}

#[test]
fn zoom_and_camera_loss_abort_live_warning_and_do_not_replay_attack_debt() {
    let (mut game, e) = fixture(Vec2::new(640.0, 360.0), -Vec2::Y * 210.0);
    to_charge(&mut game, e);
    for _ in 0..20 {
        tick(&mut game);
    }
    assert!(game.world.get::<&Mote>(e).unwrap().controller.phase_age() > 0.25);
    game.set_viewport(120.0, 80.0); // Explicitly unsupported: full cue cannot fit.
    for _ in 0..240 {
        tick(&mut game);
        assert_eq!(phase(&game, e), Phase::Waiting);
        assert!(hostile(&game).is_empty());
    }
    game.set_viewport(533.0 / 1.6, 300.0 / 1.6);
    to_charge(&mut game, e);
    assert!(hostile(&game).is_empty());
    for _ in 0..30 {
        tick(&mut game);
        assert!(hostile(&game).is_empty());
    }
    game.camera += Vec2::X * 1000.0;
    game.previous_camera = game.camera;
    tick(&mut game);
    assert_eq!(phase(&game, e), Phase::Waiting);
    assert!(hostile(&game).is_empty());
}

#[test]
fn actual_pulse_origin_is_a_live_emitter_cell_and_erased_pylons_disable_owned_rays() {
    let (mut game, e) = fixture(Vec2::new(640.0, 360.0), -Vec2::Y * 100.0);
    let pristine = game.world.get::<&PixelBody>(e).unwrap().occupied_count();
    assert_eq!(pristine, 712);
    {
        let mut body = game.world.get::<&mut PixelBody>(e).unwrap();
        for &[x, y] in &controller::emitter_masks()[0] {
            body.damage(
                Vec2::new(x as f32 + 0.5 - 20.0, y as f32 + 0.5 - 20.0),
                0.01,
            );
        }
    }
    to_charge(&mut game, e);
    let mut found = false;
    for _ in 0..100 {
        tick(&mut game);
        let shots = hostile(&game);
        if shots.is_empty() {
            continue;
        }
        let motion = *game.world.get::<&Motion>(e).unwrap();
        let actor = *game.world.get::<&Actor>(e).unwrap();
        let body = game.world.get::<&PixelBody>(e).unwrap();
        let transform = actor_transform(
            Motion {
                position: motion.previous,
                ..motion
            },
            actor,
        );
        assert!(
            shots.len() < 6,
            "removed sector must not fire from another emitter"
        );
        for b in shots {
            let point = game.world.get::<&Motion>(b).unwrap().previous;
            let local = transform.inverse_point(point);
            let x = (local.x + 20.0).floor() as usize;
            let y = (local.y + 20.0).floor() as usize;
            assert!(body.is_occupied(x, y));
            assert!(!controller::emitter_masks()[0].contains(&[x, y]));
        }
        found = true;
        break;
    }
    assert!(found);
    {
        let mut body = game.world.get::<&mut PixelBody>(e).unwrap();
        for mask in controller::emitter_masks() {
            for &[x, y] in mask {
                body.damage(
                    Vec2::new(x as f32 + 0.5 - 20.0, y as f32 + 0.5 - 20.0),
                    0.01,
                );
            }
        }
        assert!(body.occupied_count() > 200);
    }
    let bullets = hostile(&game);
    for b in bullets {
        let _ = game.world.despawn(b);
    }
    for _ in 0..600 {
        tick(&mut game);
        assert!(hostile(&game).is_empty());
    }
}

#[test]
fn shared_cap_reserves_commands_across_two_entities_and_saturated_pulses_are_not_replayed() {
    let (mut game, a) = fixture(Vec2::new(720.0, 360.0), -Vec2::Y * 100.0);
    let p = game.position();
    let b = loop {
        let e = game.spawn_enemy(FAMILY, false, p + Vec2::X * 140.0, 1);
        if game.install_mote(e, regions::at(game.profile.seed, p + Vec2::X * 140.0)) {
            break e;
        }
        game.remove_enemy(e, false);
    };
    to_charge(&mut game, a);
    for e in [a, b] {
        // Declared phase synchronization fixture still runs the real controller,
        // emitter, cap and Game tick, rather than granting a shot or damage.
        let mut mote = game.world.get::<&mut Mote>(e).unwrap();
        mote.controller.force_charge_for_test();
    }
    for _ in 0..(MAX_BULLETS - 3) {
        game.spawn_bullet(p + Vec2::X * 1000.0, Vec2::ZERO, 1.0, false, 1);
    }
    for _ in 0..105 {
        tick(&mut game);
        assert!(game.world.query::<&Bullet>().iter().count() <= MAX_BULLETS);
    }
    assert_eq!(game.world.query::<&Bullet>().iter().count(), MAX_BULLETS);
    let shots = hostile(&game);
    for b in shots {
        let _ = game.world.despawn(b);
    }
    for _ in 0..45 {
        tick(&mut game);
        assert!(
            hostile(&game).is_empty(),
            "dropped pulse must not replay after capacity returns"
        );
    }
}

fn fight(game: &mut Game, e: Entity) -> usize {
    for step in 0..1800 {
        if !game.world.contains(e) {
            return step;
        }
        let target = game.world.get::<&Motion>(e).unwrap().position;
        let aim = (target - game.position()).normalize_or_zero() * 0.081;
        game.tick(
            &InputFrame {
                joystick: Some(aim.to_array()),
                fire: true,
                ..Default::default()
            },
            DT,
        );
        assert!(
            !game.take_events().iter().any(|event| event.kind == "death"),
            "declared targeting fixture died"
        );
    }
    panic!("real primary projectiles did not defeat ordinary Mote")
}

#[test]
fn actual_player_projectiles_grant_once_only_dart_xp_queue_and_bounty_credit() {
    let mut profile = Profile::default();
    profile.kills[0] = 23;
    profile.ranks[0] = 5;
    profile.ranks[1] = 5;
    profile.ranks[2] = 5;
    profile.campaign.ship_xp[0] = 1320;
    let mut game = Game::new(profile);
    let seed = game.profile.seed;
    let site = (-12..=12)
        .flat_map(|y| {
            (-12..=12).flat_map(move |x| {
                regions::sites_near(seed, Vec2::new(x as f32, y as f32) * regions::REGION_SIZE)
            })
        })
        .find(|s| s.event == "bounty" && s.family == 0)
        .expect("actual generated Dart bounty");
    // Accept through the real site API; no fabricated completed mission.
    game.move_player(Vec2::from_array(site.position));
    game.update_exploration();
    game.interact_site().unwrap();
    assert!(
        game.profile
            .quests
            .iter()
            .any(|q| q.site_id == site.id && q.progress == 0)
    );
    let (mut game, e) = setup(game, Vec2::new(640.0, 360.0), -Vec2::Y * 100.0);
    let currency = game.profile.currency;
    let time = fight(&mut game, e);
    assert!(time < 600);
    assert_eq!(game.profile.kills, [24, 0, 0]);
    assert_eq!(game.profile.campaign.ship_xp[0], 1325);
    assert_eq!(game.profile.pending, [true, false, false]);
    assert_eq!(game.kill_credit, [0, 0, 0]);
    assert_eq!(
        game.profile
            .quests
            .iter()
            .find(|q| q.site_id == site.id)
            .unwrap()
            .progress,
        1
    );
    assert_eq!(
        game.profile.currency, currency,
        "ordinary drops must be collected to become banked currency"
    );
    let before = rewards(&game);
    game.kill_actor(e, Kind::Enemy(0), game.position()); // Repeated callback after real defeat.
    game.remove_enemy(e, true);
    assert_eq!(rewards(&game), before);
}

#[test]
fn diagnostic_real_defeat_and_sanctuary_retreat_do_not_grant_any_progression() {
    let mut profile = Profile::default();
    profile.ranks[0] = 5;
    profile.ranks[1] = 5;
    profile.ranks[2] = 5;
    profile.campaign.ship_xp[0] = 1320;
    let mut game = Game::new(profile);
    game.diagnostic("biome-mote");
    game.spawn_timer = 10000.0;
    game.boss_timer = 10000.0;
    assert!(game.boss_qa_scene);
    let e = game
        .world
        .query::<(Entity, &Mote)>()
        .iter()
        .next()
        .unwrap()
        .0;
    let before = rewards(&game);
    fight(&mut game, e);
    assert_eq!(rewards(&game), before);
    game.reset_to_dock();
    assert!(!game.boss_qa_scene);
    let (mut game, e) = fixture(Vec2::new(640.0, 360.0), -Vec2::Y * 100.0);
    to_charge(&mut game, e);
    let before = rewards(&game);
    game.move_player(Vec2::ZERO);
    for _ in 0..120 {
        tick(&mut game);
        assert!(hostile(&game).is_empty());
    }
    assert!(!game.world.contains(e));
    assert_eq!(rewards(&game), before);
}

#[test]
fn sanctuary_body_and_all_lifetime_bullet_sweeps_use_shared_world_guards() {
    let (mut game, e) = fixture(Vec2::new(640.0, 360.0), -Vec2::Y * 100.0);
    to_charge(&mut game, e);
    let body_radius = game.world.get::<&Actor>(e).unwrap().radius;
    {
        let mut m = game.world.get::<&mut Motion>(e).unwrap();
        m.position = Vec2::new(SAFE_RADIUS + body_radius + 1.0, 0.0);
        m.velocity = -Vec2::X * 90.0;
    }
    game.move_player(Vec2::new(500.0, 0.0));
    game.set_viewport(1200.0, 360.0);
    tick(&mut game);
    assert!(
        game.world.get::<&Motion>(e).unwrap().position.length() >= SAFE_RADIUS + body_radius + 5.99
    );
    assert!(hostile(&game).is_empty());
    let site = regions::sites_near(game.profile.seed, radial(game.profile.seed))
        .into_iter()
        .next()
        .unwrap();
    let center = Vec2::from_array(site.position);
    game.move_player(center + Vec2::X * (site.radius + 120.0)); // Global refuge is checked independently of enemy cache.
    let before = rewards(&game);
    game.spawn_bullet(
        center - Vec2::X * (site.radius + 1.0),
        Vec2::X * (site.radius * 2.0 + 2.0) / DT,
        9.0,
        false,
        1,
    );
    tick(&mut game);
    assert!(
        hostile(&game).is_empty(),
        "endpoints outside sanctuary must still be rejected across its interior"
    );
    assert_eq!(rewards(&game), before);
}

#[test]
fn reused_render_extracts_live_cells_materials_and_bounded_cues_without_game_mutation() {
    let (mut game, e) = fixture(Vec2::new(640.0, 360.0), -Vec2::Y * 100.0);
    to_charge(&mut game, e);
    let motion = *game.world.get::<&Motion>(e).unwrap();
    let actor = *game.world.get::<&Actor>(e).unwrap();
    let mote = *game.world.get::<&Mote>(e).unwrap();
    let mut body = (*game.world.get::<&PixelBody>(e).unwrap()).clone();
    body.damage(Vec2::ZERO, 3.0);
    let cells = body.occupied_count();
    assert!(cells < 712);
    let mut frame = FrameBuilder::new();
    game.draw_mote(&mut frame, &body, motion, actor, mote);
    let packet = frame.frame();
    assert!(packet.pixel_count >= cells && packet.pixel_count <= cells + 20);
    assert_eq!(packet.batches[0].transforms[0].b[2], 0.75);
    let before = serde_json::to_value(&game.profile).unwrap();
    let time = game.time;
    for _ in 0..120 {
        game.render_into(0.5, &mut frame);
        assert!(frame.frame().pixel_count < 384000);
        for b in &frame.frame().batches {
            for t in &b.transforms {
                assert!(t.a.iter().chain(t.b.iter()).all(|n| n.is_finite()));
            }
        }
    }
    assert_eq!(game.time, time);
    assert_eq!(serde_json::to_value(&game.profile).unwrap(), before);
}

#[test]
fn invalid_actor_input_retires_without_rewards_instead_of_freezing() {
    let (mut game, e) = fixture(Vec2::new(640.0, 360.0), -Vec2::Y * 100.0);
    game.world.get::<&mut Motion>(e).unwrap().velocity = Vec2::X * 600.0;
    let before = rewards(&game);
    tick(&mut game);
    assert!(!game.world.contains(e));
    assert_eq!(rewards(&game), before);
}

#[test]
fn normal_game_tick_spawn_hook_installs_only_actual_radial_darts() {
    let (mut game, first) = fixture(Vec2::new(533.0 / 1.6, 300.0 / 1.6), -Vec2::Y * 100.0);
    game.remove_enemy(first, false);
    let mut motes = 0;
    let mut families = [0; 3];
    for _ in 0..128 {
        game.spawn_timer = 0.0;
        tick(&mut game);
        let enemies: Vec<_> = game
            .world
            .query::<(Entity, (&Brain, &Motion, Option<&Mote>))>()
            .iter()
            .map(|(e, (b, m, v))| (e, b.family, m.previous, v.copied()))
            .collect();
        assert_eq!(
            enemies.len(),
            1,
            "Mote must replace an ordinary spawn rather than add one"
        );
        for (e, family, spawn, mote) in enemies {
            families[family] += 1;
            if let Some(mote) = mote {
                motes += 1;
                let biome = regions::at(game.profile.seed, spawn);
                assert_eq!((family, biome.kind), (FAMILY, BIOME_KIND));
                assert_eq!(mote.controller.birth().grid, biome.grid);
            }
            game.remove_enemy(e, false);
        }
    }
    assert!(
        (6..35).contains(&motes),
        "natural selection {motes}/128 from families{families:?}"
    );
    assert!(families.iter().all(|n| *n > 0));
    assert_eq!(game.profile.kills, [0; 3]);
    assert_eq!(game.profile.campaign.ship_xp, [0; 10]);
}

#[test]
fn stress_mode_excludes_an_actual_eligible_birth_without_consuming_it() {
    let mut game = Game::new(Profile::default());
    let position = radial(game.profile.seed);
    let biome = regions::at(game.profile.seed, position);
    let entity = loop {
        let e = game.spawn_enemy(FAMILY, false, position, 1);
        let ticket = game.world.get::<&Brain>(e).unwrap().phase.to_bits();
        let birth = Birth {
            seed: game.profile.seed,
            grid: biome.grid,
            ticket,
            biome: biome.kind,
            family: FAMILY,
            ordinary: true,
        };
        if birth.eligible() {
            break e;
        }
        game.remove_enemy(e, false);
    };
    let rng = game.rng;
    let rewards = rewards(&game);
    game.stress_mode = true;
    assert!(!game.maybe_mote(entity, biome));
    assert!(game.world.get::<&Mote>(entity).is_err());
    assert_eq!(game.rng, rng);
    assert_eq!(self::rewards(&game), rewards);
    game.stress_mode = false;
    assert!(game.maybe_mote(entity, biome));
}

#[test]
fn moving_player_and_accepted_dt_keep_post_follow_warning_and_pulses_in_view() {
    let mut profile = Profile::default();
    profile.ranks[3] = 5;
    profile.ranks[4] = 5;
    for (ui, levelled, dt) in [Vec2::new(533.0, 300.0), Vec2::new(320.0, 180.0)]
        .into_iter()
        .flat_map(|ui| [false, true].map(|levelled| (ui, levelled)))
        .flat_map(|(ui, levelled)| [DT, 0.05].map(|dt| (ui, levelled, dt)))
    {
        profile.campaign.ship_xp[0] = if levelled { 1320 } else { 0 };
        let (mut game, e) = setup(Game::new(profile.clone()), ui / 1.6, -Vec2::Y * 100.0);
        to_charge(&mut game, e);
        let mut cues = 0;
        let mut emitted = 0;
        let mut recovered = 0;
        for step in 0..((20.0 / dt) as usize) {
            let seconds = step as f32 * dt;
            let before: std::collections::HashSet<_> = hostile(&game).into_iter().collect();
            let direction = if seconds % 2.0 < 1.0 {
                Vec2::X
            } else {
                -Vec2::X
            };
            game.tick(
                &InputFrame {
                    joystick: (seconds < 8.0).then_some(direction.to_array()),
                    ..Default::default()
                },
                dt,
            );
            assert!(
                game.world.contains(e),
                "declared reversal/release fixture lost enemy"
            );
            let phase = phase(&game, e);
            let new = hostile(&game).into_iter().any(|b| !before.contains(&b));
            if matches!(phase, Phase::Charge | Phase::Attack) || new {
                if seconds < 8.0 {
                    cues += 1;
                    emitted += new as usize;
                } else {
                    recovered += new as usize;
                }
                let m = *game.world.get::<&Motion>(e).unwrap();
                let a = *game.world.get::<&Actor>(e).unwrap();
                for alpha in [0.0, 0.5, 1.0] {
                    let position = m.previous.lerp(m.position, alpha);
                    let camera = Vec2::from_array(game.render_camera(alpha));
                    let delta = (position - camera).abs();
                    assert!(
                        delta.x + a.radius + WARNING_REACH + 5.0 < game.viewport.x * 0.5,
                        "post-follow X clipping at dt{dt}/step{step}"
                    );
                    assert!(
                        delta.y + a.radius + WARNING_REACH + 5.0 < game.viewport.y * 0.5,
                        "post-follow Y clipping at dt{dt}/step{step}"
                    );
                }
            }
            assert!(!game.take_events().iter().any(|event| event.kind == "death"));
        }
        if ui.x == 533.0 {
            assert!(cues > 0);
            assert!(
                emitted > 0,
                "ordinary maximum-zoom movement needs actual dispatch at each step size and ship level"
            );
        }
        // A narrow view can intentionally interrupt every moving warning.
        // Stopping must permit a fresh readable attack in every declared case.
        assert!(recovered > 0, "no real pulse after releasing movement");
        eprintln!(
            "moving-camera UI{ui:?}/levelled{levelled}/dt{dt}: {cues} visible moving ticks, {emitted} moving emissions, {recovered} recovered emissions"
        );
    }
}

fn generated_shot_for_hit(game: &mut Game, e: Entity) -> Entity {
    to_charge(game, e);
    for _ in 0..200 {
        tick(game);
        let enemy = game.world.get::<&Motion>(e).unwrap().position;
        if let Some(b) = hostile(game).into_iter().find(|b| {
            game.world
                .get::<&Motion>(*b)
                .unwrap()
                .position
                .distance(enemy)
                > 65.0
        }) {
            return b;
        }
    }
    panic!("real emitted shot did not travel outside body-contact range")
}
fn put_live_ship_cell_in_shot_path(game: &mut Game, b: Entity) {
    let shot = *game.world.get::<&Motion>(b).unwrap();
    let m = *game.world.get::<&Motion>(game.player).unwrap();
    let a = *game.world.get::<&Actor>(game.player).unwrap();
    let local = game
        .world
        .get::<&PixelBody>(game.player)
        .unwrap()
        .cells()
        .min_by(|a, b| a.0.length_squared().total_cmp(&b.0.length_squared()))
        .unwrap()
        .0;
    let offset = actor_transform(
        Motion {
            position: Vec2::ZERO,
            ..m
        },
        a,
    )
    .point(local);
    // Declared geometry placement fixture uses an actual surviving ship cell and
    // an actual Mote projectile; no direct hurt/damage method is invoked.
    game.move_player(shot.position + shot.velocity * DT - offset);
    game.invulnerability = 0.0;
    game.since_damage = 0.0;
}

#[test]
fn actual_emitted_sweep_hurts_hull_and_erases_ship_cells_through_shared_collision() {
    let (mut game, e) = fixture(Vec2::new(640.0, 360.0), -Vec2::Y * 100.0);
    let b = generated_shot_for_hit(&mut game, e);
    let damage = game.world.get::<&Bullet>(b).unwrap().damage;
    let hull = game.hud().hull;
    let cells = game
        .world
        .get::<&PixelBody>(game.player)
        .unwrap()
        .occupied_count();
    put_live_ship_cell_in_shot_path(&mut game, b);
    tick(&mut game);
    assert!((game.hud().hull - (hull - damage)).abs() < 0.001);
    assert!(
        game.world
            .get::<&PixelBody>(game.player)
            .unwrap()
            .occupied_count()
            < cells
    );
    assert!(game.take_events().iter().any(|event| event.kind == "hit"));
}

#[test]
fn actual_mote_sweep_triggers_owned_nova_and_reflects_other_real_pulse_projectiles() {
    let mut profile = Profile::default();
    profile.ranks[3] = 5;
    profile.ranks[4] = 5;
    profile.ranks[7] = 5;
    profile.unlocked[1] = true;
    profile.campaign.ship_xp[0] = 1320;
    profile.kills[1] = 16 * 24;
    profile.boss_generations[1] = [16, 8, 4, 2, 1];
    profile.boss_kills[1] = 31;
    let (mut game, e) = setup(
        Game::new(profile),
        Vec2::new(640.0, 360.0),
        -Vec2::Y * 100.0,
    );
    let b = generated_shot_for_hit(&mut game, e);
    let before: Vec<_> = hostile(&game)
        .into_iter()
        .map(|e| (e, game.world.get::<&Motion>(e).unwrap().velocity))
        .collect();
    let hull = game.hud().hull;
    let shield = game.shield;
    put_live_ship_cell_in_shot_path(&mut game, b);
    tick(&mut game);
    assert_eq!(game.hud().hull, hull);
    assert!(game.shield < shield);
    assert!(game.repulsor_timer > 0.0);
    let reflected: Vec<_> = before
        .into_iter()
        .filter(|(e, _)| game.world.get::<&Bullet>(*e).is_ok_and(|b| b.player))
        .collect();
    assert!(
        !reflected.is_empty(),
        "actual evolved nova should reflect nearby surviving pulse rays"
    );
    for (e, velocity) in reflected {
        let b = game.world.get::<&Bullet>(e).unwrap();
        assert_eq!(b.remaining_hits, 2);
        assert_eq!(game.world.get::<&Motion>(e).unwrap().velocity, -velocity);
    }
    assert!(
        game.take_events()
            .iter()
            .any(|event| event.kind == "shield")
    );
}

#[test]
fn actual_projectile_emissions_preserve_minimum_charge_and_pulse_spacing_at_supported_steps() {
    let mut proof = Vec::new();
    for (steps, expected) in [
        (vec![1.0_f32 / 120.0], Some((132, 42))),
        (vec![DT], Some((66, 21))),
        (vec![0.05], Some((22, 7))),
        (vec![0.05, 0.007, 0.031, 0.000_001, 0.04], None),
    ] {
        let (mut game, e) = fixture(Vec2::new(640.0, 360.0), -Vec2::Y * 100.0);
        to_charge(&mut game, e);
        assert_eq!(
            game.world.get::<&Mote>(e).unwrap().controller.phase_age(),
            0.0
        );
        let mut elapsed = 0.0_f64;
        let mut emitted = Vec::new();
        for step in 1..=1000 {
            let dt = steps[(step - 1) % steps.len()];
            let before: std::collections::HashSet<_> = hostile(&game).into_iter().collect();
            game.tick(&InputFrame::default(), dt);
            elapsed += f64::from(dt);
            let new = hostile(&game)
                .into_iter()
                .filter(|b| !before.contains(b))
                .count();
            assert!(new <= 6, "one pulse per tick, never a catch-up volley");
            if new != 0 {
                assert_eq!(
                    new, 6,
                    "declared intact body/cap fixture emits a whole real pulse"
                );
                emitted.push((step, elapsed));
            }
            if phase(&game, e) == Phase::Recovery {
                break;
            }
        }
        assert_eq!(emitted.len(), 2);
        assert!(
            emitted[0].1 >= 1.1,
            "fresh warning fired before authored charge"
        );
        assert!(
            emitted[1].1 - emitted[0].1 >= 0.35,
            "carried phase residual shortened actual spacing"
        );
        if let Some((first, separation)) = expected {
            assert_eq!(emitted[0].0, first);
            assert_eq!(emitted[1].0 - emitted[0].0, separation);
        }
        proof.push(
            serde_json::json!({"steps":steps,"first_tick":emitted[0].0,"second_tick":emitted[1].0,
            "charge_seconds":emitted[0].1,"pulse_spacing_seconds":emitted[1].1-emitted[0].1}),
        );
    }
    if let Ok(path) = std::env::var("GALAXY_MOTE_TIMING_PROOF") {
        std::fs::write(path, serde_json::to_string_pretty(&proof).unwrap()).unwrap();
    }
}
