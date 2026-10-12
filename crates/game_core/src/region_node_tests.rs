//! Canonical placement, complete birth envelopes and real sanctuary consumers.
use super::*;
use crate::{Actor, AsteroidBirth, AsteroidSource, Game, InputFrame, Kind, Motion, Profile};

fn seeds() -> impl Iterator<Item = u32> {
    [0, 1, 17, 0x4752_4f59, u32::MAX]
        .into_iter()
        .chain((0..128).map(|n| mix(n * 7919)))
}

fn distance(a: Vec2, b: Vec2) -> f64 {
    let x = f64::from(a.x) - f64::from(b.x);
    let y = f64::from(a.y) - f64::from(b.y);
    x.hypot(y)
}

#[test]
fn typed_radii_retain_canonical_mission_metadata_and_biome_anchors() {
    let mut seen = [false; 4];
    let mut ranges = [(f32::INFINITY, 0.0_f32); 4];
    for seed in seeds() {
        for y in -3..=3 {
            for x in -3..=3 {
                let grid = [x, y];
                let s = site(seed, grid).unwrap();
                let h = hash(seed, grid, 0x7369_7465);
                let policy = s.kind.policy();
                assert!((policy.min_radius..=policy.max_radius).contains(&s.radius));
                assert_eq!(s.id, format!("{x}:{y}"));
                assert_eq!(s.event, if h & 1 == 0 { "courier" } else { "bounty" });
                assert_eq!(s.family, (mix(h ^ 37) % 3) as usize);
                assert_eq!(s.target_kills, 8 + mix(h ^ 71) % 5);
                let names = if h & 1 == 0 {
                    ["Relay Anchorage", "Wayfarer Station", "Signal Sanctuary"]
                } else {
                    ["Sentinel Refuge", "Outrider Beacon", "Frontier Bastion"]
                };
                assert_eq!(s.name, names[(mix(h ^ 11) % 3) as usize]);
                let index = s.kind as usize;
                seen[index] = true;
                ranges[index].0 = ranges[index].0.min(s.radius);
                ranges[index].1 = ranges[index].1.max(s.radius);
                // Exact old biome anchor formula, independent of the new site.
                let jitter = Vec2::new(unit(hash(seed, grid, 41)), unit(hash(seed, grid, 97)))
                    * 0.44
                    + Vec2::splat(0.28);
                let old_center = (Vec2::new(x as f32, y as f32) + jitter) * REGION_SIZE;
                assert_eq!(center(seed, grid), old_center);
                assert_eq!(at(seed, old_center).grid, grid);
                assert_ne!(Vec2::from_array(s.position), old_center);
            }
        }
    }
    assert!(seen.into_iter().all(|v| v));
    for (i, kind) in [
        NodeKind::Courier,
        NodeKind::Bounty,
        NodeKind::ChampionArena,
        NodeKind::RoyaleArena,
    ]
    .into_iter()
    .enumerate()
    {
        let policy = kind.policy();
        assert!(ranges[i].0 < policy.min_radius + 3.0);
        assert!(ranges[i].1 > policy.max_radius - 3.0);
    }
    println!("Observed seeded radius ranges: {ranges:?}");
}

