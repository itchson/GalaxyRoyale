//! Focused real Game::tick regressions; no duplicated death/regen algorithm.
use super::*;

const DT: f32 = 1.0 / 60.0;

fn survival_profile() -> Profile {
    let mut profile = Profile {
        schema_version: 3, // migrate the declared existing fully installed hardware
        currency: [7, 8, 9],
        ranks: [5, 5, 5, 5, 5, 5, 0, 0, 0],
        ..Profile::default()
    };
    // Explicit existing-profile fixture, not a reward granted by any diagnostic.
    profile.campaign.ship_xp = [1320, 130, 240, 310, 420, 520, 620, 720, 820, 1320];
    profile.campaign.stored_materials = [11, 0, 0, 0, 0];
    profile
}

fn sovereign_profile() -> Profile {
    let mut profile = survival_profile();
    profile.campaign.patches = [true; 8];
    profile.campaign.blueprints = [true; 8];
    profile.campaign.crafted = [true; 8];
    profile.campaign.royale_results = [1; 8];
    profile.campaign.victory_claimed = true;
    profile.campaign.selected_ship = 9;
    profile
}

fn deaths(events: &[GameEvent]) -> usize {
    events.iter().filter(|e| e.kind == "death").count()
}

fn profile_json(game: &Game) -> serde_json::Value {
    serde_json::to_value(game.profile()).unwrap()
}

fn assert_death_return(game: &mut Game, events: &[GameEvent]) {
    assert_eq!(deaths(events), 1);
    assert_eq!(events.iter().filter(|e| e.kind == "dock").count(), 1);
    assert_eq!(game.position(), Vec2::ZERO);
    let hud = game.hud();
    assert!(hud.safe && hud.dock);
    assert_eq!(hud.hull, hud.max_hull);
    assert_eq!(hud.shield, hud.max_shield);
    assert!(hud.boss.is_none());
    assert!(game.world.query::<&Brain>().iter().next().is_none());
    assert!(game.world.query::<&Bullet>().iter().next().is_none());
    assert!(game.world.query::<&Loot>().iter().next().is_none());
}

#[test]
fn forced_death_cannot_be_undone_by_already_eligible_regeneration() {
    for rank in 0..=5 {
        for dt in [1.0 / 120.0, DT, 0.05, 10.0] {
            let mut profile = survival_profile();
            profile.ranks[4] = rank;
            let mut game = Game::new(profile);
            game.diagnostic("boss-dart");
            game.take_events();
            assert!(game.since_damage >= 5.0);
            let before = profile_json(&game);
            game.diagnostic("die");
            assert_eq!(game.hud().hull, 0.0);
            game.tick(&InputFrame::default(), dt);
            let events = game.take_events();
            assert_death_return(&mut game, &events);
            assert_eq!(profile_json(&game), before);
        }
    }
}

#[test]
fn forced_death_precedes_a_ready_sovereign_eighth_primary_heal() {
    let mut game = Game::new(sovereign_profile());
    assert_eq!(game.profile.campaign.selected_ship, 9);
    game.diagnostic("boss-bastion");
    game.take_events();
    let firing = InputFrame {
        fire: true,
        ..Default::default()
    };
    for _ in 0..120 {
        game.tick(&firing, DT);
        game.take_events();
        if game.shot_count == 7 {
            break;
        }
    }
    assert_eq!(game.shot_count, 7);
    while game.fire_timer > 0.0 {
        game.tick(&InputFrame::default(), DT);
        assert_eq!(deaths(&game.take_events()), 0);
    }
    assert!(game.since_damage >= 5.0);
    let before = profile_json(&game);
    game.diagnostic("die");
    game.tick(&firing, DT);
    let events = game.take_events();
    assert_death_return(&mut game, &events);
    assert_eq!(
        game.shot_count, 7,
        "A dead player fired its eighth healing signature"
    );
    assert!(!events.iter().any(|e| e.kind == "shot"));
    assert_eq!(profile_json(&game), before);
}

#[test]
fn pause_invalid_time_and_duplicate_commands_defer_one_death_until_a_valid_tick() {
    let mut game = Game::new(survival_profile());
    game.diagnostic("boss-coil-5");
    game.take_events();
    let time = game.time;
    game.diagnostic("die");
    game.diagnostic("die");
    game.set_paused(true);
    for _ in 0..20 {
        game.tick(&InputFrame::default(), DT);
    }
    assert_eq!(game.time, time);
    assert!(game.take_events().is_empty());
    assert_eq!(game.hud().hull, 0.0);
    game.set_paused(false);
    for dt in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        game.tick(&InputFrame::default(), dt);
        assert!(game.take_events().is_empty());
        assert_eq!(game.hud().hull, 0.0);
    }
    game.tick(&InputFrame::default(), DT);
    let events = game.take_events();
    assert_death_return(&mut game, &events);
    assert_eq!(
        game.time, time,
        "Already dead players do not gain simulated run time"
    );
    let after = profile_json(&game);
    for _ in 0..600 {
        game.tick(
            &InputFrame {
                fire: true,
                ..Default::default()
            },
            DT,
        );
        assert_eq!(deaths(&game.take_events()), 0);
    }
    assert_eq!(profile_json(&game), after);
    assert_eq!(game.shot_count, 0);
}

