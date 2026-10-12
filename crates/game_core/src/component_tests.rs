//! Actual public constructor/dock transactions with declared inventory fixtures.
use super::*;
use crate::identity::create_profile;

fn checkpoint(game: &Game) -> serde_json::Value {
    serde_json::to_value(game.profile()).unwrap()
}
fn stocked() -> Game {
    let mut p = Profile {
        currency: [100; 3],
        ..Profile::default()
    };
    p.campaign.ship_xp[0] = 1320;
    p.campaign.stored_materials = [100; 5];
    for state in &mut p.campaign.abilities[..3] {
        *state = campaign::AbilityState {
            blueprint: true,
            rank: 2,
        };
    }
    Game::new(p)
}
fn unchanged(game: &mut Game, action: impl FnOnce(&mut Game) -> Result<(), String>) -> String {
    game.take_events();
    let before = checkpoint(game);
    let error = action(game).unwrap_err();
    assert_eq!(checkpoint(game), before);
    assert!(game.take_events().is_empty());
    error
}

#[test]
fn finite_catalog_category_costs_and_requirements_are_exact() {
    assert_eq!(components::COUNT, 3);
    for (i, def) in components::COMPONENTS.iter().enumerate() {
        assert_eq!(def.category, i);
        assert_eq!(components::by_key(def.id), Some(i));
        assert_eq!(def.materials, [2, 1, 0, 0, 0]);
        assert_eq!(def.required_level, 3);
    }
    assert_eq!(components::by_key("unknown"), None);
    for id in 0..14 {
        for rank in 0..=6 {
            assert_eq!(
                components::requirement(id, rank),
                if (3..=5).contains(&rank) {
                    Some((abilities::get(id).unwrap().category, u32::from(rank - 2)))
                } else {
                    None
                }
            );
        }
    }
    for id in [14, 15, 16, 17, u8::MAX] {
        assert_eq!(components::requirement(id, 3), None);
    }
}

#[test]
fn schema7_import_requires_exact_bounded_counts_and_never_accepts_corrupt_stacks() {
    let base = checkpoint(&stocked());
    let mut missing = base.clone();
    missing["campaign"]
        .as_object_mut()
        .unwrap()
        .remove("components");
    assert!(serde_json::from_value::<Profile>(missing).is_err());
    for bad in [
        serde_json::json!(null),
        serde_json::json!([]),
        serde_json::json!([0, 0]),
        serde_json::json!([0, 0, 0, 0]),
        serde_json::json!([-1, 0, 0]),
        serde_json::json!([0.5, 0, 0]),
        serde_json::json!([100, 0, 0]),
        serde_json::json!([4294967296u64, 0, 0]),
        serde_json::json!(["1", 0, 0]),
        serde_json::json!({"0":1}),
    ] {
        let mut value = base.clone();
        value["campaign"]["components"] = bad;
        assert!(serde_json::from_value::<Profile>(value).is_err());
    }
    let mut future = base.clone();
    future["schema_version"] = 8.into();
    assert!(serde_json::from_value::<Profile>(future).is_err());
    let mut valid = base;
    valid["campaign"]["components"] = serde_json::json!([99, 7, 1]);
    let profile: Profile = serde_json::from_value(valid.clone()).unwrap();
    assert_eq!(serde_json::to_value(&profile).unwrap(), valid);
    let reload = Game::new(profile);
    assert_eq!(checkpoint(&reload), valid);
    assert_eq!(checkpoint(&Game::new(reload.profile().clone())), valid);
}

