//! Real primary/collision ticks; fixtures only arrange builds and quiet traffic.
use super::*;
use serde_json::json;

fn id(n: u32) -> ParticipantId {
    ParticipantId::parse(&format!("{n:032x}")).unwrap()
}
fn session() -> ParticipantId {
    id(100)
}
fn profile(evolution: u8, rank: u8, xp: u32, main: &str) -> Profile {
    let mut p = identity::create_profile("talo", main, "pan", "0123456789abcdef").unwrap();
    p.campaign.ship_xp[0] = xp;
    p.campaign.abilities[3] = campaign::AbilityState {
        blueprint: true,
        rank,
    };
    p.campaign.loadouts[0].slots[0] = Some(3);
    p.boss_generations[0] = std::array::from_fn(|i| {
        if i < evolution as usize {
            1 << (evolution as usize - 1 - i)
        } else {
            0
        }
    });
    p
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
fn phase(game: &Game, a: Admission) -> u8 {
    game.world
        .get::<&Pilot>(entity(game, a.id))
        .unwrap()
        .echo_phase
}
fn place(game: &mut Game, e: Entity, position: Vec2) {
    *game.world.get::<&mut Motion>(e).unwrap() = Motion::new(position, Vec2::ZERO, 0.0);
    if let Ok(mut p) = game.world.get::<&mut Pilot>(e) {
        p.was_home = false;
    } else {
        game.was_safe = false;
    }
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
    game.take_events();
}
fn clear_bullets(game: &mut Game) {
    let shots: Vec<_> = game
        .world
        .query::<(Entity, &Bullet)>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for e in shots {
        game.world.despawn(e).unwrap();
    }
}
fn start(profiles: &[Profile]) -> (Game, Vec<Admission>) {
    let mut game = Game::new(profiles[0].clone());
    game.shot_count = 17;
    let mut admissions = vec![game.begin_co_op(PROTOCOL, session(), id(1)).unwrap()];
    for (i, p) in profiles.iter().enumerate().skip(1) {
        let solo = Game::new(p.clone());
        admissions.push(
            game.join_co_op_owned(
                session(),
                spec(solo.profile(), i as u32 + 1),
                solo.co_op_owned_loadout().unwrap(),
            )
            .unwrap(),
        );
    }
    for a in &admissions {
        let e = entity(&game, a.id);
        place(&mut game, e, Vec2::new(450.0, a.slot as f32 * 80.0));
    }
    quiet(&mut game);
    (game, admissions)
}
#[derive(Clone, Copy)]
struct Shot {
    motion: Motion,
    bullet: Bullet,
    owner: Option<ShotOwner>,
}
fn shot_entities(game: &Game) -> HashSet<Entity> {
    game.world
        .query::<(Entity, &Bullet)>()
        .iter()
        .map(|(e, _)| e)
        .collect()
}
fn fresh(game: &Game, before: &HashSet<Entity>) -> Vec<Shot> {
    game.world
        .query::<(Entity, (&Motion, &Bullet, Option<&ShotOwner>))>()
        .iter()
        .filter(|(e, _)| !before.contains(e))
        .map(|(_, (m, b, o))| Shot {
            motion: *m,
            bullet: *b,
            owner: o.copied(),
        })
        .collect()
}
fn step(
    game: &mut Game,
    admissions: &[Admission],
    seq: &mut u32,
    fire: [bool; 8],
) -> (Vec<Shot>, Vec<GameEvent>) {
    let before = shot_entities(game);
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
    (fresh(game, &before), game.take_events())
}
fn next(
    game: &mut Game,
    admissions: &[Admission],
    seq: &mut u32,
    a: Admission,
) -> (Vec<Shot>, Vec<GameEvent>) {
    let mut fire = [false; 8];
    fire[a.slot as usize] = true;
    for _ in 0..60 {
        let result = step(game, admissions, seq, fire);
        if result.1.iter().any(|e| e.kind == "shot") {
            return result;
        }
    }
    panic!("No actual primary in sixty ticks");
}
fn echo_count(evolution: u8) -> usize {
    if evolution >= 3 {
        7
    } else if evolution == 2 {
        5
    } else {
        3
    }
}
fn sorted(shots: &[Shot]) -> Vec<Shot> {
    let mut result = shots.to_vec();
    result.sort_by(|a, b| {
        a.bullet
            .remaining_hits
            .cmp(&b.bullet.remaining_hits)
            .then(a.motion.velocity.y.total_cmp(&b.motion.velocity.y))
    });
    result
}
fn same_shots(solo: &[Shot], shared: &[Shot], a: Admission) {
    assert_eq!(solo.len(), shared.len());
    for (x, y) in sorted(solo).into_iter().zip(sorted(shared)) {
        assert_eq!(
            (
                x.motion.position,
                x.motion.previous,
                x.motion.velocity,
                x.motion.heading
            ),
            (
                y.motion.position,
                y.motion.previous,
                y.motion.velocity,
                y.motion.heading
            )
        );
        assert_eq!(
            (x.bullet.damage, x.bullet.life, x.bullet.remaining_hits),
            (y.bullet.damage, y.bullet.life, y.bullet.remaining_hits)
        );
        let owner = y.owner.unwrap();
        assert_eq!((owner.id, owner.epoch), (a.id, a.epoch));
    }
}

#[test]
fn native_export_validates_evolution_without_defeat_history_or_legacy_byte_changes() {
    let legacy = Game::new(Profile::default()).co_op_owned_loadout().unwrap();
    let value = serde_json::to_value(&legacy).unwrap();
    assert!(value.get("echo_evolution").is_none());
    assert_eq!(value.as_object().unwrap().len(), 10);
    // This is the previous exact native declaration, not a JSON Profile.
    let old = r#"{"version":1,"selected_ship":0,"owned_ships":1,"patches":0,"royal_victory":false,"ship_xp":0,"hardware":[0,0,0,0,0,0],"slots":[null,null,null],"trained_starter":false,"relic_count":0}"#;
    assert_eq!(serde_json::to_string(&legacy).unwrap(), old);
    assert_eq!(serde_json::from_str::<OwnedLoadout>(old).unwrap(), legacy);
    println!(
        "actual_legacy_export={}",
        serde_json::to_string(&legacy).unwrap()
    );
    for evolution in 1..=5 {
        let source = Game::new(profile(evolution, 5, 0, "renko"));
        let saved = serde_json::to_string(source.profile()).unwrap();
        let build = source.co_op_owned_loadout().unwrap();
        assert_eq!(build.echo_evolution, (evolution > 1).then_some(evolution));
        if evolution == 5 {
            println!(
                "actual_evolved_echo_export={}",
                serde_json::to_string(&build).unwrap()
            );
        }
        let temporary = build.profile(&spec(source.profile(), 2)).unwrap();
        assert_eq!(temporary.boss_generations, [[0; 5]; 3]);
        assert_eq!(ship_progression::equipped_rank(&temporary, 3), 1);
        assert_eq!(serde_json::to_string(source.profile()).unwrap(), saved);
        let mut host = Game::new(Profile::default());
        host.begin_co_op(PROTOCOL, session(), id(1)).unwrap();
        let a = host
            .join_co_op_owned(session(), spec(source.profile(), 2), build.clone())
            .unwrap();
        let p = host.world.get::<&Pilot>(entity(&host, a.id)).unwrap();
        assert_eq!((p.echo_rank, p.echo_evolution), (1, evolution));
        drop(p);
        for bad in [0, 6, 255] {
            let mut invalid = build.clone();
            invalid.echo_evolution = Some(bad);
            assert!(matches!(
                invalid.profile(&spec(source.profile(), 3)),
                Err(Error::InvalidLoadout)
            ));
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
        for bad in [json!(-1), json!(1.5), json!("2"), json!(256)] {
            let mut v = serde_json::to_value(&build).unwrap();
            v["echo_evolution"] = bad;
            assert!(serde_json::from_value::<OwnedLoadout>(v).is_err());
        }
        let mut wrong = build.clone();
        wrong.slots[0] = None;
        wrong.echo_evolution = Some(1);
        assert!(matches!(
            wrong.profile(&spec(source.profile(), 3)),
            Err(Error::InvalidLoadout)
        ));
    }
    let mut other = profile(5, 5, 1560, "renko");
    other.campaign.loadouts[0].slots[0] = Some(0);
    assert!(
        Game::new(other)
            .co_op_owned_loadout()
            .unwrap()
            .echo_evolution
            .is_none()
    );
}

#[test]
fn actual_solo_and_guest_ticks_match_all_evolutions_and_effective_damage_ranks() {
    for evolution in 1..=5 {
        for (rank, xp) in [(1, 0), (5, 0), (5, 1560)] {
            let p = profile(evolution, rank, xp, "pan");
            let mut solo = Game::new(p.clone());
            let solo_e = solo.player;
            place(&mut solo, solo_e, Vec2::new(450.0, 80.0));
            quiet(&mut solo);
            let (mut game, admissions) = start(&[Profile::default(), p]);
            let a = admissions[1];
            let mut seq = 0;
            let mut number = 0;
            let mut fans = 0;
            let saved = serde_json::to_string(game.profile()).unwrap();
            for _ in 0..600 {
                let before = shot_entities(&solo);
                let input = InputFrame {
                    fire: true,
                    ..Default::default()
                };
                solo.tick(&input, TICK_SECONDS);
                let own = fresh(&solo, &before);
                let solo_events = solo.take_events();
                let (shared, events) = step(
                    &mut game,
                    &admissions,
                    &mut seq,
                    [false, true, false, false, false, false, false, false],
                );
                same_shots(&own, &shared, a);
                assert_eq!(
                    solo_events
                        .iter()
                        .filter(|e| e.kind == "ability-echo")
                        .count(),
                    events.iter().filter(|e| e.kind == "ability-echo").count()
                );
                if !own.is_empty() {
                    number += 1;
                    let period = if evolution == 5 { 6 } else { 8 };
                    let due = number % period == 0;
                    assert_eq!(own.len(), 1 + if due { echo_count(evolution) } else { 0 });
                    fans += usize::from(due);
                    assert_eq!(phase(&game, a), (number % period) as u8);
                    if due {
                        let stats = Stats::from_profile(solo.profile());
                        let actual_rank = ship_progression::equipped_rank(solo.profile(), 3);
                        for s in own.iter().filter(|s| s.bullet.remaining_hits > 1) {
                            assert_eq!(
                                s.bullet.damage,
                                stats.damage * (0.5 + 0.1 * actual_rank.saturating_sub(1) as f32)
                            );
                            assert_eq!(s.bullet.remaining_hits, if evolution >= 4 { 3 } else { 2 });
                        }
                    }
                }
                clear_bullets(&mut solo);
                clear_bullets(&mut game);
                if number == 18 {
                    break;
                }
            }
            assert_eq!(number, 18);
            assert_eq!(fans, if evolution == 5 { 3 } else { 2 });
            assert_eq!(serde_json::to_string(game.profile()).unwrap(), saved);
        }
    }
}

#[test]
fn eight_staggered_members_and_moving_fans_have_independent_native_owner_and_phase() {
    let profiles: Vec<_> = (0..8)
        .map(|i| {
            profile(
                (i % 5 + 1) as u8,
                5,
                1560,
                if i % 2 == 0 { "pan" } else { "renko" },
            )
        })
        .collect();
    let (mut game, admissions) = start(&profiles);
    let mut seq = 0;
    let mut primaries = [0usize; 8];
    for a in &admissions {
        game.world
            .get::<&mut Motion>(entity(&game, a.id))
            .unwrap()
            .velocity = Vec2::new(37.0, -29.0);
    }
    for tick in 0..900 {
        let fire = std::array::from_fn(|i| primaries[i] < 18 && tick % (i + 2) != 0);
        let (shots, _) = step(&mut game, &admissions, &mut seq, fire);
        for a in &admissions {
            let own: Vec<_> = shots
                .iter()
                .filter(|s| s.owner.unwrap().id == a.id)
                .collect();
            if own.is_empty() {
                continue;
            }
            let i = a.slot as usize;
            primaries[i] += 1;
            let p = game.world.get::<&Pilot>(entity(&game, a.id)).unwrap();
            let period = if p.echo_evolution == 5 { 6 } else { 8 };
            assert_eq!(p.echo_phase, (primaries[i] % period) as u8);
            assert_eq!(
                own.len(),
                1 + if p.echo_phase == 0 {
                    echo_count(p.echo_evolution)
                } else {
                    0
                }
            );
            let primary = own.iter().find(|s| s.bullet.remaining_hits == 1).unwrap();
            let m = *game.world.get::<&Motion>(entity(&game, a.id)).unwrap();
            assert_eq!(
                primary.motion.velocity,
                forward(m.heading) * 430.0 + m.velocity * 0.25
            );
            for s in own.iter().filter(|s| s.bullet.remaining_hits > 1) {
                assert!((s.motion.velocity.length() - 460.0).abs() < 0.0001);
                assert_eq!(s.motion.previous, primary.motion.previous);
                assert_eq!(s.owner.unwrap().epoch, a.epoch);
            }
        }
        clear_bullets(&mut game);
        if primaries == [18; 8] {
            break;
        }
    }
    assert_eq!(primaries, [18; 8]);
    println!(
        "actual_eight_member_primaries={primaries:?}; maximum_queued_fans=8; maximum_fan_attempts=56"
    );
}

#[test]
fn shared_cap_preserves_ordered_partial_fans_and_consumes_refused_trigger_fx() {
    for available in 1..=8 {
        let (mut game, admissions) = start(&[profile(3, 1, 0, "pan")]);
        let a = admissions[0];
        let mut seq = 0;
        for _ in 0..7 {
            next(&mut game, &admissions, &mut seq, a);
            clear_bullets(&mut game);
        }
        for _ in 0..MAX_BULLETS {
            game.spawn_bullet(Vec2::new(10000.0, 5000.0), Vec2::ZERO, 1.0, false, 1);
        }
        for _ in 0..20 {
            assert!(
                step(&mut game, &admissions, &mut seq, [true; 8])
                    .0
                    .is_empty()
            );
            assert_eq!(phase(&game, a), 7);
        }
        let filler: Vec<_> = game
            .world
            .query::<(Entity, &Bullet)>()
            .iter()
            .take(available)
            .map(|(e, _)| e)
            .collect();
        for e in filler {
            game.world.despawn(e).unwrap();
        }
        let (shots, events) = next(&mut game, &admissions, &mut seq, a);
        assert_eq!(shots.len(), available);
        assert_eq!(phase(&game, a), 0);
        assert_eq!(
            events.iter().filter(|e| e.kind == "ability-echo").count(),
            usize::from(available > 1)
        );
        assert!(
            game.transient_lights
                .iter()
                .any(|l| l.source.color == [0.67, 0.38, 1.0])
        );
        assert!(
            game.particles
                .iter()
                .any(|p| p.color == rgba(187, 159, 255, 255))
        );
        let expected = [-0.32f32, -0.16, 0.0, 0.16, 0.32, PI - 0.18, PI + 0.18];
        let fan: Vec<_> = shots
            .iter()
            .filter(|s| s.bullet.remaining_hits > 1)
            .collect();
        for angle in expected.iter().take(available - 1) {
            assert!(
                fan.iter()
                    .any(|s| s.motion.velocity == forward(*angle) * 460.0)
            );
        }
        clear_bullets(&mut game);
        assert_eq!(next(&mut game, &admissions, &mut seq, a).0.len(), 1);
        assert_eq!(phase(&game, a), 1);
    }
    let (mut game, admissions) = start(&[profile(1, 1, 0, "pan"), profile(1, 1, 0, "pan")]);
    let mut seq = 0;
    let mut count = 0;
    while count < 7 {
        let (shots, _) = step(&mut game, &admissions, &mut seq, [true; 8]);
        if !shots.is_empty() {
            assert_eq!(shots.len(), 2);
            count += 1;
        }
        clear_bullets(&mut game);
    }
    for _ in 0..MAX_BULLETS - 2 {
        game.spawn_bullet(Vec2::new(10000.0, 5000.0), Vec2::ZERO, 1.0, false, 1);
    }
    for _ in 0..60 {
        let (shots, events) = step(&mut game, &admissions, &mut seq, [true; 8]);
        if !shots.is_empty() {
            assert_eq!(shots.len(), 2);
            assert!(shots.iter().all(|s| s.bullet.remaining_hits == 1));
            assert!(!events.iter().any(|e| e.kind == "ability-echo"));
            break;
        }
    }
    assert!(admissions.iter().all(|a| phase(&game, *a) == 0));
}

#[test]
fn safe_pause_timeout_respawn_leave_and_session_end_bound_member_phase() {
    let p = profile(1, 1, 0, "pan");
    let (mut game, admissions) = start(&[p.clone(), p]);
    let a = admissions[1];
    let mut seq = 0;
    let saved = serde_json::to_string(game.profile()).unwrap();
    for _ in 0..7 {
        next(&mut game, &admissions, &mut seq, a);
        clear_bullets(&mut game);
    }
    let e = entity(&game, a.id);
    let timer = game.world.get::<&Pilot>(e).unwrap().fire_timer;
    game.set_paused(true);
    for _ in 0..120 {
        assert_eq!(game.tick_co_op(session()), Err(Error::Paused));
    }
    assert_eq!(game.world.get::<&Pilot>(e).unwrap().fire_timer, timer);
    assert_eq!(phase(&game, a), 7);
    game.set_paused(false);
    place(&mut game, e, Vec2::ZERO);
    for _ in 0..30 {
        assert!(
            step(&mut game, &admissions, &mut seq, [true; 8])
                .0
                .iter()
                .all(|s| s.owner.unwrap().id != a.id)
        );
        clear_bullets(&mut game);
    }
    for _ in 0..20 {
        game.tick_co_op(session()).unwrap();
    }
    place(&mut game, e, Vec2::new(450.0, 80.0));
    game.tick_co_op(session()).unwrap();
    assert_eq!(phase(&game, a), 7);
    game.world.get::<&mut Actor>(e).unwrap().health = 0.0;
    game.tick_co_op(session()).unwrap();
    assert_eq!(phase(&game, a), 7);
    place(&mut game, e, Vec2::new(450.0, 80.0));
    assert_eq!(next(&mut game, &admissions, &mut seq, a).0.len(), 4);
    game.leave_co_op(session(), a.id, a.epoch).unwrap();
    assert!(
        game.world
            .query::<&ShotOwner>()
            .iter()
            .all(|o| o.id != a.id)
    );
    let source = Game::new(profile(1, 1, 0, "pan"));
    let b = game
        .join_co_op_owned(
            session(),
            spec(source.profile(), 2),
            source.co_op_owned_loadout().unwrap(),
        )
        .unwrap();
    assert_ne!(b.epoch, a.epoch);
    assert_eq!(phase(&game, b), 0);
    assert_eq!(
        game.submit_guest_co_op_input(session(), a.id, a.epoch, seq + 1, InputFrame::default()),
        Err(Error::StaleMembership)
    );
    let e = entity(&game, b.id);
    place(&mut game, e, Vec2::new(450.0, 80.0));
    assert_eq!(next(&mut game, &[admissions[0], b], &mut seq, b).0.len(), 1);
    game.end_co_op(session()).unwrap();
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), saved);
    assert_eq!(game.shot_count, 17);
}

