//! Native authority proofs; private placements only arrange explicit combat fixtures.
use super::*;
use co_op::{Admission, Error, PROTOCOL, ParticipantId, ParticipantSpec, Pilot, ShotOwner};

fn id(index: u64) -> ParticipantId {
    ParticipantId::parse(&format!("{index:032x}")).unwrap()
}
fn session() -> ParticipantId {
    id(1000)
}
fn host() -> ParticipantId {
    id(1)
}
fn spec(index: u64) -> ParticipantSpec {
    let corporation = match index % 3 {
        0 => "pan",
        1 => "renko",
        _ => "hugo",
    };
    let profile =
        identity::create_profile("talo", corporation, "pan", &format!("{index:016x}")).unwrap();
    ParticipantSpec {
        id: id(index),
        local_seed: profile.seed,
        identity: profile.identity,
    }
}
fn start() -> Game {
    let mut game = Game::new(Profile::default());
    game.begin_co_op(PROTOCOL, session(), host()).unwrap();
    game.spawn_timer = 10000.0;
    game
}
fn guest(game: &mut Game, index: u64) -> Admission {
    game.join_co_op(session(), spec(index)).unwrap()
}
fn entity(game: &Game, id: ParticipantId) -> Entity {
    game.world
        .query::<(Entity, &Pilot)>()
        .iter()
        .find(|(_, p)| p.admission.id == id)
        .map(|(e, _)| e)
        .unwrap()
}
fn place(game: &mut Game, id: ParticipantId, position: Vec2, heading: f32) {
    let e = entity(game, id);
    *game.world.get::<&mut Motion>(e).unwrap() = Motion::new(position, Vec2::ZERO, heading);
    game.world.get::<&mut Pilot>(e).unwrap().was_home = position.length() <= SAFE_RADIUS;
    game.refresh_sites();
}
fn renew(game: &mut Game, a: Admission, sequence: u32, input: InputFrame) {
    if a.id == host() {
        game.submit_host_co_op_input(session(), sequence, input)
            .unwrap();
    } else {
        game.submit_guest_co_op_input(session(), a.id, a.epoch, sequence, input)
            .unwrap();
    }
}
fn hp(game: &Game, id: ParticipantId) -> (f32, f32) {
    let e = entity(game, id);
    (
        game.world.get::<&Actor>(e).unwrap().health,
        game.world.get::<&Pilot>(e).unwrap().shield,
    )
}

