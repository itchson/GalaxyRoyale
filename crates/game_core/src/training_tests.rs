//! Actual runtime event and transaction regressions for corporation training.
use super::*;
const DT: f32 = 1.0 / 60.0;
fn fresh(corp: &str) -> Game {
    Game::new(create_profile("dora", corp, "renko", "0123456789abcdef").unwrap())
}
fn json(g: &Game) -> serde_json::Value {
    serde_json::to_value(g.profile()).unwrap()
}
fn step(g: &mut Game, count: usize) {
    for _ in 0..count {
        g.tick(&InputFrame::default(), DT);
    }
}
fn rock(g: &mut Game, variety: Option<u8>) -> Entity {
    for angle in 0..8 {
        let p = forward(angle as f32 * TAU / 8.0) * 1000.0;
        g.move_player(p);
        g.update_chunks(p);
        if let Some(entity) = g
            .world
            .query::<(Entity, &AsteroidBirth)>()
            .iter()
            .find(|(_, b)| {
                b.source == AsteroidSource::HavenBelt && variety.is_none_or(|v| b.variety == v)
            })
            .map(|(e, _)| e)
        {
            return entity;
        }
    }
    panic!("Declared seed must stream a matching Haven rock");
}
// Declared high-damage real swept projectile, not a kill/reward helper.
fn defeat(g: &mut Game, e: Entity, rng: u32) {
    let m = *g.world.get::<&Motion>(e).unwrap();
    let actor = *g.world.get::<&Actor>(e).unwrap();
    let local = g
        .world
        .get::<&PixelBody>(e)
        .unwrap()
        .cells()
        .min_by(|(a, _), (b, _)| a.length_squared().total_cmp(&b.length_squared()))
        .unwrap()
        .0;
    let contact = actor_transform(m, actor).point(local);
    g.move_player(m.position - m.position.normalize_or_zero() * 70.0);
    g.spawn_bullet(contact - Vec2::X * 8.0, Vec2::X * 960.0, 1000.0, true, 1);
    g.rng = rng;
    g.take_events();
    g.tick(&InputFrame::default(), DT);
    assert!(!g.world.contains(e));
    assert!(g.take_events().iter().any(|e| e.kind == "explosion"));
}
fn collect_named(g: &mut Game, index: u8, amount: u32) {
    g.move_player(Vec2::new(600.0, 0.0));
    g.spawn_tiered_loot(g.position(), 3, 1, Some(index), amount);
    step(g, 1);
}
fn bank_by_crossing(g: &mut Game) {
    g.move_player(Vec2::new(241.0, 0.0));
    g.was_safe = false;
    g.world.get::<&mut Motion>(g.player).unwrap().velocity = -Vec2::X * 100.0;
    step(g, 1);
    assert!(g.position().length() < SAFE_RADIUS);
}
fn ten_real_kills(g: &mut Game) {
    for i in 0..10 {
        let e = rock(g, None);
        defeat(g, e, 10_000); // Declared RNG first draw>.25: no guaranteed material reward.
        assert_eq!(
            g.profile.onboarding.as_ref().unwrap().asteroids_destroyed,
            i + 1
        );
    }
}

