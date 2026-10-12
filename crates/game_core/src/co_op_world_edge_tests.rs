//! Real unfiltered chunk publication at the player coordinate boundary.
use super::*;

#[test]
fn actual_edge_chunks_commit_without_removing_generated_entities() {
    for point in [
        Vec2::new(WORLD_LIMIT, 0.0),
        Vec2::new(-WORLD_LIMIT, 0.0),
        Vec2::new(0.0, WORLD_LIMIT),
        Vec2::new(0.0, -WORLD_LIMIT),
        Vec2::splat(WORLD_LIMIT),
        Vec2::splat(-WORLD_LIMIT),
        Vec2::new(WORLD_LIMIT, -WORLD_LIMIT),
        Vec2::new(-WORLD_LIMIT, WORLD_LIMIT),
    ] {
        let mut game = Game::new(Profile {
            seed: 812345,
            ..Profile::default()
        });
        game.begin_co_op(PROTOCOL, id(100), id(1)).unwrap();
        let guest = game
            .join_co_op(
                id(100),
                ParticipantSpec {
                    id: id(2),
                    local_seed: 98767,
                    identity: None,
                },
            )
            .unwrap();
        let peer = game
            .world
            .query::<(hecs::Entity, &co_op::Pilot)>()
            .iter()
            .find(|(_, p)| p.admission.id == guest.id)
            .unwrap()
            .0;
        *game.world.get::<&mut Motion>(peer).unwrap() = Motion::new(point, Vec2::ZERO, 0.0);
        game.spawn_timer = 10000.0;
        game.tick_co_op(id(100)).unwrap();
        // Keep every real streamed asteroid. The original small replica fixture
        // deliberately removes these and cannot exercise this failure.
        let data = capture(&mut game);
        let outside: Vec<_> = data
            .entities
            .iter()
            .flat_map(|p| &p.entities)
            .filter(|row| {
                row.kind == "asteroid" && row.position.iter().any(|v| v.abs() > WORLD_LIMIT)
            })
            .collect();
        assert!(
            !outside.is_empty(),
            "actual edge chunk fixture must contain the original outlying geometry"
        );
        println!(
            "edge={point:?}; generated_outside_player_domain={}; first_position={:?}",
            outside.len(),
            outside[0].position
        );
        let mut replica = replica(guest.epoch);
        stage(&mut replica, &data, true);
        commit(&mut replica, &data);
        let scene = replica.scene().unwrap();
        assert_eq!(scene.entities.len(), data.header.counts.entities);
        for row in outside {
            let (entity, body) = scene
                .entities
                .iter()
                .find(|(entity, _)| entity.id == row.id)
                .unwrap();
            assert_eq!(entity.position, row.position);
            assert_eq!(entity.previous, row.previous);
            assert_eq!(body_signature(body), row.body_signature);
        }
    }
}

const LIMIT: f32 = crate::co_op_world_bounds::GEOMETRY_LIMIT;

fn admitted_edge() -> (Game, co_op::Admission, hecs::Entity) {
    let mut game = Game::new(Profile {
        seed: 812345,
        ..Profile::default()
    });
    game.begin_co_op(PROTOCOL, id(100), id(1)).unwrap();
    let guest = game
        .join_co_op(
            id(100),
            ParticipantSpec {
                id: id(2),
                local_seed: 98767,
                identity: None,
            },
        )
        .unwrap();
    let peer = game
        .world
        .query::<(hecs::Entity, &co_op::Pilot)>()
        .iter()
        .find(|(_, p)| p.admission.id == guest.id)
        .unwrap()
        .0;
    *game.world.get::<&mut Motion>(peer).unwrap() =
        Motion::new(Vec2::new(WORLD_LIMIT, 0.0), Vec2::ZERO, 0.0);
    game.spawn_timer = 10000.0;
    (game, guest, peer)
}

fn assert_complete(game: &mut Game, epoch: u32) {
    let data = capture(game);
    let mut r = replica(epoch);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    assert_eq!(
        r.scene().unwrap().entities().len(),
        data.header.counts.entities
    );
}