#[test]
fn schema6_migration_ignores_forged_stacks_and_preserves_every_prior_asset() {
    let current = checkpoint(&stocked());
    for bad in [
        serde_json::json!([99, 99, 99]),
        serde_json::json!("forged"),
        serde_json::json!([1]),
    ] {
        let mut old = current.clone();
        old["schema_version"] = 6.into();
        old["campaign"]["components"] = bad;
        let parsed: Profile = serde_json::from_value(old).unwrap();
        assert_eq!(parsed.campaign.components, [0; 3]);
        let mut expected = current.clone();
        expected["revision"] = (current["revision"].as_u64().unwrap() + 1).into();
        assert_eq!(checkpoint(&Game::new(parsed)), expected);
    }
    let mut typed = stocked().profile().clone();
    typed.schema_version = 6;
    typed.campaign.components = [99; 3];
    assert_eq!(Game::new(typed).profile().campaign.components, [0; 3]);
}

#[test]
fn all_profile_fields_roundtrip_with_live_cargo_but_reload_discards_only_cargo() {
    let mut game = stocked();
    game.profile.identity = Some(identity::KikaiIdentity {
        kind: identity::KikaiKind::Dora,
        corporation: identity::Corporation::Pan,
        sub_corporation: identity::Corporation::Renko,
        hex_id: "0123456789abcdef".into(),
    });
    game.profile.campaign.components = [3, 2, 1];
    game.profile.campaign.cargo_materials = [6, 2, 0, 0, 0];
    game.profile.campaign.cargo_training_materials = [1, 2, 3];
    game.profile.campaign.stored_training_materials = [2, 3, 4];
    game.profile.discoveries = vec!["0:0".into()];
    game.profile.quests = vec![QuestProgress {
        site_id: "0:0".into(),
        event: "courier".into(),
        family: 0,
        target: 1,
        progress: 0,
        completed: false,
    }];
    game.profile.relics = vec!["probe".into()];
    let before = checkpoint(&game);
    let decoded: Profile = serde_json::from_value(before.clone()).unwrap();
    assert_eq!(serde_json::to_value(&decoded).unwrap(), before);
    let reload = Game::new(decoded);
    assert_eq!(reload.profile.campaign.components, [3, 2, 1]);
    assert_eq!(reload.profile.campaign.cargo_materials, [0; 5]);
    assert_eq!(reload.profile.campaign.cargo_training_materials, [0; 3]);
    assert_eq!(reload.profile.campaign.stored_training_materials, [2, 3, 4]);
    assert_eq!(reload.hud().material_loss, [6, 2, 0, 0, 0]);
}

#[test]
fn recipes_are_hidden_until_real_category_inventory_or_blueprint_discovery() {
    let mut game = Game::new(create_profile("dora", "pan", "renko", "0123456789abcdef").unwrap());
    assert!(game.hud().components.is_empty());
    game.profile.campaign.abilities[4].blueprint = true;
    let hud = game.hud().components;
    assert_eq!(hud.len(), 1);
    assert_eq!(hud[0].id, "aegis-lattice");
    assert_eq!(hud[0].count, 0);
    assert!(!hud[0].can_craft);
    assert!(hud[0].craft_reason.contains("level 3"));
    game.profile.campaign.abilities[0].rank = 2;
    game.profile.campaign.components[2] = 1;
    assert_eq!(game.hud().components.len(), 3);
    let mut royal = Profile::default();
    royal.campaign.patches = [true; 8];
    royal.campaign.royale_results = [1; 8];
    royal.campaign.victory_claimed = true;
    ship_progression::grant_royal(&mut royal);
    // The royalty entries by themselves do not reveal ordinary part recipes.
    royal.campaign.abilities[..14].fill(campaign::AbilityState::default());
    assert!(!components::discovered(&royal, 0));
    assert!(!components::discovered(&royal, 1));
    assert!(!components::discovered(&royal, 2));
}

