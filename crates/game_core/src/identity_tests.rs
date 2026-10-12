//! Declared native fixtures exercise the real Game lifecycle, cells and transactions.
use super::*;
const DT: f32 = 1.0 / 60.0;
fn fresh(corp: &str) -> Game {
    Game::new(create_profile("dora", corp, "renko", "0123456789abcdef").unwrap())
}
fn snapshot(game: &Game) -> serde_json::Value {
    serde_json::to_value(game.profile()).unwrap()
}
fn facts(game: &Game) -> u8 {
    game.profile.onboarding.as_ref().unwrap().flags
}
fn advance(game: &mut Game, input: &InputFrame, count: usize) {
    for _ in 0..count {
        game.tick(input, DT);
    }
}

#[test]
fn creation_validates_all_27_identities_and_grants_exactly_one_main_starter() {
    let mut callsigns = std::collections::HashSet::new();
    for (kind, prefix, gender) in [
        ("dora", 'A', "Female"),
        ("doro", 'O', "Male"),
        ("talo", 'T', "Neutral"),
    ] {
        for corp in ["pan", "renko", "hugo"] {
            for sub in ["pan", "renko", "hugo"] {
                let p = create_profile(kind, corp, sub, "0123456789ABCDEF").unwrap();
                let id = p.identity.as_ref().unwrap();
                assert_eq!(id.hex_id, "0123456789abcdef");
                assert_eq!(p.currency, [0; 3]);
                assert_eq!(p.campaign.ship_xp, [0; 10]);
                assert_eq!(p.campaign.cargo_materials, [0; 5]);
                assert_eq!(p.campaign.stored_materials, [0; 5]);
                assert_eq!(
                    p.campaign.abilities.iter().filter(|a| a.rank > 0).count(),
                    1
                );
                assert_eq!(
                    p.campaign.abilities.iter().filter(|a| a.blueprint).count(),
                    1
                );
                let def = abilities::get(id.corporation.ability()).unwrap();
                assert_eq!(
                    p.campaign.loadouts[0].slots[def.category],
                    Some(id.corporation.ability())
                );
                assert!(
                    p.campaign.loadouts[1..]
                        .iter()
                        .all(|l| *l == campaign::ShipLoadout::default())
                );
                let h = hud(&p).unwrap();
                assert_eq!(h.gender, gender);
                assert_eq!(
                    h.designation,
                    format!(
                        "{}-{}-{}-0123456789ABCDEF",
                        kind.to_uppercase(),
                        corp.to_uppercase(),
                        sub.to_uppercase()
                    )
                );
                assert_eq!(
                    h.short_designation,
                    format!(
                        "{}{}{}#0123456789ABCDEF",
                        prefix,
                        corp[..1].to_uppercase(),
                        sub[..1].to_uppercase()
                    )
                );
                assert!(callsigns.insert(h.short_designation));
                assert_eq!(
                    p.seed,
                    create_profile(kind, corp, sub, "0123456789abcdef")
                        .unwrap()
                        .seed
                );
            }
        }
    }
    assert_eq!(callsigns.len(), 27);
    for invalid in [
        "",
        "1234",
        "0123456789abcdeg",
        "0123456789abcdef0",
        "é123456789abcde",
    ] {
        assert!(create_profile("dora", "pan", "renko", invalid).is_err());
    }
    assert!(create_profile("unknown", "pan", "renko", "0123456789abcdef").is_err());
    assert!(create_profile("dora", "ranko", "renko", "0123456789abcdef").is_err());
    assert!(create_profile("dora", "pan", "other", "0123456789abcdef").is_err());
}

#[test]
fn derived_callsigns_preserve_all_saved_identity_and_progression_on_hud_and_reload() {
    for (kind, prefix) in [("dora", 'A'), ("doro", 'O'), ("talo", 'T')] {
        for corp in ["pan", "renko", "hugo"] {
            for sub in ["pan", "renko", "hugo"] {
                let mut p = create_profile(kind, corp, sub, "AbCdEf0123456789").unwrap();
                p.currency = [7, 8, 9];
                p.campaign.ship_xp[0] = 200;
                p.campaign.stored_materials = [3, 2, 1, 0, 0];
                let before = serde_json::to_value(&p).unwrap();
                let game = Game::new(p);
                assert_eq!(snapshot(&game), before);
                let h = game.hud().identity.unwrap();
                assert_eq!(
                    h.short_designation,
                    format!(
                        "{}{}{}#ABCDEF0123456789",
                        prefix,
                        corp[..1].to_uppercase(),
                        sub[..1].to_uppercase()
                    )
                );
                assert_eq!(
                    h.designation,
                    format!(
                        "{}-{}-{}-ABCDEF0123456789",
                        kind.to_uppercase(),
                        corp.to_uppercase(),
                        sub.to_uppercase()
                    )
                );
                assert_eq!(h.hex_id, "abcdef0123456789");
                let reloaded = Game::new(serde_json::from_value(snapshot(&game)).unwrap());
                assert_eq!(snapshot(&reloaded), before);
                assert_eq!(
                    reloaded.hud().identity.unwrap().short_designation,
                    h.short_designation
                );
                assert_eq!(snapshot(&game), before);
            }
        }
    }
}

