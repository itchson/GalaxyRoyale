//! Declared profile fixtures use the real constructor, transaction, occupied-cell
//! projectile and tick paths. They are not natural difficulty/playtest evidence.
use crate::*;

const DT: f32 = 1.0 / 60.0;

fn close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.0002, "{actual} != {expected}");
}

fn licensed(main: &str, sub: &str) -> Profile {
    identity::create_profile("talo", main, sub, "0123456789abcdef").unwrap()
}

fn checkpoint(game: &Game) -> serde_json::Value {
    serde_json::to_value(game.profile()).unwrap()
}

fn quiet_departure(game: &mut Game) {
    // Declared outside-Haven/inside-belt-inner fixture has no ordinary waves.
    game.move_player(Vec2::new(300.0, 0.0));
    game.tick(&InputFrame::default(), DT);
    game.take_events();
    game.invulnerability = 0.0;
    assert!(!game.safe());
}

fn real_hit(game: &mut Game, damage: f32) -> Vec<GameEvent> {
    let local = game
        .world
        .get::<&PixelBody>(game.player)
        .unwrap()
        .cells()
        .min_by(|a, b| a.0.length_squared().total_cmp(&b.0.length_squared()))
        .unwrap()
        .0;
    let motion = *game.world.get::<&Motion>(game.player).unwrap();
    let actor = *game.world.get::<&Actor>(game.player).unwrap();
    let world = actor_transform(motion, actor).point(local);
    game.spawn_bullet(world, Vec2::ZERO, damage, false, 1);
    game.tick(&InputFrame::default(), DT);
    game.take_events()
}

#[test]
fn fresh_chassis_has_three_base_chunks_and_only_actual_factory_capacity() {
    let anonymous = Game::new(Profile::default());
    close(anonymous.hud().max_shield, 30.0);
    assert!(!anonymous.hud().ship_stats.shield_recharge_unlocked);
    for main in ["pan", "renko", "hugo"] {
        let game = Game::new(licensed(main, "renko"));
        let hud = game.hud();
        close(hud.max_shield, if main == "hugo" { 50.0 } else { 30.0 });
        close(hud.shield, hud.max_shield);
        close(hud.ship_stats.shield_regen, 0.0);
        assert!(!hud.ship_stats.shield_recharge_unlocked);
        assert_eq!(hud.ship_level, 1);
        assert_eq!(hud.loadout.iter().flatten().count(), 1);
    }
}

#[test]
fn real_damage_does_not_recharge_without_a_purchased_unlock() {
    for main in ["pan", "renko", "hugo"] {
        let mut game = Game::new(licensed(main, "pan"));
        quiet_departure(&mut game);
        let before = game.hud();
        let events = real_hit(&mut game, 10.0);
        assert_eq!(events.iter().filter(|e| e.kind == "shield").count(), 1);
        let expected = before.shield - if main == "hugo" { 9.0 } else { 10.0 };
        close(game.hud().shield, expected);
        close(game.hud().hull, before.hull);
        for _ in 0..720 {
            game.tick(&InputFrame::default(), DT);
            assert!(game.take_events().iter().all(|e| e.kind != "death"));
        }
        close(game.hud().shield, expected);
        assert_eq!(game.profile.currency, [0; 3]);
        assert_eq!(game.profile.campaign.ship_xp, [0; 10]);
    }
}

#[test]
fn actual_aegis_purchase_unlocks_delayed_rate_with_atomic_existing_recipe() {
    let mut profile = licensed("renko", "hugo");
    profile.currency[1] = 10;
    profile.campaign.stored_materials[0] = 1;
    let mut game = Game::new(profile);
    game.purchase("shield").unwrap();
    assert_eq!(game.profile.currency[1], 0);
    assert_eq!(game.profile.campaign.stored_materials[0], 0);
    assert_eq!(game.profile.campaign.loadouts[0].hardware[3], 1);
    assert!(game.hud().ship_stats.shield_recharge_unlocked);
    close(game.hud().max_shield, 40.0);
    close(game.hud().ship_stats.shield_regen, 3.0);
    quiet_departure(&mut game);
    real_hit(&mut game, 12.0);
    let depleted = game.hud().shield;
    while game.since_damage + DT < 4.0 {
        game.tick(&InputFrame::default(), DT);
        game.take_events();
        close(game.hud().shield, depleted);
    }
    while game.since_damage < 4.0 {
        game.tick(&InputFrame::default(), DT);
        game.take_events();
    }
    close(game.hud().shield, depleted + 3.0 * DT);
    for _ in 0..300 {
        game.tick(&InputFrame::default(), DT);
        game.take_events();
    }
    close(game.hud().shield, game.hud().max_shield);
}

