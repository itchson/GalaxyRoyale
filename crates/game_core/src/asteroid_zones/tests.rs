use super::*;

fn rock(game: &mut Game, p: Vec2, v: Vec2, birth: Option<AsteroidBirth>) -> Entity {
    let body = asteroid_body(31);
    let scale = 0.7;
    let e = game.world.spawn((
        Motion::new(p, v, 0.0),
        Actor {
            kind: Kind::Asteroid,
            health: 100.0,
            max_health: 100.0,
            radius: pixel_assets::radius(&body, scale),
            scale,
            damage: 8.0,
        },
        body,
    ));
    if let Some(b) = birth {
        game.world.insert_one(e, b).unwrap();
    }
    e
}

#[test]
fn all_asteroids_sweep_home_and_remote_shields_without_rewards_or_tunneling() {
    let mut game = Game::new(Profile::default());
    let site = regions::sites_near(game.profile.seed, Vec2::new(8000.0, 0.0)).remove(0);
    let before = serde_json::to_value(game.profile()).unwrap();
    for (center, shield) in [
        (Vec2::ZERO, SAFE_RADIUS),
        (Vec2::from_array(site.position), site.radius),
    ] {
        for angle in 0..16 {
            let direction = forward(angle as f32 * TAU / 16.0);
            let start = center + direction * (shield + 100.0);
            let e = rock(&mut game, start, -direction * 20_000.0, None);
            game.integrate(0.05);
            game.enforce_asteroid_shields(true);
            let m = *game.world.get::<&Motion>(e).unwrap();
            let radius = game.world.get::<&Actor>(e).unwrap().radius;
            assert!(m.position.distance(center) >= shield + radius);
            assert!(m.velocity.dot(direction) > 0.0);
            assert!(
                (m.velocity.length() - 14_000.0).abs() < 0.1,
                "angle {angle}: {}",
                m.velocity.length()
            );
            assert!(m.position.is_finite() && m.velocity.is_finite());
            game.world.despawn(e).unwrap();
        }
    }
    assert_eq!(serde_json::to_value(game.profile()).unwrap(), before);
    assert!(game.take_events().is_empty());
}

#[test]
fn inside_grazing_and_post_impulse_penetration_keep_real_body_clear() {
    let mut game = Game::new(Profile::default());
    let inside = rock(&mut game, Vec2::ZERO, -Vec2::X * 50.0, None);
    game.enforce_asteroid_shields(true);
    let a = *game.world.get::<&Actor>(inside).unwrap();
    let m = *game.world.get::<&Motion>(inside).unwrap();
    assert!(m.position.x >= SAFE_RADIUS + a.radius);
    assert!(m.velocity.x > 0.0);
    for alpha in [0.0, 0.25, 0.5, 0.75, 1.0] {
        assert!(m.previous.lerp(m.position, alpha).length() >= SAFE_RADIUS + a.radius);
    }
    game.world.despawn(inside).unwrap();
    let e = rock(&mut game, Vec2::new(400.0, 0.0), Vec2::ZERO, None);
    let radius = game.world.get::<&Actor>(e).unwrap().radius;
    {
        let mut m = game.world.get::<&mut Motion>(e).unwrap();
        m.previous = Vec2::new(-400.0, SAFE_RADIUS + radius + 0.1);
        m.position = Vec2::new(400.0, SAFE_RADIUS + radius + 0.1);
        m.velocity = Vec2::X * 16_000.0;
    }
    game.enforce_asteroid_shields(true);
    assert_eq!(
        game.world.get::<&Motion>(e).unwrap().velocity,
        Vec2::X * 16_000.0
    );
    {
        let mut m = game.world.get::<&mut Motion>(e).unwrap();
        m.position = Vec2::new(SAFE_RADIUS + radius - 5.0, 0.0);
        m.velocity = -Vec2::X * 100.0;
    }
    game.enforce_asteroid_shields(false);
    let m = game.world.get::<&Motion>(e).unwrap();
    assert!(m.position.length() >= SAFE_RADIUS + radius);
    assert!(m.velocity.x > 0.0);
    for alpha in [0.0, 0.25, 0.5, 0.75, 1.0] {
        assert!(m.previous.lerp(m.position, alpha).length() >= SAFE_RADIUS + radius);
    }
}