#[test]
fn real_outbound_edge_projectile_commits_with_owned_epoch_and_strict_pilot_domain() {
    let (mut game, guest, _) = admitted_edge();
    game.submit_guest_co_op_input(
        id(100),
        guest.id,
        guest.epoch,
        1,
        InputFrame {
            fire: true,
            aim: Some([WORLD_LIMIT, 0.0]),
            ..Default::default()
        },
    )
    .unwrap();
    game.tick_co_op(id(100)).unwrap();
    let data = capture(&mut game);
    let shot = data
        .header
        .snapshot
        .projectiles
        .iter()
        .find(|shot| shot.owner == Some(guest.id) && shot.position[0] > WORLD_LIMIT)
        .unwrap();
    assert!(shot.previous[0] > WORLD_LIMIT && shot.position[0] <= LIMIT);
    assert_eq!(shot.owner_epoch, Some(guest.epoch));
    assert!(
        data.header
            .snapshot
            .participants
            .iter()
            .all(|p| p.position.iter().all(|v| v.abs() <= WORLD_LIMIT))
    );
    let mut r = replica(guest.epoch);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    assert_eq!(
        r.scene().unwrap().header().snapshot.projectiles.len(),
        data.header.snapshot.projectiles.len()
    );
    println!(
        "actual_outbound_shot_position={:?}; previous={:?}; owner_epoch={}",
        shot.position, shot.previous, guest.epoch
    );
}

#[test]
fn retained_generated_asteroid_drift_reaches_outer_boundary_and_stays_publishable() {
    let (mut game, guest, _) = admitted_edge();
    game.tick_co_op(id(100)).unwrap();
    let rock = game
        .world
        .query::<(hecs::Entity, (&Actor, &AsteroidBirth))>()
        .iter()
        .find(|(_, (actor, birth))| actor.kind == Kind::Asteroid && birth.orbit.is_none())
        .unwrap()
        .0;
    let birth_chunk = game
        .chunks
        .iter()
        .find(|(_, entities)| entities.contains(&rock))
        .unwrap()
        .0;
    let birth_chunk = *birth_chunk;
    let signature = body_signature(&game.world.get::<&PixelBody>(rock).unwrap());
    let saved = serde_json::to_value(game.profile()).unwrap();
    *game.world.get::<&mut Motion>(rock).unwrap() = Motion::new(
        Vec2::splat(f32::from_bits(LIMIT.to_bits() - 1)),
        Vec2::splat(160.0),
        0.0,
    );
    for tick in 1..=300 {
        game.tick_co_op(id(100)).unwrap();
        let motion = *game.world.get::<&Motion>(rock).unwrap();
        assert_eq!(motion.position, Vec2::splat(LIMIT));
        assert_eq!(motion.velocity, Vec2::ZERO);
        assert!(motion.previous.abs().max_element() <= LIMIT);
        assert!(game.chunks.get(&birth_chunk).unwrap().contains(&rock));
        if [1, 30, 300].contains(&tick) {
            assert_complete(&mut game, guest.epoch);
        }
    }
    assert_eq!(
        body_signature(&game.world.get::<&PixelBody>(rock).unwrap()),
        signature
    );
    assert_eq!(serde_json::to_value(game.profile()).unwrap(), saved);
    println!(
        "actual_retained_asteroid_ticks=300; unchanged_body=true; retained_birth_chunk=true; corner_boundary={LIMIT}"
    );
}