#[test]
fn stored_ranks_remain_owned_and_actual_xp_gates_both_capacity_and_recharge() {
    // Actual HUD rows preserve ownedR5; level gates cap active power.
    let cases = [
        (0, 1, 40.0, 3.0),
        (60, 3, 50.0, 3.816),
        (200, 5, 60.0, 4.704),
        (560, 8, 70.0, 5.808),
        (1320, 12, 80.0, 7.182),
        (99000, 100, 80.0, 7.182),
    ];
    for (xp, level, capacity, rate) in cases {
        let mut profile = Profile::default();
        profile.campaign.loadouts[0].hardware[3] = 5;
        profile.campaign.ship_xp[0] = xp;
        let game = Game::new(profile);
        let hud = game.hud();
        assert_eq!(hud.ship_level, level);
        assert_eq!(game.profile.campaign.loadouts[0].hardware[3], 5);
        close(hud.max_shield, capacity);
        close(hud.ship_stats.shield_regen, rate);
        assert!(hud.ship_stats.shield_recharge_unlocked);
        let serialized = serde_json::to_string(game.profile()).unwrap();
        let reload = Game::new(serde_json::from_str(&serialized).unwrap());
        assert_eq!(checkpoint(&reload), checkpoint(&game));
        close(reload.hud().ship_stats.shield_regen, rate);
    }
}

fn trained_hugo() -> Profile {
    let mut profile = licensed("hugo", "pan");
    profile.campaign.abilities[1].rank = 2;
    profile.onboarding = Some(identity::OnboardingProgress {
        version: 2,
        stage: 6,
        flags: 127,
        asteroids_destroyed: 10,
        materials_collected: 10,
        legacy_complete: false,
    });
    profile
}

#[test]
fn genuine_training_rank_two_unlock_is_scoped_to_equipped_factory_hull() {
    let mut profile = trained_hugo();
    profile.campaign.patches[0] = true;
    profile.campaign.blueprints[0] = true;
    profile.campaign.crafted[0] = true;
    profile.campaign.loadouts[1].slots[1] = Some(1);
    let mut game = Game::new(profile);
    close(game.hud().max_shield, 58.0);
    close(game.hud().ship_stats.shield_regen, 3.0);
    game.switch_ship(1).unwrap();
    close(game.hud().max_shield, 50.0);
    close(game.hud().ship_stats.shield_regen, 0.0);
    assert_eq!(ship_progression::equipped_rank(game.profile(), 1), 1);
    game.switch_ship(0).unwrap();
    assert_eq!(ship_progression::equipped_rank(game.profile(), 1), 2);
    close(game.hud().ship_stats.shield_regen, 3.0);
    game.equip_ability(1, None).unwrap();
    close(game.hud().max_shield, 30.0);
    assert!(!game.hud().ship_stats.shield_recharge_unlocked);
    close(super::incoming_damage_multiplier(game.profile()), 1.0);
    let reload = Game::new(game.profile.clone());
    assert_eq!(reload.profile.campaign.abilities[1].rank, 2);
    assert_eq!(reload.profile.campaign.loadouts[0].slots[1], None);
    assert!(!reload.hud().ship_stats.shield_recharge_unlocked);
}

#[test]
fn factory_guard_first_upgrade_really_unlocks_recharge_without_fabricated_xp() {
    let mut profile = licensed("hugo", "renko");
    profile.onboarding = Some(identity::OnboardingProgress {
        version: 2,
        stage: 5,
        flags: 31,
        asteroids_destroyed: 10,
        materials_collected: 10,
        legacy_complete: false,
    });
    profile.campaign.stored_materials[0] = 10;
    profile.campaign.stored_training_materials[2] = 10;
    let mut game = Game::new(profile);
    assert!(!game.hud().ship_stats.shield_recharge_unlocked);
    game.upgrade_ability("guard-coil").unwrap();
    assert_eq!(game.profile.campaign.stored_materials[0], 0);
    assert_eq!(game.profile.campaign.stored_training_materials[2], 0);
    assert_eq!(game.profile.campaign.ship_xp, [0; 10]);
    close(game.hud().max_shield, 58.0);
    close(game.hud().ship_stats.shield_regen, 3.0);
    assert!(game.hud().ship_stats.shield_recharge_unlocked);
    let before = checkpoint(&game);
    assert!(game.upgrade_ability("guard-coil").is_err());
    assert_eq!(checkpoint(&game), before);
}