#[test]
fn direct_admission_validates_ids_protocol_real_dock_and_living_body() {
    for bad in [
        "",
        "1",
        "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        "0000000000000000000000000000000g",
        " 00000000000000000000000000000001",
    ] {
        assert_eq!(ParticipantId::parse(bad), Err(Error::InvalidId));
    }
    let mut game = Game::new(Profile::default());
    let before = serde_json::to_string(game.profile()).unwrap();
    assert_eq!(
        game.begin_co_op("other", session(), host()),
        Err(Error::InvalidProtocol)
    );
    for point in [Vec2::new(129.0, 0.0), Vec2::splat(f32::NAN)] {
        game.move_player(point);
        assert_eq!(
            game.begin_co_op(PROTOCOL, session(), host()),
            Err(Error::NotAtDock)
        );
    }
    game.move_player(Vec2::ZERO);
    game.world.get::<&mut Actor>(game.player).unwrap().health = 0.0;
    assert_eq!(
        game.begin_co_op(PROTOCOL, session(), host()),
        Err(Error::NotAtDock)
    );
    game.world.get::<&mut Actor>(game.player).unwrap().health = 100.0;
    game.diagnostic_scene = true;
    assert_eq!(
        game.begin_co_op(PROTOCOL, session(), host()),
        Err(Error::NotAtDock)
    );
    game.diagnostic_scene = false;
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), before);
    let host_admission = game.begin_co_op(PROTOCOL, session(), host()).unwrap();
    assert_eq!(host_admission.slot, 0);
    assert_eq!(
        game.begin_co_op(PROTOCOL, id(2000), host()),
        Err(Error::ActiveSession)
    );
    assert_eq!(
        game.join_co_op(
            session(),
            ParticipantSpec {
                id: id(2),
                local_seed: 0,
                identity: None
            }
        ),
        Err(Error::InvalidSeed)
    );
    let mut malformed = spec(2);
    malformed.identity.as_mut().unwrap().hex_id = "wrong".into();
    assert_eq!(
        game.join_co_op(session(), malformed),
        Err(Error::InvalidIdentity)
    );
    assert_eq!(
        game.join_co_op(
            session(),
            ParticipantSpec {
                id: host(),
                local_seed: 42,
                identity: None
            }
        ),
        Err(Error::DuplicateId)
    );
    assert!(serde_json::from_str::<ParticipantSpec>(r#"{"id":"00000000000000000000000000000002","local_seed":42,"identity":null,"ranks":[5,5]}"#).is_err());
}

#[test]
fn two_and_eight_real_ecs_pilots_move_with_independent_exact_controls() {
    for count in [2, 8] {
        let mut game = start();
        let mut admissions = vec![Admission {
            id: host(),
            slot: 0,
            epoch: 1,
        }];
        for i in 2..=count {
            admissions.push(guest(&mut game, i));
        }
        if count == 8 {
            assert_eq!(game.join_co_op(session(), spec(9)), Err(Error::Full));
        }
        let initial = game.co_op_snapshot().unwrap();
        for seq in 0..60 {
            for a in &admissions {
                renew(
                    &mut game,
                    *a,
                    seq,
                    InputFrame {
                        joystick: Some(if a.slot % 2 == 0 {
                            [1.0, 0.0]
                        } else {
                            [-1.0, 0.0]
                        }),
                        ..Default::default()
                    },
                );
            }
            game.tick_co_op(session()).unwrap();
        }
        let snapshot = game.co_op_snapshot().unwrap();
        assert_eq!(snapshot.tick, 60);
        assert_eq!(game.world.query::<&Pilot>().iter().count(), count as usize);
        assert_eq!(
            game.world
                .query::<&Actor>()
                .iter()
                .filter(|a| a.kind == Kind::Player)
                .count(),
            count as usize
        );
        for (after, before) in snapshot.participants.iter().zip(initial.participants) {
            assert!(
                Vec2::from_array(after.position).distance(Vec2::from_array(before.position)) > 55.0
            );
            assert_eq!(after.id, before.id);
            assert!(after.velocity.iter().all(|x| x.is_finite()));
        }
    }
}

#[test]
fn guest_factory_identity_seed_and_stats_never_become_host_profile() {
    let mut game = start();
    let host_profile = serde_json::to_string(game.profile()).unwrap();
    let guest_spec = spec(2);
    let own_seed = guest_spec.local_seed;
    let own_identity = guest_spec.identity.clone();
    guest(&mut game, 2);
    let snapshot = game.co_op_snapshot().unwrap();
    let pilot = &snapshot.participants[1];
    assert_eq!(pilot.local_seed, own_seed);
    assert_ne!(pilot.local_seed, snapshot.world_seed);
    assert_eq!(pilot.identity, own_identity);
    assert_eq!(pilot.selected_ship, 0);
    assert_eq!(pilot.max_hull, 100.0);
    assert_eq!(pilot.max_shield, 30.0); // HUGO's real factory Guard Coil.
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), host_profile);
    assert_eq!(snapshot.world_seed, 0x4752_4f59);
    assert!(!snapshot.progression);
}

#[test]
fn session_and_membership_tokens_close_replay_and_host_impersonation() {
    let mut game = start();
    let a = guest(&mut game, 2);
    let fire = InputFrame {
        fire: true,
        ..Default::default()
    };
    assert_eq!(
        game.submit_guest_co_op_input(session(), host(), 1, 1, fire.clone()),
        Err(Error::HostOnly)
    );
    assert_eq!(
        game.submit_guest_co_op_input(id(2000), a.id, a.epoch, 1, fire.clone()),
        Err(Error::StaleSession)
    );
    game.submit_guest_co_op_input(session(), a.id, a.epoch, 10, fire.clone())
        .unwrap();
    assert_eq!(
        game.submit_guest_co_op_input(session(), a.id, a.epoch, 10, fire.clone()),
        Err(Error::StaleSequence)
    );
    assert_eq!(
        game.submit_guest_co_op_input(session(), a.id, a.epoch, 9, fire.clone()),
        Err(Error::StaleSequence)
    );
    assert_eq!(
        game.leave_co_op(id(2000), a.id, a.epoch),
        Err(Error::StaleSession)
    );
    game.leave_co_op(session(), a.id, a.epoch).unwrap();
    let b = guest(&mut game, 2);
    assert_eq!(a.slot, b.slot);
    assert!(b.epoch > a.epoch);
    assert_eq!(
        game.submit_guest_co_op_input(session(), b.id, a.epoch, 100, fire.clone()),
        Err(Error::StaleMembership)
    );
    game.submit_guest_co_op_input(session(), b.id, b.epoch, 0, fire.clone())
        .unwrap();
    game.end_co_op(session()).unwrap();
    game.begin_co_op(PROTOCOL, id(2000), host()).unwrap();
    let c = game.join_co_op(id(2000), spec(2)).unwrap();
    assert_eq!(
        game.submit_guest_co_op_input(session(), c.id, c.epoch, 1000, fire),
        Err(Error::StaleSession)
    );
    assert_eq!(game.tick_co_op(session()), Err(Error::StaleSession));
    assert_eq!(game.end_co_op(session()), Err(Error::StaleSession));
    assert_eq!(game.co_op_snapshot().unwrap().tick, 0);
}