#[test]
fn actual_starter_stats_and_secondary_manufacturer_have_no_hidden_grants() {
    let pan = fresh("pan");
    let renko = fresh("renko");
    let hugo = fresh("hugo");
    let a = Stats::from_profile(pan.profile());
    let b = Stats::from_profile(renko.profile());
    let c = Stats::from_profile(hugo.profile());
    assert!((a.speed - 114.4).abs() < 0.001);
    assert!((a.acceleration - 297.44).abs() < 0.001);
    assert!((b.damage - 14.52).abs() < 0.001);
    assert_eq!(b.speed, 100.0);
    assert_eq!(a.shield, 0.0);
    assert_eq!(b.shield, 0.0);
    assert_eq!(c.shield, 30.0);
    assert_eq!(c.damage, 12.0);
    for sub in ["pan", "renko", "hugo"] {
        let p = create_profile("doro", "pan", sub, "0123456789abcdef").unwrap();
        assert_eq!(p.campaign, pan.profile.campaign);
    }
}

#[test]
fn constructor_reload_and_acknowledgement_are_idempotent_and_do_not_grant_twice() {
    let mut g = fresh("pan");
    let before = snapshot(&g);
    let reload = Game::new(serde_json::from_value(before.clone()).unwrap());
    assert_eq!(snapshot(&reload), before);
    assert_eq!(g.profile.revision, 0);
    g.acknowledge_license().unwrap();
    let acknowledged = snapshot(&g);
    assert_eq!(facts(&g), LICENSE);
    assert_eq!(g.profile.revision, 1);
    g.acknowledge_license().unwrap();
    assert_eq!(snapshot(&g), acknowledged);
    assert_eq!(g.profile.currency, [0; 3]);
    assert_eq!(g.profile.campaign.ship_xp, [0; 10]);
    assert!(!g.can_register_droid());
}

#[test]
fn malformed_identity_and_legacy_version_do_not_lose_earned_assets() {
    let mut p = create_profile("dora", "pan", "hugo", "0123456789abcdef").unwrap();
    p.currency = [25, 30, 35];
    p.campaign.ship_xp[0] = 560;
    p.campaign.stored_materials = [3, 4, 5, 6, 7];
    p.identity.as_mut().unwrap().hex_id = "invalid".into();
    let g = Game::new(p.clone());
    assert!(g.profile.identity.is_none());
    assert!(g.profile.onboarding.is_none());
    assert_eq!(g.profile.currency, p.currency);
    assert_eq!(g.profile.campaign.ship_xp, p.campaign.ship_xp);
    assert_eq!(
        g.profile.campaign.stored_materials,
        p.campaign.stored_materials
    );
    assert_eq!(g.profile.campaign.abilities[2].rank, 1);
    p.schema_version = 4;
    p.identity.as_mut().unwrap().hex_id = "0123456789abcdef".into();
    let g = Game::new(p);
    assert!(g.profile.identity.is_none());
    assert!(g.profile.onboarding.is_none());
    let body = g.world.get::<&PixelBody>(g.player).unwrap();
    assert_eq!(
        body.cells().collect::<Vec<_>>(),
        campaign::player_body(0).cells().collect::<Vec<_>>()
    );
}

#[test]
fn legacy_v4_pilot_preserves_every_saved_asset_and_original_scout() {
    let mut p = Profile {
        schema_version: 4,
        currency: [10, 20, 30],
        ..Profile::default()
    };
    p.campaign.ship_xp[0] = 200;
    p.campaign.stored_materials = [2, 3, 4, 5, 6];
    p.campaign.loadouts[0].hardware = [1, 0, 1, 0, 0, 0];
    ship_progression::mirrors(&mut p);
    let mut expected = serde_json::to_value(&p).unwrap();
    expected["schema_version"] = components::SCHEMA_VERSION.into();
    expected["revision"] = 1.into();
    let g = Game::new(p);
    assert_eq!(snapshot(&g), expected);
    assert_eq!(g.hud().campaign.ships[0].name, "Haven Scout");
    assert_eq!(g.hud().abilities.len(), 3);
    assert!(g.hud().identity.is_none());
}

#[test]
fn legacy_stages_migrate_without_inventing_the_new_counted_mission() {
    for flags in 0..=255 {
        let mut p = create_profile("talo", "hugo", "pan", "0123456789abcdef").unwrap();
        p.onboarding = Some(OnboardingProgress {
            version: 1,
            stage: 255,
            flags,
            ..Default::default()
        });
        let g = Game::new(p);
        let progress = g.profile.onboarding.as_ref().unwrap();
        let expected = if flags & 127 == 127 { 127 } else { flags & 3 };
        assert_eq!(progress.flags, expected);
        assert_eq!(progress.stage, stage(expected));
        assert_eq!(
            (progress.asteroids_destroyed, progress.materials_collected),
            (0, 0)
        );
    }
    assert_eq!(stage(31 | ADDITIONAL_CRAFTED), 5);
    assert_eq!(stage(31 | ADDITIONAL_EQUIPPED), 5);
    assert_eq!(stage(127), 6);
}