#[test]
fn component_failures_are_atomic_for_unknown_discovery_level_materials_location_and_full_stack() {
    let mut game = stocked();
    unchanged(&mut game, |g| g.craft_component("unknown"));
    game.profile.campaign.abilities[0].rank = 1;
    unchanged(&mut game, |g| g.craft_component("pulse-conductor"));
    game.profile.campaign.abilities[0].rank = 2;
    game.profile.campaign.ship_xp[0] = 0;
    unchanged(&mut game, |g| g.craft_component("pulse-conductor"));
    game.profile.campaign.ship_xp[0] = 60;
    for tier in 0..2 {
        let previous = game.profile.campaign.stored_materials[tier];
        game.profile.campaign.stored_materials[tier] = 0;
        game.profile.campaign.cargo_materials = [2, 1, 0, 0, 0];
        unchanged(&mut game, |g| g.craft_component("pulse-conductor"));
        game.profile.campaign.stored_materials[tier] = previous;
    }
    game.move_player(Vec2::new(500.0, 0.0));
    unchanged(&mut game, |g| g.craft_component("pulse-conductor"));
    game.move_player(Vec2::ZERO);
    game.profile.campaign.components[0] = 99;
    unchanged(&mut game, |g| g.craft_component("pulse-conductor"));
}

#[test]
fn actual_craft_spends_banked_stock_only_and_never_double_spends_named_t1() {
    let mut game = stocked();
    game.profile.campaign.stored_materials[0] = 7;
    game.profile.campaign.stored_training_materials = [2, 2, 2];
    game.profile.campaign.cargo_materials = [2, 1, 0, 0, 0];
    let currency = game.profile.currency;
    let xp = game.profile.campaign.ship_xp;
    let revision = game.profile.revision;
    for def in components::COMPONENTS {
        game.craft_component(def.id).unwrap();
    }
    assert_eq!(game.profile.campaign.components, [1; 3]);
    assert_eq!(
        game.profile.campaign.stored_materials,
        [1, 97, 100, 100, 100]
    );
    assert_eq!(game.profile.campaign.stored_training_materials, [0, 0, 1]);
    assert_eq!(game.profile.campaign.cargo_materials, [2, 1, 0, 0, 0]);
    assert_eq!(game.profile.currency, currency);
    assert_eq!(game.profile.campaign.ship_xp, xp);
    assert_eq!(game.profile.revision, revision + 3);
    assert_eq!(
        game.take_events()
            .iter()
            .filter(|e| e.kind == "upgrade")
            .count(),
        3
    );
}

#[test]
fn final_stack_unit_is_admitted_once_and_repeat_capacity_failures_cost_nothing() {
    let mut game = stocked();
    game.profile.campaign.components[0] = 98;
    game.craft_component("pulse-conductor").unwrap();
    assert_eq!(game.profile.campaign.components[0], 99);
    for _ in 0..8 {
        unchanged(&mut game, |g| g.craft_component("pulse-conductor"));
    }
}

#[test]
fn advanced_upgrade_failure_and_success_debit_points_exact_tier_and_parts_atomically() {
    let mut game = stocked();
    assert_eq!(
        game.hud().abilities[0]
            .required_component
            .as_ref()
            .unwrap()
            .amount,
        1
    );
    unchanged(&mut game, |g| g.upgrade_ability("pulse-lens"));
    game.craft_component("pulse-conductor").unwrap();
    let points = game.profile.currency;
    let materials = game.profile.campaign.stored_materials;
    game.upgrade_ability("pulse-lens").unwrap();
    assert_eq!(game.profile.campaign.abilities[0].rank, 3);
    assert_eq!(
        game.profile.currency,
        [points[0] - 10, points[1], points[2]]
    );
    assert_eq!(
        game.profile.campaign.stored_materials,
        [
            materials[0],
            materials[1],
            materials[2] - 2,
            materials[3],
            materials[4]
        ]
    );
    assert_eq!(game.profile.campaign.components, [0; 3]);
    assert_eq!(
        game.hud().abilities[0]
            .required_component
            .as_ref()
            .unwrap()
            .amount,
        2
    );
    unchanged(&mut game, |g| g.upgrade_ability("pulse-lens"));
}