#[test]
fn final_tick_confinement_handles_collision_correction_segments_and_solo_drift() {
    let (mut game, epoch) = fixture(2);
    let a = object(&mut game);
    let b = object(&mut game);
    let left = Vec2::new(LIMIT - 1.0, 0.0);
    let right = Vec2::new(f32::from_bits(LIMIT.to_bits() - 1), 0.0);
    *game.world.get::<&mut Motion>(a).unwrap() = Motion::new(left, Vec2::ZERO, 0.0);
    *game.world.get::<&mut Motion>(b).unwrap() = Motion::new(right, Vec2::ZERO, 0.0);
    game.resolve_collisions(co_op::TICK_SECONDS);
    assert!(
        game.world.get::<&Motion>(b).unwrap().position.x > LIMIT,
        "the actual late collision must cross the outer domain"
    );
    *game.world.get::<&mut Motion>(a).unwrap() = Motion::new(left, Vec2::ZERO, 0.0);
    *game.world.get::<&mut Motion>(b).unwrap() = Motion::new(right, Vec2::ZERO, 0.0);
    // A real attached chain can place a subobject beyond the head's domain.
    let head = game.world.spawn((
        Motion::new(Vec2::new(LIMIT, 300.0), Vec2::ZERO, std::f32::consts::PI),
        Actor {
            kind: Kind::Asteroid,
            health: 100.0,
            max_health: 100.0,
            radius: 2.0,
            scale: 1.0,
            damage: 0.0,
        },
        PixelBody::from_ascii(&["####"; 4], &[('#', rgba(3, 5, 7, 255))]),
    ));
    game.spawn_chain(
        head,
        PixelBody::from_ascii(&["####"; 4], &[('#', rgba(7, 9, 11, 255))]),
        1.0,
        2,
        10.0,
    );
    game.update_segments(co_op::TICK_SECONDS);
    assert!(
        game.world
            .query::<(&Motion, &Segment)>()
            .iter()
            .any(|(motion, _)| motion.position.x > LIMIT),
        "actual chain placement must exercise an outer subobject"
    );
    game.tick_co_op(id(100)).unwrap();
    assert_eq!(game.world.get::<&Motion>(b).unwrap().position.x, LIMIT);
    assert!(
        game.world
            .query::<(&Motion, &PixelBody)>()
            .iter()
            .all(|(motion, _)| motion.position.abs().max_element() <= LIMIT
                && motion.previous.abs().max_element() <= LIMIT)
    );
    assert_complete(&mut game, epoch);

    let mut solo = Game::new(Profile::default());
    solo.spawn_timer = 10000.0;
    let rock = object(&mut solo);
    *solo.world.get::<&mut Motion>(rock).unwrap() =
        Motion::new(Vec2::new(-LIMIT + 0.125, 0.0), -Vec2::X * 160.0, 0.0);
    solo.tick(&InputFrame::default(), co_op::TICK_SECONDS);
    let motion = solo.world.get::<&Motion>(rock).unwrap();
    assert_eq!(motion.position.x, -LIMIT);
    assert_eq!(motion.velocity.x, 0.0);
    assert!(motion.previous.x >= -LIMIT);
}

#[test]
fn inner_geometry_is_noop_and_generic_confinement_never_normalizes_pilots() {
    let (mut game, epoch) = fixture(2);
    let body = object(&mut game);
    let original = *game.world.get::<&Motion>(body).unwrap();
    let signature = body_signature(&game.world.get::<&PixelBody>(body).unwrap());
    game.confine_world_geometry();
    let actual = *game.world.get::<&Motion>(body).unwrap();
    assert_eq!(
        (
            actual.position,
            actual.previous,
            actual.velocity,
            actual.heading,
            actual.previous_heading
        ),
        (
            original.position,
            original.previous,
            original.velocity,
            original.heading,
            original.previous_heading
        )
    );
    assert_eq!(
        body_signature(&game.world.get::<&PixelBody>(body).unwrap()),
        signature
    );
    *game.world.get::<&mut Motion>(body).unwrap() =
        Motion::new(Vec2::splat(LIMIT + 1.0), Vec2::new(12.0, -9.0), 0.0);
    game.confine_world_geometry();
    let bounded = *game.world.get::<&Motion>(body).unwrap();
    assert_eq!(bounded.position, Vec2::splat(LIMIT));
    assert_eq!(bounded.previous, Vec2::splat(LIMIT));
    assert_eq!(
        bounded.velocity,
        Vec2::new(0.0, -9.0),
        "cancel outward motion only, retaining inward/tangent motion"
    );
    let next_pilot = f32::from_bits(WORLD_LIMIT.to_bits() + 1);
    game.world
        .get::<&mut Motion>(game.player)
        .unwrap()
        .position
        .x = next_pilot;
    game.confine_world_geometry();
    assert_eq!(
        game.world.get::<&Motion>(game.player).unwrap().position.x,
        next_pilot
    );
    assert!(matches!(
        game.acquire_co_op_publication(id(100)),
        Err(co_op::Error::PublicationBound)
    ));
    assert_eq!(
        game.submit_guest_co_op_input(
            id(100),
            id(2),
            epoch,
            1,
            InputFrame {
                thrust: Some([next_pilot, 0.0]),
                ..Default::default()
            }
        ),
        Err(co_op::Error::InvalidInput)
    );
}

