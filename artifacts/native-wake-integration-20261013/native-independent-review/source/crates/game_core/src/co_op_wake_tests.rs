//! Native Ion Wake tests. Fixtures arrange real profiles, motion and target ECS.
use super::*;
use serde_json::{Value, json};

fn id(n: u32) -> ParticipantId {
    ParticipantId::parse(&format!("{n:032x}")).unwrap()
}
fn session() -> ParticipantId {
    id(100)
}
fn profile(evolution: u8, rank: u8, xp: u32) -> Profile {
    let mut p = identity::create_profile("talo", "pan", "renko", "0123456789abcdef").unwrap();
    p.campaign.ship_xp[0] = xp;
    p.campaign.abilities[5] = campaign::AbilityState {
        blueprint: true,
        rank,
    };
    p.campaign.loadouts[0].slots[2] = Some(5);
    p.boss_generations[2] = std::array::from_fn(|i| {
        if i < evolution as usize {
            1 << (evolution as usize - 1 - i)
        } else {
            0
        }
    });
    p
}
fn quiet(game: &mut Game) {
    let remove: Vec<_> = game
        .world
        .query::<(Entity, Option<&Pilot>)>()
        .iter()
        .filter(|(e, p)| *e != game.player && p.is_none())
        .map(|(e, _)| e)
        .collect();
    for e in remove {
        game.world.despawn(e).unwrap();
    }
    game.spawn_timer = 10_000.0;
    for position in game
        .co_op_positions()
        .into_iter()
        .flatten()
        .chain([game.position()])
    {
        let center = (position / CHUNK).floor().as_ivec2();
        for dy in -1..=1 {
            for dx in -1..=1 {
                game.chunks
                    .entry((center.x + dx, center.y + dy))
                    .or_default();
            }
        }
    }
    game.take_events();
}
fn target(game: &mut Game, position: Vec2, health: f32, kind: Kind) -> Entity {
    game.world.spawn((
        Motion::new(position, Vec2::ZERO, 0.0),
        Actor {
            kind,
            health,
            max_health: health,
            radius: 3.0,
            scale: 1.0,
            damage: 0.0,
        },
        PixelBody::from_ascii(&["####"; 4], &[('#', rgba(130, 115, 100, 255))]),
    ))
}

fn spec(p: &Profile, n: u32) -> ParticipantSpec {
    ParticipantSpec {
        id: id(n),
        local_seed: p.seed,
        identity: p.identity.clone(),
    }
}
fn entity(game: &Game, peer: ParticipantId) -> Entity {
    game.world
        .query::<(Entity, &Pilot)>()
        .iter()
        .find(|(_, p)| p.admission.id == peer)
        .unwrap()
        .0
}
fn start(profiles: &[Profile]) -> (Game, Vec<Admission>) {
    let mut game = Game::new(profiles[0].clone());
    let mut admissions = vec![game.begin_co_op(PROTOCOL, session(), id(1)).unwrap()];
    for (i, p) in profiles.iter().enumerate().skip(1) {
        let source = Game::new(p.clone());
        admissions.push(
            game.join_co_op_owned(
                session(),
                spec(source.profile(), i as u32 + 1),
                source.co_op_owned_loadout().unwrap(),
            )
            .unwrap(),
        );
    }
    quiet(&mut game);
    (game, admissions)
}
fn place(game: &mut Game, e: Entity, position: Vec2, speed_fraction: f32) {
    let speed = game.world.get::<&Pilot>(e).map_or_else(
        |_| Stats::from_profile(&game.profile).speed,
        |p| p.stats.speed,
    );
    *game.world.get::<&mut Motion>(e).unwrap() =
        Motion::new(position, Vec2::X * speed * speed_fraction, 0.0);
    if let Ok(mut p) = game.world.get::<&mut Pilot>(e) {
        p.was_home = position.length_squared() <= SAFE_RADIUS * SAFE_RADIUS;
    }
    game.refresh_sites();
}
fn origins(game: &Game) -> [Option<(Entity, Vec2)>; MAX_PARTICIPANTS] {
    let mut result = [None; MAX_PARTICIPANTS];
    for (e, (p, m)) in game.world.query::<(Entity, (&Pilot, &Motion))>().iter() {
        result[p.admission.slot as usize] = Some((e, m.position));
    }
    result
}
fn advance_wakes(game: &mut Game, dt: f32) {
    game.update_co_op_wakes(origins(game), dt);
}
fn close(a: f32, b: f32) {
    assert!((a - b).abs() < 0.0002, "{a} != {b}");
}