#[test]
fn adjacent_diagonal_and_far_edge_nodes_leave_complete_belt_clearance() {
    let mut pairs = 0;
    let mut smallest = f64::INFINITY;
    for seed in seeds() {
        for anchor in [
            [-3, -3],
            [-1, -1],
            [0, 0],
            [2, 2],
            [-238, -238],
            [235, 235],
            [-238, 235],
            [235, -238],
        ] {
            for y in 0..=2 {
                for x in 0..=2 {
                    let grid = [anchor[0] + x, anchor[1] + y];
                    let a = site(seed, grid).unwrap();
                    let ap = Vec2::from_array(a.position);
                    for direction in [[1, 0], [0, 1], [1, 1], [1, -1]] {
                        let other = [grid[0] + direction[0], grid[1] + direction[1]];
                        let Some(b) = site(seed, other) else { continue };
                        let separation = distance(ap, Vec2::from_array(b.position))
                            - f64::from(a.envelope_radius() + b.envelope_radius());
                        assert!(
                            separation >= f64::from(NODE_CLEARANCE),
                            "seed {seed} {} {} separation {separation}",
                            a.id,
                            b.id
                        );
                        let belt = site_belt(seed, &a).unwrap();
                        assert_eq!(belt.outer + BELT_BODY_PADDING, a.envelope_radius());
                        assert_eq!(belt.inner, a.radius + BELT_INNER_OFFSET);
                        smallest = smallest.min(separation);
                        pairs += 1;
                    }
                }
            }
        }
    }
    println!(
        "{pairs} actual seeded adjacent/diagonal pairs; minimum padded-envelope gap {smallest}"
    );
}

#[test]
fn extrema_reservations_prove_gap_without_seed_or_query_rejection() {
    // Deliberately force both radius extrema and seed-selected center extrema.
    for radius in [160.0, 220.0, 180.0, 260.0, 340.0, 420.0, 400.0, 480.0] {
        let envelope = radius + BELT_OUTER_OFFSET + BELT_BODY_PADDING;
        let inset = envelope + NODE_CLEARANCE * 0.5;
        for seed in seeds() {
            for grid in [
                [-238, -238],
                [-1, -1],
                [-1, 0],
                [0, -1],
                [0, 0],
                [1, 1],
                [237, 237],
            ] {
                let p = node_position(seed, grid, radius).unwrap();
                let cell = Vec2::new(grid[0] as f32, grid[1] as f32) * REGION_SIZE;
                assert!((p - cell).min_element() >= inset - 0.125);
                assert!((cell + Vec2::splat(REGION_SIZE) - p).min_element() >= inset - 0.125);
                assert!(p.abs().max_element() + envelope + NODE_CLEARANCE * 0.5 <= WORLD_LIMIT);
                assert!(
                    distance(p, Vec2::ZERO) - f64::from(envelope)
                        >= f64::from(HUB_RADIUS + BELT_BODY_PADDING + NODE_CLEARANCE * 0.5)
                );
            }
        }
    }
}

#[test]
fn closest_permitted_corners_preserve_gap_even_at_radius_extrema() {
    let mut smallest = f64::INFINITY;
    for base in [[-238, -238], [-1, -1], [0, 0], [2, 2], [235, 235]] {
        for radius_a in [160.0, 260.0, 420.0, 480.0] {
            let (low_a, high_a) = node_bounds(base, radius_a).unwrap();
            for direction in [[1, 0], [0, 1], [1, 1]] {
                for radius_b in [160.0, 260.0, 420.0, 480.0] {
                    let other = [base[0] + direction[0], base[1] + direction[1]];
                    let (low_b, high_b) = node_bounds(other, radius_b).unwrap();
                    // Every corner, not merely hash values close to zero/one.
                    for a in [
                        low_a,
                        high_a,
                        Vec2::new(low_a.x, high_a.y),
                        Vec2::new(high_a.x, low_a.y),
                    ] {
                        for b in [
                            low_b,
                            high_b,
                            Vec2::new(low_b.x, high_b.y),
                            Vec2::new(high_b.x, low_b.y),
                        ] {
                            let gap = distance(a, b)
                                - f64::from(
                                    radius_a
                                        + radius_b
                                        + 2.0 * (BELT_OUTER_OFFSET + BELT_BODY_PADDING),
                                );
                            assert!(gap >= f64::from(NODE_CLEARANCE));
                            smallest = smallest.min(gap);
                        }
                    }
                }
            }
        }
    }
    assert!((smallest - f64::from(NODE_CLEARANCE + 2.0 * NODE_ROUNDING_MARGIN)).abs() < 0.125);
    for kind in [
        NodeKind::Courier,
        NodeKind::Bounty,
        NodeKind::ChampionArena,
        NodeKind::RoyaleArena,
    ] {
        let radius = kind.policy().max_radius;
        for grid in [[-1, -1], [-1, 0], [0, -1], [0, 0]] {
            let (low, high) = node_bounds(grid, radius).unwrap();
            for p in [
                low,
                high,
                Vec2::new(low.x, high.y),
                Vec2::new(high.x, low.y),
            ] {
                assert!(
                    distance(p, Vec2::ZERO)
                        - f64::from(radius + BELT_OUTER_OFFSET + BELT_BODY_PADDING)
                        > f64::from(HUB_RADIUS + BELT_BODY_PADDING + NODE_CLEARANCE * 0.5)
                );
            }
        }
    }
    println!(
        "Actual allowed rectangle corners at min/max radii: minimum padded-envelope gap {smallest}"
    );
}

