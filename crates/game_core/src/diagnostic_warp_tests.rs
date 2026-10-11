use super::*;

#[test]
fn warp_uses_real_seeded_world_and_preserves_progression_without_rewards() {
    let profile = Profile {
        seed: 17,
        currency: [7, 13, 21],
        ..Profile::default()
    };
    let mut game = Game::new(profile);
    let before = serde_json::to_value(&game.profile).unwrap();
    game.diagnostic("warp -28879.919921875 -6955.796875");
    assert_eq!(game.position(), Vec2::new(-28879.92, -6955.797));
    assert_eq!(game.camera, game.position());
    assert_eq!(game.previous_camera, game.camera);
    assert_eq!(regions::at(17, game.position()).kind, 4);
    assert!(game.boss_qa_scene);
    assert_eq!(serde_json::to_value(&game.profile).unwrap(), before);
    game.tick(&InputFrame::default(), 1.0 / 60.0);
    assert_eq!(serde_json::to_value(&game.profile).unwrap(), before);
    game.diagnostic("dock");
    assert!(!game.boss_qa_scene);
    assert_eq!(game.position(), Vec2::ZERO);
}

#[test]
fn malformed_warp_never_resets_or_moves_the_existing_scene() {
    let mut game = Game::new(Profile::default());
    game.diagnostic("combat");
    let position = game.position();
    let entities = game.world.len();
    let profile = serde_json::to_value(&game.profile).unwrap();
    for command in [
        "warp NaN 0",
        "warp 0 inf",
        "warp 1000001 0",
        "warp 0 -1000001",
        "warp 2",
        "warp one 3",
        "warp 1 2 3",
        "warp ",
    ] {
        game.diagnostic(command);
        assert_eq!(game.position(), position, "{command}");
        assert_eq!(game.world.len(), entities, "{command}");
        assert_eq!(serde_json::to_value(&game.profile).unwrap(), profile);
    }
}

#[test]
fn isolated_warp_cannot_claim_a_ready_quest_but_normal_flight_still_can() {
    let mut game = Game::new(Profile::default());
    let site = regions::sites_near(game.profile.seed, Vec2::ZERO).remove(0);
    game.profile.discoveries.push(site.id.clone());
    let target = if site.event == "courier" {
        1
    } else {
        site.target_kills
    };
    game.profile.quests.push(QuestProgress {
        site_id: site.id.clone(),
        event: site.event.into(),
        family: site.family,
        target,
        progress: target,
        completed: false,
    });
    game.diagnostic(&format!("warp {} {}", site.position[0], site.position[1]));
    let before = serde_json::to_value(&game.profile).unwrap();
    assert!(game.interact_site().unwrap_err().contains("isolated QA"));
    assert_eq!(serde_json::to_value(&game.profile).unwrap(), before);
    game.boss_qa_scene = false;
    game.interact_site().unwrap();
    assert_eq!(game.profile.currency[site.family], 3);
    assert_eq!(game.profile.relics, vec![site.id]);
    assert!(game.interact_site().is_err());
    assert_eq!(game.profile.currency[site.family], 3);
}

#[test]
fn valid_warp_explicitly_abandons_cargo_and_finishes_an_existing_run_only_once() {
    let mut game = Game::new(Profile::default());
    game.move_player(Vec2::new(1600.0, 0.0));
    game.track_run(1.0);
    game.profile.campaign.cargo_materials = [3, 0, 0, 0, 0];
    game.profile.campaign.stored_materials = [7, 0, 0, 0, 0];
    game.profile.currency = [1, 2, 3];
    game.profile.campaign.ship_xp[0] = 60;
    game.diagnostic("warp 2000 100");
    assert_eq!(game.profile.campaign.cargo_materials, [0, 0, 0, 0, 0]);
    assert_eq!(game.profile.campaign.stored_materials, [7, 0, 0, 0, 0]);
    assert_eq!(game.profile.currency, [1, 2, 3]);
    assert_eq!(game.profile.campaign.ship_xp[0], 60);
    let report = game.profile.campaign.last_run.clone().unwrap();
    assert_eq!(report.time, 1.0);
    assert_eq!(report.distance_peak, 1600.0);
    assert_eq!(report.materials_lost, [3, 0, 0, 0, 0]);
    let before = serde_json::to_value(&game.profile).unwrap();
    game.diagnostic("warp 3000 100");
    assert_eq!(serde_json::to_value(&game.profile).unwrap(), before);
}