#[test]
fn registration_guard_rejects_assets_visited_world_and_previous_expedition() {
    let p = Profile {
        revision: 20,
        ..Profile::default()
    };
    assert!(Game::new(p.clone()).can_register_droid());
    let mutations: [fn(&mut Profile); 11] = [
        |p| p.currency[1] = 1,
        |p| p.kills[1] = 1,
        |p| p.campaign.ship_xp[0] = 1,
        |p| p.campaign.stored_materials[0] = 1,
        |p| p.campaign.abilities[0].rank = 1,
        |p| p.campaign.loadouts[0].hardware[0] = 1,
        |p| p.campaign.patches[0] = true,
        |p| p.discoveries.push(campaign::arena(p.seed, 0).unwrap().id),
        |p| p.campaign.last_run = Some(campaign::RunSummary::default()),
        |p| p.pity[2] = 1,
        |p| p.campaign.cargo_materials[0] = 1,
    ];
    for mutation in mutations {
        let mut p = p.clone();
        mutation(&mut p);
        assert!(!Game::new(p).can_register_droid());
    }
    let mut g = Game::new(Profile::default());
    g.move_player(Vec2::new(400.0, 0.0));
    g.tick(&InputFrame::default(), DT);
    g.move_player(Vec2::ZERO);
    g.tick(&InputFrame::default(), DT);
    assert!(!g.can_register_droid());
}

#[test]
fn only_real_category_pickups_reveal_other_basic_blueprints() {
    let mut g = fresh("pan");
    assert_eq!(
        g.hud()
            .abilities
            .iter()
            .map(|a| a.index)
            .collect::<Vec<_>>(),
        vec![2]
    );
    g.credit_points(0, 10);
    assert_eq!(g.hud().abilities.len(), 1); // Quest reward is not a pickup.
    g.spawn_loot(g.position(), 0, 1);
    g.tick(&InputFrame::default(), DT);
    assert_eq!(
        g.hud()
            .abilities
            .iter()
            .map(|a| a.index)
            .collect::<Vec<_>>(),
        vec![0, 2]
    );
    assert_eq!(g.profile.campaign.abilities[0].rank, 0);
    assert_eq!(g.profile.currency[0], 11);
    let saved = snapshot(&g);
    assert_eq!(
        snapshot(&Game::new(serde_json::from_value(saved.clone()).unwrap())),
        saved
    );
    g.boss_qa_scene = true;
    g.spawn_loot(g.position(), 1, 1);
    g.tick(&InputFrame::default(), DT);
    assert!(!g.profile.campaign.abilities[1].blueprint);
}

#[test]
fn champion_catalog_masks_unknown_hulls_then_persists_real_approach() {
    let mut g = fresh("renko");
    let h = g.hud();
    assert_eq!(h.campaign.ships.len(), 1);
    assert!(h.campaign.champions.iter().all(|c| !c.discovered
        && c.ship.is_empty()
        && c.ability.is_empty()
        && c.name == "Unknown champion"));
    assert_eq!(h.campaign.champions.len(), 8);
    let site = campaign::arena(g.profile.seed, 1).unwrap();
    g.move_player(Vec2::from_array(site.position) + Vec2::X * 300.0);
    g.tick(&InputFrame::default(), DT);
    let h = g.hud();
    assert!(h.campaign.champions[1].discovered);
    assert_eq!(h.campaign.ships.len(), 2);
    let known = h.campaign.ships.iter().find(|s| s.id == 2).unwrap();
    assert!(!known.craftable && !known.owned);
    assert_eq!(known.name, campaign::CHAMPIONS[1].ship);
    assert!(g.profile.discoveries.contains(&site.id));
    let reload = Game::new(g.profile.clone());
    assert!(reload.hud().campaign.champions[1].discovered);
    assert_eq!(reload.hud().campaign.ships.len(), 2);
    let mut qa = fresh("renko");
    qa.boss_qa_scene = true;
    qa.move_player(Vec2::from_array(site.position));
    qa.tick(&InputFrame::default(), DT);
    assert!(!qa.hud().campaign.champions[1].discovered);
}

// Declared high-damage shot fixture uses live occupied-cell sweep and normal death
// dispatch. It does not call kill_actor, invent rewards, or claim human balance.
fn projectile_defeat(g: &mut Game, e: Entity) {
    let local = g
        .world
        .get::<&PixelBody>(e)
        .unwrap()
        .cells()
        .next()
        .unwrap()
        .0;
    let m = *g.world.get::<&Motion>(e).unwrap();
    let a = *g.world.get::<&Actor>(e).unwrap();
    g.spawn_bullet(
        actor_transform(m, a).point(local),
        Vec2::ZERO,
        1000.0,
        true,
        1,
    );
    g.take_events();
    g.tick(&InputFrame::default(), DT);
    assert!(!g.world.contains(e));
    assert!(g.take_events().iter().any(|e| e.kind == "explosion"));
}

