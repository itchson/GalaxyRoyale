//! Authored roster metadata uses actual discovery, checkpoint and combat paths.
use super::*;
use std::collections::HashSet;

const DT: f32 = 1.0 / 60.0;

fn snapshot(game: &Game) -> serde_json::Value {
    serde_json::to_value(game.profile()).unwrap()
}

#[test]
fn eight_stable_competitors_have_separate_manufacturer_and_sponsor_facts() {
    let aliases = [
        "Lyra Voss",
        "Korr-9",
        "Sable Myrr",
        "Xil of Aurora",
        "Maera Quill",
        "Orun Vale",
        "Vexa Null",
        "Atlas-0",
    ];
    let legacy = [
        "Dawnneedle",
        "Bulwark",
        "Nightrail",
        "Prism",
        "Windrunner",
        "Ironwake",
        "Voidcoil",
        "Sunspear",
    ];
    let models = [
        "AKATSUKI",
        "TATE",
        "YORUMICHI",
        "KESSHO",
        "KAZEHAYA",
        "KUROGANE",
        "UZUSHIO",
        "HINOHOKO",
    ];
    let mut identities = HashSet::new();
    let mut kinds = HashSet::new();
    let mut sponsors = HashSet::new();
    for (i, champion) in CHAMPIONS.iter().enumerate() {
        assert_eq!(champion.name, aliases[i]);
        assert_eq!(champion.species, "KIKAI");
        assert_eq!(champion.ship, models[i]);
        assert_eq!(champion.legacy_model, legacy[i]);
        assert!(!champion.legacy_species.is_empty());
        let identity = champion.identity.hud();
        assert!(identity.hex_id.bytes().all(|b| b.is_ascii_hexdigit()));
        assert!(identities.insert(identity.short_designation));
        kinds.insert(champion.identity.kind.name());
        sponsors.insert(champion.sponsor.name());
        assert!(champion.introduction().contains(champion.sponsor.name()));
        assert!(champion.introduction().contains(models[i]));
    }
    assert_eq!(kinds.len(), 3);
    assert_eq!(sponsors.len(), 3);
    // The user's exact authored examples do not acquire the player's16-hex rule.
    assert_eq!(CHAMPIONS[0].identity.hud().short_designation, "APP#F344FEE");
    assert_eq!(
        CHAMPIONS[0].identity.hud().designation,
        "DORA-PAN-PAN-F344FEE"
    );
    assert_eq!(CHAMPIONS[1].identity.hud().short_designation, "ORP#2323");
    assert_eq!(
        CHAMPIONS[1].identity.hud().designation,
        "DORO-RENKO-PAN-2323"
    );
    assert_ne!(CHAMPIONS[1].identity.corporation, CHAMPIONS[1].sponsor);
    assert_ne!(CHAMPIONS[5].identity.sub_corporation, CHAMPIONS[5].sponsor);
}

#[test]
fn roster_authoring_keeps_all_eight_combat_and_reward_tunings() {
    let families = [0, 1, 2, 0, 0, 1, 2, 1];
    let categories = [0, 1, 2, 0, 2, 1, 2, 0];
    let grids = [
        [-1, -1],
        [0, -1],
        [1, -1],
        [1, 0],
        [1, 1],
        [0, 1],
        [-1, 1],
        [-1, 0],
    ];
    let colors = [
        [255, 195, 88],
        [255, 104, 102],
        [188, 123, 255],
        [104, 230, 255],
        [113, 244, 153],
        [255, 157, 88],
        [239, 116, 228],
        [255, 233, 158],
    ];
    let hull = [260.0, 390.0, 350.0, 300.0, 290.0, 440.0, 370.0, 340.0];
    for (i, champion) in CHAMPIONS.iter().enumerate() {
        assert_eq!(champion.family, families[i]);
        assert_eq!(champion.category, categories[i]);
        assert_eq!(champion.grid, grids[i]);
        assert_eq!(champion.color, colors[i]);
        assert_eq!(champion.health, hull[i]);
        // Native ability IDs remain index+6, and each still fits its reward hull.
        assert!(abilities::compatible(i as u8 + 6, i as u8 + 1));
    }
}

#[test]
fn public_unknown_roster_hud_has_no_identity_sponsor_or_reward_text() {
    let game = Game::new(Profile::default());
    let before = snapshot(&game);
    for _ in 0..3 {
        let hud = game.hud();
        assert_eq!(hud.campaign.champions.len(), 8);
        assert_eq!(hud.campaign.ships.len(), 1);
        for (i, champion) in hud.campaign.champions.iter().enumerate() {
            assert_eq!(champion.index, i);
            assert_eq!(champion.id, format!("champion-{i}"));
            assert!(!champion.discovered);
            assert_eq!(champion.name, "Unknown champion");
            assert!(champion.identity.is_none() && champion.sponsor.is_none());
            assert!(champion.ship.is_empty() && champion.ability.is_empty());
            assert!(champion.display_model.is_empty() && champion.legacy_model.is_empty());
            assert!(champion.species.is_empty() && champion.legacy_species.is_empty());
            assert!(champion.introduction.is_empty());
        }
        assert!(
            hud.campaign
                .standings
                .iter()
                .all(|row| row.entrant == 8 || row.name == "Unknown champion")
        );
        let serialized = serde_json::to_value(hud.campaign).unwrap();
        assert!(serialized["champions"][0]["identity"].is_null());
        assert!(serialized["champions"][0]["sponsor"].is_null());
    }
    assert_eq!(snapshot(&game), before);
}

