//! Actual constructor/transactions/ticks/shared collisions, with declared fixtures.
use super::*;
const DT: f32 = 1.0 / 60.0;

fn stocked() -> Game {
    let mut p = Profile {
        currency: [100; 3],
        ..Profile::default()
    };
    p.campaign.stored_materials = [30; 5];
    Game::new(p)
}
fn royal_profile() -> Profile {
    let mut p = Profile::default();
    p.campaign.patches = [true; 8];
    p.campaign.blueprints = [true; 8];
    p.campaign.royale_results = [1; 8];
    p.campaign.victory_claimed = true;
    p.campaign.selected_ship = 9;
    ship_progression::grant_royal(&mut p);
    p
}
fn json(g: &Game) -> serde_json::Value {
    serde_json::to_value(g.profile()).unwrap()
}

#[test]
fn fresh_hull_is_level_one_with_empty_three_slots_and_six_unpurchased_hardware_rows() {
    let mut g = Game::new(Profile::default());
    let h = g.hud();
    assert_eq!(g.profile.schema_version, 6);
    assert_eq!(h.ship_level, 1);
    assert_eq!(h.ship_xp, 0);
    assert_eq!(h.upgrades.len(), 6);
    assert_eq!(h.abilities.len(), 3);
    assert!(h.loadout.iter().all(Option::is_none));
    assert!(
        g.profile
            .campaign
            .loadouts
            .iter()
            .all(|l| l.hardware == [0; 6])
    );
    assert_eq!(g.profile.revision, 0);
    g.move_player(Vec2::new(500.0, 0.0));
    g.tick(
        &InputFrame {
            fire: true,
            ..Default::default()
        },
        DT,
    );
    assert_eq!(
        g.take_events().iter().filter(|e| e.kind == "shot").count(),
        1
    );
    assert_eq!(Stats::from_profile(g.profile()).speed, 100.0);
}

#[test]
fn three_real_starters_craft_then_equip_with_exact_atomic_banked_costs() {
    let mut g = stocked();
    let base = Stats::from_profile(g.profile());
    for category in 0..3 {
        let key = abilities::ABILITIES[category].key;
        let points = g.profile.currency;
        let materials = g.profile.campaign.stored_materials;
        g.craft_ability(key).unwrap();
        assert_eq!(g.profile.currency[category], points[category] - 3);
        assert_eq!(g.profile.campaign.stored_materials[0], materials[0] - 2);
        assert_eq!(g.profile.campaign.loadouts[0].slots[category], None);
        let before = json(&g);
        assert!(g.craft_ability(key).is_err());
        assert_eq!(json(&g), before);
        g.equip_ability(category, Some(key)).unwrap();
    }
    let stats = Stats::from_profile(g.profile());
    assert!((stats.damage - base.damage * 1.1).abs() < 0.001);
    assert_eq!(stats.shield, 60.0);
    assert!((stats.speed - base.speed * 1.04).abs() < 0.001);
    assert!((stats.acceleration - base.acceleration * 1.04).abs() < 0.001);
    for category in 0..3 {
        g.equip_ability(category, None).unwrap();
    }
    let unequipped = Stats::from_profile(g.profile());
    assert_eq!(unequipped.damage, base.damage);
    assert_eq!(unequipped.shield, base.shield);
    assert_eq!(unequipped.speed, base.speed);
}