#[test]
fn completed_legacy_tutorial_keeps_earned_inventory_without_restarting_training() {
    for corp in ["pan", "renko", "hugo"] {
        let mut p = create_profile("dora", corp, "renko", "0123456789abcdef").unwrap();
        p.schema_version = 5;
        p.onboarding = Some(OnboardingProgress {
            version: 1,
            stage: 6,
            flags: 127,
            ..Default::default()
        });
        p.currency = [3, 4, 5];
        p.campaign.stored_materials[0] = 7;
        let native = p.identity.as_ref().unwrap().corporation.ability();
        let extra = if native == 0 { 1 } else { 0 };
        p.campaign.abilities[extra] = campaign::AbilityState {
            blueprint: true,
            rank: 1,
        };
        p.campaign.loadouts[0].slots[abilities::get(extra as u8).unwrap().category] =
            Some(extra as u8);
        let mut g = Game::new(p);
        assert!(g.hud().onboarding.unwrap().completed);
        assert!(g.profile.onboarding.as_ref().unwrap().legacy_complete);
        assert_eq!(g.profile.campaign.abilities[native as usize].rank, 1);
        assert_eq!(g.profile.campaign.abilities[extra].rank, 1);
        assert_eq!(g.profile.campaign.stored_materials[0], 7);
        assert_eq!(g.profile.campaign.stored_training_materials, [0; 3]);
        assert_eq!(g.profile.currency, [3, 4, 5]);
        let done = snapshot(&g);
        g.acknowledge_license().unwrap();
        g.equip_ability(
            abilities::get(extra as u8).unwrap().category,
            Some(abilities::get(extra as u8).unwrap().key),
        )
        .unwrap();
        assert_eq!(snapshot(&g), done);
        assert_eq!(snapshot(&Game::new(g.profile.clone())), done);
    }
}
#[test]
fn wrong_rock_qa_gift_and_reload_never_fake_tutorial_completion() {
    let mut g = fresh("hugo");
    g.move_player(Vec2::new(5000.0, 0.0));
    g.update_chunks(g.position());
    let rock = g
        .world
        .query::<(Entity, &AsteroidBirth)>()
        .iter()
        .find(|(_, b)| b.source != AsteroidSource::HavenBelt)
        .unwrap()
        .0;
    projectile_defeat(&mut g, rock);
    assert_eq!(facts(&g) & BELT_DESTROYED, 0);
    g.move_player(Vec2::ZERO);
    g.equip_ability(1, Some("guard-coil")).unwrap();
    assert_eq!(facts(&g) & ADDITIONAL_EQUIPPED, 0);
    assert!(g.craft_ability("guard-coil").is_err());
    g.move_player(Vec2::new(500.0, 0.0));
    g.spawn_material_loot(g.position(), 1, 2);
    g.tick(&InputFrame::default(), DT);
    let reload = Game::new(g.profile.clone());
    assert_eq!(reload.profile.campaign.stored_materials[0], 0);
    assert_eq!(facts(&reload) & T1_BANKED, 0);
    assert_eq!(reload.hud().material_loss, [2, 0, 0, 0, 0]);
    g.reset_to_dock();
    assert_eq!(facts(&g) & T1_BANKED, 0);
    assert_eq!(g.profile.campaign.stored_materials[0], 0);
}

#[test]
fn saturation_only_records_material_actually_transferred_and_death_does_not_bank() {
    let mut g = fresh("pan");
    g.profile.campaign.stored_materials[0] = u32::MAX;
    g.profile.campaign.cargo_materials[0] = 2;
    g.tick(&InputFrame::default(), DT);
    assert_eq!(facts(&g) & T1_BANKED, 0);
    assert_eq!(g.profile.campaign.cargo_materials[0], 2);
    g.profile.campaign.stored_materials[0] -= 1;
    g.tick(&InputFrame::default(), DT);
    assert_eq!(facts(&g) & T1_BANKED, 0); // Generic T1 is not the named training material.
    assert_eq!(g.profile.campaign.cargo_materials[0], 1);
    let mut dead = fresh("renko");
    dead.move_player(Vec2::new(500.0, 0.0));
    dead.spawn_material_loot(dead.position(), 1, 2);
    dead.tick(&InputFrame::default(), DT);
    dead.invulnerability = 0.0;
    dead.hurt_player(1000.0, None);
    dead.tick(&InputFrame::default(), DT);
    assert_eq!(dead.profile.campaign.stored_materials[0], 0);
    assert_eq!(facts(&dead) & T1_BANKED, 0);
    assert!(dead.take_events().iter().any(|event| event.kind == "death"));
}