#[test]
fn exact_geometry_and_velocity_limits_commit_but_camera_and_pilot_stay_strict() {
    let (mut game, epoch) = fixture(2);
    let entity = object(&mut game);
    let mut motion = Motion::new(
        Vec2::new(LIMIT, -LIMIT),
        Vec2::new(WORLD_LIMIT, -WORLD_LIMIT),
        0.0,
    );
    motion.previous = -motion.position;
    *game.world.get::<&mut Motion>(entity).unwrap() = motion;
    let data = capture(&mut game);
    let mut r = replica(epoch);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    let mut invalid_view = view();
    invalid_view.camera.x = f32::from_bits(WORLD_LIMIT.to_bits() + 1);
    assert_eq!(
        r.scene().unwrap().render_into_with_home(
            &mut FrameBuilder::new(),
            invalid_view,
            |_, _, _| false
        ),
        Err("invalid_view")
    );
    let mut bad = data.header.clone();
    bad.acquisition += 1;
    bad.snapshot.participants[1].position[0] = f32::from_bits(WORLD_LIMIT.to_bits() + 1);
    assert_eq!(r.stage(bad), Err("invalid_pilot"));
    assert_eq!(
        r.scene().unwrap().header().acquisition,
        data.header.acquisition
    );
}

#[test]
fn malformed_body_and_shot_admission_preserves_prior_native_cache_and_guest_scene() {
    let (mut game, epoch) = fixture(2);
    let entity = object(&mut game);
    let first = capture(&mut game);
    let token = first.header.acquisition;
    let revision = first.header.snapshot.revision;
    let original = *game.world.get::<&Motion>(entity).unwrap();
    for (field, value) in [
        (0, f32::from_bits(LIMIT.to_bits() + 1)),
        (1, -f32::from_bits(LIMIT.to_bits() + 1)),
        (2, f32::from_bits(WORLD_LIMIT.to_bits() + 1)),
        (0, f32::NAN),
        (1, f32::INFINITY),
        (2, f32::NAN),
    ] {
        let mut motion = original;
        match field {
            0 => motion.position.x = value,
            1 => motion.previous.y = value,
            _ => motion.velocity.x = value,
        }
        *game.world.get::<&mut Motion>(entity).unwrap() = motion;
        if !value.is_finite() {
            game.confine_world_geometry();
        }
        assert!(matches!(
            game.acquire_co_op_publication(id(100)),
            Err(co_op::Error::PublicationBound)
        ));
        assert_eq!(
            game.co_op_publication(id(100), token, revision)
                .unwrap()
                .acquisition(),
            token
        );
    }
    *game.world.get::<&mut Motion>(entity).unwrap() = original;
    let shot = game
        .spawn_bullet_entity(Vec2::new(500.0, 0.0), Vec2::X * 100.0, 1.0, true, 1)
        .unwrap();
    let shot_original = *game.world.get::<&Motion>(shot).unwrap();
    for (field, value) in [
        (0, f32::from_bits(LIMIT.to_bits() + 1)),
        (1, f32::INFINITY),
        (2, f32::NAN),
    ] {
        let mut motion = shot_original;
        match field {
            0 => motion.position.x = value,
            1 => motion.previous.y = value,
            _ => motion.velocity.x = value,
        }
        *game.world.get::<&mut Motion>(shot).unwrap() = motion;
        game.confine_world_geometry();
        if value.is_finite() {
            game.world.get::<&mut Motion>(shot).unwrap().position.x = value;
        }
        assert!(matches!(
            game.acquire_co_op_publication(id(100)),
            Err(co_op::Error::PublicationBound)
        ));
        assert!(game.co_op_publication(id(100), token, revision).is_ok());
    }
    let mut r = replica(epoch);
    stage(&mut r, &first, true);
    commit(&mut r, &first);
    for index in 0..4 {
        let mut bad = first.clone();
        bad.header.acquisition += index + 1;
        let row = bad
            .entities
            .iter_mut()
            .flat_map(|page| &mut page.entities)
            .find(|row| row.id == key(entity))
            .unwrap();
        match index {
            0 => row.position[0] = f32::from_bits(LIMIT.to_bits() + 1),
            1 => row.previous[0] = -f32::from_bits(LIMIT.to_bits() + 1),
            2 => row.velocity[0] = f32::from_bits(WORLD_LIMIT.to_bits() + 1),
            _ => row.previous[0] = f32::NAN,
        }
        r.stage(bad.header.clone()).unwrap();
        assert_eq!(
            r.entities(bad.header.acquisition, bad.entities[0].clone()),
            Err("invalid_entity")
        );
        assert_eq!(r.scene().unwrap().header().acquisition, token);
    }
    println!("actual_malformed_pose_cache_retention=true; prior_acquisition={token}");
}

