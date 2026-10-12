//! Real authoritative ticks; private placement/cleanup only arrange quiet probes.
use super::*;

fn id(n: u32) -> ParticipantId {
    ParticipantId::parse(&format!("{n:032x}")).unwrap()
}
fn session() -> ParticipantId {
    id(100)
}
fn licensed(corporation: &str, n: u32) -> Profile {
    identity::create_profile("talo", corporation, "pan", &format!("{n:016x}")).unwrap()
}
fn spec(profile: &Profile, n: u32) -> ParticipantSpec {
    ParticipantSpec {
        id: id(n),
        local_seed: profile.seed,
        identity: profile.identity.clone(),
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
fn phase(game: &Game, peer: ParticipantId) -> u8 {
    game.world
        .get::<&Pilot>(entity(game, peer))
        .unwrap()
        .primary_phase
}
fn clear_bullets(game: &mut Game) {
    let shots: Vec<_> = game
        .world
        .query::<(Entity, &Bullet)>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for shot in shots {
        game.world.despawn(shot).unwrap();
    }
}
fn place(game: &mut Game, peer: ParticipantId, position: Vec2) {
    let e = entity(game, peer);
    *game.world.get::<&mut Motion>(e).unwrap() = Motion::new(position, Vec2::ZERO, 0.0);
    game.world.get::<&mut Pilot>(e).unwrap().was_home = position.length() <= SAFE_RADIUS;
}
fn start(profile: Profile, guests: &[&str]) -> (Game, Vec<Admission>) {
    let mut game = Game::new(profile);
    // Restoration must retain this solo combat state, not any shared phase.
    game.shot_count = 17;
    let mut admissions = vec![game.begin_co_op(PROTOCOL, session(), id(1)).unwrap()];
    for (i, corporation) in guests.iter().enumerate() {
        let n = i as u32 + 2;
        admissions.push(
            game.join_co_op(session(), spec(&licensed(corporation, n), n))
                .unwrap(),
        );
    }
    let remove: Vec<_> = game
        .world
        .query::<(Entity, Option<&Pilot>)>()
        .iter()
        .filter(|(_, p)| p.is_none())
        .map(|(e, _)| e)
        .collect();
    for e in remove {
        game.world.despawn(e).unwrap();
    }
    game.spawn_timer = 10000.0;
    for a in &admissions {
        place(&mut game, a.id, Vec2::new(450.0, a.slot as f32 * 80.0));
    }
    (game, admissions)
}
#[derive(Clone, Copy)]
struct Shot {
    owner: ShotOwner,
    motion: Motion,
    bullet: Bullet,
}
fn step(game: &mut Game, admissions: &[Admission], seq: &mut u32, fire: [bool; 8]) -> Vec<Shot> {
    let previous: HashSet<_> = game
        .world
        .query::<(Entity, &Bullet)>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    *seq += 1;
    for a in admissions {
        let input = InputFrame {
            fire: fire[a.slot as usize],
            ..Default::default()
        };
        if a.slot == 0 {
            game.submit_host_co_op_input(session(), *seq, input)
                .unwrap();
        } else {
            game.submit_guest_co_op_input(session(), a.id, a.epoch, *seq, input)
                .unwrap();
        }
    }
    game.tick_co_op(session()).unwrap();
    game.take_events();
    game.world
        .query::<(Entity, (&Motion, &Bullet, &ShotOwner))>()
        .iter()
        .filter(|(e, _)| !previous.contains(e))
        .map(|(_, (m, b, owner))| Shot {
            owner: *owner,
            motion: *m,
            bullet: *b,
        })
        .collect()
}
fn next(game: &mut Game, admissions: &[Admission], seq: &mut u32, peer: Admission) -> Vec<Shot> {
    let mut fire = [false; 8];
    fire[peer.slot as usize] = true;
    for _ in 0..60 {
        let shots = step(game, admissions, seq, fire);
        if !shots.is_empty() {
            return shots;
        }
    }
    panic!("No real primary admitted within sixty authoritative ticks");
}
fn verify(shots: &[Shot], peer: Admission, damage: f32, fifth: bool) {
    assert_eq!(shots.len(), if fifth { 2 } else { 1 });
    assert!(shots.iter().all(|s| s.owner.id == peer.id
        && s.owner.epoch == peer.epoch
        && s.bullet.player
        && s.bullet.remaining_hits == 1));
    let primary = shots.iter().find(|s| s.bullet.damage == damage).unwrap();
    assert!((primary.motion.velocity.x - 430.0).abs() < 0.001);
    if fifth {
        let bonus = shots
            .iter()
            .find(|s| s.bullet.damage == damage * 1.5)
            .unwrap();
        assert!((bonus.motion.velocity.x - 500.0).abs() < 0.001);
        // Motion.previous retains the actual spawn points before integration.
        assert!((bonus.motion.previous.x - primary.motion.previous.x - 2.0).abs() < 0.001);
    }
}

#[test]
fn two_and_eight_staggered_pilots_count_only_their_real_successful_primaries() {
    for players in [2usize, 8] {
        let (mut game, admissions) = start(licensed("renko", 1), &vec!["renko"; players - 1]);
        let before = serde_json::to_string(game.profile()).unwrap();
        let mut seq = 0;
        let mut primary = [0usize; 8];
        let mut bonuses = [0usize; 8];
        for tick in 0..600 {
            let fire = std::array::from_fn(|slot| primary[slot] < 20 && tick % (slot + 2) != 0);
            let shots = step(&mut game, &admissions, &mut seq, fire);
            for a in &admissions {
                let own: Vec<_> = shots
                    .iter()
                    .filter(|s| s.owner.id == a.id)
                    .copied()
                    .collect();
                if !own.is_empty() {
                    primary[a.slot as usize] += 1;
                    let fifth = primary[a.slot as usize].is_multiple_of(5);
                    let damage = game
                        .world
                        .get::<&Pilot>(entity(&game, a.id))
                        .unwrap()
                        .stats
                        .damage;
                    verify(&own, *a, damage, fifth);
                    bonuses[a.slot as usize] += usize::from(fifth);
                }
                assert_eq!(phase(&game, a.id), (primary[a.slot as usize] % 5) as u8);
            }
            // Remove already inspected shots to isolate the next firing tick.
            clear_bullets(&mut game);
            if primary[..players].iter().all(|&n| n == 20) {
                break;
            }
        }
        assert!(primary[..players].iter().all(|&n| n == 20));
        assert!(bonuses[..players].iter().all(|&n| n == 4));
        assert_eq!(serde_json::to_string(game.profile()).unwrap(), before);
        assert!(!game.co_op_snapshot().unwrap().progression);
    }
}

#[test]
fn equipped_native_rank_not_corporation_controls_factory_and_owned_bonus() {
    for (main, equipped, xp, trained, expected_rank) in [
        ("pan", true, 0, false, 1),
        ("hugo", true, 1560, false, 5),
        ("renko", true, 0, true, 2),
        ("renko", false, 1560, false, 0),
    ] {
        let mut profile = licensed(main, 2);
        profile.campaign.ship_xp[0] = xp;
        profile.campaign.abilities[0] = campaign::AbilityState {
            blueprint: true,
            rank: 5,
        };
        profile.campaign.loadouts[0].slots[0] = equipped.then_some(0);
        if trained {
            profile.onboarding = Some(identity::OnboardingProgress {
                stage: 6,
                flags: 127,
                asteroids_destroyed: 10,
                materials_collected: 10,
                ..Default::default()
            });
        }
        let solo = Game::new(profile);
        let saved = serde_json::to_string(solo.profile()).unwrap();
        let (mut game, mut admissions) = start(Profile::default(), &[]);
        let a = game
            .join_co_op_owned(
                session(),
                spec(solo.profile(), 2),
                solo.co_op_owned_loadout().unwrap(),
            )
            .unwrap();
        admissions.push(a);
        place(&mut game, a.id, Vec2::new(450.0, 80.0));
        let damage = game
            .world
            .get::<&Pilot>(entity(&game, a.id))
            .unwrap()
            .stats
            .damage;
        assert_eq!(
            game.world
                .get::<&Pilot>(entity(&game, a.id))
                .unwrap()
                .pulse_lens_rank,
            expected_rank
        );
        assert_eq!(damage, solo.hud().ship_stats.primary_damage);
        let mut seq = 0;
        for number in 1..=10 {
            let shots = next(&mut game, &admissions, &mut seq, a);
            verify(&shots, a, damage, expected_rank > 0 && number % 5 == 0);
            clear_bullets(&mut game);
        }
        assert_eq!(serde_json::to_string(solo.profile()).unwrap(), saved);
    }
    // Missing identity in the unchanged v1 path uses the actual default slots.
    let (mut game, mut admissions) = start(Profile::default(), &[]);
    let p = Profile::default();
    let a = game.join_co_op(session(), spec(&p, 2)).unwrap();
    admissions.push(a);
    place(&mut game, a.id, Vec2::new(450.0, 80.0));
    assert_eq!(
        game.world
            .get::<&Pilot>(entity(&game, a.id))
            .unwrap()
            .pulse_lens_rank,
        ship_progression::equipped_rank(&p, 0)
    );
    let damage = Stats::from_profile(&p).damage;
    let mut seq = 0;
    for _ in 0..5 {
        verify(&next(&mut game, &admissions, &mut seq, a), a, damage, false);
        clear_bullets(&mut game);
    }
}

#[test]
fn fifth_bolt_inherits_the_actual_moving_pilot_velocity_and_spawn_direction() {
    let (mut game, admissions) = start(licensed("renko", 1), &[]);
    let a = admissions[0];
    let mut seq = 0;
    for _ in 0..4 {
        next(&mut game, &admissions, &mut seq, a);
        clear_bullets(&mut game);
    }
    let e = entity(&game, a.id);
    game.world.get::<&mut Motion>(e).unwrap().velocity = Vec2::new(37.0, -29.0);
    let shots = next(&mut game, &admissions, &mut seq, a);
    assert_eq!(shots.len(), 2);
    let pilot_motion = *game.world.get::<&Motion>(e).unwrap();
    let damage = game.world.get::<&Pilot>(e).unwrap().stats.damage;
    let primary = shots.iter().find(|s| s.bullet.damage == damage).unwrap();
    let bonus = shots
        .iter()
        .find(|s| s.bullet.damage == damage * 1.5)
        .unwrap();
    let direction = forward(pilot_motion.heading);
    assert!(
        primary
            .motion
            .velocity
            .distance(direction * 430.0 + pilot_motion.velocity * 0.25)
            < 0.001
    );
    assert!(
        bonus
            .motion
            .velocity
            .distance(direction * 500.0 + pilot_motion.velocity * 0.25)
            < 0.001
    );
    assert!((bonus.motion.previous - primary.motion.previous).distance(direction * 2.0) < 0.001);
}

#[test]
fn full_cap_does_not_count_and_refused_fifth_bonus_never_replays() {
    let (mut game, admissions) = start(licensed("renko", 1), &[]);
    let a = admissions[0];
    let damage = game
        .world
        .get::<&Pilot>(entity(&game, a.id))
        .unwrap()
        .stats
        .damage;
    let mut seq = 0;
    for _ in 0..4 {
        verify(&next(&mut game, &admissions, &mut seq, a), a, damage, false);
        clear_bullets(&mut game);
    }
    for _ in 0..MAX_BULLETS {
        game.spawn_bullet(Vec2::new(10000.0, 5000.0), Vec2::ZERO, 1.0, false, 1);
    }
    for _ in 0..20 {
        assert!(step(&mut game, &admissions, &mut seq, [true; 8]).is_empty());
        assert_eq!(phase(&game, a.id), 4);
    }
    let filler = game
        .world
        .query::<(Entity, &Bullet)>()
        .iter()
        .next()
        .unwrap()
        .0;
    game.world.despawn(filler).unwrap();
    verify(&next(&mut game, &admissions, &mut seq, a), a, damage, false);
    assert_eq!(phase(&game, a.id), 0);
    assert_eq!(game.world.query::<&Bullet>().iter().count(), MAX_BULLETS);
    clear_bullets(&mut game);
    for number in 6..=10 {
        verify(
            &next(&mut game, &admissions, &mut seq, a),
            a,
            damage,
            number == 10,
        );
        clear_bullets(&mut game);
    }
}

#[test]
fn all_queued_primaries_take_priority_over_bonuses_near_shared_cap() {
    let (mut game, admissions) = start(licensed("renko", 1), &["renko"]);
    let mut seq = 0;
    let mut counts = [0; 2];
    while counts != [4, 4] {
        let shots = step(&mut game, &admissions, &mut seq, [true; 8]);
        for shot in shots {
            counts[if shot.owner.id == id(1) { 0 } else { 1 }] += 1;
        }
        clear_bullets(&mut game);
    }
    assert_eq!([phase(&game, id(1)), phase(&game, id(2))], [4, 4]);
    for _ in 0..MAX_BULLETS - 2 {
        game.spawn_bullet(Vec2::new(10000.0, 5000.0), Vec2::ZERO, 1.0, false, 1);
    }
    let mut shots = Vec::new();
    for _ in 0..60 {
        shots = step(&mut game, &admissions, &mut seq, [true; 8]);
        if !shots.is_empty() {
            break;
        }
    }
    assert_eq!(shots.len(), 2);
    for a in &admissions {
        let own: Vec<_> = shots
            .iter()
            .filter(|s| s.owner.id == a.id)
            .copied()
            .collect();
        let damage = game
            .world
            .get::<&Pilot>(entity(&game, a.id))
            .unwrap()
            .stats
            .damage;
        verify(&own, *a, damage, false);
        assert_eq!(phase(&game, a.id), 0);
    }
    assert_eq!(game.world.query::<&Bullet>().iter().count(), MAX_BULLETS);
}

#[test]
fn safe_pause_timeout_respawn_and_rejoin_keep_only_the_intended_member_phase() {
    let (mut game, admissions) = start(licensed("pan", 1), &["renko"]);
    let saved = serde_json::to_string(game.profile()).unwrap();
    let a = admissions[1];
    let mut seq = 0;
    for _ in 0..4 {
        assert_eq!(next(&mut game, &admissions, &mut seq, a).len(), 1);
        clear_bullets(&mut game);
    }
    let timer = game
        .world
        .get::<&Pilot>(entity(&game, a.id))
        .unwrap()
        .fire_timer;
    game.set_paused(true);
    for _ in 0..120 {
        assert_eq!(game.tick_co_op(session()), Err(Error::Paused));
    }
    assert_eq!(
        game.world
            .get::<&Pilot>(entity(&game, a.id))
            .unwrap()
            .fire_timer,
        timer
    );
    assert_eq!(phase(&game, a.id), 4);
    game.set_paused(false);
    place(&mut game, a.id, Vec2::ZERO);
    let mut guest_fire = [false; 8];
    guest_fire[a.slot as usize] = true;
    for _ in 0..30 {
        assert!(step(&mut game, &admissions, &mut seq, guest_fire).is_empty());
    }
    assert_eq!(phase(&game, a.id), 4);
    // Held input expires while safe. Moving outside cannot revive it.
    for _ in 0..20 {
        game.tick_co_op(session()).unwrap();
    }
    place(&mut game, a.id, Vec2::new(450.0, 80.0));
    game.tick_co_op(session()).unwrap();
    assert_eq!(phase(&game, a.id), 4);
    assert_eq!(game.world.query::<&Bullet>().iter().count(), 0);
    for _ in 0..3 {
        assert!(step(&mut game, &admissions, &mut seq, [false; 8]).is_empty());
    }
    game.world
        .get::<&mut Actor>(entity(&game, a.id))
        .unwrap()
        .health = 0.0;
    game.tick_co_op(session()).unwrap();
    assert_eq!(phase(&game, a.id), 4);
    place(&mut game, a.id, Vec2::new(450.0, 80.0));
    assert_eq!(next(&mut game, &admissions, &mut seq, a).len(), 2);
    game.leave_co_op(session(), a.id, a.epoch).unwrap();
    assert!(
        game.world
            .query::<&ShotOwner>()
            .iter()
            .all(|s| s.id != a.id)
    );
    let b = game
        .join_co_op(session(), spec(&licensed("renko", 2), 2))
        .unwrap();
    assert_ne!(b.epoch, a.epoch);
    assert_eq!(phase(&game, b.id), 0);
    assert_eq!(
        game.submit_guest_co_op_input(session(), a.id, a.epoch, seq + 1, InputFrame::default()),
        Err(Error::StaleMembership)
    );
    place(&mut game, b.id, Vec2::new(450.0, 80.0));
    assert_eq!(next(&mut game, &[admissions[0], b], &mut seq, b).len(), 1);
    game.end_co_op(session()).unwrap();
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), saved);
    assert_eq!(game.shot_count, 17);
}