#[test]
fn actual_movement_into_discovery_radius_reveals_metadata_without_a_reward() {
    for (index, source) in CHAMPIONS.iter().enumerate() {
        let mut game = Game::new(Profile::default());
        let site = arena(game.profile.seed, index).unwrap();
        let center = Vec2::from_array(site.position);
        // Declared position fixture; subsequent admission comes from actual flight.
        game.move_player(center + Vec2::X * (site.radius.max(320.0) + 10.0));
        game.invulnerability = 10.0;
        game.tick(&InputFrame::default(), DT);
        assert!(!game.hud().campaign.champions[index].discovered);
        let points = game.profile.currency;
        let xp = game.profile.campaign.ship_xp;
        for _ in 0..40 {
            game.tick(
                &InputFrame {
                    joystick: Some([-1.0, 0.0]),
                    ..InputFrame::default()
                },
                DT,
            );
        }
        let hud = game.hud();
        let actual = &hud.campaign.champions[index];
        assert!(actual.discovered);
        assert_eq!(actual.identity.as_ref().unwrap(), &source.identity.hud());
        assert_eq!(actual.sponsor, Some(source.sponsor));
        assert_eq!(actual.ship, source.ship);
        assert_eq!(actual.display_model, source.ship);
        assert_eq!(actual.legacy_model, source.legacy_model);
        assert_eq!(actual.introduction, source.introduction());
        assert!(game.profile.discoveries.contains(&site.id));
        assert_eq!(game.profile.currency, points);
        assert_eq!(game.profile.campaign.ship_xp, xp);
        assert_eq!(game.profile.campaign.patches, [false; 8]);
        assert!(
            hud.campaign
                .ships
                .iter()
                .any(|ship| ship.id == index as u8 + 1
                    && ship.name == source.ship
                    && !ship.owned
                    && !ship.craftable)
        );
        let saved = snapshot(&game);
        let reloaded = Game::new(serde_json::from_value(saved.clone()).unwrap());
        assert_eq!(snapshot(&reloaded), saved);
        assert_eq!(
            reloaded.hud().campaign.champions[index].identity.as_ref(),
            Some(&source.identity.hud())
        );
    }
}

#[test]
fn patch_owned_legacy_profiles_reveal_aliases_without_new_saved_roster_fields() {
    let mut profile = Profile {
        currency: [7, 8, 9],
        ..Profile::default()
    };
    profile.campaign.ship_xp[0] = 200;
    profile.campaign.patches[5] = true;
    profile.campaign.blueprints[5] = true;
    let game = Game::new(profile);
    let before = snapshot(&game);
    let champion = &game.hud().campaign.champions[5];
    assert!(champion.discovered);
    assert_eq!(champion.name, "Orun Vale");
    assert_eq!(champion.legacy_model, "Ironwake");
    assert_eq!(champion.display_model, "KUROGANE");
    assert_eq!(snapshot(&game), before);
    assert_eq!(game.profile.schema_version, 6);
    assert!(before["campaign"].get("roster").is_none());
    assert!(before["campaign"].get("champions").is_none());
}

#[test]
fn all_eight_real_projectile_victories_keep_exact_reward_indices_and_once_only_awards() {
    for (index, source) in CHAMPIONS.iter().enumerate() {
        let mut game = Game::new(Profile::default());
        let site = arena(game.profile.seed, index).unwrap();
        game.move_player(Vec2::from_array(site.position));
        game.tick(&InputFrame::default(), DT);
        game.start_champion(index).unwrap();
        let opponent = game.arena.unwrap().opponent;
        let local = game
            .world
            .get::<&PixelBody>(opponent)
            .unwrap()
            .cells()
            .min_by(|(a, _), (b, _)| a.length_squared().total_cmp(&b.length_squared()))
            .unwrap()
            .0;
        let motion = *game.world.get::<&Motion>(opponent).unwrap();
        let actor = *game.world.get::<&Actor>(opponent).unwrap();
        // High-damage native fixture still traverses the real occupied-cell hit path.
        game.spawn_bullet(
            actor_transform(motion, actor).point(local) - Vec2::X * 75.0,
            Vec2::X * 9_000.0,
            10_000.0,
            true,
            1,
        );
        game.tick(&InputFrame::default(), DT);
        assert!(
            !game.world.contains(opponent),
            "champion {index} survived the declared swept shot"
        );
        for i in 0..8 {
            assert_eq!(game.profile.campaign.patches[i], i == index);
            assert_eq!(game.profile.campaign.blueprints[i], i == index);
            assert_eq!(game.profile.campaign.abilities[i + 6].blueprint, i == index);
        }
        let mut points = [0; 3];
        points[source.category] = 3;
        assert_eq!(game.profile.currency, points);
        assert_eq!(
            game.profile.campaign.ship_xp,
            [40, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(game.profile.boss_kills, [0; 3]);
        assert_eq!(game.profile.boss_generations, [[0; 5]; 3]);
        let saved = snapshot(&game);
        assert!(game.start_champion(index).is_err());
        assert_eq!(snapshot(&game), saved);
        let reloaded = Game::new(serde_json::from_value(saved.clone()).unwrap());
        assert_eq!(snapshot(&reloaded), saved);
    }
}

#[test]
fn diagnostic_champion_contact_does_not_discover_or_award_roster_content() {
    for index in 0..8 {
        let mut game = Game::new(Profile::default());
        let before = snapshot(&game);
        game.diagnostic(&format!("champion-{index}"));
        game.invulnerability = 10.0;
        for _ in 0..60 {
            game.tick(&InputFrame::default(), DT);
        }
        assert_eq!(snapshot(&game), before);
        assert!(!game.hud().campaign.champions[index].discovered);
        assert!(game.hud().campaign.champions[index].identity.is_none());
    }
}