#[test]
fn ordinary_rank_and_evolution_match_solo_damage_cadence_samples_and_expiry() {
    // Only the real wake stages are isolated here; full host/guest ticks are
    // tested separately. These are configured builds, not earned progression.
    for evolution in 1..=5 {
        for rank in 1..=5 {
            let p = profile(evolution, rank, 1560);
            let mut solo = Game::new(p.clone());
            let (mut coop, _) = start(&[p]);
            let solo_player = solo.player;
            let coop_player = coop.player;
            place(&mut solo, solo_player, Vec2::new(800.0, 0.0), 0.75);
            place(&mut coop, coop_player, Vec2::new(800.0, 0.0), 0.75);
            quiet(&mut solo);
            let a = target(&mut solo, Vec2::new(769.0, 0.0), 1000.0, Kind::Enemy(0));
            let b = target(&mut coop, Vec2::new(769.0, 0.0), 1000.0, Kind::Enemy(0));
            let body_before = format!("{:?}", &*coop.world.get::<&PixelBody>(b).unwrap());
            for tick in 0..360 {
                solo.update_wake(solo.position(), TICK_SECONDS);
                advance_wakes(&mut coop, TICK_SECONDS);
                let pilot = coop.world.get::<&Pilot>(coop_player).unwrap();
                assert_eq!(
                    pilot.wake, solo.wake,
                    "evolution {evolution}, rank {rank}, tick {tick}"
                );
                close(pilot.wake_timer, solo.wake_timer);
                close(
                    coop.world.get::<&Actor>(b).unwrap().health,
                    solo.world.get::<&Actor>(a).unwrap().health,
                );
            }
            assert_eq!(
                format!("{:?}", &*coop.world.get::<&PixelBody>(b).unwrap()),
                body_before
            );
            let expected = 1000.0 - 360.0 * 6.0 * (1.0 + 0.25 * (rank - 1) as f32) * TICK_SECONDS;
            assert!((coop.world.get::<&Actor>(b).unwrap().health - expected).abs() < 0.03);
            place(&mut solo, solo_player, Vec2::new(800.0, 0.0), 0.0);
            place(&mut coop, coop_player, Vec2::new(800.0, 0.0), 0.0);
            for _ in 0..210 {
                solo.update_wake(solo.position(), TICK_SECONDS);
                advance_wakes(&mut coop, TICK_SECONDS);
                assert_eq!(
                    coop.world.get::<&Pilot>(coop_player).unwrap().wake,
                    solo.wake
                );
            }
            assert!(
                coop.world
                    .get::<&Pilot>(coop_player)
                    .unwrap()
                    .wake
                    .is_empty()
            );
        }
    }
    println!(
        "actual_solo_differential=25 builds; 360 active + 210 stationary fixed wake stages each"
    );
}