#[test]
fn all_corporations_finish_with_real_kills_variety_drops_pickups_bank_and_equipped_upgrade() {
    for (corp, variety, index) in [("pan", 3, 0), ("renko", 1, 1), ("hugo", 2, 2)] {
        let mut g = fresh(corp);
        g.acknowledge_license().unwrap();
        for _ in 0..200 {
            g.tick(
                &InputFrame {
                    joystick: Some([1.0, 0.0]),
                    ..Default::default()
                },
                DT,
            );
        }
        assert!(g.position().length() > SAFE_RADIUS);
        ten_real_kills(&mut g);
        assert_eq!(g.hud().onboarding.unwrap().stage, 3);
        let mut attempts = 0;
        while g.profile.onboarding.as_ref().unwrap().materials_collected < 10 {
            attempts += 1;
            assert!(
                attempts <= 160,
                "Bounded declared seeded material-drop fixture"
            );
            let e = rock(&mut g, Some(variety));
            let born = *g.world.get::<&AsteroidBirth>(e).unwrap();
            assert_eq!(born.variety, variety);
            assert_eq!(born.material_tier, 1);
            assert_eq!(born.source, AsteroidSource::HavenBelt);
            defeat(&mut g, e, attempts as u32 * 997); // Actual debris and existing25% LCG drop path.
            let found = g
                .world
                .query::<(Entity, (&Motion, &Loot))>()
                .iter()
                .find(|(_, (_, l))| l.training_material == Some(index))
                .map(|(e, (m, _))| (e, m.position));
            let Some((loot, position)) = found else {
                continue;
            };
            g.move_player(position);
            step(&mut g, 1);
            assert!(!g.world.contains(loot));
            assert!(g.profile.onboarding.as_ref().unwrap().materials_collected <= 10);
        }
        eprintln!(
            "training {corp}: initial10 real kills; {attempts} matching-variety defeats to collect10 actual drops"
        );
        assert_eq!(g.profile.campaign.ship_xp, [0; 10]);
        assert!(g.profile.campaign.cargo_training_materials[index as usize] >= 10);
        assert_eq!(g.hud().onboarding.unwrap().stage, 4);
        bank_by_crossing(&mut g);
        let h = g.hud().onboarding.unwrap();
        assert_eq!(h.stage, 5);
        assert!(h.banked_matching >= 10);
        assert!(h.can_upgrade, "{}", h.upgrade_reason);
        let id = g.profile.identity.as_ref().unwrap().corporation.ability();
        let def = abilities::get(id).unwrap();
        let row = g
            .hud()
            .abilities
            .into_iter()
            .find(|a| a.index == id)
            .unwrap();
        assert!(row.training_recipe);
        assert_eq!(
            (
                row.required_level,
                row.material_tier,
                row.material_cost,
                row.point_cost
            ),
            (1, 1, 10, 0)
        );
        assert_eq!(
            row.required_training_material.unwrap().id,
            MATERIALS[index as usize].id
        );
        let currency = g.profile.currency;
        let stock = g.profile.campaign.stored_materials[0];
        let before_stats = Stats::from_profile(g.profile());
        g.upgrade_ability(def.key).unwrap();
        assert_eq!(g.profile.campaign.abilities[id as usize].rank, 2);
        assert_eq!(g.profile.currency, currency);
        assert_eq!(g.profile.campaign.stored_materials[0], stock - 10);
        assert_eq!(
            g.profile.campaign.stored_training_materials[index as usize],
            h.banked_matching - 10
        );
        assert_eq!(g.profile.campaign.ship_xp, [0; 10]);
        assert_eq!(g.ship_level(), 1);
        assert_eq!(ship_progression::equipped_rank(g.profile(), id), 2);
        let after = Stats::from_profile(g.profile());
        match id {
            0 => assert!(after.damage > before_stats.damage),
            1 => assert!(after.shield > before_stats.shield),
            2 => assert!(after.speed > before_stats.speed),
            _ => unreachable!(),
        }
        assert!(g.hud().onboarding.unwrap().completed);
        assert_eq!(g.profile.onboarding.as_ref().unwrap().flags, 127);
        let done = json(&g);
        assert!(g.upgrade_ability(def.key).is_err()); // R3 ordinary LV5/T3 gate remains.
        g.acknowledge_license().unwrap();
        assert_eq!(json(&g), done);
        let reloaded = Game::new(g.profile.clone());
        assert_eq!(json(&reloaded), done);
        assert_eq!(ship_progression::equipped_rank(reloaded.profile(), id), 2);
    }
}