#[test]
fn invalid_input_does_not_consume_sequence_and_stale_controls_brake() {
    let mut game = start();
    let a = guest(&mut game, 2);
    for input in [
        InputFrame {
            joystick: Some([f32::NAN, 0.0]),
            ..Default::default()
        },
        InputFrame {
            aim: Some([WORLD_LIMIT + 1.0, 0.0]),
            ..Default::default()
        },
        InputFrame {
            thrust: Some([f32::INFINITY, 0.0]),
            ..Default::default()
        },
    ] {
        assert_eq!(
            game.submit_guest_co_op_input(session(), a.id, a.epoch, 1, input),
            Err(Error::InvalidInput)
        );
    }
    renew(
        &mut game,
        a,
        1,
        InputFrame {
            joystick: Some([1.0, 0.0]),
            destination: Some([800.0, 0.0]),
            fire: true,
            ..Default::default()
        },
    );
    for _ in 0..co_op::INPUT_TIMEOUT_TICKS {
        game.tick_co_op(session()).unwrap();
    }
    let before = game
        .world
        .get::<&Motion>(entity(&game, a.id))
        .unwrap()
        .velocity
        .length();
    for _ in 0..120 {
        game.tick_co_op(session()).unwrap();
    }
    let e = entity(&game, a.id);
    let p = game.world.get::<&Pilot>(e).unwrap();
    assert!(p.destination.is_none());
    assert!(!p.input.fire);
    assert!(p.input.joystick.is_none());
    assert!(game.world.get::<&Motion>(e).unwrap().velocity.length() < before * 0.01);
}

#[test]
fn live_overlay_cancels_only_host_route_and_keeps_normal_momentum_decay() {
    let mut game = start();
    let a = guest(&mut game, 2);
    game.submit_host_co_op_input(
        session(),
        0,
        InputFrame {
            destination: Some([800.0, 0.0]),
            ..Default::default()
        },
    )
    .unwrap();
    renew(
        &mut game,
        a,
        0,
        InputFrame {
            destination: Some([800.0, 100.0]),
            ..Default::default()
        },
    );
    for _ in 0..10 {
        game.tick_co_op(session()).unwrap();
    }
    let previous = game.world.get::<&Motion>(game.player).unwrap().velocity;
    game.cancel_course();
    assert_eq!(
        game.world.get::<&Motion>(game.player).unwrap().velocity,
        previous
    );
    game.tick_co_op(session()).unwrap();
    assert!(
        game.world
            .get::<&Pilot>(game.player)
            .unwrap()
            .destination
            .is_none()
    );
    assert!(
        game.world
            .get::<&Pilot>(entity(&game, a.id))
            .unwrap()
            .destination
            .is_some()
    );
    assert!(game.world.get::<&Pilot>(game.player).unwrap().throttle < 1.0);
    // Existing course steering may keep accelerating briefly while throttle
    // decays. Do not invent an immediate velocity stop for a live overlay.
    for _ in 0..120 {
        game.tick_co_op(session()).unwrap();
    }
    assert!(
        game.world
            .get::<&Motion>(game.player)
            .unwrap()
            .velocity
            .length()
            < previous.length() * 0.01
    );
}

