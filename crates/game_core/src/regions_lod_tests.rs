use super::*;
use engine_core::{CameraView, GpuTransform, RenderFrame};

type Packet = ([f32; 2], u32, GpuTransform);
fn packets(frame: &RenderFrame) -> Vec<Packet> {
    frame
        .batches
        .iter()
        .flat_map(|batch| {
            batch
                .pixels
                .iter()
                .map(|p| (p.local, p.color, batch.transforms[p.transform as usize]))
        })
        .collect()
}
fn validate(frame: &RenderFrame) {
    assert_eq!(
        frame.pixel_count,
        frame.batches.iter().map(|b| b.pixels.len()).sum::<usize>()
    );
    for batch in &frame.batches {
        assert!(batch.transforms.len() <= 128);
        assert!(batch.transforms.iter().all(|t| {
            t.a.into_iter().chain(t.b).all(f32::is_finite)
                && t.b[2] == 0.0
                && [0.0, -1.0].contains(&t.b[3])
        }));
        assert!(batch.pixels.iter().all(|p| {
            p.local.into_iter().all(f32::is_finite)
                && (p.transform as usize) < batch.transforms.len()
        }));
    }
}
fn group(kind: usize, reduced: bool, transform: Transform, budget: usize) -> (RenderFrame, usize) {
    let mut frame = FrameBuilder::new();
    let mut scene = Scenery {
        frame: &mut frame,
        camera: transform.position,
        half: Vec2::new(533.0, 300.0) * 0.5,
        remaining: budget,
    };
    scene.planet_group(kind, transform, PALETTES[2], reduced);
    let remaining = scene.remaining;
    (frame.finish(), remaining)
}
fn group_cost(kind: usize, reduced: bool) -> usize {
    if kind == 1 {
        if reduced { 1364 } else { 5004 }
    } else if reduced {
        1152
    } else {
        4404
    }
}

#[test]
fn low_cache_keeps_source_coverage_existing_shades_and_authored_identity() {
    for (full, low) in PLANETS
        .iter()
        .zip(LOW_PLANETS.iter())
        .chain(RINGS.iter().zip(LOW_RINGS.iter()))
    {
        assert_eq!((low.width * 2, low.height * 2), (full.width, full.height));
        for y in 0..low.height {
            for x in 0..low.width {
                let samples: Vec<_> = (0..4)
                    .filter_map(|i| full.color_at(x * 2 + (i & 1), y * 2 + (i >> 1)))
                    .collect();
                assert_eq!(low.is_occupied(x, y), !samples.is_empty());
                if let Some(shade) = low.color_at(x, y) {
                    let count = samples.iter().filter(|&&s| s == shade).count();
                    assert!(count > 0);
                    assert!(samples.iter().all(|s| {
                        samples.iter().filter(|&&other| other == *s).count() <= count
                    }));
                }
            }
        }
    }
    for body in LOW_PLANETS.iter() {
        assert_eq!(body.occupied_count(), 1152);
    }
    for ring in LOW_RINGS.iter() {
        assert_eq!(ring.occupied_count(), 106);
    }
    for body in LOW_PLANETS.iter().take(3) {
        let shades: std::collections::BTreeSet<_> = body.cells().map(|(_, c)| c).collect();
        assert_eq!(shades, (0..8).collect());
    }
    for i in 0..3 {
        for j in i + 1..3 {
            assert_ne!(
                LOW_PLANETS[i].cells().collect::<Vec<_>>(),
                LOW_PLANETS[j].cells().collect::<Vec<_>>()
            );
        }
    }
    let tied = PixelBody::from_ascii(&["ab", "ba"], &[('a', 3), ('b', 7)]);
    assert_eq!(half_body(&tied).color_at(0, 0), Some(3));
}

#[test]
fn exact_group_edges_keep_back_surface_front_order_without_orphan_rings() {
    let original = Transform {
        position: Vec2::new(103.0, -51.0),
        rotation: -0.2,
        scale: Vec2::splat(0.67),
    };
    for reduced in [false, true] {
        for kind in 0..4 {
            let cost = group_cost(kind, reduced);
            let (rejected, remaining) = group(kind, reduced, original, cost - 1);
            assert_eq!(remaining, cost - 1);
            assert_eq!(rejected.pixel_count, 0);
            assert!(rejected.batches.is_empty());
            let (accepted, remaining) = group(kind, reduced, original, cost);
            assert_eq!(remaining, 0);
            assert_eq!(accepted.pixel_count, cost);
            validate(&accepted);
            let mut expected = FrameBuilder::new();
            let scale = if reduced { 2.0 } else { 1.0 };
            let transform = Transform {
                scale: original.scale * scale,
                ..original
            };
            let planets = if reduced { &*LOW_PLANETS } else { &*PLANETS };
            if kind == 1 {
                let rings = if reduced { &*LOW_RINGS } else { &*RINGS };
                expected.push_body_colored(&rings[0], transform, |_, c| {
                    palette_shade(PALETTES[2], c, 150)
                });
            }
            expected.push_body_colored(&planets[kind], transform, |_, c| {
                palette_shade(PALETTES[2], c, 190)
            });
            if kind == 1 {
                let rings = if reduced { &*LOW_RINGS } else { &*RINGS };
                expected.push_body_colored(&rings[1], transform, |_, c| {
                    palette_shade(PALETTES[2], c, 190)
                });
            }
            assert_eq!(packets(&accepted), packets(expected.frame()));
        }
    }
}