#[test]
fn eight_real_pilot_ticks_keep_builds_independent_and_resources_bounded() {
    let profiles: Vec<_> = (0..8)
        .map(|i| profile((i % 5 + 1) as u8, 5, if i == 0 { 0 } else { 1560 }))
        .collect();
    let (mut game, admissions) = start(&profiles);
    let saved = serde_json::to_string(game.profile()).unwrap();
    for a in &admissions {
        let e = entity(&game, a.id);
        place(&mut game, e, Vec2::new(800.0, a.slot as f32 * 100.0), 0.75);
        let p = game.world.get::<&Pilot>(e).unwrap();
        assert_eq!(
            (p.wake_rank, p.wake_evolution),
            (if a.slot == 0 { 1 } else { 5 }, a.slot % 5 + 1)
        );
    }
    quiet(&mut game);
    for sequence in 1..=300 {
        for a in &admissions {
            let input = InputFrame {
                joystick: Some([1.0, 0.0]),
                ..Default::default()
            };
            if a.slot == 0 {
                game.submit_host_co_op_input(session(), sequence, input)
                    .unwrap();
            } else {
                game.submit_guest_co_op_input(session(), a.id, a.epoch, sequence, input)
                    .unwrap();
            }
        }
        game.tick_co_op(session()).unwrap();
        let mut total = 0;
        for (_, p) in game.world.query::<(Entity, &Pilot)>().iter() {
            let (cap, ttl) = match p.wake_evolution {
                1 => (12, 2.0),
                2 | 3 => (16, 2.6),
                _ => (24, 3.4),
            };
            assert!(!p.wake.is_empty());
            assert!(p.wake.len() <= cap);
            assert!(
                p.wake
                    .iter()
                    .all(|(position, life)| position.is_finite() && *life > 0.0 && *life <= ttl)
            );
            total += p.wake.len();
        }
        assert!(total <= 192 && game.transient_lights.len() <= 32 && game.events.len() <= 128);
        assert_eq!(game.world.query::<&ShotOwner>().iter().count(), 0);
    }
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), saved);
    println!(
        "actual_real_coop_ticks=300; pilots=8; samples<=192; lights<=32; events<=128; wake_projectiles=0"
    );
}

#[test]
fn threshold_and_ordinary_equipment_gate_do_not_activate_champion_wake() {
    let (mut game, _) = start(&[profile(5, 5, 1560)]);
    let player = game.player;
    place(&mut game, player, Vec2::new(800.0, 0.0), 0.5);
    advance_wakes(&mut game, TICK_SECONDS);
    assert!(game.world.get::<&Pilot>(player).unwrap().wake.is_empty());
    place(&mut game, player, Vec2::new(800.0, 0.0), 0.501);
    advance_wakes(&mut game, TICK_SECONDS);
    assert_eq!(game.world.get::<&Pilot>(player).unwrap().wake.len(), 1);
    let mut p = profile(5, 5, 1560);
    p.campaign.loadouts[0].slots[2] = None;
    let (mut unequipped, _) = start(&[p]);
    let player = unequipped.player;
    place(&mut unequipped, player, Vec2::new(800.0, 0.0), 1.0);
    advance_wakes(&mut unequipped, TICK_SECONDS);
    assert_eq!(unequipped.world.get::<&Pilot>(player).unwrap().wake_rank, 0);
    assert!(
        unequipped
            .world
            .get::<&Pilot>(player)
            .unwrap()
            .wake
            .is_empty()
    );
    // Windrunner is a separate champion ID, outside this ordinary family scope.
    let mut p = profile(5, 5, 1560);
    p.campaign.patches[4] = true;
    p.campaign.blueprints[4] = true;
    p.campaign.crafted[4] = true;
    p.campaign.selected_ship = 5;
    p.campaign.ship_xp[5] = 1560;
    p.campaign.abilities[10] = campaign::AbilityState {
        blueprint: true,
        rank: 5,
    };
    p.campaign.loadouts[5].slots[2] = Some(10);
    let (mut champion, _) = start(&[p]);
    let player = champion.player;
    place(&mut champion, player, Vec2::new(800.0, 0.0), 1.0);
    advance_wakes(&mut champion, TICK_SECONDS);
    assert_eq!(champion.world.get::<&Pilot>(player).unwrap().wake_rank, 0);
    assert!(
        champion
            .world
            .get::<&Pilot>(player)
            .unwrap()
            .wake
            .is_empty()
    );
}

