use super::*;
use crate::{
    co_op::{Admission, PROTOCOL, ParticipantId, Pilot, ShotOwner},
    *,
};
use serde_json::{Value, json};

fn id(n: u32) -> ParticipantId {
    ParticipantId::parse(&format!("{n:032x}")).unwrap()
}
fn source(ship: u8, corporation: &str) -> Profile {
    let mut p = identity::create_profile("talo", corporation, "renko", "0123456789abcdef").unwrap();
    p.campaign.patches = [true; 8];
    p.campaign.blueprints = [true; 8];
    p.campaign.crafted = [true; 8];
    p.campaign.royale_results = [1; 8];
    p.campaign.victory_claimed = true;
    p.campaign.selected_ship = ship;
    p.campaign.ship_xp[ship as usize] = 1560;
    p.campaign.loadouts[ship as usize].hardware = [5; 6];
    for index in 0..3 {
        p.campaign.abilities[index] = campaign::AbilityState {
            blueprint: true,
            rank: 5,
        };
        p.campaign.loadouts[ship as usize].slots[index] = Some(index as u8);
    }
    p.relics = (2..7)
        .filter_map(|x| regions::site_by_id(p.seed, &format!("{x}:2")).map(|s| s.id))
        .collect();
    p.currency = [923, 412, 617];
    p.campaign.stored_materials = [32, 13, 9, 4, 2];
    p
}
fn spec(game: &Game, peer: u32) -> ParticipantSpec {
    ParticipantSpec {
        id: id(peer),
        local_seed: game.profile().seed,
        identity: game.profile().identity.clone(),
    }
}
fn start() -> Game {
    let mut game = Game::new(Profile::default());
    game.begin_co_op(PROTOCOL, id(100), id(1)).unwrap();
    game.spawn_timer = 10_000.0;
    game
}
fn pilot_entity(game: &Game, peer: ParticipantId) -> Entity {
    game.world
        .query::<(Entity, &Pilot)>()
        .iter()
        .find(|(_, p)| p.admission.id == peer)
        .map(|(e, _)| e)
        .unwrap()
}
fn encoded(value: &impl Serialize) -> String {
    serde_json::to_string(value).unwrap()
}
fn same_stats(a: Stats, b: Stats) {
    assert_eq!(
        [
            a.hull,
            a.shield,
            a.shield_regeneration,
            a.speed,
            a.acceleration,
            a.damage,
            a.fire_rate,
            a.regeneration
        ],
        [
            b.hull,
            b.shield,
            b.shield_regeneration,
            b.speed,
            b.acceleration,
            b.damage,
            b.fire_rate,
            b.regeneration
        ]
    );
}

#[test]
fn actual_native_export_is_read_only_bounded_and_keeps_all_selected_hull_stats() {
    let legacy = Game::new(Profile::default());
    let legacy_build = legacy.co_op_owned_loadout().unwrap();
    let legacy_spec = spec(&legacy, 2);
    assert!(legacy_spec.identity.is_none());
    let mut host = start();
    host.join_co_op_owned(id(100), legacy_spec, legacy_build)
        .unwrap();
    same_stats(
        host.world
            .get::<&Pilot>(pilot_entity(&host, id(2)))
            .unwrap()
            .stats,
        Stats::from_profile(legacy.profile()),
    );
    for ship in 0..=9 {
        let mut solo = Game::new(source(ship, "pan"));
        let before = encoded(solo.profile());
        let owned = solo.co_op_owned_loadout().unwrap();
        let temporary = owned.profile(&spec(&solo, 2)).unwrap();
        same_stats(
            stats(&temporary, Some(&owned)),
            Stats::from_profile(solo.profile()),
        );
        assert_eq!(owned.selected_ship, ship);
        assert_eq!(owned.ship_xp, 1560);
        assert_eq!(encoded(solo.profile()), before);
        let value = serde_json::to_value(&owned).unwrap();
        for forbidden in [
            "profile",
            "seed",
            "identity",
            "currency",
            "cargo",
            "components",
            "revision",
            "quests",
        ] {
            assert!(value.get(forbidden).is_none());
        }
        assert!(encoded(&owned).len() < 400);
        solo.begin_co_op(PROTOCOL, id(100), id(1)).unwrap();
        assert_eq!(solo.co_op_owned_loadout().unwrap(), owned);
        assert_eq!(encoded(solo.profile()), before);
        solo.end_co_op(id(100)).unwrap();
        assert_eq!(encoded(solo.profile()), before);
    }
}