#[test]
fn counts_require_real_matching_post_ten_pickups_and_respect_capacity() {
    let mut g = fresh("renko");
    collect_named(&mut g, 1, 3);
    assert_eq!(
        g.profile.onboarding.as_ref().unwrap().materials_collected,
        0
    );
    ten_real_kills(&mut g);
    collect_named(&mut g, 0, 4);
    g.spawn_material_loot(g.position(), 1, 2);
    g.spawn_tiered_loot(g.position(), 3, 2, Some(1), 2); // Declared invalid tier-label fixture is not named T1 credit.
    step(&mut g, 1);
    assert_eq!(
        g.profile.onboarding.as_ref().unwrap().materials_collected,
        0
    );
    collect_named(&mut g, 1, 10);
    assert_eq!(
        ship_progression::total(g.profile.campaign.cargo_materials),
        16
    );
    assert_eq!(
        g.profile.onboarding.as_ref().unwrap().materials_collected,
        5
    );
    assert_eq!(g.profile.campaign.cargo_training_materials, [4, 8, 0]);
    step(&mut g, 2);
    assert_eq!(
        g.profile.onboarding.as_ref().unwrap().materials_collected,
        5
    );
    assert!(
        g.world
            .query::<&Loot>()
            .iter()
            .any(|l| l.training_material == Some(1) && l.amount == 5)
    );
}

#[test]
fn named_subsets_bank_and_generic_spend_do_not_duplicate_materials() {
    let mut g = fresh("pan");
    g.profile.campaign.cargo_materials = [16, 0, 0, 0, 0];
    g.profile.campaign.cargo_training_materials = [4, 3, 2];
    bank_by_crossing(&mut g);
    assert_eq!(g.profile.campaign.stored_materials[0], 16);
    assert_eq!(g.profile.campaign.stored_training_materials, [4, 3, 2]);
    g.consume_stored_materials([6, 0, 0, 0, 0]); //7unlabeled first.
    assert_eq!(g.profile.campaign.stored_training_materials, [4, 3, 2]);
    g.consume_stored_materials([5, 0, 0, 0, 0]); // Remaining1unlabeled then4silicate.
    assert_eq!(g.profile.campaign.stored_materials[0], 5);
    assert_eq!(g.profile.campaign.stored_training_materials, [0, 3, 2]);
    assert_eq!(label_total(g.profile.campaign.stored_training_materials), 5);
    let mut labels = [u32::MAX; 3];
    clamp_labels(16, &mut labels);
    assert_eq!(labels, [16, 0, 0]);
}

#[test]
fn bank_overflow_keeps_untransferred_named_cargo_and_records_no_false_bank_goal() {
    let mut g = fresh("hugo");
    g.profile.campaign.stored_materials[0] = u32::MAX - 1;
    g.profile.campaign.cargo_materials[0] = 3;
    g.profile.campaign.cargo_training_materials = [0, 0, 3];
    bank_by_crossing(&mut g);
    assert_eq!(g.profile.campaign.stored_materials[0], u32::MAX);
    assert_eq!(g.profile.campaign.stored_training_materials, [0, 0, 1]);
    assert_eq!(g.profile.campaign.cargo_materials[0], 2);
    assert_eq!(g.profile.campaign.cargo_training_materials, [0, 0, 2]);
    assert_eq!(g.profile.onboarding.as_ref().unwrap().flags & T1_BANKED, 0);
}

#[test]
fn death_reload_and_abandon_lose_labels_but_keep_genuine_counted_history_and_stash() {
    for mode in 0..3 {
        let mut g = fresh("pan");
        ten_real_kills(&mut g);
        collect_named(&mut g, 0, 10);
        g.profile.campaign.stored_materials[0] = 4;
        g.profile.campaign.stored_training_materials = [3, 0, 0];
        g.profile.currency = [7, 8, 9];
        g.profile.campaign.ship_xp[0] = 20;
        if mode == 0 {
            g.invulnerability = 0.0;
            g.hurt_player(1000.0, None);
            step(&mut g, 1);
            assert_eq!(
                g.take_events().iter().filter(|e| e.kind == "death").count(),
                1
            );
        } else if mode == 1 {
            g = Game::new(g.profile.clone());
        } else {
            g.reset_to_dock();
        }
        assert_eq!(g.profile.campaign.cargo_materials, [0; 5]);
        assert_eq!(g.profile.campaign.cargo_training_materials, [0; 3]);
        assert_eq!(g.profile.campaign.stored_materials[0], 4);
        assert_eq!(g.profile.campaign.stored_training_materials, [3, 0, 0]);
        assert_eq!(g.profile.currency, [7, 8, 9]);
        assert_eq!(g.profile.campaign.ship_xp[0], 20);
        let p = g.profile.onboarding.as_ref().unwrap();
        assert_eq!((p.asteroids_destroyed, p.materials_collected), (10, 10));
        assert_eq!(p.flags & T1_BANKED, 0);
        assert!(!g.hud().onboarding.unwrap().can_upgrade);
    }
}