#[test]
fn diagnostic_scene_death_is_reward_isolated_and_an_explicit_retry_starts_once() {
    for scene in ["boss-dart-5", "boss-bastion-5", "boss-coil-5"] {
        let mut profile = survival_profile();
        profile.kills = [24, 0, 0];
        profile.pending = [true, false, false];
        profile.campaign.last_run = Some(RunSummary {
            time: 7.0,
            enemies: 2,
            xp_gained: 10,
            ..Default::default()
        });
        let mut game = Game::new(profile);
        let before = profile_json(&game);
        game.diagnostic(scene);
        let first_boss = game.active_boss.unwrap();
        game.take_events();
        game.diagnostic("die");
        game.tick(&InputFrame::default(), DT);
        let events = game.take_events();
        assert_death_return(&mut game, &events);
        assert!(!game.world.contains(first_boss));
        assert_eq!(profile_json(&game), before);
        for _ in 0..60 {
            game.tick(&InputFrame::default(), DT);
            assert_eq!(deaths(&game.take_events()), 0);
            assert!(game.active_boss.is_none());
        }
        game.diagnostic(scene);
        assert_eq!(game.world.query::<&BossEncounter>().iter().count(), 1);
        assert_eq!(
            game.take_events()
                .iter()
                .filter(|e| e.kind == "boss")
                .count(),
            1
        );
        assert_eq!(profile_json(&game), before);
    }
}

/// Authored fixture actors/pickups enter the real simulation; XP comes from a
/// real player projectile kill, currency/materials from real tick collection.
fn earned_run() -> Game {
    let mut game = Game::new(survival_profile());
    game.move_player(Vec2::new(500.0, 0.0));
    assert!(!game.safe());
    let before_xp = game.profile.campaign.ship_xp[0];
    let target = game.spawn_enemy(0, false, game.position() - Vec2::Y * 65.0, 1);
    for _ in 0..300 {
        let Some(target_position) = game.world.get::<&Motion>(target).ok().map(|m| m.position)
        else {
            break;
        };
        let aim = (target_position - game.position()).normalize_or_zero() * 0.081;
        game.tick(
            &InputFrame {
                joystick: Some(aim.to_array()),
                fire: true,
                ..Default::default()
            },
            DT,
        );
        assert_eq!(deaths(&game.take_events()), 0);
    }
    assert!(
        !game.world.contains(target),
        "Actual player shots failed to kill the authored enemy fixture"
    );
    assert_eq!(game.profile.campaign.ship_xp[0], before_xp + 5);
    assert_eq!(game.profile.kills[0], 1);
    for (category, amount) in [(0, 2), (1, 3), (2, 4), (3, 5)] {
        game.spawn_loot(game.position(), category, amount);
    }
    game.tick(&InputFrame::default(), DT);
    assert_eq!(deaths(&game.take_events()), 0);
    assert_eq!(game.profile.campaign.cargo_materials, [5, 0, 0, 0, 0]);
    let run = game.run.as_ref().unwrap();
    assert_eq!(run.enemies, 1);
    assert_eq!(run.xp_gained, 5);
    assert_eq!(run.materials_gained, 5);
    assert!(run.points_gained.iter().sum::<u32>() >= 9);
    game
}

#[test]
fn forced_normal_run_death_retains_earned_progress_and_finishes_report_once() {
    let mut game = earned_run();
    let before = game.profile.clone();
    let mut run = game.run.clone().unwrap();
    run.materials_lost = before.campaign.cargo_materials;
    let time = game.time;
    game.diagnostic("die");
    game.tick(&InputFrame::default(), DT);
    let events = game.take_events();
    assert_death_return(&mut game, &events);
    assert_eq!(game.time, time);
    let mut expected = before.clone();
    expected.campaign.last_run = Some(run.clone());
    expected.campaign.cargo_materials = [0, 0, 0, 0, 0];
    expected.revision += 2; // one actual cargo loss, one finished run
    assert_eq!(profile_json(&game), serde_json::to_value(expected).unwrap());
    assert_eq!(game.hud().last_run, Some(run));
    assert!(game.run.is_none());
    let after = profile_json(&game);
    for _ in 0..120 {
        game.tick(&InputFrame::default(), DT);
        assert_eq!(deaths(&game.take_events()), 0);
    }
    assert_eq!(profile_json(&game), after);
    let saved = serde_json::to_string(game.profile()).unwrap();
    let mut reload = Game::new(serde_json::from_str(&saved).unwrap());
    assert_eq!(profile_json(&reload), after);
    assert_eq!(reload.hud().last_run, game.hud().last_run);
    assert_eq!(deaths(&reload.take_events()), 0);
}

