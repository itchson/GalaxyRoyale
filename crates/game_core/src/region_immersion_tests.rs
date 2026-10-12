//! Actual cached surface and packet proofs; not browser visual acceptance.
use super::*;
use engine_core::{CameraView, GpuTransform, RenderFrame};

fn packets(frame: &RenderFrame) -> Vec<([f32; 2], u32, GpuTransform)> {
    frame
        .batches
        .iter()
        .flat_map(|b| {
            b.pixels
                .iter()
                .map(|p| (p.local, p.color, b.transforms[p.transform_slot()]))
        })
        .collect()
}

#[test]
fn night_side_stays_dark_for_every_surface_and_biome_without_erasing_sphere() {
    let light = [-0.52, -0.32, 0.79];
    for kind in 0..3 {
        assert_eq!(PLANETS[kind].occupied_count(), 4404);
        let mut nights = 0;
        let mut days = 0;
        for (p, shade) in PLANETS[kind].cells() {
            let z = (37.5_f32.powi(2) - p.length_squared()).sqrt();
            let diffuse = [p.x, p.y, z]
                .into_iter()
                .zip(light)
                .map(|(a, b)| a * b)
                .sum::<f32>()
                / 37.5;
            for palette in PALETTES {
                let rgb = planet_shade(kind, palette, shade, 230).to_le_bytes();
                assert_eq!(rgb[3], 230);
                assert!(rgb[..3].iter().all(|&c| c <= 160));
                if diffuse <= 0.0 {
                    assert_eq!(shade, 0, "unlit terrain must not brighten night");
                    assert!(rgb[..3].iter().all(|&c| c <= 8));
                }
            }
            nights += usize::from(diffuse <= 0.0);
            days += usize::from(diffuse > 0.8 && shade != 0);
        }
        assert!(nights > 300 && days > 300);
    }
}

#[test]
fn ring_shadow_does_not_black_out_the_unoccluded_right_tip() {
    let tone = |x: f32, y: f32| -> u32 {
        let shade = RINGS[usize::from(y >= 0.0)]
            .cells()
            .find(|(p, _)| *p == Vec2::new(x, y))
            .expect("probe must be an actual retained ring cell")
            .1;
        [0, 2, 4, 6, 1, 3, 5, 7][shade as usize]
    };
    // These two neighboring cells sit outside the sphere's projected shadow.
    // The old x>17/y<5 rectangle blackened only the first one, visibly severing
    // the right ring. Light/shadow must not jump at that unrelated horizontal.
    let above = tone(54.5, 4.5);
    let below = tone(54.5, 5.5);
    assert!(above >= 2 && below >= 2, "unoccluded tip must retain alloy");
    assert!(above.abs_diff(below) <= 1);
    // Same latitude, opposite sides of the sphere: the ray toward the existing
    // upper-left/front light clears the left ring and intersects from the right.
    assert!(tone(-34.5, -14.5) >= 2);
    assert_eq!(tone(34.5, -14.5), 0);
}

#[test]
fn actual_ring_material_obeys_light_ray_occlusion_with_intermediate_shades() {
    let mut umbra = 0;
    let mut clear = 0;
    let mut intermediate = 0;
    for (p, shade) in RINGS.iter().flat_map(PixelBody::cells) {
        let origin = [p.x, p.y, p.y * ((58.0_f32 / 18.0).powi(2) - 1.0).sqrt()];
        // Independent bounded ray sampling, rather than the production
        // perpendicular-distance formula. Wide guards exclude penumbra cells.
        let closest = (0..=240)
            .map(|step| {
                let t = step as f32 * 0.5;
                origin
                    .into_iter()
                    .zip([-0.52, -0.32, 0.79])
                    .map(|(p, light)| (p + t * light).powi(2))
                    .sum::<f32>()
            })
            .fold(f32::INFINITY, f32::min);
        let tone = [0, 2, 4, 6, 1, 3, 5, 7][shade as usize];
        if closest < 31.0_f32.powi(2) {
            assert_eq!(tone, 0, "actual shadowed material cell at {p:?}");
            umbra += 1;
        } else if closest > 43.0_f32.powi(2) {
            assert!(tone >= 2, "unoccluded material cell at {p:?}");
            clear += 1;
        }
        intermediate += usize::from(tone == 1);
    }
    assert!(
        umbra > 0 && clear > 0,
        "both guarded ray cases must be exercised"
    );
    assert!(
        intermediate > 4,
        "penumbra must survive eight-shade authoring"
    );
}