#[test]
fn ordinary_slot_only_excludes_lens_empty_and_royal_echo_bundles() {
    for slot in [None, Some(0), Some(14)] {
        let mut p = profile(5, 5, 1560, "renko");
        p.campaign.loadouts[0].slots[0] = slot;
        if slot == Some(14) {
            p.campaign.selected_ship = 9;
            p.campaign.patches = [true; 8];
            p.campaign.blueprints = [true; 8];
            p.campaign.crafted = [true; 8];
            p.campaign.victory_claimed = true;
            p.campaign.ship_xp[9] = 1560;
            p.campaign.abilities[14] = campaign::AbilityState {
                blueprint: true,
                rank: 5,
            };
            p.campaign.loadouts[9].slots[0] = slot;
        }
        let (mut game, admissions) = start(&[p]);
        let a = admissions[0];
        let mut seq = 0;
        assert_eq!(
            game.world
                .get::<&Pilot>(entity(&game, a.id))
                .unwrap()
                .echo_rank,
            0
        );
        for n in 1..=16 {
            let (shots, events) = next(&mut game, &admissions, &mut seq, a);
            assert!(shots.iter().all(|s| s.bullet.remaining_hits == 1));
            assert!(!events.iter().any(|e| e.kind == "ability-echo"));
            assert_eq!(shots.len(), 1 + usize::from(slot == Some(0) && n % 5 == 0));
            clear_bullets(&mut game);
        }
    }
}