#[test]
fn recipe_level_exact_tier_compatibility_and_invalid_slot_fail_without_mutation() {
    let mut g = stocked();
    g.craft_ability("pulse-lens").unwrap();
    let before = json(&g);
    assert!(g.upgrade_ability("pulse-lens").is_err());
    assert!(g.equip_ability(256, Some("pulse-lens")).is_err());
    assert!(g.equip_ability(1, Some("pulse-lens")).is_err());
    assert_eq!(json(&g), before);
    g.profile.campaign.ship_xp[0] = 60;
    g.profile.campaign.stored_materials[1] = 1;
    let before = json(&g);
    assert!(g.upgrade_ability("pulse-lens").is_err());
    assert_eq!(json(&g), before); // plentiful T1/T3 do not pay T2
    g.profile.campaign.stored_materials[1] = 2;
    g.upgrade_ability("pulse-lens").unwrap();
    assert_eq!(g.profile.campaign.stored_materials[1], 0);
    assert_eq!(g.profile.campaign.abilities[0].rank, 2);
    g.profile.campaign.patches[1] = true;
    g.profile.campaign.abilities[7] = campaign::AbilityState {
        blueprint: true,
        rank: 1,
    };
    let before = json(&g);
    assert!(g.equip_ability(0, Some("broadside")).is_err()); // Scout is Dart family
    assert_eq!(json(&g), before);
    g.move_player(Vec2::new(500.0, 0.0));
    let before = json(&g);
    assert!(g.equip_ability(0, None).is_err());
    assert!(g.purchase("speed").is_err());
    assert_eq!(json(&g), before);
}

#[test]
fn crafted_hull_has_independent_zero_xp_hardware_but_shared_rank_is_level_capped() {
    let mut g = stocked();
    g.profile.campaign.ship_xp[0] = 1320;
    g.purchase("damage").unwrap();
    g.craft_ability("pulse-lens").unwrap();
    for _ in 0..4 {
        g.upgrade_ability("pulse-lens").unwrap();
    }
    g.equip_ability(0, Some("pulse-lens")).unwrap();
    g.profile.campaign.patches[0] = true;
    g.profile.campaign.blueprints[0] = true;
    g.craft_ship(0).unwrap();
    assert_eq!(g.profile.campaign.ship_xp[1], 0);
    g.switch_ship(1).unwrap();
    assert_eq!(g.hud().ship_level, 1);
    assert_eq!(g.profile.ranks[..6], [0; 6]);
    assert_eq!(g.profile.campaign.loadouts[1].slots, [None; 3]);
    g.equip_ability(0, Some("pulse-lens")).unwrap();
    assert_eq!(g.profile.campaign.abilities[0].rank, 5);
    assert_eq!(g.hud().loadout[0].as_ref().unwrap().effective_rank, 1);
    g.switch_ship(0).unwrap();
    assert_eq!(g.profile.ranks[0], 1);
    assert_eq!(g.hud().loadout[0].as_ref().unwrap().effective_rank, 5);
}

#[test]
fn physical_home_arrival_banks_each_tier_before_checkpoint_without_a_teleport() {
    let mut g = Game::new(Profile::default());
    g.move_player(Vec2::new(255.0, 0.0));
    g.was_safe = false;
    g.track_run(DT);
    g.profile.campaign.cargo_materials = [1, 2, 3, 4, 5];
    for _ in 0..90 {
        g.tick(
            &InputFrame {
                destination: Some([200.0, 0.0]),
                ..Default::default()
            },
            DT,
        );
        if g.position().length() <= SAFE_RADIUS {
            break;
        }
    }
    assert!(g.position().length() <= SAFE_RADIUS);
    assert_eq!(g.profile.campaign.cargo_materials, [0; 5]);
    assert_eq!(g.profile.campaign.stored_materials, [1, 2, 3, 4, 5]);
    let saved = json(&g);
    let reload = Game::new(serde_json::from_value(saved.clone()).unwrap());
    assert_eq!(json(&reload), saved);
}

#[test]
fn death_abandonment_and_v4_reload_discard_only_cargo_and_report_the_real_loss() {
    for reason in ["death", "abandonment", "reload"] {
        let mut g = stocked();
        g.move_player(Vec2::new(500.0, 0.0));
        g.track_run(2.0);
        g.profile.campaign.cargo_materials = [1, 2, 3, 0, 0];
        let points = g.profile.currency;
        let stash = g.profile.campaign.stored_materials;
        match reason {
            "death" => {
                g.diagnostic("die");
                g.tick(&InputFrame::default(), DT);
            }
            "abandonment" => g.reset_to_dock(),
            _ => g = Game::new(g.profile.clone()),
        }
        assert_eq!(g.profile.currency, points);
        assert_eq!(g.profile.campaign.stored_materials, stash);
        assert_eq!(g.profile.campaign.cargo_materials, [0; 5]);
        assert_eq!(g.hud().material_loss, [1, 2, 3, 0, 0]);
        assert_eq!(g.hud().material_loss_reason, reason);
        if reason == "reload" {
            assert!(g.take_events().iter().all(|e| e.kind != "death"));
        } else {
            assert_eq!(g.hud().last_run.unwrap().materials_lost, [1, 2, 3, 0, 0]);
        }
        let saved = json(&g);
        let again = Game::new(serde_json::from_value(saved.clone()).unwrap());
        assert_eq!(json(&again), saved);
    }
}