#[test]
fn wake_targets_only_hostiles_and_preserves_boss_armor_and_pixel_bodies() {
    let (mut game, _) = start(&[profile(1, 5, 1560)]);
    let player = game.player;
    place(&mut game, player, Vec2::new(800.0, 0.0), 0.75);
    let position = Vec2::new(786.0, 0.0);
    let enemy = target(&mut game, position, 100.0, Kind::Enemy(0));
    let asteroid = target(&mut game, position, 100.0, Kind::Asteroid);
    let friendly = target(&mut game, position, 100.0, Kind::Player);
    let boss = target(&mut game, position, 100.0, Kind::Boss(0));
    game.world
        .insert(
            boss,
            (
                Brain {
                    family: 0,
                    boss: true,
                    age: 0.0,
                    shot_timer: 99.0,
                    phase: 0.0,
                },
                BossEncounter {
                    generation: 1,
                    phase: bosses::Phase::Tell,
                    phase_age: 0.0,
                    shot_timer: 99.0,
                    aim: Vec2::ZERO,
                    origin: position,
                    cycle: 0,
                    diagnostic: false,
                },
            ),
        )
        .unwrap();
    advance_wakes(&mut game, TICK_SECONDS);
    close(game.world.get::<&Actor>(enemy).unwrap().health, 99.8);
    close(game.world.get::<&Actor>(boss).unwrap().health, 99.89);
    close(game.world.get::<&Actor>(asteroid).unwrap().health, 100.0);
    close(game.world.get::<&Actor>(friendly).unwrap().health, 100.0);
    assert_eq!(
        game.world
            .get::<&PixelBody>(enemy)
            .unwrap()
            .occupied_count(),
        16
    );
    game.world.get::<&mut BossEncounter>(boss).unwrap().phase = bosses::Phase::Recover;
    advance_wakes(&mut game, TICK_SECONDS);
    close(game.world.get::<&Actor>(boss).unwrap().health, 99.69);
}

#[test]
fn guest_wake_real_destruction_has_membership_attribution_without_save_grants() {
    let (mut game, admissions) = start(&[Profile::default(), profile(5, 5, 1560)]);
    let a = admissions[1];
    let guest = entity(&game, a.id);
    let position = Vec2::new(800.0, 0.0);
    place(&mut game, guest, position, 0.75);
    quiet(&mut game);
    let enemy = target(&mut game, position - Vec2::X * 31.0, 0.05, Kind::Enemy(2));
    let saved = serde_json::to_string(game.profile()).unwrap();
    let enclosing = ShotOwner {
        id: id(99),
        epoch: 42,
    };
    game.co_op.as_mut().unwrap().hit_owner = Some(enclosing);
    game.submit_guest_co_op_input(
        session(),
        a.id,
        a.epoch,
        1,
        InputFrame {
            joystick: Some([1.0, 0.0]),
            ..Default::default()
        },
    )
    .unwrap();
    game.tick_co_op(session()).unwrap();
    assert!(!game.world.contains(enemy));
    let notices = game.take_co_op_destructions(session()).unwrap();
    let n = notices.iter().find(|n| n.id == entity_key(enemy)).unwrap();
    assert_eq!((n.owner, n.owner_epoch), (Some(a.id), Some(a.epoch)));
    let restored = game.co_op.as_ref().unwrap().hit_owner.unwrap();
    assert_eq!(
        (restored.id, restored.epoch),
        (enclosing.id, enclosing.epoch)
    );
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), saved);
    assert!(game.take_events().iter().any(|e| e.kind == "ability-wake"));
    println!(
        "actual_guest_wake_destroyed_entity={}; owner={}; epoch={}",
        n.id,
        a.id.as_hex(),
        a.epoch
    );
}

#[test]
fn safe_owner_and_safe_targets_age_trails_without_damage_or_birth() {
    let (mut game, _) = start(&[profile(5, 5, 1560)]);
    let player = game.player;
    place(&mut game, player, Vec2::ZERO, 1.0);
    let enemy = target(&mut game, Vec2::new(800.0, 0.0), 100.0, Kind::Enemy(0));
    game.world
        .get::<&mut Pilot>(player)
        .unwrap()
        .wake
        .push((Vec2::new(800.0, 0.0), 1.0));
    advance_wakes(&mut game, TICK_SECONDS);
    close(game.world.get::<&Actor>(enemy).unwrap().health, 100.0);
    close(
        game.world.get::<&Pilot>(player).unwrap().wake[0].1,
        1.0 - TICK_SECONDS,
    );
    assert_eq!(game.world.get::<&Pilot>(player).unwrap().wake.len(), 1);
    place(&mut game, player, Vec2::new(800.0, 0.0), 0.0);
    let protected = target(&mut game, Vec2::ZERO, 100.0, Kind::Enemy(0));
    game.world.get::<&mut Pilot>(player).unwrap().wake = vec![(Vec2::ZERO, 1.0)];
    advance_wakes(&mut game, TICK_SECONDS);
    close(game.world.get::<&Actor>(protected).unwrap().health, 100.0);
    let site = regions::sites_near(game.profile.seed, Vec2::new(40000.0, 30000.0))
        .into_iter()
        .next()
        .unwrap();
    let center = Vec2::from_array(site.position);
    assert!(game.outside_site_cache(center));
    let protected = target(&mut game, center, 100.0, Kind::Boss(2));
    game.world.get::<&mut Pilot>(player).unwrap().wake = vec![(center, 1.0)];
    advance_wakes(&mut game, TICK_SECONDS);
    close(game.world.get::<&Actor>(protected).unwrap().health, 100.0);
    place(&mut game, player, center, 1.0);
    advance_wakes(&mut game, TICK_SECONDS);
    assert_eq!(game.world.get::<&Pilot>(player).unwrap().wake.len(), 1);
}