#[test]
fn malformed_builds_reject_before_membership_or_publication_changes() {
    let solo = Game::new(source(6, "pan"));
    let build = solo.co_op_owned_loadout().unwrap();
    let mut game = start();
    let (acquisition, revision) = {
        let publication = game.acquire_co_op_publication(id(100)).unwrap();
        (publication.acquisition, publication.snapshot.revision)
    };
    let before = encoded(&game.co_op_snapshot().unwrap());
    let saved = encoded(game.profile());
    for case in 0..17 {
        let mut bad = build.clone();
        match case {
            0 => bad.version = 2,
            1 => bad.selected_ship = 10,
            2 => bad.owned_ships &= !(1 << 6),
            3 => bad.owned_ships &= !1,
            4 => bad.owned_ships |= 1 << 10,
            5 => bad.patches &= !(1 << 5),
            6 => bad.royal_victory = false,
            7 => bad.hardware[0] = 6,
            8 => bad.relic_count = 21,
            9 => bad.slots[0].as_mut().unwrap().rank = 0,
            10 => bad.slots[0].as_mut().unwrap().rank = 6,
            11 => bad.slots[0].as_mut().unwrap().index = 17,
            12 => bad.slots.swap(0, 1),
            13 => bad.slots[0] = Some(OwnedAbility { index: 6, rank: 1 }), // family-0 item on family-1 hull
            14 => bad.slots[1] = Some(OwnedAbility { index: 15, rank: 5 }), // royalty only on Sovereign
            15 => bad.trained_starter = true, // factory-only exception
            _ => {
                bad.owned_ships = 1 | (1 << 6);
                bad.patches = 1 << 5;
                bad.royal_victory = false;
                bad.slots[0] = Some(OwnedAbility { index: 12, rank: 1 });
            } // unearned compatible champion
        }
        assert_eq!(
            game.join_co_op_owned(id(100), spec(&solo, 2), bad),
            Err(Error::InvalidLoadout),
            "case {case}"
        );
        assert_eq!(encoded(&game.co_op_snapshot().unwrap()), before);
        assert!(
            game.co_op_publication(id(100), acquisition, revision)
                .is_ok()
        );
        assert_eq!(encoded(game.profile()), saved);
    }
    assert_eq!(
        game.join_co_op_owned(id(101), spec(&solo, 2), build.clone()),
        Err(Error::StaleSession)
    );
    let admission = game
        .join_co_op_owned(id(100), spec(&solo, 2), build.clone())
        .unwrap();
    assert_eq!(admission.epoch, 2);
    assert_eq!(
        game.join_co_op_owned(id(100), spec(&solo, 2), build),
        Err(Error::DuplicateId)
    );
}

#[test]
fn serde_rejects_raw_stats_profiles_wrong_array_lengths_and_unsigned_overflow() {
    let solo = Game::new(source(1, "renko"));
    let valid = serde_json::to_value(solo.co_op_owned_loadout().unwrap()).unwrap();
    for (field, value) in [
        ("profile", json!({"currency":([u32::MAX;3])})),
        ("damage", json!(10000)),
        ("ship_xp", json!(4294967296_u64)),
        ("ship_xp", json!(-1)),
        ("hardware", json!(([5; 7]))),
        ("slots", json!([null, null, null, null])),
        ("relic_count", json!(256)),
    ] {
        let mut bad = valid.clone();
        bad[field] = value;
        assert!(
            serde_json::from_value::<OwnedLoadout>(bad).is_err(),
            "{field}"
        );
    }
    let mut bad = valid;
    bad["slots"][0]["damage"] = json!(500);
    assert!(serde_json::from_value::<OwnedLoadout>(bad).is_err());
}