#[test]
fn corporate_hulls_are_distinct_detailed_live_cells_with_derived_collision_and_jets() {
    let mut masks = Vec::new();
    for (corp, key) in [
        (Corporation::Pan, "pan"),
        (Corporation::Renko, "renko"),
        (Corporation::Hugo, "hugo"),
    ] {
        let body = starter_body(corp);
        assert_eq!((body.width, body.height), (48, 48));
        assert!((450..=1400).contains(&body.occupied_count()));
        let palette: HashSet<_> = body.cells().map(|(_, c)| c).collect();
        assert!(palette.len() >= 7);
        masks.push(body.cells().map(|(p, _)| p.to_array()).collect::<Vec<_>>());
        let mut g = fresh(key);
        let a = *g.world.get::<&Actor>(g.player).unwrap();
        assert_eq!(a.scale, 0.7);
        assert_eq!(a.radius, pixel_assets::radius(body, a.scale));
        assert!((14.0..=23.0).contains(&a.radius));
        let ports = g.exhaust_ports();
        assert!(
            ports.iter().all(Option::is_some),
            "{key} missing live emitter"
        );
        let body = g.world.get::<&PixelBody>(g.player).unwrap();
        for port in ports.into_iter().flatten() {
            assert!(body.cells().any(|(p, c)| p == port && c == EMITTER));
        }
        drop(body);
        advance(
            &mut g,
            &InputFrame {
                joystick: Some([1.0, 0.0]),
                ..Default::default()
            },
            3,
        );
        let expected = corp.color();
        assert!(
            g.particles.iter().any(
                |p| p.style == ParticleStyle::Exhaust && p.color.to_le_bytes()[..3] == expected
            )
        );
        assert!(
            g.light_sources(1.0)
                .iter()
                .any(|l| l.color == expected.map(|c| c as f32 / 255.0))
        );
        // All emitter owner cells erased: no phantom exhaust port is selected.
        let mut body = g.world.get::<&mut PixelBody>(g.player).unwrap();
        let cells: Vec<_> = body
            .cells()
            .filter(|(_, c)| *c == EMITTER)
            .map(|(p, _)| p)
            .collect();
        for p in cells {
            body.damage(p, 0.1);
        }
        drop(body);
        assert!(g.exhaust_ports().iter().all(Option::is_none));
        g.move_player(Vec2::ZERO);
        g.enter_dock();
        assert_eq!(
            g.world
                .get::<&PixelBody>(g.player)
                .unwrap()
                .cells()
                .collect::<Vec<_>>(),
            starter_body(corp).cells().collect::<Vec<_>>()
        );
    }
    assert_ne!(masks[0], masks[1]);
    assert_ne!(masks[1], masks[2]);
    assert_ne!(masks[0], masks[2]);
}

#[test]
fn dock_corporation_mapping_preserves_structural_materials_legacy_and_facility_hitboxes() {
    let legacy = Profile::default();
    let original = dock_body();
    assert!(
        original
            .cells()
            .all(|(_, color)| dock_color(&legacy, color) == color)
    );
    for key in ["pan", "renko", "hugo"] {
        let mut g = fresh(key);
        let colors: Vec<_> = original.cells().map(|(_, c)| c).collect();
        let mapped: Vec<_> = colors.iter().map(|&c| dock_color(g.profile(), c)).collect();
        let changed = colors.iter().zip(&mapped).filter(|(a, b)| a != b).count();
        assert!(changed > 100);
        assert!(changed < colors.len() / 2);
        for (id, center) in dock_facilities::CENTERS.iter().enumerate() {
            assert_eq!(g.dock_facility_at(*center), Some(id as u8));
        }
        g.set_reduced_motion(true);
        let first = g.render(1.0);
        advance(&mut g, &InputFrame::default(), 10);
        let second = g.render(1.0);
        let crest = |frame: &RenderFrame| {
            frame
                .batches
                .iter()
                .flat_map(|b| {
                    b.pixels.iter().filter_map(|p| {
                        let t = b.transforms[p.transform_slot()];
                        (t.a[0] == 0.0 && t.a[1] == -64.0).then_some((p.local, p.color, t))
                    })
                })
                .collect::<Vec<_>>()
        };
        assert!(!crest(&first).is_empty());
        assert_eq!(crest(&first), crest(&second));
    }
}

