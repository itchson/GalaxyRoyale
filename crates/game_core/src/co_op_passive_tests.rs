//! Real solo/session ticks are independent oracles; fixtures only remove unrelated
//! traffic and arrange positions, saved builds and occupied-cell projectile hits.
use super::*;

fn id(n: u32) -> ParticipantId {
    ParticipantId::parse(&format!("{n:032x}")).unwrap()
}
fn session() -> ParticipantId {
    id(100)
}
fn licensed(main: &str) -> Profile {
    identity::create_profile("talo", main, "renko", "0123456789abcdef").unwrap()
}
fn trained(main: &str) -> Profile {
    let mut p = licensed(main);
    p.onboarding = Some(identity::OnboardingProgress {
        stage: 5,
        flags: 31,
        asteroids_destroyed: 10,
        materials_collected: 10,
        ..Default::default()
    });
    p.campaign.stored_materials[0] = 10;
    let material = training::material_for_corporation(p.identity.as_ref().unwrap().corporation);
    p.campaign.stored_training_materials[material.index] = 10;
    let mut game = Game::new(p);
    let key = abilities::get(
        game.profile()
            .identity
            .as_ref()
            .unwrap()
            .corporation
            .ability(),
    )
    .unwrap()
    .key;
    game.upgrade_ability(key).unwrap();
    assert_eq!(game.profile().campaign.ship_xp, [0; 10]);
    assert_eq!(game.profile().campaign.stored_training_materials, [0; 3]);
    game.profile().clone()
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
fn saved(game: &Game) -> String {
    serde_json::to_string(game.profile()).unwrap()
}
fn close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.0002, "{actual} != {expected}");
}
fn place(game: &mut Game, e: Entity, position: Vec2) {
    *game.world.get::<&mut Motion>(e).unwrap() = Motion::new(position, Vec2::ZERO, 0.0);
    if let Ok(mut p) = game.world.get::<&mut Pilot>(e) {
        p.was_home = false;
    } else {
        game.was_safe = false;
        game.invulnerability = 0.0;
    }
    game.refresh_sites();
    assert!(game.safe_zone_at(position).is_none());
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
    // Empty preloaded chunks isolate controls/recharge from unrelated streamed
    // rocks. Real tick, integrate, projectile and collision paths still execute.
    let positions: Vec<_> = game
        .world
        .query::<(&Motion, &Actor)>()
        .iter()
        .filter(|(_, a)| a.kind == Kind::Player)
        .map(|(m, _)| m.position)
        .collect();
    for position in positions {
        let center = (position / CHUNK).floor().as_ivec2();
        for dy in -1..=1 {
            for dx in -1..=1 {
                game.chunks
                    .entry((center.x + dx, center.y + dy))
                    .or_default();
            }
        }
    }
}
fn hostile_hit(game: &mut Game, e: Entity) {
    let local = game
        .world
        .get::<&PixelBody>(e)
        .unwrap()
        .cells()
        .min_by(|a, b| a.0.length_squared().total_cmp(&b.0.length_squared()))
        .unwrap()
        .0;
    let motion = *game.world.get::<&Motion>(e).unwrap();
    let actor = *game.world.get::<&Actor>(e).unwrap();
    let hit = actor_transform(motion, actor).point(local);
    assert!(
        game.spawn_bullet_entity(hit, Vec2::ZERO, 10.0, false, 1)
            .is_some()
    );
}
fn renew(game: &mut Game, admission: Admission, seq: u32, input: InputFrame) {
    if admission.slot == 0 {
        game.submit_host_co_op_input(session(), seq, input).unwrap();
    } else {
        game.submit_guest_co_op_input(session(), admission.id, admission.epoch, seq, input)
            .unwrap();
    }
}