#[test]
fn fifth_bolt_hits_real_cells_is_not_pvp_and_never_grants_progression() {
    for health in [200.0, 10.0] {
        let (mut game, admissions) = start(licensed("pan", 1), &["renko"]);
        let a = admissions[1];
        place(&mut game, a.id, Vec2::new(450.0, 0.0));
        place(&mut game, id(1), Vec2::new(495.0, 0.0));
        let host_health = game.world.get::<&Actor>(game.player).unwrap().health;
        let host_cells = game
            .world
            .get::<&PixelBody>(game.player)
            .unwrap()
            .occupied_count();
        let saved = serde_json::to_string(game.profile()).unwrap();
        let mut seq = 0;
        for _ in 0..4 {
            assert_eq!(next(&mut game, &admissions, &mut seq, a).len(), 1);
            clear_bullets(&mut game);
        }
        let body = PixelBody::from_ascii(
            &["####################"; 20],
            &[('#', rgba(150, 170, 180, 255))],
        );
        let target = game.world.spawn((
            Motion::new(Vec2::new(540.0, 0.0), Vec2::ZERO, 0.0),
            Actor {
                kind: Kind::Asteroid,
                health,
                max_health: health,
                radius: 15.0,
                scale: 1.0,
                damage: 0.0,
            },
            body,
        ));
        let damage = game
            .world
            .get::<&Pilot>(entity(&game, a.id))
            .unwrap()
            .stats
            .damage;
        verify(&next(&mut game, &admissions, &mut seq, a), a, damage, true);
        for _ in 0..20 {
            step(&mut game, &admissions, &mut seq, [false; 8]);
        }
        assert_eq!(
            game.world.get::<&Actor>(game.player).unwrap().health,
            host_health
        );
        assert_eq!(
            game.world
                .get::<&PixelBody>(game.player)
                .unwrap()
                .occupied_count(),
            host_cells
        );
        if health > 10.0 {
            assert!(
                (game.world.get::<&Actor>(target).unwrap().health - (health - damage * 2.5)).abs()
                    < 0.001
            );
            assert!(
                game.world
                    .get::<&PixelBody>(target)
                    .unwrap()
                    .occupied_count()
                    < 400
            );
        } else {
            assert!(!game.world.contains(target));
            let notices = game.take_co_op_destructions(session()).unwrap();
            assert_eq!(notices.len(), 1);
            assert_eq!(notices[0].owner, Some(a.id));
            assert_eq!(notices[0].owner_epoch, Some(a.epoch));
        }
        assert_eq!(serde_json::to_string(game.profile()).unwrap(), saved);
        assert_eq!(game.world.query::<&Loot>().iter().count(), 0);
    }
}

