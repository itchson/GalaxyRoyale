//! Live overlays use neutral input, not a simulation pause or a ship teleport.
use super::*;
const DT: f32 = 1.0 / 60.0;

#[test]
fn cancel_course_preserves_live_state_and_only_removes_click_navigation() {
    let mut game = Game::new(Profile::default());
    game.move_player(Vec2::new(1600.0, 0.0));
    for n in 0..8 {
        game.tick(
            &InputFrame {
                destination: (n == 0).then_some([2200.0, 0.0]),
                ..Default::default()
            },
            DT,
        );
    }
    assert!(game.destination.is_some());
    let before = *game.world.get::<&Motion>(game.player).unwrap();
    assert!(before.velocity.length() > 1.0);
    let profile = serde_json::to_value(game.profile()).unwrap();
    let time = game.time;
    let throttle = game.throttle;
    let cooldown = game.fire_timer;
    let rng = game.rng;
    let events = serde_json::to_value(&game.events).unwrap();
    game.cancel_course();
    game.cancel_course();
    let after = *game.world.get::<&Motion>(game.player).unwrap();
    assert_eq!(after.position, before.position);
    assert_eq!(after.velocity, before.velocity);
    assert_eq!(after.heading, before.heading);
    assert_eq!(game.time, time);
    assert_eq!(game.throttle, throttle);
    assert_eq!(game.fire_timer, cooldown);
    assert_eq!(game.rng, rng);
    assert!(!game.paused && game.destination.is_none());
    assert_eq!(serde_json::to_value(game.profile()).unwrap(), profile);
    assert_eq!(serde_json::to_value(&game.events).unwrap(), events);
    for _ in 0..24 {
        game.tick(&InputFrame::default(), DT);
    }
    let final_motion = *game.world.get::<&Motion>(game.player).unwrap();
    assert!(game.time > time + 0.39);
    assert!(final_motion.position.distance(before.position) > 0.1);
    assert!(final_motion.velocity.length() < before.velocity.length() * 0.25);
    assert!(game.destination.is_none());
}

#[test]
fn neutral_live_ticks_move_enemies_accept_real_hits_and_resolve_death_once() {
    let mut game = Game::new(Profile::default());
    game.move_player(Vec2::new(1600.0, 0.0));
    game.invulnerability = 0.0;
    game.shield = 0.0; // Declared depleted shield preserves this real hull/holes probe.
    game.profile.campaign.cargo_materials = [1, 2, 0, 0, 0];
    game.profile.currency = [7, 8, 9];
    let enemy = game.spawn_enemy(0, false, game.position() + Vec2::X * 160.0, 1);
    let enemy_start = game.world.get::<&Motion>(enemy).unwrap().position;
    let player = *game.world.get::<&Motion>(game.player).unwrap();
    let actor = *game.world.get::<&Actor>(game.player).unwrap();
    let occupied = game
        .world
        .get::<&PixelBody>(game.player)
        .unwrap()
        .cells()
        .min_by(|a, b| a.0.length_squared().total_cmp(&b.0.length_squared()))
        .unwrap()
        .0;
    let contact = actor_transform(player, actor).point(occupied);
    let cells_before = game
        .world
        .get::<&PixelBody>(game.player)
        .unwrap()
        .occupied_count();
    game.spawn_bullet(contact + Vec2::X * 28.0, -Vec2::X * 360.0, 12.0, false, 1);
    game.take_events();
    for _ in 0..8 {
        game.tick(&InputFrame::default(), DT);
    }
    assert!(game.time >= 0.13);
    assert_ne!(
        game.world.get::<&Motion>(enemy).unwrap().position,
        enemy_start
    );
    assert!(game.hud().hull < 100.0 && game.hud().hull > 0.0);
    assert!(
        game.world
            .get::<&PixelBody>(game.player)
            .unwrap()
            .occupied_count()
            < cells_before
    );
    assert!(game.take_events().iter().any(|e| e.kind == "hit"));
    let position = game.position();
    game.invulnerability = 0.0;
    game.spawn_bullet(
        position + Vec2::X * 28.0,
        -Vec2::X * 360.0,
        1000.0,
        false,
        1,
    );
    let mut deaths = 0;
    for _ in 0..30 {
        game.tick(&InputFrame::default(), DT);
        deaths += game
            .take_events()
            .iter()
            .filter(|e| e.kind == "death")
            .count();
    }
    assert_eq!(deaths, 1);
    assert_eq!(game.position(), Vec2::ZERO);
    assert!(game.hud().dock && game.hud().hull > 0.0);
    assert_eq!(game.profile.currency, [7, 8, 9]);
    assert_eq!(game.profile.campaign.cargo_materials, [0; 5]);
    assert_eq!(game.hud().material_loss, [1, 2, 0, 0, 0]);
    assert!(!game.paused);
}

#[test]
fn live_ship_stats_match_effective_level_caps_actual_velocity_and_primary_projectiles() {
    for xp in [0, 60, 200, 560, 1320] {
        let mut profile = Profile::default();
        profile.campaign.ship_xp[0] = xp;
        profile.campaign.loadouts[0].hardware = [5; 6];
        profile.ranks[..6].fill(5);
        let mut game = Game::new(profile);
        game.move_player(Vec2::new(500.0, 0.0));
        game.world.get::<&mut Motion>(game.player).unwrap().velocity = Vec2::new(36.0, 48.0);
        let hud = game.hud();
        let cap = campaign::rank_cap(campaign::level(xp)) as f32;
        let level_drive = 1.0 + 0.025 * campaign::level(xp).min(12).saturating_sub(1) as f32;
        let near = |a: f32, b: f32| assert!((a - b).abs() < 0.0001, "{a} != {b} / XP {xp}");
        near(hud.ship_stats.speed, 60.0);
        near(
            hud.ship_stats.top_speed,
            100.0 * level_drive * (1.0 + 0.04 * cap),
        );
        near(
            hud.ship_stats.acceleration,
            260.0 * level_drive * (1.0 + 0.04 * cap),
        );
        near(hud.ship_stats.primary_damage, 12.0 * (1.0 + 0.12 * cap));
        near(hud.ship_stats.fire_rate, 5.0 * (1.0 + 0.1 * cap));
        near(hud.ship_stats.hull_regen, 0.4 * cap);
        near(
            hud.ship_stats.shield_regen,
            3.0 * (1.0 + 0.2 * (cap - 1.0))
                * (1.0 + 0.03 * campaign::level(xp).min(12).saturating_sub(1) as f32),
        );
        let serialized = serde_json::to_value(&hud.ship_stats).unwrap();
        assert_eq!(serialized.as_object().unwrap().len(), 8);
        assert_eq!(serialized["shield_recharge_unlocked"], true);
        assert!(
            serialized
                .as_object()
                .unwrap()
                .iter()
                .filter(|(key, _)| key.as_str() != "shield_recharge_unlocked")
                .map(|(_, value)| value)
                .all(|v| v.as_f64().is_some_and(f64::is_finite))
        );
        game.tick(
            &InputFrame {
                fire: true,
                ..Default::default()
            },
            DT,
        );
        let actual = game.hud().ship_stats;
        near(
            actual.speed,
            game.world
                .get::<&Motion>(game.player)
                .unwrap()
                .velocity
                .length(),
        );
        let bullets: Vec<_> = game
            .world
            .query::<&Bullet>()
            .iter()
            .filter(|b| b.player)
            .map(|b| b.damage)
            .collect();
        assert_eq!(bullets.len(), 1);
        near(bullets[0], actual.primary_damage);
        near(game.fire_timer, 1.0 / actual.fire_rate);
    }
}