#[test]
fn every_campaign_node_and_home_corridor_survive_all_sampled_seeds() {
    for seed in seeds().chain((0..4096).map(mix)) {
        let mut ids = std::collections::HashSet::new();
        for index in 0..8 {
            let arena = crate::campaign::arena(seed, index).unwrap();
            assert_eq!(arena.kind, NodeKind::ChampionArena);
            assert_eq!(arena.grid, crate::campaign::CHAMPIONS[index].grid);
            assert!(ids.insert(arena.id.clone()));
            assert_eq!(site_by_id(seed, &arena.id), Some(arena.clone()));
            let p = Vec2::from_array(arena.position);
            assert!(
                distance(p, Vec2::ZERO) - f64::from(arena.envelope_radius())
                    >= f64::from(HUB_RADIUS + BELT_BODY_PADDING + NODE_CLEARANCE * 0.5)
            );
            assert!(
                distance(p, Vec2::ZERO) - f64::from(arena.radius) > f64::from(crate::SAFE_RADIUS)
            );
            assert!(site_belt(seed, &arena).is_some());
            assert!(sites_near(seed, p).contains(&arena));
        }
        let royale = crate::campaign::tournament_site(seed);
        assert_eq!(royale.id, "0:2");
        assert_eq!(royale.kind, NodeKind::RoyaleArena);
        assert!(ids.insert(royale.id.clone()));
        assert!(site_belt(seed, &royale).is_some());
        assert!(protected(seed, Vec2::ZERO).is_none());
        assert!(protected(seed, Vec2::X * HUB_RADIUS).is_none());
    }
    println!(
        "4,229 sampled seed cases retain all nine campaign nodes with complete Haven corridor clearance"
    );
}

#[test]
fn partial_world_cells_are_omitted_and_finite_queries_stay_bounded() {
    for seed in seeds() {
        for coordinate in [-238, -100, 0, 100, 237] {
            for edge in [-239, 238, 239] {
                for grid in [[edge, coordinate], [coordinate, edge]] {
                    assert!(site(seed, grid).is_none());
                    assert!(site_by_id(seed, &format!("{}:{}", grid[0], grid[1])).is_none());
                }
            }
            for edge in [-238, 237] {
                for grid in [[edge, coordinate], [coordinate, edge]] {
                    let s = site(seed, grid).unwrap();
                    let p = Vec2::from_array(s.position);
                    assert!(
                        p.abs().max_element() + s.envelope_radius() + NODE_CLEARANCE * 0.5
                            <= WORLD_LIMIT
                    );
                    assert!(site_belt(seed, &s).is_some());
                }
            }
        }
        for p in [
            Vec2::splat(WORLD_LIMIT),
            Vec2::splat(-WORLD_LIMIT),
            Vec2::splat(f32::MAX),
            Vec2::splat(f32::NAN),
        ] {
            let nearby = sites_near(seed, p);
            assert!(nearby.len() <= 9);
            assert!(
                nearby
                    .iter()
                    .all(|s| Vec2::from_array(s.position).is_finite())
            );
        }
    }
}