#[test]
fn remnant_keeps_open_filament_geometry_and_calm_fixed_footprint() {
    assert_eq!(PLANETS[3].occupied_count(), 323);
    assert_eq!(LOW_PLANETS[3].occupied_count(), 141);
    for reduced in [false, true] {
        let mut states = Vec::new();
        for pulse in [0.0, 0.5, 1.0] {
            let mut frame = FrameBuilder::new();
            let mut scene = Scenery {
                frame: &mut frame,
                camera: Vec2::ZERO,
                half: Vec2::splat(300.0),
                remaining: 6000,
            };
            let count = scene.nova_group(Vec2::new(17.0, -21.0), pulse, reduced);
            assert_eq!(count, if reduced { 141 } else { 323 });
            let packet = packets(frame.frame());
            assert_eq!(frame.frame().batches[0].transforms.len(), 1);
            assert!(
                packet
                    .iter()
                    .all(|(_, c, t)| (65..=73).contains(&c.to_le_bytes()[3])
                        && t.b[2..] == [0.0, 0.0])
            );
            states.push(packet);
        }
        for later in &states[1..] {
            assert!(
                states[0]
                    .iter()
                    .zip(later)
                    .all(|(a, b)| a.0 == b.0 && a.2 == b.2)
            );
        }
        let source = if reduced {
            &LOW_PLANETS[3]
        } else {
            &PLANETS[3]
        };
        assert!(source.occupied_count() < source.width * source.height / 8);
        // The reduced cache may broaden a sampled edge by one coarse cell;
        // its central cavity remains actually absent, rather than black fill.
        assert!(!source.is_occupied(source.width / 2, source.height / 2));
    }
}

#[test]
fn qa_seed_targets_are_native_selected_and_encode_parallax_once() {
    let seed = 871_238_449;
    for (layer, grid, expected_hash, expected_camera) in [
        (1_u32, [-1, 1], 2_928_467_406, [-1243.7285, 5_313.427]),
        (0, [-1, 1], 2_130_583_060, [-12_094.438, 26_339.611]),
        (0, [0, 1], 885_732_694, [4949.4165, 30685.72]),
    ] {
        let h = hash(seed, grid, layer.wrapping_mul(7919) ^ 0x706c_616e);
        assert_eq!(h, expected_hash);
        let (spacing, factor) = if layer == 1 {
            (860.0, 0.22)
        } else {
            (1800.0, 0.075)
        };
        let origin = (Vec2::new(grid[0] as f32, grid[1] as f32) + Vec2::new(unit(h), unit(mix(h))))
            * spacing;
        let camera = origin / factor;
        assert!(
            (camera - Vec2::from_array(expected_camera))
                .abs()
                .max_element()
                < 0.01
        );
        assert!((origin + camera * (1.0 - factor) - camera).length() < 0.004);
        if layer == 1 {
            assert!(h.is_multiple_of(3) && h & 1 == 0);
            assert_eq!(mix(h ^ 12) % 3, 1);
            assert_eq!(at(seed, camera).kind, 0);
        } else if grid == [0, 1] {
            assert!(h.is_multiple_of(11));
        } else {
            assert!(h.is_multiple_of(5)); // historical dotted ribbon location
        }
        eprintln!(
            "QA layer{layer} grid{grid:?} h{h}: WARP {} {}",
            camera.x, camera.y
        );
    }
}

