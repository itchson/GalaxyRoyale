//! Actual admitted-pilot resource distribution gates, independent of RTC.
use super::*;

const LOCATIONS: [Vec2; 8] = [
    Vec2::new(4096.0, 4096.0),
    Vec2::new(-4096.0, 4096.0),
    Vec2::new(4096.0, -4096.0),
    Vec2::new(-4096.0, -4096.0),
    Vec2::new(24576.0, -18432.0),
    Vec2::new(-28672.0, 17408.0),
    Vec2::new(45056.0, 38912.0),
    Vec2::new(998912.0, -998912.0),
];

fn placed(order: [usize; 8]) -> Game {
    let (mut game, _) = fixture(8);
    for (motion, pilot) in game.world.query_mut::<(&mut Motion, &mut co_op::Pilot)>() {
        *motion = Motion::new(
            LOCATIONS[order[pilot.admission.slot as usize]],
            Vec2::ZERO,
            0.0,
        );
        pilot.invulnerability = 10000.0;
    }
    game
}

fn rock_count(game: &Game, location: Vec2) -> usize {
    let center = (location / CHUNK).floor().as_ivec2();
    (-1..=1)
        .flat_map(|dy| (-1..=1).map(move |dx| (center.x + dx, center.y + dy)))
        .filter_map(|key| game.chunks.get(&key))
        .flat_map(|entities| entities.iter())
        .filter(|&&entity| {
            game.world
                .get::<&Actor>(entity)
                .is_ok_and(|a| a.kind == Kind::Asteroid)
        })
        .count()
}

#[test]
fn eight_dispersed_native_tick_admits_resources_to_every_eligible_interest() {
    let mut game = placed([0, 1, 2, 3, 4, 5, 6, 7]);
    let saved = serde_json::to_value(game.profile()).unwrap();
    game.tick_co_op(id(100)).unwrap();
    let counts = LOCATIONS.map(|position| rock_count(&game, position));
    println!("actual_first_native_tick_interest_counts={counts:?}");
    assert!(
        counts.iter().all(|&count| count > 0),
        "eligible separated interests must not starve by roster order: {counts:?}"
    );
    assert_eq!(serde_json::to_value(game.profile()).unwrap(), saved);
    assert!(
        game.world
            .query::<&Actor>()
            .iter()
            .filter(|a| a.kind == Kind::Asteroid)
            .count()
            <= 112
    );
}

fn seeded_layout(game: &Game) -> Value {
    let mut chunks: Vec<_> = game.chunks.iter().map(|(key, entities)| {
        let rows: Vec<_> = entities.iter().filter_map(|&entity| {
            let (Ok(m), Ok(a), Ok(body), Ok(birth)) = (
                game.world.get::<&Motion>(entity), game.world.get::<&Actor>(entity),
                game.world.get::<&PixelBody>(entity), game.world.get::<&AsteroidBirth>(entity),
            ) else { return None; };
            Some(json!({"position":m.position.to_array().map(f32::to_bits),
                "velocity":m.velocity.to_array().map(f32::to_bits),"heading":m.heading.to_bits(),
                "actor":[a.health.to_bits(),a.max_health.to_bits(),a.radius.to_bits(),a.scale.to_bits(),a.damage.to_bits()],
                "birth":format!("{birth:?}"),"signature":body_signature(&body),
                "occupied":body.occupied_count(),"dimensions":[body.width,body.height]}))
        }).collect();
        (*key, rows)
    }).collect();
    chunks.sort_by_key(|(key, _)| *key);
    json!({"chunks":chunks,"rng":game.rng})
}

#[test]
fn shuffled_roster_retains_exact_seeded_asteroid_admission_layout() {
    let mut first = placed([0, 1, 2, 3, 4, 5, 6, 7]);
    let mut second = placed([7, 2, 5, 0, 6, 1, 4, 3]);
    first.update_co_op_chunks();
    second.update_co_op_chunks();
    assert_eq!(seeded_layout(&first), seeded_layout(&second));
    assert_eq!(first.co_op_chunk_progress, second.co_op_chunk_progress);
    assert_eq!(first.co_op_chunk_progress.len(), 72);
    assert_eq!(first.chunks.len(), 72);
    assert_eq!(
        LOCATIONS.map(|p| rock_count(&first, p)),
        LOCATIONS.map(|p| rock_count(&second, p))
    );
}