#[test]
fn real_primary_shots_keep_owner_and_leaving_removes_only_that_members_shots() {
    let mut game = start();
    let a = guest(&mut game, 2);
    place(&mut game, host(), Vec2::new(450.0, 0.0), 0.0);
    place(&mut game, a.id, Vec2::new(450.0, 80.0), 0.0);
    game.submit_host_co_op_input(
        session(),
        0,
        InputFrame {
            fire: true,
            ..Default::default()
        },
    )
    .unwrap();
    renew(
        &mut game,
        a,
        0,
        InputFrame {
            fire: true,
            ..Default::default()
        },
    );
    game.tick_co_op(session()).unwrap();
    let snap = game.co_op_snapshot().unwrap();
    assert_eq!(snap.projectiles.len(), 2);
    assert!(
        snap.projectiles
            .iter()
            .any(|p| p.owner == Some(host()) && p.owner_epoch == Some(1))
    );
    assert!(
        snap.projectiles
            .iter()
            .any(|p| p.owner == Some(a.id) && p.owner_epoch == Some(a.epoch))
    );
    assert!(
        game.world
            .query::<(&Bullet, &ShotOwner)>()
            .iter()
            .all(|(b, _)| b.player)
    );
    game.leave_co_op(session(), a.id, a.epoch).unwrap();
    let snap = game.co_op_snapshot().unwrap();
    assert_eq!(snap.projectiles.len(), 1);
    assert_eq!(snap.projectiles[0].owner, Some(host()));
}

#[test]
fn hostile_swept_hit_damages_actual_guest_and_friendly_shots_never_pvp() {
    let mut game = start();
    let a = guest(&mut game, 2);
    place(&mut game, host(), Vec2::new(450.0, 200.0), 0.0);
    place(&mut game, a.id, Vec2::new(550.0, 0.0), 0.0);
    let host_before = hp(&game, host());
    let guest_before = hp(&game, a.id);
    game.spawn_bullet(
        Vec2::new(450.0, 0.0),
        Vec2::new(12000.0, 0.0),
        40.0,
        false,
        1,
    );
    game.tick_co_op(session()).unwrap();
    assert_eq!(hp(&game, host()), host_before);
    let guest_after = hp(&game, a.id);
    assert_eq!(guest_after.1, 0.0);
    assert!(guest_after.0 < guest_before.0); // Actual HUGO DR applies to this guest.
    game.spawn_bullet(
        Vec2::new(450.0, 0.0),
        Vec2::new(12000.0, 0.0),
        1000.0,
        true,
        1,
    );
    game.tick_co_op(session()).unwrap();
    assert_eq!(hp(&game, a.id), guest_after);
}

#[test]
fn ordinary_enemy_chooses_nearest_living_unsafe_pilot_not_host() {
    let mut game = start();
    let a = guest(&mut game, 2);
    place(&mut game, host(), Vec2::new(450.0, 0.0), 0.0);
    place(&mut game, a.id, Vec2::new(850.0, 0.0), 0.0);
    let enemy = game.spawn_enemy(1, false, Vec2::new(800.0, 0.0), 1);
    game.world.get::<&mut Brain>(enemy).unwrap().age = 1.0;
    game.world.get::<&mut Brain>(enemy).unwrap().shot_timer = 0.0;
    game.tick_co_op(session()).unwrap();
    let enemy_motion = *game.world.get::<&Motion>(enemy).unwrap();
    assert!(enemy_motion.heading.abs() < 0.01); // Guest is right; host is left.
    let shot = game
        .world
        .query::<(&Motion, &Bullet)>()
        .iter()
        .find(|(_, b)| !b.player)
        .map(|(m, _)| *m)
        .unwrap();
    assert!(shot.velocity.x > 0.0);
    place(&mut game, host(), Vec2::ZERO, 0.0);
    place(&mut game, a.id, Vec2::new(650.0, 0.0), 0.0);
    *game.world.get::<&mut Motion>(enemy).unwrap() =
        Motion::new(Vec2::new(300.0, 0.0), Vec2::ZERO, 0.0);
    game.tick_co_op(session()).unwrap();
    assert!(game.world.contains(enemy)); // Safe host cannot delete a guest's attacker.
}

