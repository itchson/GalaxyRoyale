//! Real birth, occupied geometry and render-packet evidence, independent of quality.
use super::*;
use engine_core::{CameraView, GpuTransform};
use std::collections::{HashSet, VecDeque};
const DT: f32 = 1.0 / 60.0;

fn signature(body: &PixelBody) -> Vec<(usize, u32)> {
    body.cells_indexed().map(|(i, _, c)| (i, c)).collect()
}
fn solid(body: &PixelBody) {
    let neighbors = |i: usize| {
        let x = i % body.width;
        let y = i / body.width;
        [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ]
    };
    let flood = |start: Vec<usize>, filled| {
        let mut seen: HashSet<_> = start.iter().copied().collect();
        let mut queue = VecDeque::from(start);
        while let Some(i) = queue.pop_front() {
            for (x, y) in neighbors(i) {
                if x < body.width && y < body.height && body.is_occupied(x, y) == filled {
                    let next = y * body.width + x;
                    if seen.insert(next) {
                        queue.push_back(next);
                    }
                }
            }
        }
        seen
    };
    let first = body.cells_indexed().next().unwrap().0;
    assert_eq!(flood(vec![first], true).len(), body.occupied_count());
    let boundary = (0..body.width * body.height)
        .filter(|i| {
            let x = i % body.width;
            let y = i / body.width;
            (x == 0 || y == 0 || x == body.width - 1 || y == body.height - 1)
                && !body.is_occupied(x, y)
        })
        .collect();
    assert_eq!(
        flood(boundary, false).len() + body.occupied_count(),
        body.width * body.height,
        "Crater/seam shading must not create enclosed collision holes"
    );
}

#[test]
fn distinct_seeded_instances_are_solid_and_tier_grading_does_not_redraw_the_shape() {
    let mut totals = [usize::MAX, 0];
    for family in 0..4u32 {
        let mut identities = HashSet::new();
        let mut silhouettes = HashSet::new();
        for n in 0..48u32 {
            let seed = (mix(n.wrapping_mul(7919)) & !96) | (family * 32);
            let a = pixel_assets::asteroid_instance(seed, 1);
            assert_eq!(pixel_assets::asteroid_variety(seed), family as u8);
            assert_eq!((a.width, a.height), (64, 64));
            assert!(a.occupied_count() <= asteroid_art::MAX_CELLS);
            totals[0] = totals[0].min(a.occupied_count());
            totals[1] = totals[1].max(a.occupied_count());
            solid(&a);
            assert!(
                identities.insert(signature(&a)),
                "same-family seeded surface repeated"
            );
            silhouettes.insert(a.cells_indexed().map(|(i, _, _)| i).collect::<Vec<_>>());
            assert_eq!(
                signature(&a),
                signature(&pixel_assets::asteroid_instance(seed, 1))
            );
            for tier in [3, 5] {
                let b = pixel_assets::asteroid_instance(seed, tier);
                assert_eq!(
                    a.cells_indexed().map(|(i, _, _)| i).collect::<Vec<_>>(),
                    b.cells_indexed().map(|(i, _, _)| i).collect::<Vec<_>>()
                );
                assert_ne!(signature(&a), signature(&b));
                assert!(b.cells().all(|(_, c)| c >> 24 == 255));
                assert!(b.cells().map(|(_, c)| c).collect::<HashSet<_>>().len() >= 7);
            }
        }
        assert!(
            silhouettes.len() >= 44,
            "Instance variety cannot be palette-only"
        );
        println!(
            "Family{family}: {} unique surfaces / {} silhouettes",
            identities.len(),
            silhouettes.len()
        );
    }
    println!("Sampled192 instance occupied-cell range {totals:?}");
}

#[test]
fn actual_mineral_cells_form_compact_deposits_instead_of_long_surface_streaks() {
    let mut components = 0;
    for family in 0..4u32 {
        for n in 0..32u32 {
            let seed = (mix(n.wrapping_mul(7919)) & !96) | (family * 32);
            for tier in [1, 5] {
                let body = pixel_assets::asteroid_instance(seed, tier);
                let colors = asteroid_art::mineral_colors(seed, tier);
                let mut remaining: HashSet<_> = body
                    .cells_indexed()
                    .filter_map(|(i, _, c)| colors.contains(&c).then_some(i))
                    .collect();
                let mut deposits = 0;
                while let Some(&first) = remaining.iter().next() {
                    remaining.remove(&first);
                    let mut queue = VecDeque::from([first]);
                    let mut cells = Vec::new();
                    while let Some(i) = queue.pop_front() {
                        cells.push((i % body.width, i / body.width));
                        let x = i % body.width;
                        let y = i / body.width;
                        for dy in -1..=1isize {
                            for dx in -1..=1isize {
                                let nx = x.wrapping_add_signed(dx);
                                let ny = y.wrapping_add_signed(dy);
                                if nx < body.width && ny < body.height {
                                    let next = ny * body.width + nx;
                                    if remaining.remove(&next) {
                                        queue.push_back(next);
                                    }
                                }
                            }
                        }
                    }
                    let width = cells.iter().map(|p| p.0).max().unwrap()
                        - cells.iter().map(|p| p.0).min().unwrap()
                        + 1;
                    let height = cells.iter().map(|p| p.1).max().unwrap()
                        - cells.iter().map(|p| p.1).min().unwrap()
                        + 1;
                    assert!(
                        (6..=40).contains(&cells.len()),
                        "deposit lost clustered area"
                    );
                    assert!(width <= 9 && height <= 9, "long mineral overlay returned");
                    deposits += 1;
                    components += 1;
                }
                assert_eq!(deposits, 4, "separated mineral deposits joined or vanished");
            }
        }
    }
    println!("Verified {components} actual compact mineral components across 256 bodies");
}