#[test]
fn shuffled_lookup_discovery_and_reload_never_change_node_identity() {
    let grids: Vec<_> = (-4..=4)
        .flat_map(|y| (-4..=4).map(move |x| [x, y]))
        .collect();
    for seed in seeds().take(21) {
        let query = |ordered: &[[i32; 2]]| {
            ordered
                .iter()
                .map(|&grid| {
                    let id = format!("{}:{}", grid[0], grid[1]);
                    (id.clone(), site_by_id(seed, &id).unwrap())
                })
                .collect::<std::collections::BTreeMap<_, _>>()
        };
        let canonical = query(&grids);
        let mut reversed = grids.clone();
        reversed.reverse();
        assert_eq!(canonical, query(&reversed));
        reversed.sort_by_key(|&grid| hash(seed, grid, 0x6f72_6465));
        assert_eq!(canonical, query(&reversed));
        let mut p = Profile {
            seed,
            ..Profile::default()
        };
        p.discoveries = canonical.keys().cloned().collect();
        let g = Game::new(p);
        let reload =
            Game::new(serde_json::from_str(&serde_json::to_string(g.profile()).unwrap()).unwrap());
        assert_eq!(
            serde_json::to_value(g.profile()).unwrap(),
            serde_json::to_value(reload.profile()).unwrap()
        );
        assert_eq!(canonical, query(&reversed));
    }
}

#[test]
fn actual_streamed_body_radii_and_belt_birth_fit_reserved_envelopes() {
    let mut maximum = 0.0_f32;
    for geology in 0..4 {
        for style in 0..4 {
            let seed = (geology << 11) | (style << 5);
            let body = crate::pixel_assets::asteroid(seed);
            let radius = crate::pixel_assets::radius(&body, 1.0); // Real maximum streamed scale.
            maximum = maximum.max(radius);
            assert!(radius <= BELT_BODY_PADDING);
        }
    }
    let seed = 17;
    for s in sites_near(seed, Vec2::new(6000.0, 6000.0)) {
        let belt = site_belt(seed, &s).unwrap();
        let center = Vec2::from_array(s.position);
        for angle in 0..64 {
            let direction = crate::forward(angle as f32 * TAU / 64.0);
            let p = center + direction * (belt.outer - 0.25);
            let birth = AsteroidBirth::at(seed, p);
            assert_eq!(birth.source, AsteroidSource::SiteBelt(s.grid));
            assert_eq!(birth.material_tier, belt.material_tier);
            assert_eq!(birth.orbit.unwrap().center, center);
            assert!(p.distance(center) + maximum <= s.envelope_radius());
        }
    }
    println!(
        "16 actual asteroid templates at maximum scale1: largest conservative radius {maximum}; reserve {BELT_BODY_PADDING}"
    );
}