#[test]
fn reflected_long_path_queries_a_second_canonical_site_outside_original_corridor() {
    let mut game = Game::new(Profile::default());
    let first = regions::sites_near(game.profile.seed, Vec2::new(8000.0, 0.0)).remove(0);
    let center = Vec2::from_array(first.position);
    let mut proved = false;
    for offset in [[4, 0], [0, 4], [-4, 0], [0, -4], [4, 4], [-4, 4]] {
        let id = format!(
            "{}:{}",
            first.grid[0] + offset[0],
            first.grid[1] + offset[1]
        );
        let Some(second) = regions::site_by_id(game.profile.seed, &id) else {
            continue;
        };
        let target = Vec2::from_array(second.position);
        let direction = (target - center).normalize();
        let e = rock(
            &mut game,
            center + direction * (first.radius + 100.0),
            Vec2::ZERO,
            None,
        );
        let radius = game.world.get::<&Actor>(e).unwrap().radius;
        let start = game.world.get::<&Motion>(e).unwrap().position;
        let travel = (target.distance(center) + second.radius + radius + 400.0) / RESTITUTION
            + start.distance(center)
            - (first.radius + radius);
        let end = start - direction * travel;
        assert!(
            shield_circles(game.profile.seed, start, end, radius)
                .iter()
                .all(|(c, _)| c.distance(target) > 1.0),
            "fixture must put the second site outside original query coverage"
        );
        {
            let mut m = game.world.get::<&mut Motion>(e).unwrap();
            m.previous = start;
            m.position = end;
            m.velocity = -direction * travel / 0.05;
        }
        game.enforce_asteroid_shields(true);
        let m = *game.world.get::<&Motion>(e).unwrap();
        if (m.previous.distance(target) - (second.radius + radius + SLOP)).abs() < 0.02 {
            assert!(m.position.distance(target) >= second.radius + radius);
            assert!(m.velocity.dot(direction) < 0.0);
            proved = true;
        }
        game.world.despawn(e).unwrap();
        if proved {
            break;
        }
    }
    assert!(
        proved,
        "an actual seeded secondary refuge must reflect the redirected path"
    );
}

#[test]
fn belt_orbits_are_seeded_bounded_and_preserve_real_shot_impulse() {
    let mut game = Game::new(Profile::default());
    let p = Vec2::new(1050.0, 0.0);
    let birth = AsteroidBirth::at(game.profile.seed, p);
    let orbit = birth.orbit.unwrap();
    let e = rock(&mut game, p, orbit.tangent(p) * orbit.speed, Some(birth));
    let initial = *game.world.get::<&Motion>(e).unwrap();
    for _ in 0..3600 {
        game.update_asteroid_orbits(1.0 / 60.0);
        game.integrate(1.0 / 60.0);
        game.enforce_asteroid_shields(true);
        let m = game.world.get::<&Motion>(e).unwrap();
        assert!((BELT_INNER..COMBAT_RADIUS).contains(&m.position.length()));
        assert!(m.velocity.is_finite() && m.velocity.length() < 15.0);
    }
    assert!(
        game.world
            .get::<&Motion>(e)
            .unwrap()
            .position
            .distance(initial.position)
            > 200.0
    );
    let before = game.world.get::<&Motion>(e).unwrap().velocity;
    game.damage_actor(e, 1.0, Vec2::ZERO, Vec2::X * 180.0);
    let knocked = game.world.get::<&Motion>(e).unwrap().velocity;
    assert!(
        knocked.distance(before) > 10.0,
        "actual shot impulse remains live"
    );
    game.update_asteroid_orbits(1.0 / 60.0);
    assert!(
        game.world
            .get::<&Motion>(e)
            .unwrap()
            .velocity
            .distance(knocked)
            <= 0.201
    );
}