#[test]
fn evolution_five_connects_short_samples_but_never_crosses_sanctuary_or_large_gap() {
    let (mut game, _) = start(&[profile(5, 5, 1560)]);
    let player = game.player;
    place(&mut game, player, Vec2::new(800.0, 0.0), 0.0);
    let enemy = target(&mut game, Vec2::new(800.0, 0.0), 100.0, Kind::Enemy(0));
    // Endpoints are outside reach. Only the connected segment may hit.
    game.world.get::<&mut Pilot>(player).unwrap().wake = vec![
        (Vec2::new(800.0, -60.0), 1.0),
        (Vec2::new(800.0, 60.0), 1.0),
    ];
    advance_wakes(&mut game, TICK_SECONDS);
    close(game.world.get::<&Actor>(enemy).unwrap().health, 99.8);
    game.world.get::<&mut Pilot>(player).unwrap().wake_evolution = 4;
    advance_wakes(&mut game, TICK_SECONDS);
    close(game.world.get::<&Actor>(enemy).unwrap().health, 99.8);
    game.world.get::<&mut Pilot>(player).unwrap().wake_evolution = 5;
    game.world.get::<&mut Pilot>(player).unwrap().wake = vec![
        (Vec2::new(800.0, -81.0), 1.0),
        (Vec2::new(800.0, 81.0), 1.0),
    ];
    advance_wakes(&mut game, TICK_SECONDS);
    close(game.world.get::<&Actor>(enemy).unwrap().health, 99.8);
    // A 150-unit tangent chord crosses Haven while the target lies outside it.
    let x = SAFE_RADIUS - 1.0;
    let protected_target = target(&mut game, Vec2::new(x + 20.0, 0.0), 100.0, Kind::Enemy(0));
    game.world.get::<&mut Pilot>(player).unwrap().wake =
        vec![(Vec2::new(x, -75.0), 1.0), (Vec2::new(x, 75.0), 1.0)];
    assert!(game.safe_zone_at(Vec2::new(x + 20.0, 0.0)).is_none());
    advance_wakes(&mut game, TICK_SECONDS);
    close(
        game.world.get::<&Actor>(protected_target).unwrap().health,
        100.0,
    );
    // Same exact fence for a canonical site outside the host's cache.
    let site = regions::sites_near(game.profile.seed, Vec2::new(40000.0, 30000.0))
        .into_iter()
        .next()
        .unwrap();
    let center = Vec2::from_array(site.position);
    let tangent = center + Vec2::X * (site.radius - 1.0);
    let remote_position = tangent + Vec2::X * 20.0;
    assert!(game.outside_site_cache(tangent));
    assert!(game.safe_zone_at(remote_position).is_none());
    let remote = target(&mut game, remote_position, 100.0, Kind::Enemy(0));
    game.world.get::<&mut Pilot>(player).unwrap().wake = vec![
        (tangent - Vec2::Y * 75.0, 1.0),
        (tangent + Vec2::Y * 75.0, 1.0),
    ];
    advance_wakes(&mut game, TICK_SECONDS);
    close(game.world.get::<&Actor>(remote).unwrap().health, 100.0);
}