#[test]
fn two_players_destroy_one_shared_asteroid_cells_once_without_rewards_or_loot_grants() {
    let mut game = start();
    let a = guest(&mut game, 2);
    place(&mut game, host(), Vec2::new(450.0, 0.0), 0.0);
    place(&mut game, a.id, Vec2::new(450.0, 15.0), 0.0);
    let rows = vec!["XXXXXXXXXXXXXXXX"; 16];
    let body = PixelBody::from_ascii(&rows, &[('X', rgba(120, 110, 100, 255))]);
    let asteroid = game.world.spawn((
        Motion::new(Vec2::new(570.0, 0.0), Vec2::ZERO, 0.0),
        Actor {
            kind: Kind::Asteroid,
            health: 48.0,
            max_health: 48.0,
            radius: 12.0,
            scale: 1.0,
            damage: 0.0,
        },
        body,
    ));
    let key = format!("{:016x}", asteroid.to_bits().get());
    let before = serde_json::to_string(game.profile()).unwrap();
    let mut saw_shared_hole = false;
    let mut previous = 256;
    for sequence in 0..120 {
        for admission in [
            Admission {
                id: host(),
                slot: 0,
                epoch: 1,
            },
            a,
        ] {
            renew(
                &mut game,
                admission,
                sequence,
                InputFrame {
                    fire: true,
                    aim: Some([570.0, 0.0]),
                    ..Default::default()
                },
            );
        }
        game.tick_co_op(session()).unwrap();
        if let Ok(body) = game.world.get::<&PixelBody>(asteroid) {
            let count = body.occupied_count();
            if count < previous {
                saw_shared_hole = true;
            }
            assert!(count <= previous);
            previous = count;
        } else {
            break;
        }
    }
    assert!(saw_shared_hole);
    assert!(!game.world.contains(asteroid));
    let notices = game.take_co_op_destructions(session()).unwrap();
    let deaths: Vec<_> = notices.iter().filter(|n| n.id == key).collect();
    assert_eq!(deaths.len(), 1);
    assert!(deaths[0].owner == Some(host()) || deaths[0].owner == Some(a.id));
    assert!(game.take_co_op_destructions(session()).unwrap().is_empty());
    assert_eq!(game.world.query::<&Loot>().iter().count(), 0);
    // Real existing pickup path cannot give either participant points/materials.
    game.spawn_loot(game.position(), 0, 10);
    game.spawn_material_loot(Vec2::new(450.0, 15.0), 1, 10);
    game.tick_co_op(session()).unwrap();
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), before);
}

#[test]
fn dead_guest_respawns_only_its_body_and_does_not_reset_host_or_profile() {
    let mut game = start();
    let a = guest(&mut game, 2);
    place(&mut game, host(), Vec2::new(450.0, 150.0), 0.0);
    place(&mut game, a.id, Vec2::new(550.0, 0.0), 0.0);
    let e = entity(&game, a.id);
    let original_count = game.world.get::<&PixelBody>(e).unwrap().occupied_count();
    game.spawn_bullet(
        Vec2::new(450.0, 0.0),
        Vec2::new(12000.0, 0.0),
        1000.0,
        false,
        1,
    );
    game.tick_co_op(session()).unwrap();
    let snapshot = game.co_op_snapshot().unwrap();
    assert!(Vec2::from_array(snapshot.participants[1].position).length() < SAFE_RADIUS);
    assert!(Vec2::from_array(snapshot.participants[0].position).length() > SAFE_RADIUS);
    assert_eq!(
        snapshot.participants[1].hull,
        snapshot.participants[1].max_hull
    );
    assert_eq!(
        game.world.get::<&PixelBody>(e).unwrap().occupied_count(),
        original_count
    );
    assert_eq!(
        game.take_events()
            .iter()
            .filter(|event| event.kind == "death")
            .count(),
        1
    );
    game.tick_co_op(session()).unwrap();
    assert_eq!(
        game.take_events()
            .iter()
            .filter(|event| event.kind == "death")
            .count(),
        0
    );
}