#[test]
fn ship_switch_arena_and_real_death_keep_the_identity_body_only_on_hull_zero() {
    let mut p = create_profile("talo", "pan", "hugo", "0123456789abcdef").unwrap();
    p.campaign.patches[0] = true;
    p.campaign.blueprints[0] = true;
    p.campaign.crafted[0] = true;
    p.campaign.ship_xp[0] = 200;
    p.campaign.ship_xp[1] = 60;
    let mut g = Game::new(p);
    g.switch_ship(1).unwrap();
    assert_eq!(
        g.world
            .get::<&PixelBody>(g.player)
            .unwrap()
            .cells()
            .collect::<Vec<_>>(),
        campaign::player_body(1).cells().collect::<Vec<_>>()
    );
    g.switch_ship(0).unwrap();
    assert_eq!(g.profile.campaign.ship_xp[..2], [200, 60]);
    let site = campaign::arena(g.profile.seed, 1).unwrap();
    g.move_player(Vec2::from_array(site.position));
    g.start_champion(1).unwrap();
    assert_eq!(
        g.world
            .get::<&PixelBody>(g.player)
            .unwrap()
            .cells()
            .collect::<Vec<_>>(),
        starter_body(Corporation::Pan).cells().collect::<Vec<_>>()
    );
    g.invulnerability = 0.0;
    g.hurt_player(1000.0, None);
    g.tick(&InputFrame::default(), DT);
    assert_eq!(g.position(), Vec2::ZERO);
    assert!(g.take_events().iter().any(|e| e.kind == "death"));
    assert_eq!(
        g.world
            .get::<&PixelBody>(g.player)
            .unwrap()
            .cells()
            .collect::<Vec<_>>(),
        starter_body(Corporation::Pan).cells().collect::<Vec<_>>()
    );
    assert_eq!(g.profile.campaign.ship_xp[..2], [200, 60]);
    assert_eq!(
        g.profile.identity.as_ref().unwrap().corporation,
        Corporation::Pan
    );
}

#[test]
fn actual_diagnostic_belt_defeat_does_not_record_license_or_tutorial_facts() {
    let mut g = fresh("hugo");
    let before = snapshot(&g);
    g.boss_qa_scene = true;
    g.move_player(Vec2::new(950.0, 0.0));
    g.update_chunks(g.position());
    let rock = g
        .world
        .query::<(Entity, &AsteroidBirth)>()
        .iter()
        .find(|(_, b)| b.source == AsteroidSource::HavenBelt)
        .unwrap()
        .0;
    projectile_defeat(&mut g, rock);
    assert_eq!(snapshot(&g), before);
    g.acknowledge_license().unwrap();
    assert_eq!(facts(&g), 0);
}

#[test]
fn public_combat_scene_cannot_turn_a_license_into_a_fake_departure() {
    let mut g = fresh("hugo");
    g.acknowledge_license().unwrap();
    let before = snapshot(&g);
    assert_eq!(facts(&g), LICENSE);
    g.diagnostic("combat");
    assert_eq!(g.position(), Vec2::new(1600.0, 0.0));
    assert!(g.diagnostic_scene && !g.boss_qa_scene);
    let first = g.world.query::<(Entity, &Brain)>().iter().next().unwrap().0;
    let initial = g.world.get::<&Motion>(first).unwrap().position;
    advance(&mut g, &InputFrame::default(), 30);
    assert!(g.time > 0.0);
    assert_ne!(g.world.get::<&Motion>(first).unwrap().position, initial);
    assert_eq!(facts(&g), LICENSE);
    assert_eq!(g.hud().onboarding.unwrap().stage, 1);
    assert_eq!(snapshot(&g), before);
}