fn target(game: &mut Game, position: Vec2, kind: Kind, health: f32, color: u32) -> Entity {
    game.world.spawn((
        Motion::new(position, Vec2::ZERO, 0.0),
        Actor {
            kind,
            health,
            max_health: health,
            radius: 15.0,
            scale: 1.0,
            damage: 0.0,
        },
        PixelBody::from_ascii(&["####################"; 20], &[('#', color)]),
    ))
}
#[test]
fn actual_echo_pierces_cells_and_segments_with_debris_attribution_without_pvp_or_grants() {
    for (evolution, health) in [(1, 200.0), (1, 1.0), (4, 200.0), (4, 1.0)] {
        let p = profile(evolution, 1, 0, "pan");
        let (mut game, admissions) = start(&[Profile::default(), p]);
        let a = admissions[1];
        let mut seq = 0;
        let e = entity(&game, a.id);
        place(&mut game, e, Vec2::new(450.0, 0.0));
        let host = game.player;
        place(&mut game, host, Vec2::new(495.0, 0.0));
        let host_health = game.world.get::<&Actor>(host).unwrap().health;
        let host_cells = game.world.get::<&PixelBody>(host).unwrap().occupied_count();
        let saved = serde_json::to_string(game.profile()).unwrap();
        for _ in 0..7 {
            next(&mut game, &admissions, &mut seq, a);
            clear_bullets(&mut game);
        }
        let (shots, _) = next(&mut game, &admissions, &mut seq, a);
        assert_eq!(shots.len(), 1 + echo_count(evolution));
        let damage = shots
            .iter()
            .find(|s| s.bullet.remaining_hits > 1)
            .unwrap()
            .bullet
            .damage;
        // Keep an actual emitted center Echo, not a hand-made projectile. This
        // isolates piercing from primary/off-axis holes in the collision oracle.
        let remove: Vec<_> = game
            .world
            .query::<(Entity, (&Motion, &Bullet))>()
            .iter()
            .filter(|(_, (m, b))| b.remaining_hits == 1 || m.velocity.y != 0.0)
            .map(|(e, _)| e)
            .collect();
        for e in remove {
            game.world.despawn(e).unwrap();
        }
        assert_eq!(game.world.query::<&Bullet>().iter().count(), 1);
        let rock = target(
            &mut game,
            Vec2::new(540.0, 0.0),
            Kind::Asteroid,
            health,
            rgba(150, 170, 180, 255),
        );
        let enemy = target(
            &mut game,
            Vec2::new(580.0, 0.0),
            Kind::Enemy(0),
            health,
            rgba(200, 80, 70, 255),
        );
        let mut chain = None;
        if evolution == 4 {
            let head = target(
                &mut game,
                Vec2::new(660.0, 0.0),
                Kind::Enemy(1),
                200.0,
                rgba(70, 180, 90, 255),
            );
            let body = PixelBody::from_ascii(
                &["####################"; 20],
                &[('#', rgba(80, 190, 100, 255))],
            );
            game.spawn_chain(head, body, 1.0, 1, 200.0);
            let segment = game
                .world
                .query::<(Entity, &Segment)>()
                .iter()
                .next()
                .unwrap()
                .0;
            chain = Some((head, segment));
        }
        let mut debris = false;
        for _ in 0..30 {
            step(&mut game, &admissions, &mut seq, [false; 8]);
            debris |= game
                .particles
                .iter()
                .any(|p| p.color == rgba(150, 170, 180, 255) || p.color == rgba(200, 80, 70, 255));
        }
        assert!(
            debris,
            "Actual removed body colors must create bounded debris"
        );
        assert_eq!(game.world.get::<&Actor>(host).unwrap().health, host_health);
        assert_eq!(
            game.world.get::<&PixelBody>(host).unwrap().occupied_count(),
            host_cells
        );
        if health > 1.0 {
            for t in [rock, enemy] {
                assert!(
                    (game.world.get::<&Actor>(t).unwrap().health - (health - damage)).abs()
                        < 0.0001
                );
                assert!(game.world.get::<&PixelBody>(t).unwrap().occupied_count() < 400);
                assert!(game.world.get::<&Motion>(t).unwrap().velocity.x > 0.0);
            }
            assert!(game.take_co_op_destructions(session()).unwrap().is_empty());
        } else {
            assert!(!game.world.contains(rock) && !game.world.contains(enemy));
            let notices = game.take_co_op_destructions(session()).unwrap();
            assert_eq!(notices.len(), 2);
            assert!(
                notices
                    .iter()
                    .all(|n| n.owner == Some(a.id) && n.owner_epoch == Some(a.epoch))
            );
        }
        if let Some((head, segment)) = chain {
            assert!(
                (game.world.get::<&Segment>(segment).unwrap().health - (200.0 - damage)).abs()
                    < 0.0001
            );
            assert!(
                game.world
                    .get::<&PixelBody>(segment)
                    .unwrap()
                    .occupied_count()
                    < 400
            );
            assert!(
                (game.world.get::<&Actor>(head).unwrap().health - (200.0 - damage * 0.6)).abs()
                    < 0.0001
            );
        }
        assert_eq!(game.world.query::<&Bullet>().iter().count(), 0);
        assert_eq!(game.world.query::<&Loot>().iter().count(), 0);
        assert_eq!(serde_json::to_string(game.profile()).unwrap(), saved);
    }
}