#[test]
fn actual_last_occupied_ship_cell_loss_discards_cargo_even_with_positive_hull_and_ready_regeneration()
 {
    let mut g = Game::new(Profile::default());
    g.move_player(Vec2::new(500.0, 0.0));
    g.track_run(DT);
    g.profile.campaign.cargo_materials = [1, 2, 3, 4, 5];
    g.profile.campaign.stored_materials = [7, 8, 9, 10, 11];
    g.profile.campaign.loadouts[0].hardware[4] = 1;
    ship_progression::mirrors(&mut g.profile);
    g.since_damage = 6.0;
    g.shield = 0.0;
    // Declared last-cell damaged-body fixture. A real hostile projectile must
    // erase it, despite one point of damage being far below the positive hull.
    *g.world.get::<&mut PixelBody>(g.player).unwrap() =
        PixelBody::from_ascii(&["x"], &[('x', rgba(180, 210, 225, 255))]);
    let local = g
        .world
        .get::<&PixelBody>(g.player)
        .unwrap()
        .cells()
        .next()
        .unwrap()
        .0;
    let motion = *g.world.get::<&Motion>(g.player).unwrap();
    let actor = *g.world.get::<&Actor>(g.player).unwrap();
    assert!(actor.health > 90.0);
    let points = g.profile.currency;
    let xp = g.profile.campaign.ship_xp;
    g.spawn_bullet(
        actor_transform(motion, actor).point(local),
        Vec2::ZERO,
        1.0,
        false,
        1,
    );
    g.take_events();
    g.tick(&InputFrame::default(), DT);
    let events = g.take_events();
    assert_eq!(events.iter().filter(|e| e.kind == "hit").count(), 1);
    assert_eq!(events.iter().filter(|e| e.kind == "death").count(), 1);
    assert_eq!(events.iter().filter(|e| e.kind == "dock").count(), 1);
    assert_eq!(g.position(), Vec2::ZERO);
    assert_eq!(g.profile.campaign.cargo_materials, [0; 5]);
    assert_eq!(g.profile.campaign.stored_materials, [7, 8, 9, 10, 11]);
    assert_eq!(g.profile.currency, points);
    assert_eq!(g.profile.campaign.ship_xp, xp);
    assert_eq!(g.hud().material_loss, [1, 2, 3, 4, 5]);
    assert_eq!(g.hud().material_loss_reason, "death");
    assert_eq!(g.hud().last_run.unwrap().materials_lost, [1, 2, 3, 4, 5]);
    assert!(
        g.world
            .get::<&PixelBody>(g.player)
            .unwrap()
            .occupied_count()
            > 1
    );
    g.tick(&InputFrame::default(), DT);
    assert!(g.take_events().iter().all(|e| e.kind != "death"));
}

#[test]
fn legacy_assets_migrate_once_selected_hardware_and_native_slots_without_fabricated_xp() {
    let p:Profile=serde_json::from_value(serde_json::json!({"schema_version":3,"revision":7,
        "currency":[2,3,4],"ranks":[5,4,3,2,1,0,5,4,3],"unlocked":[true,true,true],
        "campaign":{"patches":[true,false,false,false,false,false,false,false],
        "blueprints":[true,false,false,false,false,false,false,false],"crafted":[true,false,false,false,false,false,false,false],
        "selected_ship":1,"cargo_materials":6,"stored_materials":11,"ship_xp":[60,0,0,0,0,0,0,0,0,0]}})).unwrap();
    let g = Game::new(p);
    assert_eq!(g.profile.campaign.stored_materials, [17, 0, 0, 0, 0]);
    assert_eq!(g.profile.campaign.cargo_materials, [0; 5]);
    assert_eq!(g.profile.campaign.loadouts[0].hardware, [0; 6]);
    assert_eq!(g.profile.campaign.loadouts[1].hardware, [5, 4, 3, 2, 1, 0]);
    assert_eq!(
        g.profile.campaign.loadouts[1].slots,
        [Some(6), Some(4), Some(5)]
    );
    assert_eq!(g.profile.campaign.abilities[3].rank, 5);
    assert_eq!(g.profile.campaign.abilities[6].rank, 1);
    assert_eq!(g.profile.campaign.ship_xp[1], 0);
    let saved = json(&g);
    let reload = Game::new(serde_json::from_value(saved.clone()).unwrap());
    assert_eq!(json(&reload), saved);
}