#[test]
fn rejected_group_keeps_existing_packets_and_batch_rollover_keeps_materials() {
    let dot = PixelBody::from_ascii(&["X"], &[('X', 0xff_abcdef)]);
    let mut frame = FrameBuilder::new();
    for i in 0..127 {
        frame.push_body(
            &dot,
            Transform {
                position: Vec2::new(i as f32, 30.0),
                ..Transform::default()
            },
        );
    }
    let before = packets(frame.frame());
    let mut scene = Scenery {
        frame: &mut frame,
        camera: Vec2::ZERO,
        half: Vec2::splat(400.0),
        remaining: 1363,
    };
    assert_eq!(
        scene.planet_group(1, Transform::default(), PALETTES[0], true),
        0
    );
    assert_eq!(scene.remaining, 1363);
    assert_eq!(packets(scene.frame.frame()), before);
    scene.remaining = 1364;
    assert_eq!(
        scene.planet_group(1, Transform::default(), PALETTES[0], true),
        1364
    );
    assert_eq!(scene.remaining, 0);
    assert_eq!(scene.frame.frame().pixel_count, 127 + 1364);
    assert_eq!(scene.frame.frame().batches.len(), 2);
    validate(scene.frame.frame());
}

#[test]
fn nova_core_and_visible_spokes_are_atomic_with_exact_edges() {
    for reduced in [false, true] {
        for (position, half) in [
            (Vec2::ZERO, Vec2::splat(100.0)),
            (Vec2::new(105.0, 0.0), Vec2::splat(100.0)),
        ] {
            let pulse = 0.65;
            let size = 0.8 + pulse * 0.7;
            let sparks = (0..24)
                .filter(|&i| {
                    let angle = i as f32 * TAU / 24.0;
                    let p = position + Vec2::new(angle.cos(), angle.sin()) * (15.0 + pulse * 19.0);
                    let d = p.abs();
                    d.x <= half.x + size && d.y <= half.y + size
                })
                .count();
            let cost = if reduced { 1152 } else { 4404 } + sparks;
            for budget in [cost - 1, cost, cost + 7] {
                let mut frame = FrameBuilder::new();
                let mut scene = Scenery {
                    frame: &mut frame,
                    camera: Vec2::ZERO,
                    half,
                    remaining: budget,
                };
                let written = scene.nova_group(position, pulse, reduced);
                if budget < cost {
                    assert_eq!(written, 0);
                    assert_eq!(scene.remaining, budget);
                    assert_eq!(scene.frame.frame().pixel_count, 0);
                    assert!(scene.frame.frame().batches.is_empty());
                } else {
                    assert_eq!(written, cost);
                    assert_eq!(scene.remaining, budget - cost);
                    assert_eq!(scene.frame.frame().pixel_count, cost);
                    validate(scene.frame.frame());
                }
            }
        }
    }
}

#[test]
fn fitting_full_nova_keeps_existing_body_and_spoke_packets() {
    let position = Vec2::new(24.0, -17.0);
    for pulse in [0.0, 0.3, 1.0] {
        let mut actual = FrameBuilder::new();
        let mut scene = Scenery {
            frame: &mut actual,
            camera: Vec2::ZERO,
            half: Vec2::splat(200.0),
            remaining: 6000,
        };
        assert_eq!(scene.nova_group(position, pulse, false), 4428);
        let mut expected = FrameBuilder::new();
        expected.push_body_colored(
            &PLANETS[3],
            Transform {
                position,
                scale: Vec2::splat(0.32 + pulse * 0.2),
                ..Transform::default()
            },
            |_, c| palette_shade(PALETTES[4], c, (90.0 + pulse * 100.0) as u8),
        );
        for i in 0..24 {
            let angle = i as f32 * TAU / 24.0;
            let direction = Vec2::new(angle.cos(), angle.sin());
            expected.push_pixel(
                position + direction * (15.0 + pulse * 19.0),
                0.8 + pulse * 0.7,
                0x80_8fcaff,
            );
        }
        assert_eq!(packets(actual.frame()), packets(expected.frame()));
    }
}