#[test]
fn death_leave_rejoin_and_session_end_drop_all_membership_trails() {
    let p = profile(5, 5, 1560);
    let mut game = Game::new(p.clone());
    // A retained solo trail is restored exactly, not adopted into co-op.
    game.wake = vec![(Vec2::new(800.0, 0.0), 1.7)];
    game.wake_timer = 0.123;
    let solo_profile = serde_json::to_string(game.profile()).unwrap();
    let solo_body = format!("{:?}", &*game.world.get::<&PixelBody>(game.player).unwrap());
    game.begin_co_op(PROTOCOL, session(), id(1)).unwrap();
    let source = Game::new(p);
    let build = source.co_op_owned_loadout().unwrap();
    let a = game
        .join_co_op_owned(session(), spec(source.profile(), 2), build.clone())
        .unwrap();
    let guest = entity(&game, a.id);
    place(&mut game, guest, Vec2::new(800.0, 0.0), 0.75);
    advance_wakes(&mut game, TICK_SECONDS);
    assert!(!game.world.get::<&Pilot>(guest).unwrap().wake.is_empty());
    game.world.get::<&mut Actor>(guest).unwrap().health = 0.0;
    game.tick_co_op(session()).unwrap();
    let pilot = game.world.get::<&Pilot>(guest).unwrap();
    assert!(pilot.wake.is_empty());
    close(pilot.wake_timer, 0.0);
    assert_eq!(pilot.admission.epoch, a.epoch);
    drop(pilot);
    place(&mut game, guest, Vec2::new(800.0, 0.0), 0.75);
    advance_wakes(&mut game, TICK_SECONDS);
    game.leave_co_op(session(), a.id, a.epoch).unwrap();
    assert!(!game.world.contains(guest));
    let b = game
        .join_co_op_owned(session(), spec(source.profile(), 2), build)
        .unwrap();
    assert_eq!(b.slot, a.slot);
    assert_ne!(b.epoch, a.epoch);
    let rejoined = entity(&game, b.id);
    assert!(game.world.get::<&Pilot>(rejoined).unwrap().wake.is_empty());
    assert_eq!(
        game.submit_guest_co_op_input(session(), a.id, a.epoch, 9, InputFrame::default()),
        Err(Error::StaleMembership)
    );
    game.end_co_op(session()).unwrap();
    assert_eq!(game.world.query::<&Pilot>().iter().count(), 0);
    assert_eq!(game.wake, vec![(Vec2::new(800.0, 0.0), 1.7)]);
    close(game.wake_timer, 0.123);
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), solo_profile);
    assert_eq!(
        format!("{:?}", &*game.world.get::<&PixelBody>(game.player).unwrap()),
        solo_body
    );
}