#[test]
fn public_scenes_kills_pickups_crafting_and_home_travel_cannot_advance_training() {
    let mut g = fresh("pan");
    g.acknowledge_license().unwrap();
    g.diagnostic("belt");
    let before = json(&g);
    let e = g
        .world
        .query::<(Entity, &AsteroidBirth)>()
        .iter()
        .next()
        .unwrap()
        .0;
    defeat(&mut g, e, 0);
    collect_named(&mut g, 0, 10);
    bank_by_crossing(&mut g);
    assert!(g.upgrade_ability("flux-drive").is_err());
    assert_eq!(json(&g), before);
    g.diagnostic("home");
    ten_real_kills(&mut g);
    assert_eq!(
        g.profile.onboarding.as_ref().unwrap().asteroids_destroyed,
        10
    );
}

#[test]
fn migration_and_impossible_v2_facts_preserve_assets_without_inventing_counts() {
    for flags in [1, 3, 7, 31, 127] {
        let mut p = create_profile("dora", "renko", "pan", "0123456789abcdef").unwrap();
        p.schema_version = 5;
        p.onboarding = Some(OnboardingProgress {
            version: 1,
            flags,
            stage: 6,
            ..Default::default()
        });
        p.currency = [11, 12, 13];
        p.campaign.ship_xp[0] = 60;
        p.campaign.stored_materials[0] = 9;
        p.campaign.stored_training_materials = [9, 0, 0]; // Never known in schema5.
        let g = Game::new(p);
        let mission = g.profile.onboarding.as_ref().unwrap();
        assert_eq!(mission.version, 2);
        assert_eq!(
            (mission.asteroids_destroyed, mission.materials_collected),
            (0, 0)
        );
        assert_eq!(mission.legacy_complete, flags == 127);
        assert_eq!(mission.flags, if flags == 127 { 127 } else { flags & 3 });
        assert_eq!(g.profile.currency, [11, 12, 13]);
        assert_eq!(g.profile.campaign.ship_xp[0], 60);
        assert_eq!(g.profile.campaign.stored_materials[0], 9);
        assert_eq!(g.profile.campaign.stored_training_materials, [0; 3]);
        assert_eq!(ship_progression::equipped_rank(g.profile(), 0), 1);
        assert_eq!(json(&Game::new(g.profile.clone())), json(&g));
    }
    let mut p = create_profile("talo", "pan", "hugo", "0123456789abcdef").unwrap();
    p.onboarding = Some(OnboardingProgress {
        flags: 127,
        asteroids_destroyed: 2,
        materials_collected: 255,
        ..Default::default()
    });
    p.campaign.cargo_materials[0] = 3;
    p.campaign.cargo_training_materials = [u32::MAX; 3];
    p.campaign.stored_materials[0] = 5;
    p.campaign.stored_training_materials = [4, 4, 4];
    let g = Game::new(p);
    assert_eq!(g.profile.onboarding.as_ref().unwrap().flags, 3);
    assert_eq!(
        g.profile.onboarding.as_ref().unwrap().materials_collected,
        0
    );
    assert_eq!(g.profile.campaign.stored_training_materials, [4, 1, 0]);
    assert_eq!(g.profile.campaign.cargo_training_materials, [0; 3]);
    assert_eq!(g.hud().material_loss, [3, 0, 0, 0, 0]);
}