#[test]
fn stored_ranks_use_actual_selected_xp_and_scoped_factory_training_exception() {
    for corporation in ["pan", "renko", "hugo"] {
        let mut p = source(0, corporation);
        p.campaign.ship_xp[0] = 0;
        p.onboarding = Some(identity::OnboardingProgress {
            stage: 6,
            flags: 127,
            asteroids_destroyed: 10,
            materials_collected: 10,
            ..Default::default()
        });
        let solo = Game::new(p);
        let owned = solo.co_op_owned_loadout().unwrap();
        assert!(owned.trained_starter);
        let mut profile = owned.profile(&spec(&solo, 2)).unwrap();
        assert_eq!(ship_progression::hardware_ranks(&profile), [1; 6]);
        for index in 0..3 {
            assert_eq!(
                ship_progression::equipped_rank(&profile, index),
                if index == profile.identity.as_ref().unwrap().corporation.ability() {
                    2
                } else {
                    1
                }
            );
        }
        same_stats(
            stats(&profile, Some(&owned)),
            Stats::from_profile(solo.profile()),
        );
        for xp in [60, 200, 560, 1320, u32::MAX] {
            let mut higher = owned.clone();
            higher.ship_xp = xp;
            profile = higher.profile(&spec(&solo, 2)).unwrap();
            assert_eq!(
                ship_progression::hardware_ranks(&profile),
                [campaign::rank_cap(campaign::level(xp)); 6]
            );
        }
        let mut wrong = owned.clone();
        wrong.slots[profile.identity.as_ref().unwrap().corporation.ability() as usize] = None;
        assert!(wrong.profile(&spec(&solo, 2)).is_err());
        let anonymous = ParticipantSpec {
            identity: None,
            ..spec(&solo, 2)
        };
        assert!(owned.profile(&anonymous).is_err());
    }
    let mut royal = source(9, "pan");
    royal.campaign.ship_xp[9] = 0;
    royal.campaign.loadouts[9].slots = [Some(14), Some(15), Some(16)];
    let solo = Game::new(royal);
    let owned = solo.co_op_owned_loadout().unwrap();
    let p = owned.profile(&spec(&solo, 2)).unwrap();
    assert_eq!(ship_progression::hardware_ranks(&p), [5; 6]);
    assert_eq!(
        [14, 15, 16].map(|id| ship_progression::equipped_rank(&p, id)),
        [5; 3]
    );
    same_stats(stats(&p, Some(&owned)), Stats::from_profile(solo.profile()));
}

#[test]
fn two_and_eight_owned_pilots_use_real_bodies_movement_shot_damage_and_affinity() {
    for total in [2, 8] {
        let mut host = start();
        let saved = encoded(host.profile());
        let mut admitted = Vec::<(Admission, Stats)>::new();
        for n in 2..=total {
            let solo = Game::new(source((n - 1) as u8, "renko"));
            let owned = solo.co_op_owned_loadout().unwrap();
            let a = host
                .join_co_op_owned(id(100), spec(&solo, n), owned.clone())
                .unwrap();
            let e = pilot_entity(&host, id(n));
            let p = host.world.get::<&Pilot>(e).unwrap();
            let expected = Stats::from_profile(solo.profile());
            same_stats(p.stats, expected);
            assert_eq!(
                p.damage_multiplier,
                crate::shield_progression::incoming_damage_multiplier(solo.profile())
            );
            assert_eq!(p.owned_loadout.as_ref(), Some(&owned));
            let body = host.world.get::<&PixelBody>(e).unwrap();
            assert_eq!(
                body.cells_indexed()
                    .map(|(i, _, c)| (i, c))
                    .collect::<Vec<_>>(),
                identity::player_body(solo.profile())
                    .cells_indexed()
                    .map(|(i, _, c)| (i, c))
                    .collect::<Vec<_>>()
            );
            let actor = host.world.get::<&Actor>(e).unwrap();
            assert_eq!(actor.scale, campaign::player_scale((n - 1) as u8));
            assert_eq!(actor.radius, pixel_assets::radius(&body, actor.scale));
            drop((p, body, actor));
            *host.world.get::<&mut Motion>(e).unwrap() =
                Motion::new(Vec2::new(450.0 + n as f32 * 50.0, 0.0), Vec2::ZERO, 0.0);
            host.world.get::<&mut Pilot>(e).unwrap().was_home = false;
            admitted.push((a, expected));
        }
        for (a, _) in &admitted {
            host.submit_guest_co_op_input(
                id(100),
                a.id,
                a.epoch,
                1,
                InputFrame {
                    joystick: Some([1.0, 0.0]),
                    fire: true,
                    ..Default::default()
                },
            )
            .unwrap();
        }
        host.tick_co_op(id(100)).unwrap();
        for (a, expected) in &admitted {
            let e = pilot_entity(&host, a.id);
            assert!(host.world.get::<&Motion>(e).unwrap().velocity.x > 0.0);
            let damage = host
                .world
                .query::<(&Bullet, &ShotOwner)>()
                .iter()
                .find(|(_, owner)| owner.id == a.id)
                .unwrap()
                .0
                .damage;
            assert_eq!(damage, expected.damage);
        }
        let snapshot = host.co_op_snapshot().unwrap();
        assert_eq!(snapshot.participants.len(), total as usize);
        assert_eq!(
            snapshot
                .participants
                .iter()
                .filter(|p| p.owned_loadout.is_some())
                .count(),
            total as usize - 1
        );
        assert_eq!(encoded(host.profile()), saved);
        assert!(!snapshot.progression);
        host.end_co_op(id(100)).unwrap();
        assert_eq!(encoded(host.profile()), saved);
        assert_eq!(host.world.query::<&Pilot>().iter().count(), 0);
        assert_eq!(host.world.query::<&ShotOwner>().iter().count(), 0);
    }
}

