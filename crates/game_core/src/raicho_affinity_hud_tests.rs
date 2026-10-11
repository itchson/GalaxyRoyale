//! Actual public HUD/serialization and transaction checks on declared owned
//! hulls. These fixtures are not natural discovery or balance acceptance.
use crate::*;
use serde_json::{Value, json};

fn checkpoint(game: &Game) -> Value {
    serde_json::to_value(game.profile()).unwrap()
}

fn licensed(main: &str, sub: &str) -> Profile {
    identity::create_profile("talo", main, sub, "0123456789abcdef").unwrap()
}

fn owned_hull(mut profile: Profile, ship: u8) -> Profile {
    let index = ship as usize - 1;
    profile.campaign.patches[index] = true;
    profile.campaign.blueprints[index] = true;
    profile.campaign.crafted[index] = true;
    profile.campaign.selected_ship = ship;
    for id in 0..3 {
        profile.campaign.abilities[id] = campaign::AbilityState {
            blueprint: true,
            rank: 1,
        };
        profile.campaign.loadouts[ship as usize].slots[id] = Some(id as u8);
    }
    profile
}

fn serialized_affinity(game: &Game) -> Value {
    let hud = serde_json::to_value(game.hud()).unwrap();
    hud["ship_stats"]["affinity"].clone()
}

#[test]
fn actual_factory_hud_serializes_canonical_affinity_and_public_clear_state() {
    for (main, slot) in [("pan", 2), ("renko", 0), ("hugo", 1)] {
        for sub in ["pan", "renko", "hugo"] {
            let mut game = Game::new(licensed(main, sub));
            let before = checkpoint(&game);
            assert_eq!(
                serialized_affinity(&game),
                json!({
                    "corporation": main, "slot": slot,
                    "active": true, "source": "factory_license"
                })
            );
            for _ in 0..3 {
                game.hud();
            }
            assert_eq!(checkpoint(&game), before);
            game.equip_ability(slot, None).unwrap();
            let cleared = checkpoint(&game);
            assert_eq!(
                serialized_affinity(&game),
                json!({
                    "corporation": main, "slot": slot,
                    "active": false, "source": "factory_license"
                })
            );
            assert_eq!(checkpoint(&game), cleared);
            assert_eq!(cleared["identity"], before["identity"]);
            assert_eq!(game.profile().currency, [0; 3]);
            assert_eq!(game.profile().campaign.ship_xp, [0; 10]);
        }
    }
}

#[test]
fn actual_roster_hud_reports_craft_main_for_cross_corporation_and_anonymous_pilots() {
    let expected = [
        ("pan", 2),
        ("renko", 0),
        ("hugo", 1),
        ("pan", 2),
        ("pan", 2),
        ("hugo", 1),
        ("renko", 0),
        ("hugo", 1),
    ];
    for ship in 1..=8 {
        let (main, slot) = expected[ship as usize - 1];
        for pilot in [None, Some("pan"), Some("renko"), Some("hugo")] {
            let profile = pilot.map_or_else(Profile::default, |corp| licensed(corp, "hugo"));
            let mut game = Game::new(owned_hull(profile, ship));
            let before = checkpoint(&game);
            let affinity = game.hud().ship_stats.affinity.unwrap();
            assert_eq!(
                affinity.corporation,
                campaign::CHAMPIONS[ship as usize - 1].identity.corporation
            );
            assert_eq!(affinity.source, RaichoAffinitySource::RosterManufacturer);
            assert_eq!(
                serialized_affinity(&game),
                json!({
                    "corporation": main, "slot": slot,
                    "active": true, "source": "roster_manufacturer"
                })
            );
            assert_eq!(checkpoint(&game), before);
            game.equip_ability(slot, None).unwrap();
            assert!(!game.hud().ship_stats.affinity.unwrap().active);
            assert_eq!(checkpoint(&game)["identity"], before["identity"]);
        }
    }
    // TATE's PAN pilot/sponsor/horns must not override its RENKO manufacturer.
    let game = Game::new(owned_hull(licensed("pan", "pan"), 2));
    assert_eq!(campaign::CHAMPIONS[1].sponsor, identity::Corporation::Pan);
    assert_eq!(serialized_affinity(&game)["corporation"], "renko");
}

#[test]
fn invalid_unowned_and_wrong_category_slots_serialize_disabled_without_losing_manufacturer() {
    for id in [0, 2, 4, 11, 17, 255] {
        let mut profile = owned_hull(Profile::default(), 6);
        profile.campaign.abilities[1].rank = 0;
        profile.campaign.loadouts[6].slots[1] = Some(id);
        let game = Game::new(profile);
        assert_eq!(game.profile().campaign.loadouts[6].slots[1], None);
        let before = checkpoint(&game);
        assert_eq!(
            serialized_affinity(&game),
            json!({
                "corporation": "hugo", "slot": 1,
                "active": false, "source": "roster_manufacturer"
            })
        );
        assert_eq!(checkpoint(&game), before);
        let reloaded = Game::new(game.profile().clone());
        assert_eq!(serialized_affinity(&reloaded), serialized_affinity(&game));
        assert_eq!(checkpoint(&reloaded), before);
    }
}

#[test]
fn anonymous_factory_and_sovereign_fallback_have_truthful_nullable_sources() {
    let anonymous = Game::new(Profile::default());
    assert_eq!(serialized_affinity(&anonymous), Value::Null);
    for main in [None, Some("pan"), Some("renko"), Some("hugo")] {
        let mut profile = main.map_or_else(Profile::default, |corp| licensed(corp, "renko"));
        profile.campaign.patches = [true; 8];
        profile.campaign.blueprints = [true; 8];
        profile.campaign.royale_results = [1; 8];
        profile.campaign.victory_claimed = true;
        profile.campaign.selected_ship = 9;
        ship_progression::grant_royal(&mut profile);
        let mut game = Game::new(profile);
        let before = checkpoint(&game);
        if let Some(main) = main {
            let slot = match main {
                "pan" => 2,
                "renko" => 0,
                _ => 1,
            };
            assert_eq!(
                serialized_affinity(&game),
                json!({
                    "corporation": main, "slot": slot,
                    "active": true, "source": "sovereign_license"
                })
            );
            assert_eq!(checkpoint(&game), before);
            game.equip_ability(slot, None).unwrap();
            let value = serialized_affinity(&game);
            assert_eq!(value["active"], false);
            assert_eq!(value["source"], "sovereign_license");
        } else {
            assert_eq!(serialized_affinity(&game), Value::Null);
            assert_eq!(checkpoint(&game), before);
        }
        assert_eq!(checkpoint(&game)["identity"], before["identity"]);
        assert_eq!(game.profile().campaign.ship_xp, [0; 10]);
    }
}