#[test]
fn eight_real_pilots_recharge_only_their_unlocked_build_after_their_own_hit_delay() {
    let mut aegis = licensed("renko");
    aegis.currency[1] = 10;
    aegis.campaign.stored_materials[0] = 1;
    let mut aegis = Game::new(aegis);
    aegis.purchase("shield").unwrap();
    let mut empty = Game::new(licensed("hugo"));
    empty.equip_ability(1, None).unwrap();
    let builds = [
        licensed("hugo"),
        trained("hugo"),
        aegis.profile().clone(),
        empty.profile().clone(),
    ];
    let mut game = Game::new(builds[0].clone());
    let host_save = saved(&game);
    let mut admissions = vec![game.begin_co_op(PROTOCOL, session(), id(1)).unwrap()];
    let mut solos = Vec::new();
    let mut solo_saves = Vec::new();
    for n in 0..8 {
        let solo = Game::new(builds[n % 4].clone());
        solo_saves.push(saved(&solo));
        if n > 0 {
            admissions.push(
                game.join_co_op_owned(
                    session(),
                    spec(solo.profile(), n as u32 + 1),
                    solo.co_op_owned_loadout().unwrap(),
                )
                .unwrap(),
            );
        }
        solos.push(solo);
    }
    quiet(&mut game);
    for (n, solo) in solos.iter_mut().enumerate() {
        let position = Vec2::new(300.0, n as f32 * 80.0);
        let e = entity(&game, admissions[n].id);
        place(&mut game, e, position);
        place(solo, solo.player, position);
        quiet(solo);
        // Ordinary solo departure truthfully records its onboarding fact. Settle
        // it before the no-further-grants baseline; sessions suppress that fact.
        solo.tick(&InputFrame::default(), TICK_SECONDS);
        solo_saves[n] = saved(solo);
        close(solo.hud().max_shield, [30.0, 38.0, 10.0, 0.0][n % 4]);
        close(
            solo.hud().ship_stats.shield_regen,
            [0.0, 3.0, 3.0, 0.0][n % 4],
        );
        hostile_hit(&mut game, e);
        hostile_hit(solo, solo.player);
    }
    game.tick_co_op(session()).unwrap();
    for (n, solo) in solos.iter_mut().enumerate() {
        solo.tick(&InputFrame::default(), TICK_SECONDS);
        let p = game
            .world
            .get::<&Pilot>(entity(&game, admissions[n].id))
            .unwrap();
        close(p.shield, [21.0, 29.0, 0.0, 0.0][n % 4]);
        close(p.shield, solo.hud().shield);
        close(p.since_damage, 0.0);
        close(
            game.world
                .get::<&Actor>(entity(&game, admissions[n].id))
                .unwrap()
                .health,
            [100.0, 100.0, 100.0, 90.0][n % 4],
        );
    }
    let mut first_recharge = [None; 8];
    let mut reset_shield = 0.0;
    for step in 1..=850u32 {
        if step == 120 {
            // Pause does not age any participant's four-second recharge gate.
            game.set_paused(true);
            let before = serde_json::to_string(&game.co_op_snapshot().unwrap()).unwrap();
            let timers: Vec<_> = admissions
                .iter()
                .map(|a| {
                    game.world
                        .get::<&Pilot>(entity(&game, a.id))
                        .unwrap()
                        .since_damage
                })
                .collect();
            assert_eq!(game.tick_co_op(session()), Err(Error::Paused));
            assert_eq!(
                serde_json::to_string(&game.co_op_snapshot().unwrap()).unwrap(),
                before
            );
            assert_eq!(
                admissions
                    .iter()
                    .map(|a| game
                        .world
                        .get::<&Pilot>(entity(&game, a.id))
                        .unwrap()
                        .since_damage)
                    .collect::<Vec<_>>(),
                timers
            );
            game.set_paused(false);
        }
        quiet(&mut game);
        for solo in &mut solos {
            quiet(solo);
        }
        if step == 121 {
            // Re-hit one R2 pilot while its matching R2 peer keeps its old delay.
            let e = entity(&game, admissions[1].id);
            hostile_hit(&mut game, e);
            let player = solos[1].player;
            hostile_hit(&mut solos[1], player);
        }
        game.tick_co_op(session()).unwrap();
        for (n, solo) in solos.iter_mut().enumerate() {
            solo.tick(&InputFrame::default(), TICK_SECONDS);
            let e = entity(&game, admissions[n].id);
            let p = game.world.get::<&Pilot>(e).unwrap();
            let expected = solo.hud();
            close(p.shield, expected.shield);
            close(p.since_damage, solo.since_damage);
            close(game.world.get::<&Actor>(e).unwrap().health, expected.hull);
            if step == 121 && n == 1 {
                close(p.since_damage, 0.0);
                reset_shield = p.shield;
                close(reset_shield, 20.0);
            }
            let depleted = if n == 1 && step >= 121 {
                reset_shield
            } else {
                [21.0, 29.0, 0.0, 0.0][n % 4]
            };
            if p.shield > depleted && first_recharge[n].is_none() {
                first_recharge[n] = Some(step);
                close(p.shield - depleted, TICK_SECONDS * 3.0);
                assert!(p.since_damage >= 4.0 && p.since_damage < 4.0 + TICK_SECONDS * 1.01);
            }
            if step <= 239 || (n == 1 && (121..=360).contains(&step)) {
                close(p.shield, depleted);
            }
        }
        game.take_events();
        for solo in &mut solos {
            solo.take_events();
        }
    }
    for n in 0..8 {
        let p = game
            .world
            .get::<&Pilot>(entity(&game, admissions[n].id))
            .unwrap();
        match n % 4 {
            1 | 2 => {
                let delay = if n == 1 { 121 } else { 0 };
                assert!((240 + delay..=241 + delay).contains(&first_recharge[n].unwrap()));
                close(p.shield, p.stats.shield);
            }
            _ => {
                assert!(first_recharge[n].is_none());
                close(p.shield, [21.0, 0.0][usize::from(n % 4 == 3)]);
            }
        }
        assert_eq!(saved(&solos[n]), solo_saves[n]);
    }
    assert_eq!(saved(&game), host_save);
    let snapshot = game.co_op_snapshot().unwrap();
    assert!(!snapshot.progression);
    assert_eq!(snapshot.participants.len(), 8);
    for row in snapshot.participants {
        close(
            row.shield,
            game.world
                .get::<&Pilot>(entity(&game, row.id))
                .unwrap()
                .shield,
        );
    }
    println!("actual_eight_pilot_first_recharge_ticks={first_recharge:?}; pilot_1_rehit_tick=121");
    game.end_co_op(session()).unwrap();
    assert_eq!(saved(&game), host_save);
}