fn live_cell_center(game: &Game) -> Vec2 {
    let body = game.world.get::<&PixelBody>(game.player).unwrap();
    let local = body
        .cells()
        .min_by(|a, b| a.0.length_squared().total_cmp(&b.0.length_squared()))
        .unwrap()
        .0;
    let motion = *game.world.get::<&Motion>(game.player).unwrap();
    let actor = *game.world.get::<&Actor>(game.player).unwrap();
    actor_transform(motion, actor).point(local)
}

#[test]
fn natural_lethal_occupied_cell_collision_keeps_the_existing_end_of_tick_path() {
    let mut game = earned_run();
    // The actual enemy never reached this fixture; damage remains eligible.
    assert_eq!(game.invulnerability, 0.0);
    assert!(game.since_damage >= 5.0);
    let before = game.profile.clone();
    let mut expected_run = game.run.clone().unwrap();
    let time = game.time;
    let position = game.position();
    let stats = Stats::from_profile(&before);
    game.spawn_bullet(
        live_cell_center(&game),
        Vec2::ZERO,
        stats.hull + stats.shield + 100.0,
        false,
        1,
    );
    game.tick(&InputFrame::default(), DT);
    let events = game.take_events();
    assert_eq!(events.iter().filter(|e| e.kind == "hit").count(), 1);
    assert_eq!(events.iter().filter(|e| e.kind == "shield").count(), 1);
    assert_death_return(&mut game, &events);
    assert_eq!(
        game.since_damage, 0.0,
        "The real hurt_player collision reset was bypassed"
    );
    assert!((game.time - time - DT).abs() < 0.00001);
    expected_run.time += DT;
    expected_run.distance_peak = expected_run.distance_peak.max(position.length());
    expected_run.materials_lost = before.campaign.cargo_materials;
    let mut expected = before;
    expected.campaign.last_run = Some(expected_run);
    expected.campaign.cargo_materials = [0, 0, 0, 0, 0];
    expected.revision += 2;
    assert_eq!(profile_json(&game), serde_json::to_value(expected).unwrap());
}

#[test]
fn natural_nonlethal_shield_and_hull_damage_wait_then_regenerate_unchanged() {
    let mut game = Game::new(survival_profile());
    game.move_player(Vec2::new(500.0, 0.0));
    game.tick(&InputFrame::default(), DT);
    game.take_events();
    let initial = game.hud();
    game.spawn_bullet(
        live_cell_center(&game),
        Vec2::ZERO,
        initial.shield + 10.0,
        false,
        1,
    );
    game.tick(&InputFrame::default(), DT);
    let events = game.take_events();
    assert_eq!(deaths(&events), 0);
    assert_eq!(game.hud().hull, initial.hull - 10.0);
    assert_eq!(game.hud().shield, 0.0);
    assert_eq!(game.since_damage, 0.0);
    for _ in 0..299 {
        game.tick(&InputFrame::default(), DT);
        assert_eq!(deaths(&game.take_events()), 0);
    }
    assert_eq!(game.hud().hull, initial.hull - 10.0);
    for _ in 0..3 {
        game.tick(&InputFrame::default(), DT);
        game.take_events();
    }
    assert!(game.hud().hull > initial.hull - 10.0);
    assert!(game.hud().hull < initial.hull);
    assert!(game.hud().shield > 0.0);
}

#[test]
fn atomic_unplayed_diagnostic_departure_keeps_the_previous_report_and_haven_death_is_safe() {
    let mut profile = survival_profile();
    profile.campaign.last_run = Some(RunSummary {
        time: 4.0,
        enemies: 1,
        xp_gained: 5,
        ..Default::default()
    });
    let mut game = Game::new(profile);
    let before = profile_json(&game);
    game.diagnostic("combat");
    game.take_events();
    assert!(game.run.is_none());
    game.diagnostic("die");
    game.tick(&InputFrame::default(), DT);
    let events = game.take_events();
    assert_death_return(&mut game, &events);
    assert_eq!(
        profile_json(&game),
        before,
        "No simulated flight means no invented new report"
    );
    game.diagnostic("die");
    game.tick(
        &InputFrame {
            fire: true,
            ..Default::default()
        },
        DT,
    );
    let events = game.take_events();
    assert_death_return(&mut game, &events);
    assert_eq!(profile_json(&game), before);
}
