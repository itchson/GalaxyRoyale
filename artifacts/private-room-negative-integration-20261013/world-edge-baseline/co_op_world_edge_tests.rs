//! Real unfiltered chunk publication at the player coordinate boundary.
use super::*;

#[test]
fn actual_edge_chunks_commit_without_removing_generated_entities() {
    for point in [
        Vec2::new(WORLD_LIMIT, 0.0),
        Vec2::new(-WORLD_LIMIT, 0.0),
        Vec2::new(0.0, WORLD_LIMIT),
        Vec2::new(0.0, -WORLD_LIMIT),
        Vec2::splat(WORLD_LIMIT),
        Vec2::splat(-WORLD_LIMIT),
    ] {
        let mut game = Game::new(Profile {
            seed: 812345,
            ..Profile::default()
        });
        game.begin_co_op(PROTOCOL, id(100), id(1)).unwrap();
        let guest = game.join_co_op(id(100), ParticipantSpec {
            id: id(2), local_seed: 98767, identity: None,
        }).unwrap();
        let peer = game.world.query::<(hecs::Entity, &co_op::Pilot)>()
            .iter().find(|(_, p)| p.admission.id == guest.id).unwrap().0;
        *game.world.get::<&mut Motion>(peer).unwrap() = Motion::new(point, Vec2::ZERO, 0.0);
        game.spawn_timer = 10000.0;
        game.tick_co_op(id(100)).unwrap();
        // Keep every real streamed asteroid. The original small replica fixture
        // deliberately removes these and cannot exercise this failure.
        let data = capture(&mut game);
        let outside: Vec<_> = data.entities.iter().flat_map(|p| &p.entities)
            .filter(|row| row.kind == "asteroid" && row.position.iter().any(|v| v.abs() > WORLD_LIMIT))
            .collect();
        assert!(!outside.is_empty(), "actual edge chunk fixture must contain the original outlying geometry");
        println!("edge={point:?}; generated_outside_player_domain={}; first_position={:?}", outside.len(), outside[0].position);
        let mut replica = replica(guest.epoch);
        stage(&mut replica, &data, true);
        commit(&mut replica, &data);
        let scene = replica.scene().unwrap();
        assert_eq!(scene.entities.len(), data.header.counts.entities);
        for row in outside {
            assert!(scene.entities.iter().any(|(entity, _)| entity.id == row.id));
        }
    }
}
