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