#[test]
fn all_public_progression_routes_and_diagnostics_cannot_mutate_retained_save() {
    let mut game = Game::new(Profile::default());
    game.profile.currency = [100; 3];
    game.profile.campaign.stored_materials = [100; 5];
    game.profile.campaign.components = [50; 3];
    game.profile.campaign.blueprints = [true; 8];
    game.profile.campaign.patches = [true; 8];
    game.profile.campaign.victory_claimed = true;
    let before = serde_json::to_string(&game.profile).unwrap();
    game.begin_co_op(PROTOCOL, session(), host()).unwrap();
    assert!(game.purchase("damage").is_err());
    assert!(game.craft_ship(0).is_err());
    assert!(game.switch_ship(0).is_err());
    assert!(game.craft_ability("pulse-lens").is_err());
    assert!(game.upgrade_ability("pulse-lens").is_err());
    assert!(game.equip_ability(0, None).is_err());
    assert!(game.craft_component("pulse-conductor").is_err());
    assert!(game.start_champion(0).is_err());
    assert!(game.start_tournament().is_err());
    assert!(game.interact_site().is_err());
    assert!(!game.can_register_droid());
    game.acknowledge_credits();
    game.diagnostic("warp 8000 0");
    game.diagnostic("home");
    let position = game.position();
    game.tick(
        &InputFrame {
            fire: true,
            joystick: Some([1.0, 0.0]),
            ..Default::default()
        },
        1.0,
    );
    assert_eq!(game.position(), position);
    assert_eq!(game.co_op_snapshot().unwrap().tick, 0);
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), before);
    assert!(!game.profile.campaign.credits_seen);
}

#[test]
fn identified_host_license_acknowledgement_rejects_without_changing_saved_training() {
    let profile = identity::create_profile("dora", "pan", "renko", "0123456789abcdef").unwrap();
    let mut game = Game::new(profile);
    let before = serde_json::to_string(game.profile()).unwrap();
    game.begin_co_op(PROTOCOL, session(), host()).unwrap();
    assert!(game.acknowledge_license().is_err());
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), before);
    game.end_co_op(session()).unwrap();
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), before);
    game.acknowledge_license().unwrap();
    assert_ne!(serde_json::to_string(game.profile()).unwrap(), before);
}

#[test]
fn end_host_leave_and_return_home_restore_exact_original_solo_instance() {
    for mode in 0..3 {
        let mut game = Game::new(Profile::default());
        game.profile.campaign.cargo_materials = [7, 0, 0, 0, 0];
        game.time = 42.5;
        game.rng = 919;
        game.shield = 12.0;
        game.destination = Some(Vec2::new(500.0, 0.0));
        game.throttle = 0.7;
        game.fire_timer = 0.125;
        game.camera = Vec2::new(80.0, 90.0);
        game.previous_camera = Vec2::new(79.0, 89.0);
        game.world.get::<&mut Actor>(game.player).unwrap().health = 75.0;
        game.world
            .get::<&mut PixelBody>(game.player)
            .unwrap()
            .damage(Vec2::ZERO, 1.0);
        game.spawn_bullet(Vec2::new(100.0, 100.0), Vec2::new(30.0, 40.0), 1.0, true, 1);
        game.event("test", Vec2::ZERO, 0.25);
        let original_player = game.player;
        let profile = serde_json::to_string(game.profile()).unwrap();
        let original_cells: Vec<_> = game
            .world
            .get::<&PixelBody>(game.player)
            .unwrap()
            .cells_indexed()
            .map(|(index, _, color)| (index, color))
            .collect();
        let original_entities = game.world.len();
        game.begin_co_op(PROTOCOL, session(), host()).unwrap();
        let a = guest(&mut game, 2);
        place(&mut game, host(), Vec2::new(450.0, 0.0), 0.0);
        game.submit_host_co_op_input(
            session(),
            0,
            InputFrame {
                fire: true,
                ..Default::default()
            },
        )
        .unwrap();
        game.tick_co_op(session()).unwrap();
        game.world
            .get::<&mut PixelBody>(game.player)
            .unwrap()
            .damage(Vec2::ZERO, 100.0);
        match mode {
            0 => {
                game.end_co_op(session()).unwrap();
            }
            1 => {
                game.leave_co_op(session(), host(), 1).unwrap();
            }
            _ => game.reset_to_dock(),
        }
        assert!(game.co_op_snapshot().is_none());
        assert_eq!(game.player, original_player);
        assert_eq!(game.world.len(), original_entities);
        assert!(game.world.query::<&Pilot>().iter().next().is_none());
        assert_eq!(serde_json::to_string(game.profile()).unwrap(), profile);
        assert_eq!(game.time, 42.5);
        assert_eq!(game.rng, 919);
        assert_eq!(game.shield, 12.0);
        assert_eq!(game.throttle, 0.7);
        assert_eq!(game.fire_timer, 0.125);
        assert_eq!(game.destination, Some(Vec2::new(500.0, 0.0)));
        assert_eq!(game.camera, Vec2::new(80.0, 90.0));
        assert_eq!(game.world.get::<&Actor>(game.player).unwrap().health, 75.0);
        assert_eq!(
            game.world
                .get::<&PixelBody>(game.player)
                .unwrap()
                .cells_indexed()
                .map(|(index, _, color)| (index, color))
                .collect::<Vec<_>>(),
            original_cells
        );
        assert_eq!(game.take_events()[0].kind, "test");
        assert_eq!(
            game.submit_guest_co_op_input(session(), a.id, a.epoch, 100, InputFrame::default()),
            Err(Error::NoSession)
        );
    }
}