#[test]
fn actual_planets_keep_fractional_reflected_cell_corners_and_bounded_group_cost() {
    for kind in 0..3 {
        for reduced in [false, true] {
            let mut frame = FrameBuilder::new();
            let camera = Vec2::new(999_000.25, -999_000.25);
            let mut scene = Scenery {
                frame: &mut frame,
                camera,
                half: Vec2::splat(300.0),
                remaining: 6000,
            };
            let transform = Transform {
                position: camera,
                rotation: 0.37,
                scale: Vec2::new(-0.32, 0.61),
            };
            let count = scene.planet_group(kind, transform, PALETTES[2], reduced);
            assert!(count > 0 && count <= if reduced { 1364 } else { 5004 });
            for zoom in [0.6, 1.0, 1.6] {
                let view = CameraView::new(camera, Vec2::new(533.0, 300.0), zoom).unwrap();
                for (local, _, t) in packets(frame.frame()) {
                    let t = view.relative_transform(t);
                    let local = Vec2::from_array(local);
                    for corner in [
                        Vec2::splat(-0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::splat(0.5),
                        Vec2::new(-0.5, 0.5),
                    ] {
                        let p = t.point(local + corner);
                        assert!(p.is_finite() && p.length() < 50.0);
                    }
                    let actual_width = (t.point(local + Vec2::X) - t.point(local)).length();
                    assert!((actual_width - 0.32 * if reduced { 2.0 } else { 1.0 }).abs() < 0.0001);
                }
            }
        }
    }
}

#[test]
#[ignore = "explicit actual packet export; requires owned output directory"]
fn export_actual_region_surfaces_for_software_contact() {
    let output = std::path::PathBuf::from(std::env::var_os("REGION_IMMERSION_PROOF_DIR").unwrap());
    assert!(output.is_absolute());
    std::fs::create_dir_all(&output).unwrap();
    for kind in 0..4 {
        for reduced in [false, true] {
            let mut frame = FrameBuilder::new();
            let mut scene = Scenery {
                frame: &mut frame,
                camera: Vec2::ZERO,
                half: Vec2::splat(200.0),
                remaining: 6000,
            };
            let count = if kind == 3 {
                scene.nova_group(Vec2::ZERO, 0.5, reduced)
            } else {
                scene.planet_group(
                    kind,
                    Transform {
                        scale: Vec2::splat(0.72),
                        rotation: -0.2,
                        ..Transform::default()
                    },
                    PALETTES[0],
                    reduced,
                )
            };
            let views: Vec<_> = [0.6, 1.0, 1.6]
                .into_iter()
                .map(|zoom| {
                    let view = CameraView::new(Vec2::ZERO, Vec2::new(192.0, 144.0), zoom).unwrap();
                    let cells: Vec<_> = packets(frame.frame())
                        .into_iter()
                        .map(|(local, c, t)| {
                            let corners: Vec<_> = [
                                Vec2::splat(-0.5),
                                Vec2::new(0.5, -0.5),
                                Vec2::splat(0.5),
                                Vec2::new(-0.5, 0.5),
                            ]
                            .into_iter()
                            .map(|corner| {
                                view.world_to_screen(t.point(Vec2::from_array(local) + corner))
                                    .to_array()
                            })
                            .collect();
                            serde_json::json!({"corners":corners,"rgba":c.to_le_bytes()})
                        })
                        .collect();
                    serde_json::json!({"zoom":zoom,"cells":cells})
                })
                .collect();
            let data = serde_json::json!({"kind":format!("Region{kind}"),"reduced":reduced,"time":0.0,
                "quads":count,"transforms":frame.frame().batches.iter().map(|b|b.transforms.len()).sum::<usize>(),
                "scope":"posed actual cached Scenery helper/FrameBuilder/CameraView; not full Game, GPU or browser", "views":views});
            std::fs::write(
                output.join(format!("region-{kind}-reduced-{reduced}.json")),
                serde_json::to_vec(&data).unwrap(),
            )
            .unwrap();
        }
    }
}

const RING_POSES: [([i32; 2], u32, f32); 3] = [
    ([-1, 1], 2_928_467_406, -0.4049601),
    ([-3, -1], 2_525_719_458, 0.0098429),
    ([-4, 0], 2_113_593_648, -0.0085866),
];
type RingPacket = ([f32; 2], u32, GpuTransform);

// Select the actual region extraction, rather than posing an angle in a helper.
fn ring_pose(grid: [i32; 2], reduced: bool, time: f32) -> (Vec2, Transform, Vec<RingPacket>) {
    let seed = 871_238_449;
    let h = hash(seed, grid, 7919 ^ 0x706c_616e);
    assert!(h.is_multiple_of(3) && h & 1 == 0);
    assert_eq!(mix(h ^ 12) % 3, 1);
    let origin =
        (Vec2::new(grid[0] as f32, grid[1] as f32) + Vec2::new(unit(h), unit(mix(h)))) * 860.0;
    let camera = origin / 0.22 + Vec2::new(400.0, 0.0);
    let transform = Transform {
        position: origin + camera * 0.78,
        rotation: -0.2,
        scale: Vec2::splat(0.32 + unit(mix(h ^ 37)) * 0.4),
    };
    let mut frame = FrameBuilder::new();
    background_with_density(
        &mut frame,
        seed,
        camera,
        Vec2::new(533.0, 300.0) / 1.6,
        time,
        reduced,
    );
    assert!(frame.frame().pixel_count <= if reduced { 3000 } else { 6000 });
    let group = packets(frame.frame())
        .into_iter()
        .filter(|(_, _, t)| Vec2::new(t.a[0], t.a[1]).distance(transform.position) < 0.01)
        .collect::<Vec<_>>();
    assert_eq!(group.len(), if reduced { 1364 } else { 5004 });
    (camera, transform, group)
}

#[test]
fn ringed_births_use_distinct_seeded_angles_in_both_density_modes() {
    let mut angles = Vec::new();
    for (grid, expected_hash, expected_angle) in RING_POSES {
        assert_eq!(hash(871_238_449, grid, 7919 ^ 0x706c_616e), expected_hash);
        let mut paired = Vec::new();
        for reduced in [false, true] {
            let (_, _, group) = ring_pose(grid, reduced, 20.0);
            let actual = group[0].2.a[3].atan2(group[0].2.a[2]);
            assert!(
                (actual - expected_angle).abs() < 0.000001,
                "grid{grid:?}: {actual}"
            );
            assert!((-0.45..=0.05).contains(&actual));
            assert!(group.iter().all(|(_, _, t)| *t == group[0].2));
            // Independent extraction at a later cosmetic time retains all cells,
            // materials and the complete group's affine record.
            assert_eq!(group, ring_pose(grid, reduced, 91.0).2);
            paired.push(actual);
        }
        assert_eq!(paired[0], paired[1]);
        angles.push(paired[0]);
    }
    assert!((angles[0] - angles[1]).abs() > 0.4);
    assert!((angles[1] - angles[2]).abs() > 0.015);
}

#[test]
fn ringed_birth_rotation_preserves_ordered_materials_and_original_bounds() {
    for (grid, _, _) in RING_POSES {
        for reduced in [false, true] {
            let (camera, original, group) = ring_pose(grid, reduced, 20.0);
            let mut frame = FrameBuilder::new();
            let mut scene = Scenery {
                frame: &mut frame,
                camera,
                half: Vec2::splat(300.0),
                remaining: 6000,
            };
            let birth = at(871_238_449, (original.position - camera * 0.78) / 0.22);
            assert_ne!(birth.kind, 4);
            scene.planet_group(1, original, birth.palette, reduced);
            let old = packets(frame.frame());
            assert_eq!(old.len(), group.len());
            assert!(
                old.iter()
                    .zip(&group)
                    .all(|(a, b)| a.0 == b.0 && a.1 == b.1)
            );
            let ring_count = if reduced { 106 } else { 300 };
            let body_count = if reduced { 1152 } else { 4404 };
            for (i, (local, color, transform)) in group.iter().enumerate() {
                let alpha = if i < ring_count {
                    120
                } else if i < ring_count + body_count {
                    230
                } else {
                    145
                };
                assert_eq!(color.to_le_bytes()[3], alpha);
                assert_eq!(transform.b[2..], [0.0, 0.0]);
                let bound = Vec2::new(64.0, 24.0).length() * original.scale.x;
                for zoom in [0.6, 1.0, 1.6] {
                    let view = CameraView::new(camera, Vec2::new(533.0, 300.0), zoom).unwrap();
                    let relative = view.relative_transform(*transform);
                    let center = Vec2::new(relative.a[0], relative.a[1]);
                    for corner in [
                        Vec2::splat(-0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::splat(0.5),
                        Vec2::new(-0.5, 0.5),
                    ] {
                        let world = relative.point(Vec2::from_array(*local) + corner);
                        assert!(world.is_finite() && world.distance(center) <= bound);
                        let screen = world * zoom + Vec2::new(533.0, 300.0) * 0.5;
                        assert!(screen.is_finite());
                    }
                    let local = Vec2::from_array(*local);
                    let width = relative
                        .point(local + Vec2::X)
                        .distance(relative.point(local))
                        * zoom;
                    let expected = original.scale.x * if reduced { 2.0 } else { 1.0 } * zoom;
                    assert!((width - expected).abs() < 0.00003);
                }
            }
        }
    }
}

#[test]
fn ringed_births_keep_atomic_existing_group_admission() {
    for (grid, _, _) in RING_POSES {
        for reduced in [false, true] {
            let (camera, mut transform, group) = ring_pose(grid, reduced, 20.0);
            transform.rotation = group[0].2.a[3].atan2(group[0].2.a[2]);
            let required = if reduced { 1364 } else { 5004 };
            for budget in [required - 1, required] {
                let mut frame = FrameBuilder::new();
                let mut scene = Scenery {
                    frame: &mut frame,
                    camera,
                    half: Vec2::splat(300.0),
                    remaining: budget,
                };
                let admitted = scene.planet_group(1, transform, PALETTES[0], reduced);
                if budget < required {
                    assert_eq!(admitted, 0);
                    assert_eq!(scene.remaining, budget);
                    assert_eq!(frame.frame().pixel_count, 0);
                    assert!(frame.frame().batches.is_empty());
                } else {
                    assert_eq!(admitted, required);
                    assert_eq!(scene.remaining, 0);
                    assert_eq!(frame.frame().pixel_count, required);
                    assert_eq!(frame.frame().batches[0].transforms.len(), 3);
                }
            }
        }
    }
}

#[test]
#[ignore = "explicit native seeded-group export; requires owned output directory"]
fn export_actual_seeded_ringed_groups() {
    let output = std::path::PathBuf::from(std::env::var_os("RING_ORIENTATION_PROOF_DIR").unwrap());
    assert!(output.is_absolute());
    std::fs::create_dir_all(&output).unwrap();
    for (grid, h, _) in RING_POSES {
        for reduced in [false, true] {
            let (camera, original, group) = ring_pose(grid, reduced, 20.0);
            let cells: Vec<_> = group.into_iter().map(|(local, color, t)| {
                serde_json::json!({"local":local,"rgba":color.to_le_bytes(),"a":t.a,"b":t.b})
            }).collect();
            let data = serde_json::json!({"seed":871238449,"hash":h,"grid":grid,"camera":camera.to_array(),"position":original.position.to_array(),"scale":original.scale.x,"reduced":reduced,"time":20.0,"cells":cells,"scope":"actual background_with_density/FrameBuilder seeded group at533x300/zoom1.6; not Game/GPU/browser"});
            std::fs::write(
                output.join(format!("ring-{h}-reduced-{reduced}.json")),
                serde_json::to_vec(&data).unwrap(),
            )
            .unwrap();
        }
    }
}