#[test]
fn exact_projectile_geometry_limits_and_invalid_headers_keep_prior_scene() {
    let (mut game, epoch) = fixture(2);
    let shot = game
        .spawn_bullet_entity(
            Vec2::new(LIMIT, -LIMIT),
            Vec2::new(WORLD_LIMIT, -WORLD_LIMIT),
            1.0,
            true,
            1,
        )
        .unwrap();
    game.world.get::<&mut Motion>(shot).unwrap().previous = Vec2::new(-LIMIT, LIMIT);
    let data = capture(&mut game);
    let mut r = replica(epoch);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    for (index, (field, value)) in [
        (0, f32::from_bits(LIMIT.to_bits() + 1)),
        (1, -f32::from_bits(LIMIT.to_bits() + 1)),
        (2, f32::from_bits(WORLD_LIMIT.to_bits() + 1)),
        (0, f32::NAN),
        (1, f32::INFINITY),
        (2, f32::NAN),
    ]
    .into_iter()
    .enumerate()
    {
        let mut bad = data.header.clone();
        bad.acquisition += index as u32 + 1;
        let projectile = &mut bad.snapshot.projectiles[0];
        match field {
            0 => projectile.position[0] = value,
            1 => projectile.previous[0] = value,
            _ => projectile.velocity[0] = value,
        }
        assert_eq!(r.stage(bad), Err("invalid_projectile"));
        assert_eq!(
            r.scene().unwrap().header().acquisition,
            data.header.acquisition
        );
    }
    for (field, value) in [
        (0, f32::from_bits(WORLD_LIMIT.to_bits() + 1)),
        (1, -f32::from_bits(WORLD_LIMIT.to_bits() + 1)),
        (2, f32::NAN),
    ] {
        let original = *game.world.get::<&Motion>(game.player).unwrap();
        let mut invalid = original;
        match field {
            0 => invalid.position[0] = value,
            1 => invalid.previous[1] = value,
            _ => invalid.velocity[0] = value,
        }
        *game.world.get::<&mut Motion>(game.player).unwrap() = invalid;
        game.confine_world_geometry();
        assert!(matches!(
            game.acquire_co_op_publication(id(100)),
            Err(co_op::Error::PublicationBound)
        ));
        assert!(
            game.co_op_publication(
                id(100),
                data.header.acquisition,
                data.header.snapshot.revision
            )
            .is_ok()
        );
        *game.world.get::<&mut Motion>(game.player).unwrap() = original;
    }
}