#[test]
fn real_publication_and_native_replica_preserve_evolved_owned_fan_and_holes() {
    let (mut game, admissions) = start(&[Profile::default(), profile(5, 5, 0, "renko")]);
    let a = admissions[1];
    let mut seq = 0;
    for _ in 0..5 {
        next(&mut game, &admissions, &mut seq, a);
        clear_bullets(&mut game);
    }
    assert_eq!(next(&mut game, &admissions, &mut seq, a).0.len(), 8);
    let rock = target(
        &mut game,
        Vec2::new(1000.0, 500.0),
        Kind::Asteroid,
        200.0,
        rgba(150, 170, 180, 255),
    );
    // Retain a real damaged occupied body in the same native publication.
    game.world
        .get::<&mut PixelBody>(rock)
        .unwrap()
        .damage(Vec2::ZERO, 3.0);
    let header: crate::co_op_replica::Header = serde_json::from_value(
        serde_json::to_value(game.acquire_co_op_publication(session()).unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(header.snapshot.projectiles.len(), 8);
    assert!(
        header
            .snapshot
            .projectiles
            .iter()
            .all(|s| s.owner == Some(a.id) && s.owner_epoch == Some(a.epoch))
    );
    assert_eq!(
        header
            .snapshot
            .projectiles
            .iter()
            .filter(|s| s.remaining_hits == 3)
            .count(),
        7
    );
    let row = header
        .snapshot
        .participants
        .iter()
        .find(|p| p.id == a.id)
        .unwrap();
    assert_eq!(row.owned_loadout.as_ref().unwrap().echo_evolution, Some(5));
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
        capture.counts().occupied_cells + 8 * 3
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
            && *color
                == if shot.remaining_hits > 1 {
                    rgba(191, 167, 255, 255)
                } else {
                    rgba(255, 237, 156, 255)
                }));
    }
    let body = game.world.get::<&PixelBody>(rock).unwrap();
    assert!(body.occupied_count() < 400);
    let rock_row = capture
        .entities()
        .iter()
        .find(|row| row.id == entity_key(rock))
        .unwrap();
    assert_eq!(rock_row.occupied, body.occupied_count());
    println!(
        "actual_evolved_publication_bytes={} projectiles={} native_replica_cells={} damaged_rock_cells={}",
        serde_json::to_vec(&header).unwrap().len(),
        header.snapshot.projectiles.len(),
        frame.frame().pixel_count,
        body.occupied_count()
    );
}