#[test]
fn complete_cell_corners_fit_the_existing_zone_padding_for_every_configured_scale() {
    // Even an entirely filled64-grid would fit; real occupied extents are smaller.
    let theoretical = (31.5_f32 * std::f32::consts::SQRT_2 + std::f32::consts::FRAC_1_SQRT_2)
        * asteroid_art::CELL_SCALE
        * asteroid_art::MAX_SIZE;
    assert!(theoretical < regions::BELT_BODY_PADDING);
    let mut maximum = 0.0_f32;
    for seed in (0..160u32).map(|n| mix(n.wrapping_mul(977))) {
        for tier in 1..=5 {
            for belt in [false, true] {
                let size = asteroid_art::size(seed, tier, belt);
                assert!(size > 0.0 && size <= asteroid_art::MAX_SIZE);
                let body = pixel_assets::asteroid_instance(seed, tier);
                let scale = size * asteroid_art::CELL_SCALE;
                let radius = pixel_assets::radius(&body, scale);
                maximum = maximum.max(radius);
                assert!(radius < regions::BELT_BODY_PADDING);
                let speed = asteroid_art::speed(seed, tier, belt);
                assert!(speed.is_finite() && (3.0..=16.0).contains(&speed));
            }
        }
    }
    println!(
        "Observed maximum radius{maximum}; theoretical all-filled bound{theoretical}; reserved{}",
        regions::BELT_BODY_PADDING
    );
}

type BirthPacket = (u32, u32, u8, u8, f32, Vec<(usize, u32)>);
fn stream(game: &mut Game, at: Vec2) -> Vec<BirthPacket> {
    game.move_player(at);
    game.update_chunks(at);
    let mut result: Vec<_> = game
        .world
        .query::<(&Motion, &Actor, &PixelBody, &AsteroidBirth)>()
        .iter()
        .map(|(m, a, b, birth)| {
            assert_eq!(a.kind, Kind::Asteroid);
            assert_eq!(a.radius, pixel_assets::radius(b, a.scale));
            assert!(a.radius < regions::BELT_BODY_PADDING);
            let key = (
                (m.position.x / CHUNK).floor() as i32,
                (m.position.y / CHUNK).floor() as i32,
            );
            let seed = (0..12)
                .map(|i| hash3(game.profile.seed, key.0, key.1, i))
                .find(|h| {
                    Vec2::new(
                        key.0 as f32 * CHUNK + unit(*h) * CHUNK,
                        key.1 as f32 * CHUNK + unit(mix(*h)) * CHUNK,
                    ) == m.position
                })
                .unwrap();
            assert_eq!(birth.variety, pixel_assets::asteroid_variety(seed));
            assert_eq!(
                signature(b),
                signature(&pixel_assets::asteroid_instance(seed, birth.material_tier))
            );
            assert_eq!(
                a.scale,
                asteroid_art::size(
                    seed,
                    birth.material_tier,
                    birth.source != AsteroidSource::Outer
                ) * asteroid_art::CELL_SCALE
            );
            (
                m.position.x.to_bits(),
                m.position.y.to_bits(),
                birth.variety,
                birth.material_tier,
                a.scale,
                signature(b),
            )
        })
        .collect();
    result.sort_by_key(|r| (r.0, r.1));
    result
}

#[test]
fn actual_streamed_births_are_deterministic_bounded_and_do_not_advance_combat_rng_or_saves() {
    for position in [
        Vec2::new(1000.0, 0.0),
        Vec2::new(5000.0, 0.0),
        Vec2::new(990_000.0, -999_000.0),
    ] {
        let mut a = Game::new(Profile::default());
        let mut b = Game::new(Profile::default());
        let before = serde_json::to_value(a.profile()).unwrap();
        let rng = a.rng;
        let rocks = stream(&mut a, position);
        assert!(!rocks.is_empty());
        assert!(rocks.len() <= 112);
        assert_eq!(rocks, stream(&mut b, position));
        assert_eq!(a.rng, rng);
        assert_eq!(serde_json::to_value(a.profile()).unwrap(), before);
        let unique: HashSet<_> = rocks.iter().map(|r| r.5.clone()).collect();
        assert_eq!(unique.len(), rocks.len());
        if position.x == 1000.0 {
            assert_eq!(rocks.iter().map(|r| r.2).collect::<HashSet<_>>().len(), 4);
            assert!(rocks.iter().all(|r| r.3 == 1));
        }
    }
}