#[test]
fn bounded_snapshot_pages_preserve_all_cells_and_reject_same_tick_admission_changes() {
    let mut game = start();
    let snapshot = game.co_op_snapshot().unwrap();
    let page = game
        .co_op_entity_page(session(), snapshot.revision, 0)
        .unwrap();
    let row = &page.entities[0];
    let mut cells = Vec::new();
    let mut offset = 0;
    loop {
        let part = game
            .co_op_body_page(session(), snapshot.revision, &row.id, offset)
            .unwrap();
        assert!(part.cells.len() <= co_op::CELL_PAGE_SIZE);
        cells.extend(part.cells);
        if let Some(next) = part.next_offset {
            offset = next;
        } else {
            break;
        }
    }
    let expected: Vec<_> = game
        .world
        .get::<&PixelBody>(game.player)
        .unwrap()
        .cells_indexed()
        .map(|(index, _, color)| [index as u32, color])
        .collect();
    assert_eq!(cells, expected);
    let a = guest(&mut game, 2);
    assert_eq!(game.co_op_snapshot().unwrap().tick, snapshot.tick);
    assert!(matches!(
        game.co_op_body_page(session(), snapshot.revision, &row.id, 0),
        Err(Error::StaleSnapshot)
    ));
    let after_join = game.co_op_snapshot().unwrap();
    game.leave_co_op(session(), a.id, a.epoch).unwrap();
    assert!(matches!(
        game.co_op_entity_page(session(), after_join.revision, 0),
        Err(Error::StaleSnapshot)
    ));
    let current = game.co_op_snapshot().unwrap();
    assert!(matches!(
        game.co_op_body_page(id(2000), current.revision, &row.id, 0),
        Err(Error::StaleSession)
    ));
    assert!(matches!(
        game.co_op_body_page(session(), current.revision, "0000000000000000", 0),
        Err(Error::InvalidEntity)
    ));
    assert!(matches!(
        game.co_op_body_page(session(), current.revision, &row.id, usize::MAX),
        Err(Error::InvalidOffset)
    ));
}

#[test]
fn authority_fills_shared_projectile_cap_and_actual_payload_remains_under_transport_limit() {
    let mut game = start();
    for i in 2..=8 {
        guest(&mut game, i);
    }
    // Actual authoritative serialization at edge coordinates, all 180 bullets.
    for index in 0..MAX_BULLETS {
        let position = Vec2::new(999_999.94 - index as f32 * 0.37, -999_997.3);
        let shot = game
            .spawn_bullet_entity(position, Vec2::new(430.125, -399.9375), 123.45678, true, 3)
            .unwrap();
        game.world
            .insert_one(
                shot,
                ShotOwner {
                    id: id((index % 8 + 1) as u64),
                    epoch: (index % 8 + 1) as u32,
                },
            )
            .unwrap();
    }
    assert!(
        game.spawn_bullet_entity(Vec2::ZERO, Vec2::X, 1.0, true, 1)
            .is_none()
    );
    let snapshot = game.co_op_snapshot().unwrap();
    assert_eq!(snapshot.projectiles.len(), 180);
    assert_eq!(snapshot.participants.len(), 8);
    let bytes = serde_json::to_vec(&snapshot).unwrap();
    println!("actual bounded snapshot bytes={}", bytes.len());
    assert!(bytes.len() < 64 * 1024 - 1024); // Leave room for transport envelope.
    let body = game
        .co_op_entity_page(session(), snapshot.revision, 0)
        .unwrap();
    assert!(body.entities.len() <= 64);
    assert!(serde_json::to_vec(&body).unwrap().len() < 64 * 1024 - 1024);
}