#[test]
fn canonical_main_bonus_is_equipped_and_independent_of_secondary_manufacturer() {
    for main in ["pan", "renko", "hugo"] {
        for sub in ["pan", "renko", "hugo"] {
            let mut game = Game::new(licensed(main, sub));
            let before = game.profile.campaign.clone();
            let stats = Stats::from_profile(game.profile());
            close(stats.speed, if main == "pan" { 114.4 } else { 100.0 });
            close(
                stats.acceleration,
                if main == "pan" { 297.44 } else { 260.0 },
            );
            close(stats.damage, if main == "renko" { 14.52 } else { 12.0 });
            close(
                super::incoming_damage_multiplier(game.profile()),
                if main == "hugo" { 0.9 } else { 1.0 },
            );
            assert_eq!(
                serde_json::to_value(&game.profile.campaign).unwrap(),
                serde_json::to_value(&before).unwrap()
            );
            let category = abilities::get(
                game.profile
                    .identity
                    .as_ref()
                    .unwrap()
                    .corporation
                    .ability(),
            )
            .unwrap()
            .category;
            game.equip_ability(category, None).unwrap();
            let empty = Stats::from_profile(game.profile());
            close(empty.speed, 100.0);
            close(empty.acceleration, 260.0);
            close(empty.damage, 12.0);
            close(super::incoming_damage_multiplier(game.profile()), 1.0);
            assert_eq!(game.profile.campaign.ship_xp, [0; 10]);
        }
    }
}

#[test]
fn actual_hugo_projectile_damage_reduces_shield_and_hull_with_no_phantom_defense() {
    for equipped in [true, false] {
        let mut game = Game::new(licensed("hugo", "pan"));
        if !equipped {
            game.equip_ability(1, None).unwrap();
        }
        quiet_departure(&mut game);
        game.shield = 3.0; // Declared already-depleted real shield fixture.
        let body_cells = game
            .world
            .get::<&PixelBody>(game.player)
            .unwrap()
            .occupied_count();
        let events = real_hit(&mut game, 10.0);
        close(game.hud().shield, 0.0);
        close(game.hud().hull, if equipped { 94.0 } else { 93.0 });
        assert_eq!(events.iter().filter(|e| e.kind == "shield").count(), 1);
        assert!(
            game.world
                .get::<&PixelBody>(game.player)
                .unwrap()
                .occupied_count()
                < body_cells
        );
        assert_eq!(game.profile.currency, [0; 3]);
        assert_eq!(game.profile.campaign.ship_xp, [0; 10]);
    }
}

#[test]
fn actual_renko_primary_and_champion_volley_share_the_weapon_bonus() {
    for main in ["renko", "pan"] {
        let mut profile = licensed(main, "hugo");
        profile.campaign.patches[0] = true;
        profile.campaign.blueprints[0] = true;
        profile.campaign.abilities[6] = campaign::AbilityState {
            blueprint: true,
            rank: 1,
        };
        profile.campaign.loadouts[0].slots[0] = Some(6);
        let mut game = Game::new(profile);
        quiet_departure(&mut game);
        let mut shots = 0;
        for _ in 0..90 {
            game.tick(
                &InputFrame {
                    fire: true,
                    ..Default::default()
                },
                DT,
            );
            shots += game
                .take_events()
                .iter()
                .filter(|e| e.kind == "shot")
                .count();
            if shots == 4 {
                break;
            }
        }
        assert_eq!(shots, 4);
        let multiplier = if main == "renko" { 1.1 } else { 1.0 };
        let mut primary = 0;
        let mut signature = 0;
        for (motion, bullet) in game.world.query::<(&Motion, &Bullet)>().iter() {
            if (motion.velocity.length() - 520.0).abs() < 0.01 {
                close(bullet.damage, 12.0 * multiplier * 0.65);
                signature += 1;
            } else if (motion.velocity.length() - 430.0).abs() < 0.01 {
                close(bullet.damage, 12.0 * multiplier);
                primary += 1;
            }
        }
        assert_eq!(primary, 4);
        assert_eq!(signature, 2);
    }
}

#[test]
fn actual_repeated_hits_reset_the_delay_and_locked_shield_still_repairs_at_haven() {
    let mut profile = Profile::default();
    profile.campaign.loadouts[0].hardware[3] = 1;
    let mut game = Game::new(profile);
    quiet_departure(&mut game);
    real_hit(&mut game, 10.0);
    for _ in 0..239 {
        game.tick(&InputFrame::default(), DT);
        game.take_events();
    }
    close(game.hud().shield, 30.0);
    real_hit(&mut game, 10.0);
    close(game.since_damage, 0.0);
    for _ in 0..239 {
        game.tick(&InputFrame::default(), DT);
        game.take_events();
    }
    close(game.hud().shield, 20.0);
    game.move_player(Vec2::ZERO);
    game.tick(&InputFrame::default(), DT);
    close(game.hud().shield, 40.0);
    let mut locked = Game::new(Profile::default());
    quiet_departure(&mut locked);
    real_hit(&mut locked, 10.0);
    locked.move_player(Vec2::ZERO);
    locked.tick(&InputFrame::default(), DT);
    close(locked.hud().shield, 30.0);
    assert!(!locked.hud().ship_stats.shield_recharge_unlocked);
}