#[test]
fn manufacturer_bonus_uses_selected_champion_not_pilot_or_sub_corporation() {
    for ship in 1..=8 {
        let builds: Vec<_> = ["pan", "renko", "hugo"]
            .map(|corp| Game::new(source(ship, corp)))
            .into_iter()
            .collect();
        let mut expected = None;
        for solo in builds {
            let owned = solo.co_op_owned_loadout().unwrap();
            let p = owned.profile(&spec(&solo, 2)).unwrap();
            let affinity = shield_progression::hud(&p).unwrap();
            assert_eq!(
                affinity.corporation,
                campaign::CHAMPIONS[ship as usize - 1].identity.corporation
            );
            assert!(affinity.active);
            let now = stats(&p, Some(&owned));
            if let Some(previous) = expected {
                same_stats(previous, now);
            } else {
                expected = Some(now);
            }
            let mut inactive = owned.clone();
            inactive.slots[affinity.slot as usize] = None;
            assert!(
                !shield_progression::hud(&inactive.profile(&spec(&solo, 2)).unwrap())
                    .unwrap()
                    .active
            );
        }
    }
}

#[test]
fn owned_publication_replica_hud_and_body_are_coherent_without_local_progression() {
    for ship in [0, 2, 9] {
        let solo = Game::new(source(ship, "hugo"));
        let guest_save = encoded(solo.profile());
        let owned = solo.co_op_owned_loadout().unwrap();
        let mut host = start();
        let host_save = encoded(host.profile());
        let a = host
            .join_co_op_owned(id(100), spec(&solo, 2), owned.clone())
            .unwrap();
        let header: crate::co_op_replica::Header = {
            let publication = host.acquire_co_op_publication(id(100)).unwrap();
            serde_json::from_value(serde_json::to_value(&publication).unwrap()).unwrap()
        };
        let acquisition = header.acquisition;
        let revision = header.snapshot.revision;
        let capture = host
            .co_op_publication(id(100), acquisition, revision)
            .unwrap();
        let mut replica =
            crate::co_op_replica::Replica::new(id(100), id(2), a.epoch, host.profile().seed)
                .unwrap();
        replica.stage(header.clone()).unwrap();
        for offset in (0..capture.counts().entities).step_by(64) {
            replica
                .entities(
                    acquisition,
                    serde_json::from_value(
                        serde_json::to_value(capture.entity_page(offset).unwrap()).unwrap(),
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        for row in capture.entities() {
            for offset in (0..row.occupied).step_by(1024) {
                replica
                    .body(
                        acquisition,
                        serde_json::from_value(
                            serde_json::to_value(capture.body_page(&row.id, offset).unwrap())
                                .unwrap(),
                        )
                        .unwrap(),
                    )
                    .unwrap();
            }
        }
        replica.commit(acquisition, revision).unwrap();
        let scene = replica.scene().unwrap();
        let hud = scene.guest_hud(id(2), Vec2::ZERO, host.hud()).unwrap();
        let original = solo.hud();
        assert_eq!(hud.campaign.selected_ship, ship);
        assert_eq!(
            [
                hud.max_hull,
                hud.max_shield,
                hud.ship_stats.top_speed,
                hud.ship_stats.primary_damage
            ],
            [
                original.max_hull,
                original.max_shield,
                original.ship_stats.top_speed,
                original.ship_stats.primary_damage
            ]
        );
        assert_eq!(hud.ship_xp, original.ship_xp);
        assert_eq!(hud.ship_level, original.ship_level);
        assert_eq!(
            hud.loadout
                .iter()
                .flatten()
                .map(|a| (&a.id, a.rank, a.effective_rank))
                .collect::<Vec<_>>(),
            original
                .loadout
                .iter()
                .flatten()
                .map(|a| (&a.id, a.rank, a.effective_rank))
                .collect::<Vec<_>>()
        );
        assert_eq!(hud.currency, [0; 3]);
        assert!(hud.quest_log.is_empty());
        assert!(hud.abilities.is_empty());
        assert!(hud.components.is_empty());
        assert!(hud.onboarding.is_none());
        assert_eq!(hud.campaign.ships.len(), 1);
        assert_eq!(hud.campaign.ships[0].id, ship);
        let body = &scene
            .entities()
            .iter()
            .find(|(e, _)| e.participant == Some(id(2)))
            .unwrap()
            .1;
        assert_eq!(
            crate::co_op_publish::body_signature(body),
            crate::co_op_publish::body_signature(&identity::player_body(solo.profile()))
        );
        let mut frame = FrameBuilder::new();
        scene.append_occupied(&mut frame, 1.0);
        assert_eq!(frame.frame().pixel_count, capture.counts().occupied_cells);
        assert_eq!(encoded(host.profile()), host_save);
        assert_eq!(encoded(solo.profile()), guest_save);
        let mut invalid = header.clone();
        invalid.acquisition += 1;
        invalid.snapshot.participants[1]
            .owned_loadout
            .as_mut()
            .unwrap()
            .hardware[0] = 6;
        assert!(replica.stage(invalid).is_err());
        assert_eq!(replica.scene().unwrap().header().acquisition, acquisition);
        let mut changed = header.clone();
        changed.acquisition += 1;
        // Same level and therefore same stats, but a membership's admitted
        // declaration is immutable until leave/rejoin with a fresh epoch.
        changed.snapshot.participants[1]
            .owned_loadout
            .as_mut()
            .unwrap()
            .ship_xp += 1;
        assert_eq!(replica.stage(changed), Err("invalid_pilot"));
        assert_eq!(replica.scene().unwrap().header().acquisition, acquisition);
        println!(
            "owned_hull={ship} loadout_bytes={} header_bytes={} entities={} cells={}",
            encoded(&owned).len(),
            encoded(&header).len(),
            capture.counts().entities,
            capture.counts().occupied_cells
        );
        if ship == 2 {
            println!("actual_owned_loadout={}", encoded(&owned));
        }
    }
}

#[test]
fn legacy_v1_metadata_stays_absent_and_retained_real_fixture_roundtrips() {
    let mut host = start();
    let solo = Game::new(source(8, "hugo"));
    host.join_co_op(id(100), spec(&solo, 2)).unwrap();
    let snapshot = serde_json::to_value(host.co_op_snapshot().unwrap()).unwrap();
    assert!(
        snapshot["participants"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p.get("owned_loadout").is_none())
    );
    assert_eq!(snapshot["participants"][1]["selected_ship"], 0);
    let original: Value = serde_json::from_str(include_str!(
        "../../../scripts/test/fixtures/co-op-publication/capture.json"
    ))
    .unwrap();
    let header: crate::co_op_replica::Header =
        serde_json::from_value(original["results"][0]["value"].clone()).unwrap();
    assert_eq!(
        serde_json::to_value(header).unwrap(),
        original["results"][0]["value"]
    );
}

#[test]
fn eight_owned_builds_and_full_projectile_cap_fit_existing_transport_frame_limit() {
    let mut host = start();
    for peer in 2..=8 {
        let mut p = source(9, "hugo");
        p.campaign.ship_xp[9] = u32::MAX;
        p.campaign.loadouts[9].slots = [Some(14), Some(15), Some(16)];
        let solo = Game::new(p);
        host.join_co_op_owned(
            id(100),
            spec(&solo, peer),
            solo.co_op_owned_loadout().unwrap(),
        )
        .unwrap();
    }
    for n in 0..MAX_BULLETS {
        let entity = host
            .spawn_bullet_entity(
                Vec2::new(999_999.94 - n as f32 * 0.37, -999_997.3),
                Vec2::new(-431.625, 328.75),
                37.3,
                true,
                64,
            )
            .unwrap();
        host.world
            .insert_one(
                entity,
                ShotOwner {
                    id: id(2),
                    epoch: 2,
                },
            )
            .unwrap();
    }
    assert!(
        host.spawn_bullet_entity(Vec2::ZERO, Vec2::ZERO, 1.0, true, 1)
            .is_none()
    );
    let capture = host.acquire_co_op_publication(id(100)).unwrap();
    assert_eq!(capture.snapshot.participants.len(), 8);
    assert_eq!(capture.snapshot.projectiles.len(), MAX_BULLETS);
    let frame =
        json!({"v":1,"session":id(100),"seq":u32::MAX,"kind":"publication","payload":capture});
    let bytes = encoded(&frame).len();
    assert!(bytes <= 65_536, "actual framed publication={bytes}");
    println!("actual_8_owned_180_projectiles_frame_bytes={bytes}");
}