#[test]
fn all_ordinary_advanced_ranks_consume_their_matching_part_and_keep_slot_capacity() {
    let mut game = stocked();
    for category in 0..3 {
        let key = abilities::ABILITIES[category].key;
        for _ in 0..6 {
            game.craft_component(components::COMPONENTS[category].id)
                .unwrap();
        }
        for rank in 3..=5 {
            game.upgrade_ability(key).unwrap();
            assert_eq!(game.profile.campaign.abilities[category].rank, rank);
        }
        assert_eq!(game.profile.campaign.components[category], 0);
        assert!(game.hud().abilities[category].required_component.is_none());
        unchanged(&mut game, |g| g.upgrade_ability(key));
        game.equip_ability(category, Some(key)).unwrap();
    }
    assert_eq!(
        game.profile.campaign.loadouts[0].slots,
        [Some(0), Some(1), Some(2)]
    );
    assert_eq!(game.profile.currency, [70; 3]);
    assert_eq!(game.profile.campaign.stored_materials, [64, 82, 94, 94, 94]);
}

#[test]
fn level_compatibility_points_and_rank_tier_failures_never_consume_parts() {
    let mut game = stocked();
    game.profile.campaign.components = [3; 3];
    game.profile.campaign.ship_xp[0] = 60;
    unchanged(&mut game, |g| g.upgrade_ability("pulse-lens"));
    game.profile.campaign.ship_xp[0] = 200;
    game.profile.currency[0] = 9;
    unchanged(&mut game, |g| g.upgrade_ability("pulse-lens"));
    game.profile.currency[0] = 10;
    game.profile.campaign.stored_materials[2] = 1;
    unchanged(&mut game, |g| g.upgrade_ability("pulse-lens"));
    game.profile.campaign.abilities[7] = campaign::AbilityState {
        blueprint: true,
        rank: 2,
    };
    unchanged(&mut game, |g| g.upgrade_ability("broadside"));
    // A high-level owned hull cannot lend its gate to a level-one selected hull.
    game.profile.campaign.patches[0] = true;
    game.profile.campaign.crafted[0] = true;
    game.switch_ship(1).unwrap();
    unchanged(&mut game, |g| g.craft_component("pulse-conductor"));
}

#[test]
fn first_craft_and_rank2_upgrade_remain_component_free() {
    let mut game = stocked();
    game.profile.campaign.abilities[0].rank = 0;
    game.profile.campaign.ship_xp[0] = 60;
    game.craft_ability("pulse-lens").unwrap();
    assert!(game.hud().abilities[0].required_component.is_none());
    game.upgrade_ability("pulse-lens").unwrap();
    assert_eq!(game.profile.campaign.abilities[0].rank, 2);
    assert_eq!(game.profile.campaign.components, [0; 3]);
}

#[test]
fn actual_corporate_training_first_upgrade_keeps_its_named_recipe_and_level1_exception() {
    for corp in ["pan", "renko", "hugo"] {
        let mut p = create_profile("talo", corp, "renko", "0123456789abcdef").unwrap();
        let material = training::material_for_corporation(p.identity.as_ref().unwrap().corporation);
        p.onboarding = Some(identity::OnboardingProgress {
            version: 2,
            stage: 5,
            flags: 31,
            asteroids_destroyed: 10,
            materials_collected: 10,
            legacy_complete: false,
        });
        p.campaign.stored_materials[0] = 10;
        p.campaign.stored_training_materials[material.index] = 10;
        let mut game = Game::new(p);
        let id = game
            .profile
            .identity
            .as_ref()
            .unwrap()
            .corporation
            .ability();
        let key = abilities::get(id).unwrap().key;
        let row = game
            .hud()
            .abilities
            .into_iter()
            .find(|row| row.index == id)
            .unwrap();
        assert!(row.training_recipe && row.required_component.is_none());
        assert_eq!(row.point_cost, 0);
        game.upgrade_ability(key).unwrap();
        assert_eq!(game.profile.campaign.components, [0; 3]);
        assert_eq!(game.profile.campaign.stored_materials[0], 0);
        assert_eq!(
            game.profile.campaign.stored_training_materials[material.index],
            0
        );
        assert_eq!(
            game.hud().loadout[abilities::get(id).unwrap().category]
                .as_ref()
                .unwrap()
                .effective_rank,
            2
        );
        assert_eq!(game.hud().onboarding.unwrap().stage, 6);
    }
}