#[test]
fn earned_training_effect_is_only_equipped_corporate_starter_on_hull_zero() {
    let mut p = create_profile("dora", "pan", "renko", "0123456789abcdef").unwrap();
    p.onboarding = Some(OnboardingProgress {
        flags: 127,
        stage: 6,
        asteroids_destroyed: 10,
        materials_collected: 10,
        ..Default::default()
    });
    p.campaign.abilities[2].rank = 2;
    p.campaign.patches[0] = true;
    p.campaign.blueprints[0] = true;
    p.campaign.crafted[0] = true;
    let mut g = Game::new(p);
    assert_eq!(ship_progression::equipped_rank(g.profile(), 2), 2);
    g.equip_ability(2, None).unwrap();
    assert_eq!(ship_progression::equipped_rank(g.profile(), 2), 0);
    assert_eq!(
        g.hud()
            .abilities
            .into_iter()
            .find(|a| a.index == 2)
            .unwrap()
            .effective_rank,
        1
    );
    g.equip_ability(2, Some("flux-drive")).unwrap();
    g.switch_ship(1).unwrap();
    g.equip_ability(2, Some("flux-drive")).unwrap();
    assert_eq!(ship_progression::equipped_rank(g.profile(), 2), 1);
    assert_eq!(g.profile.campaign.abilities[2].rank, 2);
    g.switch_ship(0).unwrap();
    assert_eq!(ship_progression::equipped_rank(g.profile(), 2), 2);
}

#[test]
fn missing_materials_wrong_slot_and_repeated_upgrade_are_atomic_without_rank_reset() {
    let mut g = fresh("hugo");
    ten_real_kills(&mut g);
    collect_named(&mut g, 2, 10);
    bank_by_crossing(&mut g);
    g.equip_ability(1, None).unwrap();
    let before = json(&g);
    assert!(g.upgrade_ability("guard-coil").is_err());
    assert_eq!(json(&g), before);
    g.equip_ability(1, Some("guard-coil")).unwrap();
    g.consume_stored_materials([1, 0, 0, 0, 0]);
    let before = json(&g);
    assert!(g.upgrade_ability("guard-coil").is_err());
    assert_eq!(json(&g), before);
    collect_named(&mut g, 2, 1);
    bank_by_crossing(&mut g);
    g.upgrade_ability("guard-coil").unwrap();
    let complete = json(&g);
    assert!(g.upgrade_ability("guard-coil").is_err());
    assert_eq!(json(&g), complete);
}

#[test]
fn already_earned_higher_rank_is_recognized_without_downgrade_or_new_reward() {
    let mut g = fresh("renko");
    g.profile.campaign.abilities[0].rank = 4;
    g.profile.campaign.ship_xp[0] = 560;
    ten_real_kills(&mut g);
    collect_named(&mut g, 1, 10);
    bank_by_crossing(&mut g);
    assert_eq!(g.profile.campaign.abilities[0].rank, 4);
    assert_eq!(g.profile.campaign.ship_xp[0], 560);
    assert_eq!(g.profile.campaign.stored_training_materials[1], 10);
    let p = g.profile.onboarding.as_ref().unwrap();
    assert_eq!(p.flags & 96, 96);
    assert!(p.stage < 6); // Actual ACK still needed; earned rank is not a license grant.
    g.acknowledge_license().unwrap();
    assert!(g.hud().onboarding.unwrap().completed);
}

#[test]
fn ordinary_craft_hardware_and_ship_recipes_debit_the_same_t1_subset_ledger() {
    let mut p = Profile {
        currency: [100; 3],
        ..Profile::default()
    };
    p.campaign.stored_materials[0] = 20;
    p.campaign.stored_training_materials = [5, 5, 5];
    p.campaign.patches[0] = true;
    p.campaign.blueprints[0] = true;
    let mut g = Game::new(p);
    g.craft_ability("pulse-lens").unwrap();
    assert_eq!(g.profile.campaign.stored_materials[0], 18);
    assert_eq!(g.profile.campaign.stored_training_materials, [5, 5, 5]);
    g.purchase("damage").unwrap();
    assert_eq!(g.profile.campaign.stored_materials[0], 17);
    assert_eq!(g.profile.campaign.stored_training_materials, [5, 5, 5]);
    g.craft_ship(0).unwrap();
    assert_eq!(g.profile.campaign.stored_materials[0], 11);
    assert_eq!(g.profile.campaign.stored_training_materials, [1, 5, 5]);
    assert_eq!(g.hud().material_training_stash, [1, 5, 5]);
    assert_eq!(
        label_total(g.profile.campaign.stored_training_materials),
        11
    );
}

