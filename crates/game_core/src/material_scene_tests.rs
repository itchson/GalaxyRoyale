use super::*;

fn pan() -> Game {
    Game::new(identity::create_profile("talo", "pan", "renko", "0011223344556677").unwrap())
}
#[test]
fn material_scene_is_truthful_bounded_and_does_not_mutate_progression_or_rng() {
    let mut game = pan();
    let profile = serde_json::to_string(game.profile()).unwrap();
    let rng = game.rng;
    assert!(game.diagnostic("material"));
    assert!(game.material_scene && game.diagnostic_scene);
    assert_eq!(game.position(), Vec2::new(450.0, 0.0));
    assert_eq!(game.world.len(), 1);
    let first = game.light_sources(1.0);
    assert_eq!(first.len(), 2);
    for _ in 0..600 {
        game.tick(
            &InputFrame {
                fire: true,
                ..Default::default()
            },
            1.0 / 60.0,
        );
    }
    let final_lights = game.light_sources(1.0);
    assert_ne!(first, final_lights);
    assert_eq!(final_lights.len(), 2);
    assert!(
        final_lights
            .iter()
            .all(|l| l.position.iter().all(|v| v.is_finite())
                && l.radius <= 110.0
                && l.intensity <= 1.0)
    );
    assert_eq!(game.world.len(), 1);
    assert!(game.particles.is_empty());
    assert!(game.transient_lights.is_empty());
    assert_eq!(profile, serde_json::to_string(game.profile()).unwrap());
    assert_eq!(rng, game.rng);
    let hud = game.hud();
    assert!((hud.time - 10.0).abs() < 0.001);
    assert_eq!(hud.position, [450.0, 0.0]);
    assert!(
        game.render(1.0)
            .batches
            .iter()
            .flat_map(|b| &b.pixels)
            .filter(|p| p.surface().is_some())
            .count()
            == 1867
    );
}
#[test]
fn material_scene_gates_preserve_profile_run_world_and_ready_result() {
    for gate in 0..7 {
        let mut game = pan();
        match gate {
            0 => game.profile.identity.as_mut().unwrap().corporation = identity::Corporation::Hugo,
            1 => game.profile.campaign.selected_ship = 1,
            2 => game.profile.campaign.cargo_materials[0] = 1,
            3 => game.profile.campaign.cargo_training_materials[0] = 1,
            4 => {
                game.run = Some(RunSummary {
                    time: 12.0,
                    ..Default::default()
                })
            }
            5 => {
                let boss = game.spawn_enemy(0, true, Vec2::new(1800.0, 0.0), 1);
                game.active_boss = Some(boss);
            }
            _ => {
                game.arena = Some(ArenaEncounter {
                    index: 0,
                    tournament: false,
                    diagnostic: false,
                    center: Vec2::new(1800.0, 0.0),
                    radius: 300.0,
                    opponent: game.player,
                })
            }
        }
        let profile = serde_json::to_string(game.profile()).unwrap();
        let entities = game.world.len();
        let position = game.position();
        assert!(!game.diagnostic("material"));
        assert!(!game.material_scene && !game.diagnostic_scene);
        assert_eq!(profile, serde_json::to_string(game.profile()).unwrap());
        assert_eq!(game.world.len(), entities);
        assert_eq!(game.position(), position);
        if gate == 4 {
            assert_eq!(game.run.as_ref().unwrap().time, 12.0);
        }
    }
    let mut game = pan();
    let id = |n: u32| co_op::ParticipantId::parse(&format!("{n:032x}")).unwrap();
    game.begin_co_op(co_op::PROTOCOL, id(1), id(2)).unwrap();
    let profile = serde_json::to_string(game.profile()).unwrap();
    assert!(!game.diagnostic("material"));
    assert!(game.co_op.is_some());
    assert_eq!(profile, serde_json::to_string(game.profile()).unwrap());
}
#[test]
fn material_controller_rotates_moves_and_exits_without_enabling_world_ticks_in_scene() {
    let mut game = pan();
    assert!(game.diagnostic("material"));
    let initial = game.position();
    for _ in 0..60 {
        game.tick(
            &InputFrame {
                thrust: Some([0.0, 1.0]),
                fire: true,
                ..Default::default()
            },
            1.0 / 60.0,
        );
    }
    assert!(game.position().distance(initial) > 30.0);
    assert!(game.throttle > 0.5);
    assert_eq!(game.world.len(), 1);
    game.reduced_motion = true;
    let frozen = game.light_sources(1.0);
    game.tick(&InputFrame::default(), 1.0 / 60.0);
    // Position-relative light vectors freeze under reduced motion.
    let next = game.light_sources(1.0);
    for (a, b) in frozen.iter().zip(next) {
        let delta = Vec2::from_array(b.position) - Vec2::from_array(a.position);
        assert!(
            delta.distance(
                game.position() - game.world.get::<&Motion>(game.player).unwrap().previous
            ) < 0.0001
        );
    }
    assert!(game.diagnostic("warp 600 0"));
    assert!(!game.material_scene);
    assert!(game.diagnostic_scene);
    assert!(game.diagnostic("home"));
    assert!(!game.diagnostic_scene);
    assert!(!game.material_scene);
    assert!(game.diagnostic("material"));
    game.world.get::<&mut Motion>(game.player).unwrap().position = Vec2::ZERO;
    game.tick(&InputFrame::default(), 1.0 / 60.0);
    assert!(!game.material_scene && !game.diagnostic_scene);
    assert_eq!(game.position(), Vec2::ZERO);
    assert!(!game.diagnostic("does-not-exist"));
}