#[test]
fn wake_wire_exports_effect_claim_only_and_keeps_legacy_omission_bytes() {
    let legacy = Game::new(Profile::default()).co_op_owned_loadout().unwrap();
    let old = r#"{"version":1,"selected_ship":0,"owned_ships":1,"patches":0,"royal_victory":false,"ship_xp":0,"hardware":[0,0,0,0,0,0],"slots":[null,null,null],"trained_starter":false,"relic_count":0}"#;
    assert_eq!(serde_json::to_string(&legacy).unwrap(), old);
    assert_eq!(serde_json::from_str::<OwnedLoadout>(old).unwrap(), legacy);
    println!("actual_legacy_export={old}");
    for evolution in 1..=5 {
        let source = Game::new(profile(evolution, 5, 0));
        let saved = serde_json::to_string(source.profile()).unwrap();
        let build = source.co_op_owned_loadout().unwrap();
        assert_eq!(build.wake_evolution, (evolution > 1).then_some(evolution));
        assert_eq!(build.echo_evolution, None);
        assert_eq!(build.nova_evolution, None);
        let temporary = build.profile(&spec(source.profile(), 2)).unwrap();
        assert_eq!(temporary.boss_generations, [[0; 5]; 3]);
        assert_eq!(ship_progression::equipped_rank(&temporary, 5), 1);
        let (mut host, _) = start(&[Profile::default()]);
        let a = host
            .join_co_op_owned(session(), spec(source.profile(), 2), build.clone())
            .unwrap();
        let pilot = host.world.get::<&Pilot>(entity(&host, a.id)).unwrap();
        assert_eq!((pilot.wake_rank, pilot.wake_evolution), (1, evolution));
        drop(pilot);
        assert_eq!(
            host.co_op_snapshot().unwrap().participants[1].owned_loadout,
            Some(build.clone())
        );
        assert_eq!(serde_json::to_string(source.profile()).unwrap(), saved);
        println!(
            "actual_wake_export_evolution_{evolution}={}",
            serde_json::to_string(&build).unwrap()
        );
        for bad in [0, 6, 255] {
            let mut invalid = build.clone();
            invalid.wake_evolution = Some(bad);
            let before = serde_json::to_string(&host.co_op_snapshot()).unwrap();
            assert_eq!(
                host.join_co_op_owned(session(), spec(source.profile(), 3), invalid),
                Err(Error::InvalidLoadout)
            );
            assert_eq!(
                serde_json::to_string(&host.co_op_snapshot()).unwrap(),
                before
            );
        }
        let mut mismatched = build.clone();
        mismatched.wake_evolution = Some(1);
        mismatched.slots[2] = None;
        assert!(matches!(
            mismatched.profile(&spec(source.profile(), 2)),
            Err(Error::InvalidLoadout)
        ));
        let mut unequipped = source.profile().clone();
        unequipped.campaign.loadouts[0].slots[2] = None;
        assert_eq!(
            Game::new(unequipped)
                .co_op_owned_loadout()
                .unwrap()
                .wake_evolution,
            None
        );
    }
    let source = Game::new(profile(1, 1, 1560));
    let mut value = serde_json::to_value(source.co_op_owned_loadout().unwrap()).unwrap();
    value["wake_evolution"] = json!(1);
    let explicit = serde_json::from_value::<OwnedLoadout>(value.clone()).unwrap();
    assert!(explicit.profile(&spec(source.profile(), 2)).is_ok());
    for invalid in [
        Value::Null,
        json!(-1),
        json!(1.5),
        json!("1"),
        json!(256),
        json!(true),
    ] {
        value["wake_evolution"] = invalid;
        assert!(serde_json::from_value::<OwnedLoadout>(value.clone()).is_err());
    }
    value["wake_evolution"] = json!(1);
    value["wake_points"] = json!([]);
    assert!(serde_json::from_value::<OwnedLoadout>(value).is_err());
}

#[test]
fn guest_owned_rank_obeys_actual_ship_xp_caps() {
    for (xp, expected) in [(0, 1), (60, 2), (200, 3), (560, 4), (1320, 5)] {
        let source = Game::new(profile(3, 5, xp));
        assert_eq!(
            ship_progression::equipped_rank(source.profile(), 5),
            expected
        );
        let (mut game, _) = start(&[Profile::default()]);
        let a = game
            .join_co_op_owned(
                session(),
                spec(source.profile(), 2),
                source.co_op_owned_loadout().unwrap(),
            )
            .unwrap();
        let guest = entity(&game, a.id);
        assert_eq!(game.world.get::<&Pilot>(guest).unwrap().wake_rank, expected);
        place(&mut game, guest, Vec2::new(800.0, 0.0), 0.75);
        quiet(&mut game);
        let enemy = target(&mut game, Vec2::new(769.0, 0.0), 100.0, Kind::Enemy(0));
        game.submit_guest_co_op_input(
            session(),
            a.id,
            a.epoch,
            1,
            InputFrame {
                joystick: Some([1.0, 0.0]),
                ..Default::default()
            },
        )
        .unwrap();
        game.tick_co_op(session()).unwrap();
        close(
            game.world.get::<&Actor>(enemy).unwrap().health,
            100.0 - 0.1 * (1.0 + 0.25 * (expected - 1) as f32),
        );
    }
}