#[test]
fn actual_game_ticks_reflect_real_projectile_knockback_before_home_penetration() {
    let mut game = Game::new(Profile::default());
    let temporary = rock(&mut game, Vec2::new(400.0, 0.0), Vec2::ZERO, None);
    let radius = game.world.get::<&Actor>(temporary).unwrap().radius;
    let e = temporary;
    game.world.get::<&mut Motion>(e).unwrap().position = Vec2::new(SAFE_RADIUS + radius + 0.2, 0.0);
    {
        let mut m = game.world.get::<&mut Motion>(e).unwrap();
        m.previous = m.position;
    }
    let m = *game.world.get::<&Motion>(e).unwrap();
    let a = *game.world.get::<&Actor>(e).unwrap();
    let transform = actor_transform(m, a);
    let hit = game
        .world
        .get::<&PixelBody>(e)
        .unwrap()
        .cells()
        .map(|(p, _)| transform.point(p))
        .max_by(|a, b| a.x.total_cmp(&b.x))
        .unwrap();
    let count = game.world.get::<&PixelBody>(e).unwrap().occupied_count();
    let before = serde_json::to_value(game.profile()).unwrap();
    let hull = game.hud().hull;
    game.spawn_bullet(hit + Vec2::X * 3.0, -Vec2::X * 1800.0, 1.0, true, 1);
    game.tick(&InputFrame::default(), 1.0 / 60.0);
    assert_eq!(game.world.get::<&Actor>(e).unwrap().health, 99.0);
    assert!(game.world.get::<&PixelBody>(e).unwrap().occupied_count() < count);
    assert!(game.world.get::<&Motion>(e).unwrap().velocity.x < 0.0);
    let mut reflected = false;
    for _ in 0..60 {
        game.tick(&InputFrame::default(), 1.0 / 60.0);
        let m = game.world.get::<&Motion>(e).unwrap();
        assert!(m.position.length() >= SAFE_RADIUS + radius);
        if m.velocity.x > 0.0 {
            reflected = true;
            break;
        }
    }
    assert!(
        reflected,
        "real projectile momentum must reach and reflect at the shield"
    );
    assert_eq!(game.hud().hull, hull);
    assert_eq!(serde_json::to_value(game.profile()).unwrap(), before);
}

#[test]
fn remote_orbit_stays_in_its_site_annulus_and_invalid_velocity_recovers_finitely() {
    let mut game = Game::new(Profile::default());
    let site = regions::sites_near(game.profile.seed, Vec2::new(10_000.0, 0.0))
        .into_iter()
        .find(|s| regions::site_belt(game.profile.seed, s).is_some())
        .unwrap();
    let belt = regions::site_belt(game.profile.seed, &site).unwrap();
    let center = Vec2::from_array(site.position);
    let p = center + Vec2::X * ((belt.inner + belt.outer) * 0.5);
    let birth = AsteroidBirth::at(game.profile.seed, p);
    let e = rock(
        &mut game,
        p,
        Vec2::new(f32::NAN, f32::INFINITY),
        Some(birth),
    );
    for _ in 0..3600 {
        game.update_asteroid_orbits(1.0 / 60.0);
        game.integrate(1.0 / 60.0);
        game.enforce_asteroid_shields(true);
        let m = game.world.get::<&Motion>(e).unwrap();
        assert!((belt.inner..belt.outer).contains(&m.position.distance(center)));
        assert!(m.velocity.is_finite() && m.velocity.length() < 15.0);
    }
    assert!(game.world.get::<&Motion>(e).unwrap().position.distance(p) > 200.0);
    assert_eq!(
        game.world.get::<&AsteroidBirth>(e).unwrap().source,
        AsteroidSource::SiteBelt(site.grid)
    );
}

#[test]
fn canonical_belt_queries_and_streaming_are_deterministic_dense_and_safe() {
    let seed = Profile::default().seed;
    let mut a = Game::new(Profile::default());
    let mut b = Game::new(Profile::default());
    let site = regions::sites_near(seed, Vec2::new(10_000.0, 0.0))
        .into_iter()
        .find(|s| regions::site_belt(seed, s).is_some_and(|belt| belt.material_tier >= 4))
        .unwrap();
    let belt = regions::site_belt(seed, &site).unwrap();
    let center = Vec2::from_array(site.position);
    let p = center + Vec2::X * ((belt.inner + belt.outer) * 0.5);
    a.update_chunks(p);
    b.update_chunks(p);
    let snapshot = |g: &Game| {
        let mut records: Vec<_> = g
            .world
            .query::<(&Motion, &Actor, &AsteroidBirth)>()
            .iter()
            .map(|(m, actor, birth)| {
                assert!(m.position.length() >= SAFE_RADIUS + actor.radius);
                assert!(
                    regions::sites_near(seed, m.position)
                        .iter()
                        .all(|s| m.position.distance(Vec2::from_array(s.position))
                            >= s.radius + actor.radius)
                );
                (
                    m.position.x.to_bits(),
                    m.position.y.to_bits(),
                    birth.material_tier,
                    birth.source,
                    m.velocity.x.to_bits(),
                    m.velocity.y.to_bits(),
                )
            })
            .collect();
        records.sort_by_key(|r| (r.0, r.1));
        records
    };
    let first = snapshot(&a);
    assert_eq!(first, snapshot(&b));
    assert!(first.len() <= 112);
    assert!(
        first
            .iter()
            .filter(|r| r.3 == AsteroidSource::SiteBelt(site.grid))
            .count()
            >= 8
    );
    let born = AsteroidBirth::at(seed, p);
    assert_eq!(born.source, AsteroidSource::SiteBelt(site.grid));
    assert_eq!(born.material_tier, belt.material_tier);
    assert_eq!(born.orbit.unwrap().center, center);
    let mut fake = site.clone();
    fake.radius += 1.0;
    assert!(regions::site_belt(seed, &fake).is_none());
    assert_eq!(
        AsteroidBirth::at(seed, Vec2::new(1000.0, 0.0)).material_tier,
        1
    );
}