#[test]
#[ignore = "explicit native material scene exporter; requires GALAXY_PAN_MATERIAL_EVIDENCE"]
fn export_actual_material_scene_packets_and_metadata() {
    let directory = std::path::PathBuf::from(
        std::env::var("GALAXY_PAN_MATERIAL_EVIDENCE").expect("explicit directory"),
    );
    std::fs::create_dir_all(&directory).unwrap();
    let mut game = pan();
    let profile = serde_json::to_string(game.profile()).unwrap();
    assert!(game.diagnostic("material"));
    game.set_viewport(640.0, 360.0);
    let mut views = Vec::new();
    for tick in [0, 120] {
        if tick > 0 {
            for _ in 0..tick {
                game.tick(&InputFrame::default(), 1.0 / 60.0);
            }
        }
        let frame = game.render(1.0);
        let cells:Vec<_>=frame.batches.iter().flat_map(|b|b.pixels.iter().filter(|p|p.surface().is_some()).map(|p|serde_json::json!({"local":p.local,"rgba":p.color,"surface":p.surface().unwrap().packed(),"a":b.transforms[p.transform_slot()].a,"b":b.transforms[p.transform_slot()].b}))).collect();
        assert_eq!(cells.len(), 1867);
        views.push(serde_json::json!({"tick":tick,"position":game.hud().position,"camera":game.hud().camera,"time":game.hud().time,"lights":game.light_sources(1.0),"materialCells":cells}));
    }
    assert_eq!(profile, serde_json::to_string(game.profile()).unwrap());
    let fixture = serde_json::json!({"version":1,"scope":"Actual native Game material scene, cells and moving light inputs only; no renderer/browser screenshot or performance claim.","seed":game.profile.seed,"logicalViewport":[640,360],"profileUnchanged":true,"worldEntities":game.world.len(),"views":views});
    std::fs::write(
        directory.join("native-material-scene.json"),
        serde_json::to_string_pretty(&fixture).unwrap() + "\n",
    )
    .unwrap();
    println!(
        "actual native material scene views=2 cells_per_view=1867 lights_per_view=2 world_entities=1 profile_unchanged=true"
    );
}