#[test]
fn stale_input_stops_births_and_pause_keeps_effect_phase_unchanged() {
    let (mut game, _) = start(&[profile(5, 5, 1560)]);
    let player = game.player;
    place(&mut game, player, Vec2::new(800.0, 0.0), 1.0);
    quiet(&mut game);
    game.submit_host_co_op_input(
        session(),
        1,
        InputFrame {
            joystick: Some([1.0, 0.0]),
            ..Default::default()
        },
    )
    .unwrap();
    for _ in 0..30 {
        game.tick_co_op(session()).unwrap();
    }
    assert!(
        game.world
            .get::<&Pilot>(player)
            .unwrap()
            .input
            .joystick
            .is_none()
    );
    game.paused = true;
    let points = game.world.get::<&Pilot>(player).unwrap().wake.clone();
    let timer = game.world.get::<&Pilot>(player).unwrap().wake_timer;
    assert_eq!(game.tick_co_op(session()), Err(Error::Paused));
    assert_eq!(game.world.get::<&Pilot>(player).unwrap().wake, points);
    close(game.world.get::<&Pilot>(player).unwrap().wake_timer, timer);
    game.paused = false;
    for _ in 0..300 {
        game.tick_co_op(session()).unwrap();
    }
    assert!(game.world.get::<&Pilot>(player).unwrap().wake.is_empty());
    assert!(game.transient_lights.is_empty());
    game.take_events();
    // Idle cooldown debt emits at most one sample on a returning valid input.
    place(&mut game, player, Vec2::new(800.0, 0.0), 1.0);
    game.submit_host_co_op_input(
        session(),
        2,
        InputFrame {
            joystick: Some([1.0, 0.0]),
            ..Default::default()
        },
    )
    .unwrap();
    game.tick_co_op(session()).unwrap();
    assert_eq!(game.world.get::<&Pilot>(player).unwrap().wake.len(), 1);
    assert_eq!(
        game.take_events()
            .iter()
            .filter(|e| e.kind == "ability-wake")
            .count(),
        1
    );
}

#[test]
fn overlapping_trails_kill_once_in_slot_order_and_restore_enclosing_owner() {
    let (mut game, admissions) = start(&[profile(1, 1, 1560), profile(1, 1, 1560)]);
    for a in &admissions {
        let e = entity(&game, a.id);
        place(&mut game, e, Vec2::new(800.0, a.slot as f32 * 100.0), 0.0);
        game.world.get::<&mut Pilot>(e).unwrap().wake = vec![(Vec2::new(900.0, 0.0), 1.0)];
    }
    let boss = target(&mut game, Vec2::new(900.0, 0.0), 0.05, Kind::Boss(2));
    advance_wakes(&mut game, TICK_SECONDS);
    assert!(!game.world.contains(boss));
    let notices = game.take_co_op_destructions(session()).unwrap();
    assert_eq!(notices.len(), 1);
    assert_eq!(
        (notices[0].owner, notices[0].owner_epoch),
        (Some(admissions[0].id), Some(admissions[0].epoch))
    );
    assert!(game.co_op.as_ref().unwrap().hit_owner.is_none());
}

#[test]
fn admitted_ordinary_ion_wake_damages_real_enemy_during_fast_host_tick() {
    let mut game = Game::new(profile(1, 1, 1560));
    assert_eq!(ship_progression::equipped_rank(game.profile(), 5), 1);
    game.begin_co_op(PROTOCOL, session(), id(1)).unwrap();
    let position = Vec2::new(800.0, 0.0);
    let speed = game.world.get::<&Pilot>(game.player).unwrap().stats.speed;
    *game.world.get::<&mut Motion>(game.player).unwrap() =
        Motion::new(position, Vec2::X * speed * 0.75, 0.0);
    game.world.get::<&mut Pilot>(game.player).unwrap().was_home = false;
    quiet(&mut game);
    let enemy = target(&mut game, position - Vec2::X * 31.0, 100.0, Kind::Enemy(0));
    let profile_before = serde_json::to_string(game.profile()).unwrap();
    game.submit_host_co_op_input(
        session(),
        1,
        InputFrame {
            joystick: Some([1.0, 0.0]),
            ..Default::default()
        },
    )
    .unwrap();
    game.tick_co_op(session()).unwrap();
    let health = game.world.get::<&Actor>(enemy).unwrap().health;
    println!("actual_coop_enemy_health={health}; ordinary_ion_wake_expected=99.9");
    assert!(
        (health - 99.9).abs() < 0.0002,
        "ordinary admitted Ion Wake must apply real 6 DPS"
    );
    assert_eq!(
        serde_json::to_string(game.profile()).unwrap(),
        profile_before
    );
}