#[test]
fn legacy_owned_hardware_is_level_capped_then_real_ship_kills_activate_it_without_repurchase() {
    let p = Profile {
        schema_version: 3,
        ranks: [5, 5, 5, 5, 5, 5, 0, 0, 0],
        ..Profile::default()
    };
    let mut g = Game::new(p);
    assert_eq!(g.hud().ship_level, 1);
    assert_eq!(g.profile.campaign.loadouts[0].hardware, [5; 6]);
    assert!(
        g.hud()
            .upgrades
            .iter()
            .all(|u| u.rank == 5 && u.effective_rank == 1)
    );
    let first = Stats::from_profile(g.profile());
    assert_eq!(first.hull, 115.0);
    assert_eq!(first.shield, 40.0);
    assert!((first.damage - 13.44).abs() < 0.001);
    assert_eq!(first.fire_rate, 5.5);
    assert_eq!(first.regeneration, 0.4);
    assert!((first.speed - 104.0).abs() < 0.001);
    assert!((first.acceleration - 270.4).abs() < 0.001);
    let points = g.profile.currency;
    let stash = g.profile.campaign.stored_materials;
    // Twelve ordinary fixture enemies are defeated by the actual held primary
    // input, shared swept occupied-cell collision and once-only XP award path.
    // This demonstrates progression, not natural enemy difficulty acceptance.
    for _ in 0..12 {
        g.move_player(Vec2::new(500.0, 0.0));
        let enemy = g.spawn_enemy(0, false, g.position() - Vec2::Y * 65.0, 1);
        let mut shots = 0;
        for _ in 0..300 {
            let Some(target) = g.world.get::<&Motion>(enemy).ok().map(|m| m.position) else {
                break;
            };
            let aim = (target - g.position()).normalize_or_zero() * 0.081;
            g.tick(
                &InputFrame {
                    joystick: Some(aim.to_array()),
                    fire: true,
                    ..Default::default()
                },
                DT,
            );
            let events = g.take_events();
            assert!(events.iter().all(|e| e.kind != "death"));
            shots += events.iter().filter(|e| e.kind == "shot").count();
        }
        assert!(!g.world.contains(enemy));
        assert!(shots > 0);
    }
    assert_eq!(g.profile.kills, [12, 0, 0]);
    assert_eq!(g.profile.campaign.ship_xp, [60, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(g.hud().ship_level, 3);
    assert!(
        g.hud()
            .upgrades
            .iter()
            .all(|u| u.rank == 5 && u.effective_rank == 2)
    );
    assert_eq!(g.profile.campaign.loadouts[0].hardware, [5; 6]);
    assert!(
        g.profile
            .currency
            .iter()
            .zip(points)
            .all(|(after, before)| *after >= before)
    );
    assert_eq!(g.profile.campaign.stored_materials, stash);
    let second = Stats::from_profile(g.profile());
    assert_eq!(second.hull, 130.0);
    assert_eq!(second.shield, 50.0);
    assert!((second.damage - 14.88).abs() < 0.001);
    assert_eq!(second.fire_rate, 6.0);
    assert_eq!(second.regeneration, 0.8);
    assert!((second.speed - 113.4).abs() < 0.001);
    assert!((second.acceleration - 294.84).abs() < 0.001);
    let reload = Game::new(g.profile.clone());
    assert!(
        reload
            .hud()
            .upgrades
            .iter()
            .all(|u| u.rank == 5 && u.effective_rank == 2)
    );
}

#[test]
fn uncrafted_boss_blueprint_empty_cargo_reload_is_a_true_revision_no_op() {
    let mut p = Profile::default();
    p.campaign.abilities[3].blueprint = true;
    ship_progression::mirrors(&mut p);
    p.revision = 17;
    let g = Game::new(p);
    assert_eq!(g.profile.revision, 17);
    let saved = json(&g);
    let again = Game::new(serde_json::from_value(saved.clone()).unwrap());
    assert_eq!(json(&again), saved);
}

#[test]
fn royal_bundles_are_three_real_equipped_slots_at_level_one_and_clearing_survives_reload() {
    let mut g = Game::new(royal_profile());
    assert_eq!(g.hud().ship_level, 1);
    assert_eq!(g.profile.campaign.ship_xp[9], 0);
    assert_eq!(
        g.profile.campaign.loadouts[9].slots,
        [Some(14), Some(15), Some(16)]
    );
    assert_eq!(
        (0..3)
            .map(|f| bosses::evolution(&g.profile, f))
            .collect::<Vec<_>>(),
        vec![5; 3]
    );
    for category in 0..3 {
        g.equip_ability(category, None).unwrap();
    }
    assert_eq!(
        (0..3)
            .map(|f| bosses::evolution(&g.profile, f))
            .collect::<Vec<_>>(),
        vec![1; 3]
    );
    let saved = json(&g);
    let mut reload = Game::new(serde_json::from_value(saved.clone()).unwrap());
    assert_eq!(json(&reload), saved);
    reload.move_player(Vec2::new(500.0, 0.0));
    reload
        .world
        .get::<&mut Actor>(reload.player)
        .unwrap()
        .health -= 10.0;
    reload.since_damage = 0.0;
    let hull = reload.hud().hull;
    while reload.shot_count < 8 {
        reload.tick(
            &InputFrame {
                fire: true,
                ..Default::default()
            },
            DT,
        );
    }
    assert_eq!(reload.hud().hull, hull); // no unequipped RoyalAegis repair
    assert!(
        reload
            .world
            .query::<&Bullet>()
            .iter()
            .all(|b| b.remaining_hits == 1)
    );
    reload.invulnerability = 0.0;
    reload.hurt_player(1.0, None);
    assert!(reload.repulsor_timer <= 0.0);
    reload
        .world
        .get::<&mut Motion>(reload.player)
        .unwrap()
        .velocity = Vec2::X * 180.0;
    reload.update_wake(reload.position(), DT);
    assert!(reload.wake.is_empty());
}

#[test]
fn unequipping_champion_passive_and_defense_removes_every_hidden_native_effect() {
    let mut p = Profile::default();
    p.campaign.patches = [true; 8];
    p.campaign.blueprints = [true; 8];
    p.campaign.crafted = [true; 8];
    p.campaign.abilities[10] = campaign::AbilityState {
        blueprint: true,
        rank: 1,
    };
    p.campaign.abilities[11] = campaign::AbilityState {
        blueprint: true,
        rank: 1,
    };
    let mut g = Game::new(p);
    g.switch_ship(5).unwrap();
    g.equip_ability(2, Some("windrunner-wake")).unwrap();
    g.wake.push((Vec2::new(500.0, 0.0), 1.0));
    g.equip_ability(2, None).unwrap();
    assert!(g.wake.is_empty());
    g.move_player(Vec2::new(500.0, 0.0));
    g.world.get::<&mut Motion>(g.player).unwrap().velocity = Vec2::X * 200.0;
    g.update_wake(g.position(), DT);
    assert!(g.wake.is_empty());
    g.reset_to_dock();
    g.switch_ship(6).unwrap();
    g.move_player(Vec2::new(500.0, 0.0));
    g.invulnerability = 0.0;
    g.hurt_player(1.0, None);
    assert_eq!(g.repulsor_timer, 0.0);
}

#[test]
fn shared_typescript_migration_fixtures_match_actual_constructor_fields_and_loss() {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../../scripts/test/fixtures/ship-save-migrations.json"
    ))
    .unwrap();
    for case in cases {
        let input: Profile = serde_json::from_value(case["input"].clone()).unwrap();
        let version = input.schema_version;
        let before_revision = input.revision;
        let has_cargo = ship_progression::total(input.campaign.cargo_materials) > 0;
        let g = Game::new(input);
        let mut actual = json(&g);
        let mut expected = case["expected"].clone();
        expected["schema_version"] = 6.into();
        for key in ["cargo_training_materials", "stored_training_materials"] {
            expected["campaign"][key] = serde_json::json!([0, 0, 0]);
        }
        // The older shared schema-4 fixtures omit the two new nullable fields.
        // New schema-5 identity fixtures keep their actual expected values.
        expected
            .as_object_mut()
            .unwrap()
            .entry("identity")
            .or_insert(serde_json::Value::Null);
        expected
            .as_object_mut()
            .unwrap()
            .entry("onboarding")
            .or_insert(serde_json::Value::Null);
        actual.as_object_mut().unwrap().remove("revision");
        expected.as_object_mut().unwrap().remove("revision");
        assert_eq!(actual, expected, "{}", case["name"]);
        assert_eq!(
            g.profile.revision,
            before_revision + u64::from(version < 6 || has_cargo),
            "{}",
            case["name"]
        );
        if !case["loss"].is_null() {
            assert_eq!(
                serde_json::to_value(g.hud().material_loss).unwrap(),
                case["loss"]
            );
        }
        let saved = json(&g);
        let next = Game::new(serde_json::from_value(saved.clone()).unwrap());
        assert_eq!(json(&next), saved, "Second load must be idempotent");
    }
}