#[test]
fn real_lens_bonus_keeps_priority_before_echo_at_shared_short_capacity() {
    let mut lens = profile(1, 1, 0, "renko");
    lens.campaign.loadouts[0].slots[0] = Some(0);
    let (mut game, admissions) = start(&[lens, profile(1, 1, 0, "pan")]);
    let mut seq = 0;
    for _ in 0..4 {
        next(&mut game, &admissions, &mut seq, admissions[0]);
        clear_bullets(&mut game);
    }
    for _ in 0..7 {
        next(&mut game, &admissions, &mut seq, admissions[1]);
        clear_bullets(&mut game);
    }
    for _ in 0..60 {
        step(&mut game, &admissions, &mut seq, [false; 8]);
    }
    for _ in 0..MAX_BULLETS - 4 {
        game.spawn_bullet(Vec2::new(10000.0, 5000.0), Vec2::ZERO, 1.0, false, 1);
    }
    let (shots, events) = step(&mut game, &admissions, &mut seq, [true; 8]);
    assert_eq!(shots.len(), 4);
    let lens: Vec<_> = shots
        .iter()
        .filter(|s| s.owner.unwrap().id == admissions[0].id)
        .collect();
    assert_eq!(lens.len(), 2);
    assert!(lens.iter().any(|s| s.motion.velocity.x == 500.0));
    let echo: Vec<_> = shots
        .iter()
        .filter(|s| s.owner.unwrap().id == admissions[1].id)
        .collect();
    assert_eq!(echo.len(), 2);
    let fan = echo.iter().find(|s| s.bullet.remaining_hits == 2).unwrap();
    assert_eq!(fan.motion.velocity, forward(-0.16) * 460.0);
    assert_eq!(
        events.iter().filter(|e| e.kind == "ability-echo").count(),
        1
    );
    assert_eq!(game.world.query::<&Bullet>().iter().count(), MAX_BULLETS);
    clear_bullets(&mut game);
    assert_eq!(
        next(&mut game, &admissions, &mut seq, admissions[1])
            .0
            .len(),
        1
    );
}
