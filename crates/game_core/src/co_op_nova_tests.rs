//! Fixtures configure saved builds/quiet traffic; reactions use real Game damage,
//! collision and fixed ticks. Defeat history here is configured, not earned play.
use super::*;
use serde_json::{Value, json};

fn id(n: u32) -> ParticipantId {
    ParticipantId::parse(&format!("{n:032x}")).unwrap()
}
fn session() -> ParticipantId {
    id(100)
}
fn profile(evolution: u8, rank: u8, xp: u32) -> Profile {
    let mut p = identity::create_profile("talo", "hugo", "pan", "0123456789abcdef").unwrap();
    p.campaign.ship_xp[0] = xp;
    p.campaign.abilities[4] = campaign::AbilityState {
        blueprint: true,
        rank,
    };
    p.campaign.loadouts[0].slots[1] = Some(4);
    p.campaign.loadouts[0].hardware[1] = 5;
    p.boss_generations[1] = std::array::from_fn(|i| {
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
fn place(game: &mut Game, e: Entity, position: Vec2) {
    *game.world.get::<&mut Motion>(e).unwrap() = Motion::new(position, Vec2::ZERO, 0.0);
    if let Ok(mut p) = game.world.get::<&mut Pilot>(e) {
        p.was_home = position.length_squared() <= SAFE_RADIUS * SAFE_RADIUS;
    } else {
        game.was_safe = false;
        game.invulnerability = 0.0;
    }
    game.refresh_sites();
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
    for p in positions {
        let center = (p / CHUNK).floor().as_ivec2();
        for y in -1..=1 {
            for x in -1..=1 {
                game.chunks.entry((center.x + x, center.y + y)).or_default();
            }
        }
    }
    game.take_events();
}
fn start(profiles: &[Profile]) -> (Game, Vec<Admission>) {
    let mut g = Game::new(profiles[0].clone());
    let mut admissions = vec![g.begin_co_op(PROTOCOL, session(), id(1)).unwrap()];
    for (i, p) in profiles.iter().enumerate().skip(1) {
        let solo = Game::new(p.clone());
        admissions.push(
            g.join_co_op_owned(
                session(),
                spec(solo.profile(), i as u32 + 1),
                solo.co_op_owned_loadout().unwrap(),
            )
            .unwrap(),
        );
    }
    for a in &admissions {
        let e = entity(&g, a.id);
        place(&mut g, e, Vec2::new(450.0, a.slot as f32 * 150.0));
    }
    quiet(&mut g);
    (g, admissions)
}
fn target(game: &mut Game, p: Vec2, health: f32) -> Entity {
    game.world.spawn((
        Motion::new(p, Vec2::ZERO, 0.0),
        Actor {
            kind: Kind::Asteroid,
            health,
            max_health: health,
            radius: 15.0,
            scale: 1.0,
            damage: 1.0,
        },
        PixelBody::from_ascii(
            &["####################"; 20],
            &[('#', rgba(130, 115, 100, 255))],
        ),
    ))
}
fn close(a: f32, b: f32) {
    assert!((a - b).abs() < 0.0002, "{a} != {b}");
}
fn timer(g: &Game, a: Admission) -> f32 {
    g.world.get::<&Pilot>(entity(g, a.id)).unwrap().nova_timer
}
fn hostile_hit(g: &mut Game, e: Entity, damage: f32) {
    let local = g
        .world
        .get::<&PixelBody>(e)
        .unwrap()
        .cells()
        .min_by(|a, b| a.0.length_squared().total_cmp(&b.0.length_squared()))
        .unwrap()
        .0;
    let motion = *g.world.get::<&Motion>(e).unwrap();
    let actor = *g.world.get::<&Actor>(e).unwrap();
    let point = actor_transform(motion, actor).point(local);
    assert!(
        g.spawn_bullet_entity(point, Vec2::ZERO, damage, false, 1)
            .is_some()
    );
}
fn nova_events(g: &mut Game) -> usize {
    g.take_events()
        .iter()
        .filter(|e| e.kind == "ability-nova")
        .count()
}

#[test]
fn native_export_preserves_legacy_bytes_and_validates_effect_only_nova_metadata() {
    let legacy = Game::new(Profile::default()).co_op_owned_loadout().unwrap();
    let old = r#"{"version":1,"selected_ship":0,"owned_ships":1,"patches":0,"royal_victory":false,"ship_xp":0,"hardware":[0,0,0,0,0,0],"slots":[null,null,null],"trained_starter":false,"relic_count":0}"#;
    assert_eq!(serde_json::to_string(&legacy).unwrap(), old);
    assert_eq!(serde_json::from_str::<OwnedLoadout>(old).unwrap(), legacy);
    println!("actual_legacy_export={old}");
    for evolution in 1..=5 {
        let source = Game::new(profile(evolution, 5, 0));
        let saved = serde_json::to_string(source.profile()).unwrap();
        let build = source.co_op_owned_loadout().unwrap();
        assert_eq!(build.nova_evolution, (evolution > 1).then_some(evolution));
        assert_eq!(build.echo_evolution, None);
        let temporary = build.profile(&spec(source.profile(), 2)).unwrap();
        assert_eq!(temporary.boss_generations, [[0; 5]; 3]);
        assert_eq!(ship_progression::equipped_rank(&temporary, 4), 1);
        assert_eq!(serde_json::to_string(source.profile()).unwrap(), saved);
        let (mut host, _) = start(&[Profile::default()]);
        let a = host
            .join_co_op_owned(session(), spec(source.profile(), 2), build.clone())
            .unwrap();
        let p = host.world.get::<&Pilot>(entity(&host, a.id)).unwrap();
        assert_eq!((p.nova_rank, p.nova_evolution), (1, evolution));
        drop(p);
        assert_eq!(
            host.co_op_snapshot().unwrap().participants[1].owned_loadout,
            Some(build.clone())
        );
        for bad in [0, 6, 255] {
            let mut invalid = build.clone();
            invalid.nova_evolution = Some(bad);
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
        let mut invalid = build.clone();
        invalid.slots[1] = Some(OwnedAbility { index: 1, rank: 1 });
        invalid.nova_evolution = Some(evolution);
        assert!(matches!(
            invalid.profile(&spec(source.profile(), 2)),
            Err(Error::InvalidLoadout)
        ));
        if evolution == 5 {
            println!(
                "actual_evolved_nova_export={}",
                serde_json::to_string(&build).unwrap()
            );
        }
        let mut p = source.profile().clone();
        p.campaign.loadouts[0].slots[1] = None;
        assert_eq!(
            Game::new(p).co_op_owned_loadout().unwrap().nova_evolution,
            None
        );
    }
    let mut value: Value = serde_json::from_str(old).unwrap();
    for bad in [
        Value::Null,
        json!(-1),
        json!(1.5),
        json!("5"),
        json!(256),
        json!({"boss_generations":[1]}),
    ] {
        value["nova_evolution"] = bad;
        assert!(serde_json::from_value::<OwnedLoadout>(value.clone()).is_err());
    }
    value["nova_evolution"] = json!(1);
    let declared: OwnedLoadout = serde_json::from_value(value).unwrap();
    assert!(matches!(
        declared.profile(&spec(&Profile::default(), 2)),
        Err(Error::InvalidLoadout)
    ));
}

#[test]
fn solo_and_owned_guest_nova_match_real_actor_damage_holes_impulses_and_cooldowns() {
    for evolution in 1..=5 {
        for xp in [0, 1560] {
            let p = profile(evolution, 5, xp);
            let mut solo = Game::new(p.clone());
            let solo_player = solo.player;
            place(&mut solo, solo_player, Vec2::new(450.0, 150.0));
            quiet(&mut solo);
            let (mut host, admissions) = start(&[Profile::default(), p]);
            let a = admissions[1];
            let guest = entity(&host, a.id);
            let position = solo.position();
            let mut targets = Vec::new();
            for distance in [65.0, 85.0, 110.0, 125.0] {
                let p = position + Vec2::X * distance;
                targets.push((target(&mut solo, p, 100.0), target(&mut host, p, 100.0)));
            }
            let host_hull = host.world.get::<&Actor>(host.player).unwrap().health;
            solo.hurt_player(1.0, None);
            host.hurt_co_op_pilot(guest, 1.0, None);
            close(timer(&host, a), solo.repulsor_timer);
            close(host.world.get::<&Pilot>(guest).unwrap().shield, solo.shield);
            let reached = match evolution {
                1 => [true, false, false, false],
                2 | 3 => [true, true, false, false],
                _ => [true, true, true, false],
            };
            for ((se, ge), reached) in targets.into_iter().zip(reached) {
                assert_eq!(
                    host.world.get::<&Actor>(ge).unwrap().health,
                    solo.world.get::<&Actor>(se).unwrap().health
                );
                assert_eq!(
                    host.world.get::<&Actor>(ge).unwrap().health,
                    if reached { 88.0 } else { 100.0 }
                );
                let velocity = host.world.get::<&Motion>(ge).unwrap().velocity;
                close(velocity.x, if reached { 110.0 } else { 0.0 });
                close(velocity.y, 0.0);
                assert_eq!(
                    host.world.get::<&PixelBody>(ge).unwrap().occupied_count() < 400,
                    reached
                );
                assert_eq!(
                    host.world.get::<&Motion>(ge).unwrap().velocity,
                    solo.world.get::<&Motion>(se).unwrap().velocity
                );
                assert_eq!(
                    crate::co_op_publish::body_signature(
                        &host.world.get::<&PixelBody>(ge).unwrap()
                    ),
                    crate::co_op_publish::body_signature(
                        &solo.world.get::<&PixelBody>(se).unwrap()
                    )
                );
            }
            assert_eq!(
                host.world.get::<&Actor>(host.player).unwrap().health,
                host_hull
            );
            assert_eq!(nova_events(&mut host), 1);
            assert_eq!(nova_events(&mut solo), 1);
        }
    }
}

#[test]
fn real_ticks_keep_rank_and_evolution_cooldown_boundaries_per_pilot() {
    for (rank, xp, evolution, expected_timer) in [
        (1, 0, 1, 18.0),
        (5, 0, 4, 15.3),
        (5, 1560, 1, 10.0),
        (5, 1560, 4, 8.5),
    ] {
        let p = profile(evolution, rank, xp);
        let mut solo = Game::new(p.clone());
        let player = solo.player;
        place(&mut solo, player, Vec2::new(450.0, 0.0));
        quiet(&mut solo);
        let (mut host, a) = start(&[p]);
        let a = a[0];
        solo.hurt_player(1.0, None);
        let player = host.player;
        host.hurt_co_op_pilot(player, 1.0, None);
        close(timer(&host, a), expected_timer);
        nova_events(&mut solo);
        nova_events(&mut host);
        let mut ticks = 0;
        while solo.repulsor_timer > TICK_SECONDS {
            solo.tick(&InputFrame::default(), TICK_SECONDS);
            host.tick_co_op(session()).unwrap();
            close(timer(&host, a), solo.repulsor_timer);
            ticks += 1;
            assert!(ticks < 1200);
        }
        assert!(solo.repulsor_timer > 0.0);
        solo.invulnerability = 0.0;
        host.world
            .get::<&mut Pilot>(host.player)
            .unwrap()
            .invulnerability = 0.0;
        solo.hurt_player(1.0, None);
        host.hurt_co_op_pilot(host.player, 1.0, None);
        assert_eq!(nova_events(&mut solo), 0);
        assert_eq!(nova_events(&mut host), 0);
        solo.tick(&InputFrame::default(), TICK_SECONDS);
        host.tick_co_op(session()).unwrap();
        assert!(solo.repulsor_timer <= 0.0);
        solo.invulnerability = 0.0;
        host.world
            .get::<&mut Pilot>(host.player)
            .unwrap()
            .invulnerability = 0.0;
        solo.hurt_player(1.0, None);
        host.hurt_co_op_pilot(host.player, 1.0, None);
        close(timer(&host, a), solo.repulsor_timer);
        assert_eq!(nova_events(&mut solo), 1);
        assert_eq!(nova_events(&mut host), 1);
        println!(
            "cooldown_fixture stored_rank={rank} xp={xp} evolution={evolution} first_retrigger_tick={}",
            ticks + 1
        );
    }
}

#[test]
fn accepted_localized_hull_hit_keeps_solo_damage_mask_and_triggers_once() {
    let mut p = profile(2, 1, 0);
    p.campaign.loadouts[0].hardware[1] = 0;
    let mut solo = Game::new(p.clone());
    let player = solo.player;
    place(&mut solo, player, Vec2::new(450.0, 150.0));
    quiet(&mut solo);
    let (mut host, admissions) = start(&[Profile::default(), p]);
    let a = admissions[1];
    let guest = entity(&host, a.id);
    let local = solo
        .world
        .get::<&PixelBody>(player)
        .unwrap()
        .cells()
        .next()
        .unwrap()
        .0;
    let before = solo
        .world
        .get::<&PixelBody>(player)
        .unwrap()
        .occupied_count();
    solo.hurt_player(3.0, Some(local));
    host.hurt_co_op_pilot(guest, 3.0, Some(local));
    assert!(
        solo.world
            .get::<&PixelBody>(player)
            .unwrap()
            .occupied_count()
            < before
    );
    assert_eq!(
        crate::co_op_publish::body_signature(&host.world.get::<&PixelBody>(guest).unwrap()),
        crate::co_op_publish::body_signature(&solo.world.get::<&PixelBody>(player).unwrap())
    );
    close(
        host.world.get::<&Actor>(guest).unwrap().health,
        solo.world.get::<&Actor>(player).unwrap().health,
    );
    close(timer(&host, a), solo.repulsor_timer);
    host.hurt_co_op_pilot(guest, 3.0, Some(local));
    assert_eq!(nova_events(&mut host), 1);
    assert_eq!(nova_events(&mut solo), 1);
}

#[test]
fn actual_hostile_collision_targets_guest_and_attributes_nova_destruction_without_grants() {
    let p = profile(3, 1, 0);
    let (mut host, a) = start(&[Profile::default(), p]);
    let a = a[1];
    let guest = entity(&host, a.id);
    let position = host.world.get::<&Motion>(guest).unwrap().position;
    let rock = target(&mut host, position + Vec2::X * 55.0, 6.0);
    let saved = serde_json::to_string(host.profile()).unwrap();
    let host_hull = host.world.get::<&Actor>(host.player).unwrap().health;
    hostile_hit(&mut host, guest, 1.0);
    host.tick_co_op(session()).unwrap();
    assert!(!host.world.contains(rock));
    assert!(timer(&host, a) > 0.0);
    assert_eq!(nova_events(&mut host), 1);
    let notices = host.take_co_op_destructions(session()).unwrap();
    assert_eq!(notices.len(), 1);
    assert_eq!(
        (notices[0].owner, notices[0].owner_epoch),
        (Some(a.id), Some(a.epoch))
    );
    assert_eq!(
        host.world.get::<&Actor>(host.player).unwrap().health,
        host_hull
    );
    assert!(host.co_op.as_ref().unwrap().hit_owner.is_none());
    assert_eq!(host.world.query::<&Loot>().iter().count(), 0);
    assert_eq!(serde_json::to_string(host.profile()).unwrap(), saved);
}

#[test]
fn evolved_clear_and_reflection_match_solo_and_keep_shared_caps_and_membership_owner() {
    for evolution in [2, 3, 5] {
        let p = profile(evolution, 1, 0);
        let (mut host, a) = start(std::slice::from_ref(&p));
        let a = a[0];
        let mut solo = Game::new(p);
        let player = solo.player;
        place(&mut solo, player, Vec2::new(450.0, 0.0));
        quiet(&mut solo);
        for g in [&mut host, &mut solo] {
            for _ in 0..24 {
                g.spawn_bullet(
                    g.position() + Vec2::X * 60.0,
                    -Vec2::X * 150.0,
                    9.0,
                    false,
                    1,
                );
            }
            g.spawn_bullet(g.position() + Vec2::X * 60.0, Vec2::X * 170.0, 2.0, true, 1);
            g.spawn_bullet(
                g.position() + Vec2::X * 150.0,
                -Vec2::X * 150.0,
                9.0,
                false,
                1,
            );
        }
        host.hurt_co_op_pilot(host.player, 1.0, None);
        solo.hurt_player(1.0, None);
        let summarize = |g: &Game| {
            let mut rows: Vec<_> = g
                .world
                .query::<(&Motion, &Bullet)>()
                .iter()
                .map(|(m, b)| {
                    (
                        b.player,
                        b.remaining_hits,
                        b.damage as u32,
                        b.life.to_bits(),
                        m.velocity.x as i32,
                    )
                })
                .collect();
            rows.sort();
            rows
        };
        assert_eq!(summarize(&host), summarize(&solo));
        assert_eq!(
            host.world.query::<&Bullet>().iter().count(),
            match evolution {
                2 => 26,
                3 => 2,
                _ => 18,
            }
        );
        let owned: Vec<_> = host
            .world
            .query::<(&Bullet, &ShotOwner)>()
            .iter()
            .map(|(b, o)| (*b, *o))
            .collect();
        assert_eq!(owned.len(), if evolution == 5 { 16 } else { 0 });
        for (b, o) in owned {
            assert!(b.player);
            assert_eq!((o.id, o.epoch), (a.id, a.epoch));
            assert_eq!((b.remaining_hits, b.previous_hit, b.life), (2, None, 1.5));
        }
    }
    let (mut host, _) = start(&[profile(5, 1, 0)]);
    for _ in 0..MAX_BULLETS {
        host.spawn_bullet(
            host.position() + Vec2::X * 60.0,
            -Vec2::X * 150.0,
            1.0,
            false,
            1,
        );
    }
    for _ in 0..MAX_PARTICLES - 10 {
        host.particle(Vec2::ZERO, Vec2::ZERO, 1.0, rgba(20, 20, 20, 255), 1.0);
    }
    for _ in 0..32 {
        host.transient_light(Vec2::ZERO, 50.0, [0.1; 3], 0.1, 1.0);
    }
    host.hurt_co_op_pilot(host.player, 1.0, None);
    assert_eq!(host.world.query::<&Bullet>().iter().count(), 16);
    assert_eq!(host.particles.len(), MAX_PARTICLES);
    assert_eq!(host.transient_lights.len(), 32);
}

#[test]
fn reflected_real_projectile_damages_actor_and_never_other_pilots() {
    let (mut host, a) = start(&[profile(5, 1, 0), Profile::default()]);
    let a = a[0];
    let position = host.position();
    let other = entity(&host, id(2));
    place(&mut host, other, position + Vec2::X * 75.0);
    let other_hull = host.world.get::<&Actor>(other).unwrap().health;
    let rock = target(&mut host, position + Vec2::X * 110.0, 20.0);
    host.spawn_bullet(position + Vec2::X * 75.0, -Vec2::X * 430.0, 9.0, false, 1);
    host.hurt_co_op_pilot(host.player, 1.0, None);
    assert_eq!(host.world.get::<&Actor>(rock).unwrap().health, 8.0);
    let saved = serde_json::to_string(host.profile()).unwrap();
    for _ in 0..10 {
        host.tick_co_op(session()).unwrap();
        if !host.world.contains(rock) {
            break;
        }
    }
    assert!(!host.world.contains(rock));
    assert_eq!(host.world.get::<&Actor>(other).unwrap().health, other_hull);
    let notices = host.take_co_op_destructions(session()).unwrap();
    assert_eq!(
        (notices[0].owner, notices[0].owner_epoch),
        (Some(a.id), Some(a.epoch))
    );
    assert_eq!(serde_json::to_string(host.profile()).unwrap(), saved);
}

#[test]
fn safe_invulnerable_invalid_and_unequipped_damage_cannot_trigger_nova() {
    let (mut g, a) = start(&[profile(5, 1, 0)]);
    let a = a[0];
    for damage in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        g.hurt_co_op_pilot(g.player, damage, None);
    }
    assert_eq!(timer(&g, a), 0.0);
    assert_eq!(nova_events(&mut g), 0);
    g.world.get::<&mut Pilot>(g.player).unwrap().invulnerability = 1.0;
    g.hurt_co_op_pilot(g.player, 1.0, None);
    assert_eq!(timer(&g, a), 0.0);
    g.world.get::<&mut Pilot>(g.player).unwrap().invulnerability = 0.0;
    let player = g.player;
    place(&mut g, player, Vec2::ZERO);
    g.hurt_co_op_pilot(player, 1.0, None);
    assert_eq!(timer(&g, a), 0.0);
    let site = (-8..=8)
        .flat_map(|y| (-8..=8).map(move |x| format!("{x}:{y}")))
        .find_map(|key| regions::site_by_id(g.profile.seed, &key))
        .unwrap();
    place(&mut g, player, site.position.into());
    assert!(g.safe_zone_at(site.position.into()).is_some());
    g.hurt_co_op_pilot(player, 1.0, None);
    assert_eq!(timer(&g, a), 0.0);
    let mut p = profile(5, 5, 1560);
    p.campaign.loadouts[0].slots[1] = Some(1);
    let (mut g, a) = start(&[p]);
    g.hurt_co_op_pilot(g.player, 1.0, None);
    assert_eq!(timer(&g, a[0]), 0.0);
    assert_eq!(nova_events(&mut g), 0);
    // A nova outside Haven does not damage a protected target just inside it.
    let (mut g, _) = start(&[profile(5, 1, 0)]);
    let player = g.player;
    place(&mut g, player, Vec2::new(SAFE_RADIUS + 20.0, 0.0));
    let protected = target(&mut g, Vec2::new(SAFE_RADIUS - 10.0, 0.0), 100.0);
    g.hurt_co_op_pilot(player, 1.0, None);
    assert_eq!(g.world.get::<&Actor>(protected).unwrap().health, 100.0);
    assert_eq!(nova_events(&mut g), 1);
}

#[test]
fn champion_impact_and_royal_defense_stay_outside_ordinary_nova_slice() {
    for (ship, ability) in [(6, 11), (9, 15)] {
        let mut p = profile(5, 5, 1560);
        p.campaign.patches = [true; 8];
        p.campaign.blueprints = [true; 8];
        p.campaign.crafted = [true; 8];
        p.campaign.royale_results = [1; 8];
        p.campaign.victory_claimed = true;
        p.campaign.selected_ship = ship;
        p.campaign.ship_xp[ship as usize] = 1560;
        p.campaign.abilities[ability as usize] = campaign::AbilityState {
            blueprint: true,
            rank: 5,
        };
        p.campaign.loadouts[ship as usize].slots[1] = Some(ability);
        let solo = Game::new(p.clone());
        assert!(ship_progression::equipped_rank(solo.profile(), ability) > 0);
        assert_eq!(solo.co_op_owned_loadout().unwrap().nova_evolution, None);
        let (mut g, admissions) = start(&[Profile::default(), p]);
        let a = admissions[1];
        let guest = entity(&g, a.id);
        assert_eq!(g.world.get::<&Pilot>(guest).unwrap().nova_rank, 0);
        g.hurt_co_op_pilot(guest, 1.0, None);
        assert_eq!(timer(&g, a), 0.0);
        assert_eq!(nova_events(&mut g), 0);
    }
}

#[test]
fn eight_staggered_collision_triggers_keep_timers_and_destruction_owners_independent() {
    let profiles: Vec<_> = (0..8)
        .map(|i| profile(i % 5 + 1, 5, if i % 2 == 0 { 0 } else { 1560 }))
        .collect();
    let (mut g, admissions) = start(&profiles);
    let saved = serde_json::to_string(g.profile()).unwrap();
    for (index, a) in admissions.iter().enumerate() {
        let e = entity(&g, a.id);
        let position = g.world.get::<&Motion>(e).unwrap().position;
        let rock = target(&mut g, position + Vec2::X * 55.0, 6.0);
        hostile_hit(&mut g, e, 1.0);
        g.tick_co_op(session()).unwrap();
        assert!(!g.world.contains(rock));
        assert_eq!(nova_events(&mut g), 1);
        let notices = g.take_co_op_destructions(session()).unwrap();
        assert_eq!(notices.len(), 1);
        assert_eq!(
            (notices[0].owner, notices[0].owner_epoch),
            (Some(a.id), Some(a.epoch))
        );
        for later in &admissions[index + 1..] {
            assert!(timer(&g, *later) <= 0.0);
        }
        assert!(g.world.query::<&Bullet>().iter().count() <= MAX_BULLETS);
        assert!(g.particles.len() <= MAX_PARTICLES);
        assert!(g.transient_lights.len() <= 32);
    }
    assert_eq!(g.co_op_snapshot().unwrap().participants.len(), 8);
    assert_eq!(serde_json::to_string(g.profile()).unwrap(), saved);
}

#[test]
fn death_preserves_solo_cooldown_policy_but_leave_rejoin_resets_membership_and_shots() {
    let p = profile(5, 1, 0);
    let mut g = Game::new(Profile::default());
    g.repulsor_timer = 7.0;
    let original = serde_json::to_string(g.profile()).unwrap();
    g.begin_co_op(PROTOCOL, session(), id(1)).unwrap();
    let solo = Game::new(p);
    let build = solo.co_op_owned_loadout().unwrap();
    let a = g
        .join_co_op_owned(session(), spec(solo.profile(), 2), build.clone())
        .unwrap();
    let guest = entity(&g, a.id);
    place(&mut g, guest, Vec2::new(450.0, 0.0));
    quiet(&mut g);
    g.spawn_bullet(Vec2::new(510.0, 0.0), -Vec2::X * 150.0, 1.0, false, 1);
    g.hurt_co_op_pilot(guest, 1.0, None);
    let before = timer(&g, a);
    assert_eq!(g.world.query::<&ShotOwner>().iter().count(), 1);
    g.world.get::<&mut Actor>(guest).unwrap().health = 0.0;
    g.tick_co_op(session()).unwrap();
    close(timer(&g, a), before - TICK_SECONDS);
    assert_eq!(g.world.query::<&ShotOwner>().iter().count(), 0);
    assert!(g.world.get::<&Motion>(guest).unwrap().position.length() < SAFE_RADIUS);
    g.leave_co_op(session(), a.id, a.epoch).unwrap();
    let fresh = g
        .join_co_op_owned(session(), spec(solo.profile(), 2), build)
        .unwrap();
    assert!(fresh.epoch > a.epoch);
    assert_eq!(timer(&g, fresh), 0.0);
    assert_eq!(
        g.submit_guest_co_op_input(session(), a.id, a.epoch, 99, InputFrame::default()),
        Err(Error::StaleMembership)
    );
    let guest = entity(&g, fresh.id);
    place(&mut g, guest, Vec2::new(450.0, 0.0));
    g.spawn_bullet(Vec2::new(510.0, 0.0), -Vec2::X * 150.0, 1.0, false, 1);
    g.hurt_co_op_pilot(guest, 1.0, None);
    assert!(timer(&g, fresh) > 0.0);
    g.leave_co_op(session(), fresh.id, fresh.epoch).unwrap();
    assert_eq!(g.world.query::<&ShotOwner>().iter().count(), 0);
    g.end_co_op(session()).unwrap();
    assert_eq!(g.repulsor_timer, 7.0);
    assert_eq!(serde_json::to_string(g.profile()).unwrap(), original);
    assert!(g.world.query::<&Pilot>().iter().next().is_none());
}