fn real_shot_destroys_asteroid(g: &mut Game, e: Entity) {
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
    // Declared high-damage collision fixture, not a player balance claim.
    g.spawn_bullet(
        actor_transform(m, a).point(local),
        Vec2::ZERO,
        1000.0,
        true,
        1,
    );
    g.take_events();
    g.tick(&InputFrame::default(), DT);
    assert!(
        !g.world.contains(e),
        "Real occupied-cell projectile did not defeat asteroid"
    );
    assert!(
        g.take_events().iter().any(|event| event.kind == "explosion"
            && Vec2::new(event.x, event.y).distance(m.position) < 2.0),
        "Chunk removal must not masquerade as an actual projectile defeat"
    );
}

#[test]
fn streamed_belt_birth_survives_physics_drift_and_real_defeats_drop_only_t1_or_points() {
    let mut g = Game::new(Profile::default());
    g.move_player(Vec2::new(1000.0, 0.0));
    g.update_chunks(Vec2::new(1000.0, 0.0));
    let rocks: Vec<_> = g
        .world
        .query::<(Entity, &AsteroidBirth)>()
        .iter()
        .filter(|(_, b)| b.source == AsteroidSource::HavenBelt)
        .map(|(e, _)| e)
        .collect();
    assert!(rocks.len() >= 12);
    let points = g.profile.currency;
    let xp = g.profile.campaign.ship_xp;
    for (i, e) in rocks.into_iter().enumerate() {
        if !g.world.contains(e) {
            continue;
        }
        // Real integration crosses out of its birth annulus before the shot.
        {
            let mut m = g.world.get::<&mut Motion>(e).unwrap();
            m.position = Vec2::new(1399.0, 80.0 + i as f32 * 40.0);
            m.previous = m.position;
            m.velocity = Vec2::X * 1000.0;
        }
        g.integrate(1.0);
        assert!(g.world.get::<&Motion>(e).unwrap().position.length() > COMBAT_RADIUS);
        g.world.get::<&mut Motion>(e).unwrap().velocity = Vec2::ZERO;
        real_shot_destroys_asteroid(&mut g, e);
    }
    assert_eq!(g.profile.campaign.ship_xp, xp);
    assert_eq!(g.profile.currency, points); // pickups are actual separate entities
    let loot: Vec<_> = g
        .world
        .query::<&Loot>()
        .iter()
        .map(|l| (l.category, l.material_tier))
        .collect();
    assert!(!loot.is_empty());
    assert!(loot.iter().any(|(c, _)| *c == 3));
    assert!(loot.iter().any(|(c, _)| *c < 3));
    assert!(loot.iter().all(|(c, t)| *c < 3 || *t == 1));
    assert_eq!(g.profile.kills, [0; 3]);
    assert_eq!(g.profile.boss_generations, [[0; 5]; 3]);
}