#[test]
fn real_game_sanctuary_shields_and_arena_walls_use_canonical_radius() {
    let mut g = Game::new(Profile {
        seed: 17,
        ..Profile::default()
    });
    for s in sites_near(17, Vec2::new(6000.0, 6000.0)) {
        let center = Vec2::from_array(s.position);
        for direction in [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y] {
            g.move_player(center + direction * (s.radius - 1.0));
            g.invulnerability = 0.0;
            g.shield = 0.0;
            let before = g.hud().hull;
            g.hurt_player(9.0, None);
            assert_eq!(g.hud().hull, before);
            let body = crate::pixel_assets::asteroid(32);
            let radius = crate::pixel_assets::radius(&body, 1.0);
            let e = g.world.spawn((
                Motion::new(
                    center + direction * (s.radius + 100.0),
                    -direction * 20_000.0,
                    0.0,
                ),
                Actor {
                    kind: Kind::Asteroid,
                    health: 50.0,
                    max_health: 50.0,
                    radius,
                    scale: 1.0,
                    damage: 8.0,
                },
                body,
            ));
            g.integrate(0.05);
            g.enforce_asteroid_shields(true);
            let m = *g.world.get::<&Motion>(e).unwrap();
            for alpha in [0.0, 0.25, 0.5, 0.75, 1.0] {
                assert!(
                    m.previous.lerp(m.position, alpha).distance(center)
                        >= s.radius + radius - 0.005
                );
            }
            assert!(m.velocity.dot(direction) > 0.0);
            g.world.despawn(e).unwrap();
        }
    }
    for index in 0..8 {
        let s = crate::campaign::arena(17, index).unwrap();
        let center = Vec2::from_array(s.position);
        g.move_player(center);
        g.start_champion(index).unwrap();
        assert_eq!(g.arena.unwrap().radius, s.radius);
        assert!(!g.safe());
        let e = g.arena.unwrap().opponent;
        g.world.get::<&mut Motion>(e).unwrap().position = center + Vec2::X * (s.radius + 30.0);
        g.confine_arena();
        let a = *g.world.get::<&Actor>(e).unwrap();
        assert!(
            g.world.get::<&Motion>(e).unwrap().position.distance(center) + a.radius
                <= s.radius - 3.9
        );
        g.retreat_arena().unwrap();
        assert!(g.profile.campaign.patches.iter().all(|v| !v));
    }
    let mut qualified = Profile {
        seed: 17,
        ..Profile::default()
    };
    qualified.campaign.patches = [true; 8];
    qualified.campaign.blueprints = [true; 8];
    let mut royale = Game::new(qualified);
    let s = crate::campaign::tournament_site(17);
    royale.move_player(Vec2::from_array(s.position));
    royale.update_exploration(); // Actual arrival records the canonical discovery once.
    royale.start_tournament().unwrap();
    assert_eq!(royale.arena.unwrap().radius, s.radius);
    let before = serde_json::to_value(royale.profile()).unwrap();
    royale.tick(&InputFrame::default(), 1.0 / 60.0);
    assert_eq!(serde_json::to_value(royale.profile()).unwrap(), before);
}

#[test]
fn real_player_and_hostile_projectile_sweeps_respect_every_campaign_zone() {
    for index in 0..9 {
        let mut g = Game::new(Profile {
            seed: 17,
            ..Profile::default()
        });
        let s = if index < 8 {
            crate::campaign::arena(17, index).unwrap()
        } else {
            crate::campaign::tournament_site(17)
        };
        let center = Vec2::from_array(s.position);
        g.move_player(center);
        let before = serde_json::to_value(g.profile()).unwrap();
        let enemy = g.spawn_enemy(0, false, center, 1);
        g.enforce_actor_sanctuaries();
        let a = *g.world.get::<&Actor>(enemy).unwrap();
        assert!(
            g.world
                .get::<&Motion>(enemy)
                .unwrap()
                .position
                .distance(center)
                > s.radius + a.radius
        );
        g.world.despawn(enemy).unwrap();
        for angle in 0..16 {
            let direction = crate::forward(angle as f32 * TAU / 16.0);
            for player in [false, true] {
                let outside = direction * (s.radius + 50.0);
                g.spawn_bullet(center + outside, -direction * 430.0, 13.0, player, 1);
                let entity = g
                    .world
                    .query::<(crate::Entity, &crate::Bullet)>()
                    .iter()
                    .next()
                    .unwrap()
                    .0;
                {
                    let mut m = g.world.get::<&mut Motion>(entity).unwrap();
                    m.previous = center + outside;
                    m.position = center - outside;
                }
                g.resolve_collisions(1.0 / 60.0);
                assert!(
                    !g.world.contains(entity),
                    "{} actual projectile crossed protection",
                    s.id
                );
            }
        }
        assert_eq!(serde_json::to_value(g.profile()).unwrap(), before);
        assert!(!g.hud().dock); // Remote protection never grants Haven services.
    }
}
