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
    for (kind, prefix) in [("dora", 'O'), ("doro", 'A'), ("talo", 'T')] {
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
    for (kind, prefix) in [("dora", 'O'), ("doro", 'A'), ("talo", 'T')] {
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
    assert!((a.speed - 104.0).abs() < 0.001);
    assert!((a.acceleration - 270.4).abs() < 0.001);
    assert!((b.damage - 13.2).abs() < 0.001);
    assert_eq!(b.speed, 100.0);
    assert_eq!(c.shield, 20.0);
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
    expected["schema_version"] = 5.into();
    expected["revision"] = 1.into();
    let g = Game::new(p);
    assert_eq!(snapshot(&g), expected);
    assert_eq!(g.hud().campaign.ships[0].name, "Haven Scout");
    assert_eq!(g.hud().abilities.len(), 3);
    assert!(g.hud().identity.is_none());
}

#[test]
fn stage_is_derived_from_bounded_flags_without_fabricated_facts() {
    for flags in 0..=255 {
        let mut p = create_profile("talo", "hugo", "pan", "0123456789abcdef").unwrap();
        p.onboarding = Some(OnboardingProgress { stage: 255, flags });
        let g = Game::new(p);
        let progress = g.profile.onboarding.as_ref().unwrap();
        assert_eq!(progress.flags, flags & 127);
        assert_eq!(progress.stage, stage(flags & 127));
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
fn complete_tutorial_uses_real_departure_streamed_belt_hit_pickup_bank_craft_and_equip() {
    for corp in ["pan", "renko", "hugo"] {
        let mut g = fresh(corp);
        g.acknowledge_license().unwrap();
        advance(
            &mut g,
            &InputFrame {
                joystick: Some([1.0, 0.0]),
                ..Default::default()
            },
            200,
        );
        assert!(g.position().length() > SAFE_RADIUS);
        assert_eq!(g.profile.onboarding.as_ref().unwrap().stage, 2);
        // Set travel position for an isolated collision fixture; departure above
        // is actual movement. The rock itself is deterministic streamed terrain.
        g.move_player(Vec2::new(950.0, 0.0));
        g.update_chunks(g.position());
        let e = g
            .world
            .query::<(Entity, &AsteroidBirth)>()
            .iter()
            .find(|(_, b)| b.source == AsteroidSource::HavenBelt)
            .unwrap()
            .0;
        projectile_defeat(&mut g, e);
        assert_eq!(g.profile.onboarding.as_ref().unwrap().stage, 3);
        assert_eq!(g.profile.campaign.ship_xp, [0; 10]);
        let extra = if corp == "renko" { 1 } else { 0 };
        let category = abilities::get(extra).unwrap().category;
        // Declared loot fixtures enter the real collection path. There is no
        // chance assertion pretending every rock guarantees material.
        g.spawn_material_loot(g.position(), 1, 2);
        g.spawn_loot(g.position(), category, 3);
        g.tick(&InputFrame::default(), DT);
        assert_eq!(g.profile.campaign.cargo_materials[0], 2);
        assert_eq!(g.profile.onboarding.as_ref().unwrap().stage, 4);
        assert!(g.profile.campaign.abilities[extra as usize].blueprint);
        // Actual final crossing, rather than reset_to_dock, banks carried cargo.
        g.move_player(Vec2::new(241.0, 0.0));
        g.was_safe = false;
        g.world.get::<&mut Motion>(g.player).unwrap().velocity = Vec2::new(-100.0, 0.0);
        g.tick(&InputFrame::default(), DT);
        assert!(g.position().length() < SAFE_RADIUS);
        assert_eq!(g.profile.campaign.cargo_materials, [0; 5]);
        assert_eq!(g.profile.campaign.stored_materials[0], 2);
        assert_eq!(g.profile.onboarding.as_ref().unwrap().stage, 5);
        let key = abilities::get(extra).unwrap().key;
        g.craft_ability(key).unwrap();
        assert_eq!(g.profile.onboarding.as_ref().unwrap().stage, 5);
        g.equip_ability(category, Some(key)).unwrap();
        assert_eq!(facts(&g), 127);
        assert!(g.hud().onboarding.unwrap().completed);
        assert_eq!(g.profile.currency[category], 0);
        assert_eq!(g.profile.campaign.stored_materials[0], 0);
        let done = snapshot(&g);
        g.acknowledge_license().unwrap();
        g.equip_ability(category, Some(key)).unwrap();
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
    assert_ne!(facts(&g) & T1_BANKED, 0);
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
                        let t = b.transforms[p.transform as usize];
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