#[test]
fn streamed_remote_birth_keeps_tier_after_declared_reposition_and_real_projectile_defeat() {
    let mut game = Game::new(Profile::default());
    let site = regions::sites_near(game.profile.seed, Vec2::new(10_000.0, 0.0))
        .into_iter()
        .find(|s| regions::site_belt(game.profile.seed, s).is_some_and(|b| b.material_tier >= 4))
        .unwrap();
    let belt = regions::site_belt(game.profile.seed, &site).unwrap();
    let center = Vec2::from_array(site.position);
    game.move_player(center + Vec2::X * ((belt.inner + belt.outer) * 0.5));
    game.update_chunks(game.position());
    let rocks: Vec<_> = game
        .world
        .query::<(Entity, &AsteroidBirth)>()
        .iter()
        .filter(|(_, b)| b.source == AsteroidSource::SiteBelt(site.grid))
        .map(|(e, _)| e)
        .collect();
    assert!(rocks.len() >= 8);
    let mut defeats = 0;
    for e in rocks {
        let born = *game.world.get::<&AsteroidBirth>(e).unwrap();
        // Controlled reposition to a different global material tier: this proves
        // immutable streamed provenance, not a naturally travelled drift route.
        // Real occupied-cell projectile collisions still perform every defeat.
        {
            let mut m = game.world.get::<&mut Motion>(e).unwrap();
            m.position = Vec2::new(1000.0, 0.0);
            m.previous = m.position;
            m.velocity = Vec2::ZERO;
        }
        let m = *game.world.get::<&Motion>(e).unwrap();
        let actor = *game.world.get::<&Actor>(e).unwrap();
        let local = game
            .world
            .get::<&PixelBody>(e)
            .unwrap()
            .cells()
            .next()
            .unwrap()
            .0;
        game.spawn_bullet(
            actor_transform(m, actor).point(local),
            Vec2::ZERO,
            1000.0,
            true,
            1,
        );
        game.resolve_collisions(1.0 / 60.0);
        assert!(!game.world.contains(e));
        assert_eq!(born.material_tier, belt.material_tier);
        defeats += 1;
    }
    assert_eq!(game.profile.campaign.ship_xp[0], defeats * 2);
    let materials: Vec<_> = game
        .world
        .query::<&Loot>()
        .iter()
        .filter(|l| l.category == 3)
        .map(|l| l.material_tier)
        .collect();
    assert!(!materials.is_empty());
    assert!(materials.iter().all(|t| *t == belt.material_tier));
    game.boss_qa_scene = true;
    let before = serde_json::to_value(game.profile()).unwrap();
    let birth = AsteroidBirth::at(game.profile.seed, center + Vec2::X * (belt.inner + 10.0));
    let e = rock(&mut game, Vec2::new(1000.0, 0.0), Vec2::ZERO, Some(birth));
    let local = game
        .world
        .get::<&PixelBody>(e)
        .unwrap()
        .cells()
        .next()
        .unwrap()
        .0;
    let m = *game.world.get::<&Motion>(e).unwrap();
    let a = *game.world.get::<&Actor>(e).unwrap();
    game.spawn_bullet(
        actor_transform(m, a).point(local),
        Vec2::ZERO,
        1000.0,
        true,
        1,
    );
    game.resolve_collisions(1.0 / 60.0);
    assert!(!game.world.contains(e));
    assert_eq!(serde_json::to_value(game.profile()).unwrap(), before);
}