#[test]
fn all_three_named_varieties_are_real_deterministic_home_terrain_and_do_not_change_on_drift() {
    let mut a = fresh("pan");
    let mut b = fresh("pan");
    for game in [&mut a, &mut b] {
        game.move_player(Vec2::new(1000.0, 0.0));
        game.update_chunks(game.position());
    }
    let scan = |g: &Game| {
        let mut cells = g
            .world
            .query::<(&AsteroidBirth, &Motion, &PixelBody)>()
            .iter()
            .filter(|(birth, _, _)| birth.source == AsteroidSource::HavenBelt)
            .map(|(birth, m, body)| {
                (
                    (m.position.x.to_bits(), m.position.y.to_bits()),
                    birth.variety,
                    birth.material_tier,
                    body.cells().count(),
                )
            })
            .collect::<Vec<_>>();
        cells.sort_by_key(|e| e.0);
        cells
    };
    assert_eq!(scan(&a), scan(&b));
    for variety in [1, 2, 3] {
        assert!(scan(&a).iter().any(|(_, v, t, _)| *v == variety && *t == 1));
    }
    for variety in [1, 2, 3] {
        let e = rock(&mut a, Some(variety));
        let before = *a.world.get::<&AsteroidBirth>(e).unwrap();
        let mut motion = a.world.get::<&mut Motion>(e).unwrap();
        motion.velocity = Vec2::new(100.0, 50.0);
        let origin = motion.position;
        drop(motion);
        a.integrate(0.05);
        a.enforce_asteroid_shields(true);
        let after = *a.world.get::<&AsteroidBirth>(e).unwrap();
        assert!(a.world.get::<&Motion>(e).unwrap().position.distance(origin) > 1.0);
        assert_eq!(after.variety, before.variety);
        assert_eq!(after.material_tier, 1);
        assert_eq!(after.source, AsteroidSource::HavenBelt);
        assert_eq!(
            material_for_variety(after.variety),
            Some([1, 2, 0][variety as usize - 1])
        );
    }
}

#[test]
fn first_mission_counts_outer_and_site_belt_asteroids_with_their_existing_xp_policy() {
    let mut g = fresh("pan");
    g.move_player(Vec2::new(5000.0, 0.0));
    g.update_chunks(g.position());
    let e = g
        .world
        .query::<(Entity, &AsteroidBirth)>()
        .iter()
        .find(|(_, b)| b.source == AsteroidSource::Outer)
        .unwrap()
        .0;
    defeat(&mut g, e, 10_000);
    assert_eq!(
        g.profile.onboarding.as_ref().unwrap().asteroids_destroyed,
        1
    );
    assert_eq!(g.profile.campaign.ship_xp[0], 2);
    let site = regions::sites_near(g.profile.seed, Vec2::new(5000.0, 0.0))
        .into_iter()
        .find(|s| regions::site_belt(g.profile.seed, s).is_some())
        .expect("Declared remote belt");
    let belt = regions::site_belt(g.profile.seed, &site).unwrap();
    let center = Vec2::from_array(site.position);
    g.move_player(center + Vec2::X * ((belt.inner + belt.outer) * 0.5));
    g.update_chunks(g.position());
    let e = g
        .world
        .query::<(Entity, &AsteroidBirth)>()
        .iter()
        .find(|(_, b)| b.source == AsteroidSource::SiteBelt(site.grid))
        .unwrap()
        .0;
    defeat(&mut g, e, 10_000);
    assert_eq!(
        g.profile.onboarding.as_ref().unwrap().asteroids_destroyed,
        2
    );
    assert_eq!(g.profile.campaign.ship_xp[0], 4);
}