#[test]
fn outer_birth_xp_and_tier_survive_drift_back_to_belt_and_qa_stays_reward_isolated() {
    let mut g = Game::new(Profile::default());
    g.move_player(Vec2::new(500.0, 0.0));
    g.update_chunks(Vec2::new(5000.0, 0.0));
    g.move_player(Vec2::new(5000.0, 0.0)); // keep its actual birth chunk loaded
    let e = g
        .world
        .query::<(Entity, &AsteroidBirth)>()
        .iter()
        .find(|(_, b)| b.source != AsteroidSource::HavenBelt && b.material_tier >= 3)
        .map(|(e, _)| e)
        .unwrap();
    let tier = g.world.get::<&AsteroidBirth>(e).unwrap().material_tier;
    {
        let mut m = g.world.get::<&mut Motion>(e).unwrap();
        m.position = Vec2::new(1000.0, 0.0);
        m.previous = m.position;
        m.velocity = Vec2::ZERO;
    }
    real_shot_destroys_asteroid(&mut g, e);
    assert_eq!(g.profile.campaign.ship_xp[0], 2);
    assert!(
        g.world
            .query::<&Loot>()
            .iter()
            .all(|l| l.category < 3 || l.material_tier == tier)
    );
    g.diagnostic("warp 5000 0");
    let before = json(&g);
    g.update_chunks(g.position());
    let e = g
        .world
        .query::<(Entity, (&Actor, &AsteroidBirth))>()
        .iter()
        .next()
        .unwrap()
        .0;
    real_shot_destroys_asteroid(&mut g, e);
    assert_eq!(json(&g), before);
}