#[test]
fn every_public_diagnostic_scene_tick_is_progression_isolated() {
    let mut commands: Vec<String> = [
        "belt",
        "combat",
        "damage",
        "site",
        "tournament",
        "stress",
        "die",
        "biome-riveter",
        "biome-mote",
        "boss-dart",
        "boss-bastion",
        "boss-coil",
        "warp 950 0",
        "warp 0 0",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    for family in ["dart", "bastion", "coil"] {
        for generation in 1..=5 {
            commands.push(format!("boss-{family}-{generation}"));
        }
    }
    for index in 0..8 {
        commands.push(format!("champion-{index}"));
    }
    for command in commands {
        let mut g = fresh("hugo");
        g.acknowledge_license().unwrap();
        g.profile.currency = [7, 8, 9];
        g.profile.campaign.ship_xp[0] = 60;
        g.profile.campaign.last_run = Some(RunSummary {
            time: 7.0,
            enemies: 2,
            xp_gained: 10,
            ..Default::default()
        });
        let before = snapshot(&g);
        g.diagnostic(&command);
        advance(&mut g, &InputFrame::default(), 3);
        assert_eq!(snapshot(&g), before, "{command}");
        assert_eq!(facts(&g), LICENSE, "{command}");
    }
}

// A center-crossing real swept shot tolerates the ordinary scene actor moving
// during the tick. The existing first-cell stationary helper is inappropriate
// for this live combat setup; no kill/damage/reward helper is called here.
fn diagnostic_projectile_defeat(g: &mut Game, entity: Entity) {
    let local = g
        .world
        .get::<&PixelBody>(entity)
        .unwrap()
        .cells()
        .min_by(|(a, _), (b, _)| a.length_squared().total_cmp(&b.length_squared()))
        .unwrap()
        .0;
    let motion = *g.world.get::<&Motion>(entity).unwrap();
    let actor = *g.world.get::<&Actor>(entity).unwrap();
    let contact = actor_transform(motion, actor).point(local);
    g.spawn_bullet(contact - Vec2::X * 8.0, Vec2::X * 960.0, 1000.0, true, 1);
    g.take_events();
    g.tick(&InputFrame::default(), DT);
    assert!(!g.world.contains(entity));
    assert!(
        g.take_events()
            .iter()
            .any(|event| event.kind == "explosion")
    );
}

#[test]
fn public_combat_and_belt_real_projectile_defeats_do_not_earn_any_progress() {
    let mut g = fresh("hugo");
    g.acknowledge_license().unwrap();
    // Declared near-promotion checkpoint proves a real defeat cannot reserve G1.
    g.profile.kills[0] = 23;
    g.profile.currency = [7, 8, 9];
    g.profile.campaign.ship_xp[0] = 60;
    let before = snapshot(&g);
    g.diagnostic("combat");
    let enemy = g
        .world
        .query::<(Entity, &Brain)>()
        .iter()
        .find(|(_, brain)| brain.family == 0)
        .unwrap()
        .0;
    diagnostic_projectile_defeat(&mut g, enemy);
    assert_eq!(snapshot(&g), before);
    assert!(!g.profile.pending[0]);
    // Authored pickups are processed by the same real tick/collection path.
    g.spawn_loot(g.position(), 0, 3);
    g.spawn_material_loot(g.position(), 1, 2);
    advance(&mut g, &InputFrame::default(), 2);
    assert_eq!(snapshot(&g), before);
    assert_eq!(g.world.query::<&Loot>().iter().count(), 0);
    g.diagnostic("belt");
    let rock = g
        .world
        .query::<(Entity, &AsteroidBirth)>()
        .iter()
        .find(|(_, birth)| birth.source == AsteroidSource::HavenBelt)
        .unwrap()
        .0;
    diagnostic_projectile_defeat(&mut g, rock);
    assert_eq!(snapshot(&g), before);
    assert_eq!(facts(&g), LICENSE);
}

#[test]
fn public_site_and_tournament_fixtures_cannot_discover_or_start_rewarded_trials() {
    let mut g = fresh("hugo");
    g.acknowledge_license().unwrap();
    let before = snapshot(&g);
    g.diagnostic("site");
    assert!(g.profile.discoveries.is_empty());
    assert!(g.interact_site().is_err());
    advance(&mut g, &InputFrame::default(), 2);
    assert_eq!(snapshot(&g), before);
    let index = 0;
    let site = campaign::arena(g.profile.seed, index).unwrap();
    g.diagnostic(&format!("warp {} {}", site.position[0], site.position[1]));
    assert!(g.start_champion(index).is_err());
    assert!(!g.hud().campaign.arena.unwrap().can_challenge);
    assert_eq!(snapshot(&g), before);
    g.diagnostic("tournament");
    assert!(g.start_tournament().is_err());
    assert_eq!(snapshot(&g), before);
}

#[test]
fn a_physical_fixture_return_to_haven_cannot_end_isolation_or_bank_materials() {
    let mut g = fresh("hugo");
    g.acknowledge_license().unwrap();
    g.diagnostic("combat");
    // Declared carried-material fixture tests the banking boundary independently.
    g.profile.campaign.cargo_materials[0] = 2;
    g.move_player(Vec2::ZERO);
    let before = snapshot(&g);
    g.tick(&InputFrame::default(), DT);
    assert!(g.diagnostic_scene);
    assert_eq!(snapshot(&g), before);
    assert_eq!(g.profile.campaign.stored_materials, [0; 5]);
    assert_eq!(facts(&g), LICENSE);
}

#[test]
fn explicit_normal_reset_restores_real_departure_and_keeps_saved_assets() {
    for exit in ["home", "dock", "reset"] {
        let mut g = fresh("hugo");
        g.acknowledge_license().unwrap();
        g.profile.currency = [7, 8, 9];
        g.profile.campaign.ship_xp[0] = 60;
        g.profile.campaign.stored_materials = [2, 3, 4, 5, 6];
        let before = snapshot(&g);
        g.diagnostic("combat");
        g.tick(&InputFrame::default(), DT);
        if exit == "reset" {
            g.reset_to_dock();
        } else {
            g.diagnostic(exit);
        }
        assert!(!g.diagnostic_scene && !g.boss_qa_scene);
        assert_eq!(snapshot(&g), before);
        advance(
            &mut g,
            &InputFrame {
                joystick: Some([1.0, 0.0]),
                ..Default::default()
            },
            200,
        );
        assert!(g.position().length() > SAFE_RADIUS);
        assert_eq!(facts(&g), LICENSE | DEPARTED);
        assert_eq!(g.profile.currency, [7, 8, 9]);
        assert_eq!(g.profile.campaign.ship_xp[0], 60);
        assert_eq!(g.profile.campaign.stored_materials, [2, 3, 4, 5, 6]);
    }
}

#[test]
fn invalid_public_diagnostics_do_not_abandon_a_real_run_or_change_isolation() {
    let mut g = fresh("hugo");
    g.acknowledge_license().unwrap();
    advance(
        &mut g,
        &InputFrame {
            joystick: Some([1.0, 0.0]),
            ..Default::default()
        },
        200,
    );
    g.spawn_material_loot(g.position(), 1, 2);
    g.tick(&InputFrame::default(), DT);
    assert!(g.run.is_some());
    assert_eq!(g.profile.campaign.cargo_materials[0], 2);
    let before = snapshot(&g);
    let position = g.position();
    let run = serde_json::to_value(&g.run).unwrap();
    let entities = g.world.len();
    g.take_events();
    for command in [
        "unknown",
        "warp NaN 0",
        "warp 1000001 0",
        "boss-dart-6",
        "champion-8",
        "warp 1 2 3",
    ] {
        g.diagnostic(command);
        assert!(!g.diagnostic_scene, "{command}");
        assert_eq!(snapshot(&g), before, "{command}");
        assert_eq!(g.position(), position, "{command}");
        assert_eq!(serde_json::to_value(&g.run).unwrap(), run, "{command}");
        assert_eq!(g.world.len(), entities, "{command}");
        assert!(g.take_events().is_empty(), "{command}");
    }
    g.diagnostic("combat");
    let before = snapshot(&g);
    for command in ["unknown", "warp NaN 0", "champion-8"] {
        g.diagnostic(command);
        assert!(g.diagnostic_scene);
        assert_eq!(snapshot(&g), before);
    }
}

#[test]
fn diagnostic_crafting_and_equipping_do_not_record_tutorial_facts() {
    let mut g = fresh("hugo");
    g.acknowledge_license().unwrap();
    g.profile.currency[0] = 3;
    g.profile.campaign.stored_materials[0] = 2;
    g.profile.campaign.abilities[0].blueprint = true;
    g.diagnostic("warp 0 0");
    // Real, fully paid inventory actions stay valid; only earned tutorial facts
    // are suppressed. No free ability/currency is injected by a QA command.
    g.craft_ability("pulse-lens").unwrap();
    g.equip_ability(0, Some("pulse-lens")).unwrap();
    assert_eq!(g.profile.currency[0], 0);
    assert_eq!(g.profile.campaign.stored_materials[0], 0);
    assert_eq!(facts(&g), LICENSE);
}

#[test]
fn public_fixture_reset_does_not_deliver_a_pending_courier_packet() {
    for command in [
        "combat",
        "belt",
        "damage",
        "site",
        "tournament",
        "stress",
        "boss-dart",
        "champion-0",
        "biome-mote",
        "warp 950 0",
        "die",
    ] {
        let mut g = fresh("hugo");
        g.acknowledge_license().unwrap();
        let site = regions::sites_near(g.profile.seed, Vec2::ZERO)
            .into_iter()
            .find(|site| site.event == "courier")
            .unwrap();
        g.move_player(Vec2::from_array(site.position));
        g.interact_site().unwrap();
        assert_eq!(g.profile.quests[0].progress, 0);
        let before = snapshot(&g);
        g.diagnostic(command);
        g.tick(&InputFrame::default(), DT);
        assert_eq!(snapshot(&g), before, "{command}");
        assert_eq!(g.profile.quests[0].progress, 0, "{command}");
    }
}

#[test]
fn diagnostic_setup_forfeits_a_real_league_match_without_awarding_a_fixture_win() {
    let mut g = fresh("hugo");
    g.acknowledge_license().unwrap();
    // Explicit earned-qualification checkpoint, not granted by a scene command.
    g.profile.campaign.patches = [true; 8];
    g.profile.campaign.blueprints = [true; 8];
    g.profile.currency = [7, 8, 9];
    g.profile.campaign.ship_xp[0] = 60;
    g.move_player(Vec2::from_array(
        campaign::tournament_site(g.profile.seed).position,
    ));
    g.start_tournament().unwrap();
    assert!(!g.arena.unwrap().diagnostic);
    g.diagnostic("combat");
    assert_eq!(g.profile.campaign.royale_results, [2, 0, 0, 0, 0, 0, 0, 0]);
    assert!(!g.profile.campaign.victory_claimed);
    let before = snapshot(&g);
    g.tick(&InputFrame::default(), DT);
    assert_eq!(snapshot(&g), before);
    g.reset_to_dock();
    assert_eq!(snapshot(&g), before);
    assert_eq!(g.profile.currency, [7, 8, 9]);
    assert_eq!(g.profile.campaign.ship_xp[0], 60);
}

#[test]
fn diagnostic_flight_refreshes_canonical_site_geometry_without_discovering_it() {
    let mut g = fresh("hugo");
    g.acknowledge_license().unwrap();
    g.diagnostic("combat");
    g.move_player(Vec2::new(regions::REGION_SIZE - 0.2, 0.0));
    g.world.get::<&mut Motion>(g.player).unwrap().velocity = Vec2::X * 100.0;
    let before = snapshot(&g);
    advance(&mut g, &InputFrame::default(), 2);
    assert!(g.position().x > regions::REGION_SIZE);
    assert_eq!(g.site_grid, region_grid(g.position()));
    assert_eq!(
        g.nearby_sites
            .iter()
            .map(|site| site.id.clone())
            .collect::<Vec<_>>(),
        regions::sites_near(g.profile.seed, g.position())
            .iter()
            .map(|site| site.id.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(snapshot(&g), before);
}