#[test]
fn real_publication_and_replica_render_both_owned_fifth_shots_without_wire_fields() {
    let (mut game, admissions) = start(licensed("pan", 1), &["renko"]);
    let a = admissions[1];
    let mut seq = 0;
    for _ in 0..4 {
        next(&mut game, &admissions, &mut seq, a);
        clear_bullets(&mut game);
    }
    assert_eq!(next(&mut game, &admissions, &mut seq, a).len(), 2);
    let header: crate::co_op_replica::Header = serde_json::from_value(
        serde_json::to_value(game.acquire_co_op_publication(session()).unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(header.snapshot.projectiles.len(), 2);
    assert!(
        header
            .snapshot
            .projectiles
            .iter()
            .all(|s| s.owner == Some(a.id) && s.owner_epoch == Some(a.epoch))
    );
    let json = serde_json::to_value(&header).unwrap();
    assert!(
        json["snapshot"]["participants"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p.get("primary_phase").is_none()
                && p.get("pulse_lens_rank").is_none()
                && p.get("owned_loadout").is_none())
    );
    let capture = game
        .co_op_publication(session(), header.acquisition, header.snapshot.revision)
        .unwrap();
    let mut replica =
        crate::co_op_replica::Replica::new(session(), a.id, a.epoch, game.profile().seed).unwrap();
    replica.stage(header.clone()).unwrap();
    for offset in (0..capture.counts().entities).step_by(ENTITY_PAGE_SIZE) {
        replica
            .entities(
                header.acquisition,
                serde_json::from_value(
                    serde_json::to_value(capture.entity_page(offset).unwrap()).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
    }
    for row in capture.entities() {
        for offset in (0..row.occupied).step_by(CELL_PAGE_SIZE) {
            replica
                .body(
                    header.acquisition,
                    serde_json::from_value(
                        serde_json::to_value(capture.body_page(&row.id, offset).unwrap()).unwrap(),
                    )
                    .unwrap(),
                )
                .unwrap();
        }
    }
    replica
        .commit(header.acquisition, header.snapshot.revision)
        .unwrap();
    let scene = replica.scene().unwrap();
    let mut frame = FrameBuilder::new();
    scene.append_occupied(&mut frame, 1.0);
    assert_eq!(
        frame.frame().pixel_count,
        capture.counts().occupied_cells + 6
    );
    let pixels: Vec<_> = frame
        .frame()
        .batches
        .iter()
        .flat_map(|b| {
            b.pixels.iter().map(move |p| {
                (
                    b.transforms[p.transform_slot()].point(Vec2::from_array(p.local)),
                    p.color,
                )
            })
        })
        .collect();
    for shot in &header.snapshot.projectiles {
        assert!(pixels.iter().any(|(position, color)| *position
            == Vec2::from_array(shot.position)
            && *color == rgba(255, 237, 156, 255)));
    }
    println!(
        "actual_fifth_publication_bytes={} projectiles={} native_replica_cells={}",
        serde_json::to_vec(&header).unwrap().len(),
        header.snapshot.projectiles.len(),
        frame.frame().pixel_count
    );
}