#[test]
fn real_destroyed_candidates_stay_destroyed_while_deferred_candidates_retry() {
    let mut game = placed([0, 1, 2, 3, 4, 5, 6, 7]);
    let saved = serde_json::to_value(game.profile()).unwrap();
    let mut admitted = HashSet::new();
    let mut total = 0;
    for _ in 0..4 {
        game.update_co_op_chunks();
        let rocks: Vec<_> = game
            .world
            .query::<(hecs::Entity, (&Motion, &Actor))>()
            .iter()
            .filter(|(_, (_, a))| a.kind == Kind::Asteroid)
            .map(|(e, (m, _))| (e, m.position))
            .collect();
        assert!(rocks.len() <= 112);
        for (entity, position) in &rocks {
            assert!(
                admitted.insert(position.to_array().map(f32::to_bits)),
                "destroyed seeded birth regenerated"
            );
            total += 1;
            game.damage_actor(*entity, f32::MAX, Vec2::ZERO, Vec2::ZERO);
            assert!(!game.world.contains(*entity));
        }
        if rocks.is_empty() {
            break;
        }
    }
    assert!(
        total > 112,
        "capacity release must admit previously deferred births"
    );
    assert!(
        game.co_op_chunk_progress.is_empty(),
        "finite candidates finish after capacity release"
    );
    game.update_co_op_chunks();
    assert_eq!(
        game.world
            .query::<&Actor>()
            .iter()
            .filter(|a| a.kind == Kind::Asteroid)
            .count(),
        0
    );
    assert_eq!(game.chunks.len(), 72);
    assert_eq!(serde_json::to_value(game.profile()).unwrap(), saved);
    println!("real_destroyed_unique_candidates={total}; no_loaded_birth_regeneration=true");
}

#[test]
fn ordinary_interest_unload_releases_capacity_without_evicting_retained_rocks() {
    let mut game = placed([0, 1, 2, 3, 4, 5, 6, 7]);
    game.update_co_op_chunks();
    let old_host_center = (LOCATIONS[0] / CHUNK).floor().as_ivec2();
    let retained: Vec<_> = game
        .chunks
        .iter()
        .filter(|(key, _)| {
            (key.0 - old_host_center.x).abs() > 2 || (key.1 - old_host_center.y).abs() > 2
        })
        .flat_map(|(_, es)| es.iter().copied())
        .collect();
    let next = Vec2::new(73728.0, -53248.0);
    game.world.get::<&mut Motion>(game.player).unwrap().position = next;
    game.update_co_op_chunks();
    assert!(retained.iter().all(|&e| game.world.contains(e)));
    assert!(rock_count(&game, LOCATIONS[0]) == 0);
    assert!(
        rock_count(&game, next) > 0,
        "unloaded capacity must reach newly active interest"
    );
    assert_eq!(game.chunks.len(), 72);
    assert!(game.co_op_chunk_progress.len() <= co_op::MAX_PARTICIPANTS * 25);
    assert!(
        game.world
            .query::<&Actor>()
            .iter()
            .filter(|a| a.kind == Kind::Asteroid)
            .count()
            <= 112
    );
}

#[test]
fn pending_admission_cursors_are_session_transient_and_leave_with_boxed_world() {
    let mut game = placed([0, 1, 2, 3, 4, 5, 6, 7]);
    game.update_co_op_chunks();
    assert!(!game.co_op_chunk_progress.is_empty());
    game.end_co_op(id(100)).unwrap();
    assert!(game.co_op_chunk_progress.is_empty());
    assert!(game.chunks.is_empty());
}

#[test]
#[ignore = "explicit native solo seeded-layout comparison export; requires new output"]
fn solo_seeded_chunk_layout_export() {
    let output = std::path::PathBuf::from(std::env::var("GALAXY_SOLO_CHUNKS_OUT").unwrap());
    assert!(output.is_absolute() && !output.exists());
    let mut captures = Vec::new();
    for seed in [812345, 0, 0x0a2d7f9b] {
        let mut game = Game::new(Profile {
            seed,
            ..Profile::default()
        });
        for position in [
            Vec2::ZERO,
            Vec2::new(1000.0, 0.0),
            Vec2::new(1200.0, 480.0),
            Vec2::new(1400.0, 512.0),
            Vec2::new(4096.0, 4096.0),
            Vec2::new(4608.0, 4096.0),
            Vec2::new(5120.0, 4608.0),
            Vec2::new(998912.0, -998912.0),
            Vec2::ZERO,
        ] {
            game.update_chunks(position);
            assert!(game.chunks.len() <= 25);
            captures.push(
                json!({"seed":seed,"position":position.to_array(),"layout":seeded_layout(&game)}),
            );
        }
    }
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .unwrap();
    file.write_all(serde_json::to_string_pretty(&captures).unwrap().as_bytes())
        .unwrap();
    println!("actual_solo_seeded_layout_captures={}", captures.len());
}