#[test]
fn solo_control_refactor_retains_current_cursor_thrust_cadence_and_quality_determinism() {
    let mut normal = Game::new(Profile::default());
    let mut reduced = Game::new(Profile::default());
    reduced.set_reduced_motion(true);
    reduced.set_cosmetic_density(true);
    for game in [&mut normal, &mut reduced] {
        game.move_player(Vec2::new(450.0, 0.0));
        game.spawn_timer = 10000.0;
        game.boss_timer = 10000.0;
    }
    for _ in 0..60 {
        let input = InputFrame {
            joystick: Some([0.5, 0.5]),
            aim: Some([800.0, 0.0]),
            fire: true,
            ..Default::default()
        };
        normal.tick(&input, co_op::TICK_SECONDS);
        reduced.tick(&input, co_op::TICK_SECONDS);
    }
    assert_eq!(normal.position(), reduced.position());
    assert_eq!(
        serde_json::to_string(normal.profile()).unwrap(),
        serde_json::to_string(reduced.profile()).unwrap()
    );
    let shots = |g: &Game| {
        g.world
            .query::<(&Motion, &Bullet)>()
            .iter()
            .map(|(m, b)| {
                (
                    m.position.to_array(),
                    m.velocity.to_array(),
                    b.damage,
                    b.life,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(shots(&normal), shots(&reduced));
    assert_eq!(normal.shot_count, 5);
}

/// Deliberately separate from ordinary tests: writes only native contract
/// fixtures under this checkout's work directory, not a network/play capture.
#[test]
#[ignore = "explicit native serialization fixture export"]
fn export_actual_authority_serialization_fixtures() {
    let mut game = start();
    let mut admissions = vec![Admission {
        id: host(),
        slot: 0,
        epoch: 1,
    }];
    for index in 2..=8 {
        admissions.push(guest(&mut game, index));
    }
    let input = InputFrame {
        joystick: Some([0.5, -0.25]),
        aim: Some([850.0, 80.0]),
        fire: true,
        ..Default::default()
    };
    for (index, admission) in admissions.iter().copied().enumerate() {
        place(
            &mut game,
            admission.id,
            Vec2::new(450.0 + index as f32 * 44.0, 0.0),
            0.0,
        );
        renew(&mut game, admission, 1, input.clone());
    }
    game.tick_co_op(session()).unwrap();
    let bullets = game.world.query::<&Bullet>().iter().count();
    for index in bullets..MAX_BULLETS {
        let shot = game
            .spawn_bullet_entity(
                Vec2::new(999_999.94 - index as f32 * 0.37, -999_997.3),
                Vec2::new(430.125, -399.9375),
                123.45678,
                true,
                3,
            )
            .unwrap();
        game.world
            .insert_one(
                shot,
                ShotOwner {
                    id: admissions[index % 8].id,
                    epoch: admissions[index % 8].epoch,
                },
            )
            .unwrap();
    }
    let snapshot = game.co_op_snapshot().unwrap();
    let entities = game
        .co_op_entity_page(session(), snapshot.revision, 0)
        .unwrap();
    let row = entities
        .entities
        .iter()
        .find(|row| row.participant == Some(id(2)))
        .unwrap();
    let body = game
        .co_op_body_page(session(), snapshot.revision, &row.id, 0)
        .unwrap();
    assert_eq!(snapshot.participants.len(), 8);
    assert_eq!(snapshot.projectiles.len(), 180);
    assert!(body.cells.len() <= 1024);
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("work/co-op-authority/serialization");
    std::fs::create_dir_all(&output).unwrap();
    let files = [
        ("snapshot.json", serde_json::to_vec(&snapshot).unwrap()),
        ("entities.json", serde_json::to_vec(&entities).unwrap()),
        ("body.json", serde_json::to_vec(&body).unwrap()),
        ("admissions.json", serde_json::to_vec(&admissions).unwrap()),
        (
            "input.json",
            serde_json::to_vec(&serde_json::json!({
                "epoch": admissions[1].epoch, "seq": 1, "input": input,
            }))
            .unwrap(),
        ),
    ];
    for (name, mut bytes) in files {
        assert!(bytes.len() < 64 * 1024 - 1024);
        println!("actual authority fixture {name}: {} bytes", bytes.len());
        bytes.push(b'\n');
        std::fs::write(output.join(name), bytes).unwrap();
    }
}