fn movement_builds() -> Vec<Profile> {
    let mut empty = Game::new(licensed("pan"));
    empty.equip_ability(2, None).unwrap();
    let mut other = licensed("hugo");
    other.campaign.abilities[2] = campaign::AbilityState {
        blueprint: true,
        rank: 1,
    };
    let mut other = Game::new(other);
    other.equip_ability(2, Some("flux-drive")).unwrap();
    let mut result = vec![
        licensed("pan"),
        trained("pan"),
        empty.profile().clone(),
        other.profile().clone(),
    ];
    for (ship, pilot) in [(1, "hugo"), (2, "pan")] {
        let mut p = licensed(pilot);
        p.campaign.patches[ship - 1] = true;
        p.campaign.blueprints[ship - 1] = true;
        p.campaign.crafted[ship - 1] = true;
        p.campaign.selected_ship = ship as u8;
        p.campaign.abilities[2] = campaign::AbilityState {
            blueprint: true,
            rank: 1,
        };
        p.campaign.loadouts[ship].slots[2] = Some(2);
        result.push(Game::new(p).profile().clone());
    }
    result
}
fn controls(tick: u32) -> InputFrame {
    match tick {
        0..=89 => InputFrame {
            joystick: Some([1.0, 0.0]),
            ..Default::default()
        },
        90..=109 => InputFrame {
            joystick: Some([0.3, 0.8]),
            aim: Some([600.0, -100.0]),
            fire: true,
            ..Default::default()
        },
        110..=129 => InputFrame {
            thrust: Some([450.0, 180.0]),
            aim: Some([300.0, 200.0]),
            fire: true,
            ..Default::default()
        },
        130 => InputFrame {
            destination: Some([350.0, 100.0]),
            ..Default::default()
        },
        _ => InputFrame::default(),
    }
}
#[test]
fn actual_solo_and_admitted_pan_movement_match_for_slots_training_and_other_hull_manufacturers() {
    let mut steady_speeds = Vec::new();
    for profile in movement_builds() {
        let mut solo = Game::new(profile);
        let mut game = Game::new(Profile::default());
        let host_save = saved(&game);
        game.begin_co_op(PROTOCOL, session(), id(1)).unwrap();
        let admission = game
            .join_co_op_owned(
                session(),
                spec(solo.profile(), 2),
                solo.co_op_owned_loadout().unwrap(),
            )
            .unwrap();
        let e = entity(&game, admission.id);
        let position = Vec2::new(300.0, 0.0);
        place(&mut game, e, position);
        let player = solo.player;
        place(&mut solo, player, position);
        quiet(&mut solo);
        solo.tick(&InputFrame::default(), TICK_SECONDS);
        let solo_save = saved(&solo);
        for tick in 0..210 {
            quiet(&mut game);
            quiet(&mut solo);
            let input = controls(tick);
            renew(&mut game, admission, tick, input.clone());
            solo.tick(&input, TICK_SECONDS);
            game.tick_co_op(session()).unwrap();
            let a = *solo.world.get::<&Motion>(solo.player).unwrap();
            let b = *game.world.get::<&Motion>(e).unwrap();
            assert_eq!(
                (
                    a.position,
                    a.previous,
                    a.velocity,
                    a.heading,
                    a.previous_heading
                ),
                (
                    b.position,
                    b.previous,
                    b.velocity,
                    b.heading,
                    b.previous_heading
                ),
                "tick {tick}"
            );
            assert_eq!(solo.throttle, game.world.get::<&Pilot>(e).unwrap().throttle);
            assert_eq!(
                solo.destination,
                game.world.get::<&Pilot>(e).unwrap().destination
            );
            if tick == 89 {
                close(b.velocity.length(), solo.hud().ship_stats.top_speed);
                steady_speeds.push(b.velocity.length());
            }
            game.take_events();
            solo.take_events();
        }
        assert_eq!(saved(&solo), solo_save);
        assert_eq!(saved(&game), host_save);
        let row = game
            .co_op_snapshot()
            .unwrap()
            .participants
            .into_iter()
            .find(|p| p.id == admission.id)
            .unwrap();
        assert_eq!(
            row.velocity,
            solo.world
                .get::<&Motion>(solo.player)
                .unwrap()
                .velocity
                .to_array()
        );
        game.end_co_op(session()).unwrap();
        assert_eq!(saved(&game), host_save);
    }
    // Contract values make the comparison sensitive to losing Flux, training,
    // equipped-slot gating, or selecting pilot rather than hull manufacturer.
    println!(
        "actual_six_admitted_steady_speeds={steady_speeds:?}; equal_motion_ticks_per_build=210"
    );
    for (actual, expected) in steady_speeds
        .into_iter()
        .zip([114.4, 118.8, 100.0, 104.0, 205.92, 172.224])
    {
        close(actual, expected);
    }
}