#[test]
fn hull_repair_remains_independent_after_five_seconds() {
    let mut profile = Profile::default();
    profile.campaign.loadouts[0].hardware[4] = 1;
    let mut game = Game::new(profile);
    quiet_departure(&mut game);
    real_hit(&mut game, 40.0);
    close(game.hud().hull, 90.0);
    close(game.hud().shield, 0.0);
    close(game.hud().ship_stats.hull_regen, 0.4);
    while game.since_damage + DT < 5.0 {
        game.tick(&InputFrame::default(), DT);
        game.take_events();
        close(game.hud().hull, 90.0);
    }
    while game.since_damage < 5.0 {
        game.tick(&InputFrame::default(), DT);
        game.take_events();
    }
    assert!(game.hud().hull > 90.0);
    close(game.hud().shield, 0.0);
}

#[test]
fn enhanced_sovereign_is_explicit_and_all_valid_capacity_is_bounded() {
    let mut p = Profile::default();
    p.campaign.patches = [true; 8];
    p.campaign.blueprints = [true; 8];
    p.campaign.royale_results = [1; 8];
    p.campaign.victory_claimed = true;
    p.campaign.selected_ship = 9;
    ship_progression::grant_royal(&mut p);
    let mut game = Game::new(p);
    assert_eq!(game.hud().ship_level, 1);
    close(game.hud().max_shield, 120.0);
    close(game.hud().ship_stats.shield_regen, 18.0);
    assert_eq!(game.profile.campaign.ship_xp, [0; 10]);
    game.profile.campaign.abilities[1] = campaign::AbilityState {
        blueprint: true,
        rank: 5,
    };
    game.profile.campaign.ship_xp[9] = 1320;
    game.equip_ability(1, Some("guard-coil")).unwrap();
    close(game.hud().max_shield, 172.0);
    close(game.hud().ship_stats.shield_regen, 18.0);
    game.switch_ship(0).unwrap();
    assert!(!game.hud().ship_stats.shield_recharge_unlocked);
    close(game.hud().max_shield, 30.0);
    assert_eq!(game.profile.campaign.abilities[15].rank, 5);
}

#[test]
fn all_ten_owned_maximum_rank_hulls_fit_the_authoritative_eighteen_chunk_bound() {
    for ship in 0..10 {
        let mut profile = Profile::default();
        profile.campaign.patches = [true; 8];
        profile.campaign.blueprints = [true; 8];
        profile.campaign.crafted = [true; 8];
        profile.campaign.royale_results = [1; 8];
        profile.campaign.victory_claimed = true;
        profile.campaign.selected_ship = ship;
        profile.campaign.ship_xp = [1320; 10];
        profile.campaign.abilities[1] = campaign::AbilityState {
            blueprint: true,
            rank: 5,
        };
        profile.campaign.loadouts[ship as usize].hardware[3] = 5;
        profile.campaign.loadouts[ship as usize].slots[1] = Some(1);
        let game = Game::new(profile);
        let maximum = game.hud().max_shield;
        close(maximum, if ship == 9 { 172.0 } else { 132.0 });
        assert!((maximum / super::SHIELD_CHUNK_HP).ceil() <= 18.0);
    }
}

#[test]
fn unavailable_or_wrong_category_items_cannot_create_a_corporate_bonus() {
    for id in [1, 2, 11] {
        let mut profile = licensed("hugo", "pan");
        profile.campaign.abilities[1].rank = 0;
        if id != 1 {
            profile.campaign.abilities[id] = campaign::AbilityState {
                blueprint: true,
                rank: 1,
            };
        }
        if id == 11 {
            profile.campaign.patches[5] = true;
            profile.campaign.blueprints[5] = true;
        }
        profile.campaign.loadouts[0].slots[1] = Some(id as u8);
        let game = Game::new(profile);
        assert_eq!(game.profile.campaign.loadouts[0].slots[1], None);
        close(super::incoming_damage_multiplier(game.profile()), 1.0);
        close(game.hud().max_shield, 30.0);
        assert!(!game.hud().ship_stats.shield_recharge_unlocked);
    }
}