#[test]
fn components_survive_real_death_abandonment_and_reload_without_becoming_cargo() {
    for mode in ["death", "abandonment", "reload"] {
        let mut game = stocked();
        game.profile.campaign.components = [3, 2, 1];
        game.profile.campaign.cargo_materials = [1, 2, 0, 0, 0];
        let points = game.profile.currency;
        let xp = game.profile.campaign.ship_xp;
        let stash = game.profile.campaign.stored_materials;
        game.move_player(Vec2::new(500.0, 0.0));
        game.invulnerability = 0.0;
        game.shield = 0.0;
        match mode {
            "death" => {
                game.hurt_player(10000.0, None);
                game.tick(&InputFrame::default(), 1.0 / 60.0);
            }
            "abandonment" => game.reset_to_dock(),
            _ => game = Game::new(serde_json::from_value(checkpoint(&game)).unwrap()),
        }
        assert_eq!(game.profile.campaign.components, [3, 2, 1]);
        assert_eq!(game.profile.campaign.cargo_materials, [0; 5]);
        assert_eq!(game.profile.currency, points);
        assert_eq!(game.profile.campaign.ship_xp, xp);
        assert_eq!(game.profile.campaign.stored_materials, stash);
    }
}

#[test]
fn public_diagnostic_warp_cannot_craft_or_spend_parts_and_reset_restores_real_dock_actions() {
    let mut game = stocked();
    game.diagnostic("warp 0 0");
    assert!(game.diagnostic_scene);
    unchanged(&mut game, |g| g.craft_component("pulse-conductor"));
    game.profile.campaign.components[0] = 1;
    unchanged(&mut game, |g| g.upgrade_ability("pulse-lens"));
    game.reset_to_dock();
    game.craft_component("pulse-conductor").unwrap();
    game.upgrade_ability("pulse-lens").unwrap();
    assert_eq!(game.profile.campaign.components[0], 1);
}

#[test]
fn existing_components_prevent_reissuing_a_pristine_license() {
    let mut p = Profile::default();
    p.campaign.components[0] = 1;
    assert!(!Game::new(p).can_register_droid());
}

#[test]
fn actual_hud_serializer_exposes_bounded_recipe_eligibility_and_owned_part_requirements() {
    let mut game = stocked();
    game.craft_component("pulse-conductor").unwrap();
    let hud = serde_json::to_value(game.hud()).unwrap();
    let row = &hud["components"][0];
    assert_eq!(row["id"], "pulse-conductor");
    assert_eq!(row["index"], 0);
    assert_eq!(row["category"], 0);
    assert_eq!(row["count"], 1);
    assert_eq!(row["max_count"], 99);
    assert_eq!(row["required_level"], 3);
    assert_eq!(row["materials"], serde_json::json!([2, 1, 0, 0, 0]));
    assert_eq!(row["can_craft"], true);
    assert_eq!(row["craft_reason"], "");
    assert_eq!(
        hud["abilities"][0]["required_component"],
        serde_json::json!({"id":"pulse-conductor", "name":"Pulse Conductor", "amount":1, "owned":1})
    );
    game.profile.campaign.ship_xp[0] = 0;
    let hud = serde_json::to_value(game.hud()).unwrap();
    assert_eq!(hud["components"][0]["can_craft"], false);
    assert!(
        hud["components"][0]["craft_reason"]
            .as_str()
            .unwrap()
            .contains("level 3")
    );
    assert_eq!(hud["abilities"][0]["can_upgrade"], false);
    assert_eq!(hud["abilities"][0]["required_component"]["owned"], 1);
}