#[test]
fn actual_final_league_projectile_awards_royal_three_slots_once_without_injecting_ship_xp() {
    let mut p = Profile::default();
    p.campaign.patches = [true; 8];
    p.campaign.blueprints = [true; 8];
    p.campaign.royale_results = [1, 1, 1, 1, 1, 1, 1, 0];
    let mut g = Game::new(p);
    g.move_player(Vec2::from_array(
        campaign::tournament_site(g.profile.seed).position,
    ));
    g.start_tournament().unwrap();
    let target = g.arena.unwrap().opponent;
    let local = g
        .world
        .get::<&PixelBody>(target)
        .unwrap()
        .cells()
        .min_by(|a, b| a.0.length_squared().total_cmp(&b.0.length_squared()))
        .unwrap()
        .0;
    let motion = *g.world.get::<&Motion>(target).unwrap();
    let actor = *g.world.get::<&Actor>(target).unwrap();
    // Declared high-damage final-round fixture exercises real shared pixel collision.
    g.spawn_bullet(
        actor_transform(motion, actor).point(local),
        Vec2::ZERO,
        10000.0,
        true,
        1,
    );
    g.take_events();
    g.tick(&InputFrame::default(), DT);
    assert!(g.profile.campaign.victory_claimed);
    assert_eq!(
        g.take_events()
            .iter()
            .filter(|e| e.kind == "credits")
            .count(),
        1
    );
    assert_eq!(g.profile.campaign.ship_xp[9], 0);
    assert_eq!(
        g.profile.campaign.loadouts[9].slots,
        [Some(14), Some(15), Some(16)]
    );
    assert_eq!(
        g.profile.campaign.abilities[14..],
        [campaign::AbilityState {
            blueprint: true,
            rank: 5
        }; 3]
    );
    let earned = json(&g);
    g.tick(&InputFrame::default(), DT);
    assert_eq!(json(&g), earned);
    assert!(g.take_events().iter().all(|e| e.kind != "credits"));
    assert!(g.start_tournament().is_err());
    g.reset_to_dock();
    g.switch_ship(9).unwrap();
    assert_eq!(g.hud().ship_level, 1);
}