#[test]
fn actual_render_uses_the_collision_body_and_removed_cells_do_not_reappear() {
    let mut game = Game::new(Profile::default());
    game.set_reduced_motion(true);
    game.set_viewport(960.0, 540.0);
    stream(&mut game, Vec2::new(1000.0, 0.0));
    let entity = game
        .world
        .query::<(Entity, (&Motion, &AsteroidBirth))>()
        .iter()
        .min_by(|(_, (a, _)), (_, (b, _))| {
            a.position
                .distance_squared(game.position())
                .total_cmp(&b.position.distance_squared(game.position()))
        })
        .unwrap()
        .0;
    let m = *game.world.get::<&Motion>(entity).unwrap();
    let a = *game.world.get::<&Actor>(entity).unwrap();
    let gpu = GpuTransform::from(actor_transform(m, a).affine()).with_material(ACTOR_MATERIAL);
    let extract = |game: &Game| {
        let current_motion = *game.world.get::<&Motion>(entity).unwrap();
        let current_actor = *game.world.get::<&Actor>(entity).unwrap();
        let current_gpu =
            GpuTransform::from(actor_transform(current_motion, current_actor).affine())
                .with_material(ACTOR_MATERIAL);
        game.render(1.0)
            .batches
            .into_iter()
            .flat_map(|b| {
                b.pixels.into_iter().filter_map(move |p| {
                    (b.transforms[p.transform_slot()] == current_gpu).then_some((p.local, p.color))
                })
            })
            .collect::<Vec<_>>()
    };
    let intact = extract(&game);
    assert_eq!(
        intact.len(),
        game.world
            .get::<&PixelBody>(entity)
            .unwrap()
            .occupied_count()
    );
    for zoom in [0.6, 1.0, 1.6] {
        let view = CameraView::new(m.position, Vec2::new(640.0, 360.0), zoom).unwrap();
        for (local, _) in &intact {
            let local = Vec2::from_array(*local);
            assert!(
                game.world
                    .get::<&PixelBody>(entity)
                    .unwrap()
                    .hit_segment(local, local)
                    .is_some()
            );
            for corner in [
                Vec2::new(-0.5, -0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::new(0.5, -0.5),
                Vec2::new(0.5, 0.5),
            ] {
                let world = gpu.point(local + corner);
                assert!(world.distance(m.position) <= a.radius + 0.001);
                assert!(view.world_to_screen(world).is_finite());
            }
        }
    }
    // One real projectile strips live occupied cells without defeating the actor.
    let local = Vec2::from_array(intact[intact.len() / 2].0);
    let world = actor_transform(m, a).point(local);
    game.spawn_bullet(world, Vec2::ZERO, 1.0, true, 1);
    game.tick(&InputFrame::default(), DT);
    assert!(game.world.contains(entity));
    let damaged = extract(&game);
    assert!(damaged.len() < intact.len());
    assert_eq!(
        damaged.len(),
        game.world
            .get::<&PixelBody>(entity)
            .unwrap()
            .occupied_count()
    );
}

#[test]
fn actual_packets_export_only_when_requested_and_keep_fractional_local_geometry() {
    let mut records = Vec::new();
    for tier in [1, 5] {
        for family in 0..4u32 {
            for n in 0..4u32 {
                let seed = (mix(n.wrapping_mul(7919)) & !96) | (family * 32);
                let body = pixel_assets::asteroid_instance(seed, tier);
                let mut frame = FrameBuilder::new();
                frame.push_body(
                    &body,
                    Transform {
                        scale: Vec2::splat(asteroid_art::CELL_SCALE),
                        ..Default::default()
                    },
                );
                assert_eq!(frame.frame().pixel_count, body.occupied_count());
                assert_eq!(frame.frame().batches[0].transforms.len(), 1);
                let pixels: Vec<_> = frame.frame().batches[0]
                    .pixels
                    .iter()
                    .map(|p| serde_json::json!({"local":p.local,"color":p.color}))
                    .collect();
                let t = frame.frame().batches[0].transforms[0];
                assert_eq!(t.a[2], asteroid_art::CELL_SCALE);
                assert_eq!(t.b[1], asteroid_art::CELL_SCALE);
                records.push(serde_json::json!({"family":family,"tier":tier,"seed":seed,"count":pixels.len(),"transform":{"a":t.a,"b":t.b},"pixels":pixels}));
            }
        }
    }
    assert_eq!(records.len(), 32);
    if let Ok(path) = std::env::var("GALAXY_ASTEROID_PACKET_EXPORT") {
        std::fs::write(path, serde_json::to_vec(&records).unwrap()).unwrap();
    }
}