/// Explicit native fixture export only. It uses the same real unfiltered Game
/// captures as the edge regressions; no Profile/loadout/header value is authored.
#[test]
#[ignore = "explicit native edge serializer export; requires GALAXY_NATIVE_EDGE_FIXTURES"]
fn export_actual_edge_publication_fixtures() {
    use std::io::Write;
    let output = std::path::PathBuf::from(std::env::var("GALAXY_NATIVE_EDGE_FIXTURES").unwrap());
    assert!(
        output.is_absolute(),
        "export requires an explicit absolute owned output path"
    );
    std::fs::create_dir_all(&output).unwrap();
    let write = |name: &str, value: &Value| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output.join(name))
            .unwrap();
        file.write_all(serde_json::to_string_pretty(value).unwrap().as_bytes())
            .unwrap();
        file.write_all(b"\n").unwrap();
    };
    for (name, point) in [
        ("positive-x", Vec2::new(WORLD_LIMIT, 0.0)),
        ("negative-x", Vec2::new(-WORLD_LIMIT, 0.0)),
        ("positive-y", Vec2::new(0.0, WORLD_LIMIT)),
        ("negative-y", Vec2::new(0.0, -WORLD_LIMIT)),
        ("positive-x-positive-y", Vec2::splat(WORLD_LIMIT)),
        ("negative-x-negative-y", Vec2::splat(-WORLD_LIMIT)),
        (
            "positive-x-negative-y",
            Vec2::new(WORLD_LIMIT, -WORLD_LIMIT),
        ),
        (
            "negative-x-positive-y",
            Vec2::new(-WORLD_LIMIT, WORLD_LIMIT),
        ),
    ] {
        let (mut game, guest, peer) = admitted_edge();
        *game.world.get::<&mut Motion>(peer).unwrap() = Motion::new(point, Vec2::ZERO, 0.0);
        game.tick_co_op(id(100)).unwrap();
        let data = capture(&mut game);
        let mut r = replica(guest.epoch);
        stage(&mut r, &data, true);
        commit(&mut r, &data);
        let rows = data.entities.iter().flat_map(|page| &page.entities).count();
        let outside = data
            .entities
            .iter()
            .flat_map(|page| &page.entities)
            .filter(|row| {
                row.kind == "asteroid" && row.position.iter().any(|value| value.abs() > WORLD_LIMIT)
            })
            .count();
        assert!(outside > 0);
        assert_eq!(r.scene().unwrap().entities().len(), rows);
        write(
            &format!("{name}.header.json"),
            &serde_json::to_value(&data.header).unwrap(),
        );
        write(
            &format!("{name}.entities.json"),
            &serde_json::to_value(&data.entities).unwrap(),
        );
        write(
            &format!("{name}.json"),
            &json!({
                "header": data.header, "entities": data.entities, "bodies": data.bodies,
            }),
        );
        println!(
            "actual_edge_export={name}; rows={rows}; outside_asteroids={outside}; complete_native_commit=true"
        );
    }
    let (mut game, guest, _) = admitted_edge();
    game.submit_guest_co_op_input(
        id(100),
        guest.id,
        guest.epoch,
        1,
        InputFrame {
            fire: true,
            aim: Some([WORLD_LIMIT, 0.0]),
            ..Default::default()
        },
    )
    .unwrap();
    game.tick_co_op(id(100)).unwrap();
    let data = capture(&mut game);
    let shot = data
        .header
        .snapshot
        .projectiles
        .iter()
        .find(|shot| shot.owner == Some(guest.id) && shot.position[0] > WORLD_LIMIT)
        .unwrap();
    assert_eq!(shot.owner_epoch, Some(guest.epoch));
    assert!(shot.previous[0] > WORLD_LIMIT);
    assert!(
        data.header
            .snapshot
            .participants
            .iter()
            .all(|p| p.owned_loadout.is_none()),
        "this fixture proves ShotOwner ownership only, not an owned-loadout build"
    );
    let mut r = replica(guest.epoch);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    println!(
        "actual_outward_export_position={:?}; previous={:?}; shot_owner_epoch={}; owned_loadout=false; complete_native_commit=true",
        shot.position, shot.previous, guest.epoch
    );
    write(
        "outward-projectile.header.json",
        &serde_json::to_value(&data.header).unwrap(),
    );
    write(
        "outward-projectile.entities.json",
        &serde_json::to_value(&data.entities).unwrap(),
    );
    write(
        "outward-projectile.json",
        &json!({
            "header": data.header, "entities": data.entities, "bodies": data.bodies,
        }),
    );
}
