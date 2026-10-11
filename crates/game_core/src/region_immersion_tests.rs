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
                .map(|p| (p.local, p.color, b.transforms[p.transform as usize]))
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