#[test]
fn actual_seeded_reduced_scenes_retain_each_ordinary_same_art_landmark() {
    for biome in [0, 1, 2, 3, 5] {
        for kind in 0..3 {
            let mut retained = false;
            'search: for y in -32..=32 {
                for x in -32..=32 {
                    let seed = 0x4752_4f59;
                    let grid = [x, y];
                    let h = hash(seed, grid, 7919 ^ 0x706c_616e);
                    if !h.is_multiple_of(3) || h & 1 != 0 || (mix(h ^ 12) % 3) as usize != kind {
                        continue;
                    }
                    let origin =
                        (Vec2::new(x as f32, y as f32) + Vec2::new(unit(h), unit(mix(h)))) * 860.0;
                    let camera = origin / 0.22;
                    if at(seed, camera).kind != biome {
                        continue;
                    }
                    let transform = Transform {
                        position: origin + camera * 0.78,
                        rotation: if kind == 1 {
                            -0.2
                        } else {
                            unit(mix(h ^ 93)) * 0.6
                        },
                        scale: Vec2::splat(0.32 + unit(mix(h ^ 37)) * 0.4),
                    };
                    let mut expected = FrameBuilder::new();
                    Scenery {
                        frame: &mut expected,
                        camera,
                        half: Vec2::new(533.0, 300.0) / 3.2,
                        remaining: 3000,
                    }
                    .planet_group(
                        kind,
                        transform,
                        at(seed, camera).palette,
                        true,
                    );
                    let mut actual = FrameBuilder::new();
                    background_with_density(
                        &mut actual,
                        seed,
                        camera,
                        Vec2::new(533.0, 300.0) / 1.6,
                        0.0,
                        true,
                    );
                    assert!(actual.frame().pixel_count <= 3000);
                    validate(actual.frame());
                    let wanted = packets(expected.frame());
                    if packets(actual.frame())
                        .windows(wanted.len())
                        .any(|w| w == wanted)
                    {
                        retained = true;
                        break 'search;
                    }
                }
            }
            assert!(retained, "missing retained biome {biome} kind {kind}");
        }
    }
}

#[test]
fn reduced_motion_zero_time_scenes_are_deterministic_and_keep_caps() {
    for reduced in [false, true] {
        for camera in [Vec2::ZERO, Vec2::new(999000.0, -999000.0)] {
            for viewport in [Vec2::new(533.0, 300.0) / 0.6, Vec2::splat(8192.0)] {
                let mut a = FrameBuilder::new();
                let mut b = FrameBuilder::new();
                // Game forwards cosmetic_time() == 0 under Reduced Motion.
                background_with_density(&mut a, 17, camera, viewport, 0.0, reduced);
                background_with_density(&mut b, 17, camera, viewport, 0.0, reduced);
                assert_eq!(packets(a.frame()), packets(b.frame()));
                assert!(a.frame().pixel_count <= if reduced { 3000 } else { 6000 });
                validate(a.frame());
            }
        }
    }
}

#[test]
fn projected_group_corners_keep_nominal_center_and_conservative_culling_extent() {
    for camera in [Vec2::ZERO, Vec2::splat(999000.3), Vec2::splat(-999000.3)] {
        for rotation in [-0.2, 0.0, 0.6] {
            let transform = Transform {
                position: camera,
                rotation,
                scale: Vec2::splat(0.72),
            };
            for reduced in [false, true] {
                let (frame, _) = group(1, reduced, transform, 6000);
                let view = CameraView::new(camera, Vec2::new(533.0, 300.0), 0.6).unwrap();
                for batch in &frame.batches {
                    for p in &batch.pixels {
                        let t = view.relative_transform(batch.transforms[p.transform as usize]);
                        assert_eq!([t.a[0], t.a[1]], [0.0, 0.0]);
                        for corner in [
                            Vec2::splat(-0.5),
                            Vec2::new(0.5, -0.5),
                            Vec2::splat(0.5),
                            Vec2::new(-0.5, 0.5),
                        ] {
                            let local = Vec2::from_array(p.local) + corner;
                            let relative = Vec2::new(
                                t.a[2] * local.x + t.a[3] * local.y,
                                t.b[0] * local.x + t.b[1] * local.y,
                            );
                            assert!(relative.is_finite());
                            // Every actual corner remains far inside the unchanged
                            // 135-world-unit group visibility margin at max scale.
                            assert!(relative.length() < 46.0);
                        }
                    }
                }
                validate(&frame);
            }
        }
    }
}