#[test]
fn royal_repair_and_echo_are_real_but_ordinary_replacement_gets_only_earned_evolution_and_level_cap()
 {
    let mut g = Game::new(royal_profile());
    g.move_player(Vec2::new(500.0, 0.0));
    g.world.get::<&mut Actor>(g.player).unwrap().health -= 10.0;
    g.since_damage = 0.0;
    let hull = g.hud().hull;
    while g.shot_count < 8 {
        g.tick(
            &InputFrame {
                fire: true,
                ..Default::default()
            },
            DT,
        );
    }
    assert_eq!(g.hud().hull, hull + 4.0);
    assert!(
        g.world
            .query::<&Bullet>()
            .iter()
            .any(|b| b.remaining_hits == 3)
    );
    g.reset_to_dock();
    g.profile.campaign.abilities[3] = campaign::AbilityState {
        blueprint: true,
        rank: 5,
    };
    g.equip_ability(0, Some("pulse-echo")).unwrap();
    assert_eq!(ship_progression::family_rank(g.profile(), 0), 1);
    assert_eq!(bosses::evolution(g.profile(), 0), 1);
    g.move_player(Vec2::new(500.0, 0.0));
    while g.shot_count < 8 {
        g.tick(
            &InputFrame {
                fire: true,
                ..Default::default()
            },
            DT,
        );
    }
    assert!(
        g.world
            .query::<&Bullet>()
            .iter()
            .any(|b| b.remaining_hits == 2)
    );
    assert!(
        g.world
            .query::<&Bullet>()
            .iter()
            .all(|b| b.remaining_hits != 3)
    );
    assert_eq!(g.profile.campaign.abilities[3].rank, 5);
}

#[test]
fn full_tier_stash_retains_unbankable_cargo_without_cross_tier_conversion_and_reload_loses_it() {
    let mut g = Game::new(Profile::default());
    g.profile.campaign.stored_materials = [u32::MAX, u32::MAX - 1, 4, 5, 6];
    g.profile.campaign.cargo_materials = [2, 2, 3, 0, 0];
    g.bank_cargo();
    assert_eq!(
        g.profile.campaign.stored_materials,
        [u32::MAX, u32::MAX, 7, 5, 6]
    );
    assert_eq!(g.profile.campaign.cargo_materials, [2, 1, 0, 0, 0]);
    let again = Game::new(g.profile.clone());
    assert_eq!(
        again.profile.campaign.stored_materials,
        g.profile.campaign.stored_materials
    );
    assert_eq!(again.hud().material_loss, [2, 1, 0, 0, 0]);
}

#[test]
fn tiered_cargo_capacity_and_cached_pickup_geometry_are_real_and_bounded() {
    let mut g = Game::new(Profile::default());
    g.move_player(Vec2::new(500.0, 0.0));
    for tier in 1..=5 {
        g.spawn_material_loot(g.position(), tier, 4);
    }
    g.update_pickups(g.position(), DT);
    assert_eq!(
        ship_progression::total(g.profile.campaign.cargo_materials),
        16
    );
    assert_eq!(
        g.world
            .query::<&Loot>()
            .iter()
            .map(|l| l.amount)
            .sum::<u32>(),
        4
    );
    let mut packets = Vec::new();
    for tier in 1..=5 {
        let mut frame = FrameBuilder::new();
        ship_progression::draw_material(&mut frame, Vec2::new(20.0, 30.0), tier);
        let frame = frame.finish();
        assert!((10..=48).contains(&frame.pixel_count));
        assert_eq!(
            frame
                .batches
                .iter()
                .map(|b| b.transforms.len())
                .sum::<usize>(),
            1
        );
        assert!(
            frame
                .batches
                .iter()
                .flat_map(|b| b.transforms.iter())
                .all(|t| t.b[2..] == [0.0, 0.0])
        );
        packets.push(
            frame
                .batches
                .iter()
                .flat_map(|b| b.pixels.iter())
                .map(|p| (p.local, p.color))
                .collect::<Vec<_>>(),
        );
    }
    for a in 0..5 {
        for b in a + 1..5 {
            assert_ne!(packets[a], packets[b]);
        }
    }
}
