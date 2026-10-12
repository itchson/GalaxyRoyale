//! Actual cached body / FrameBuilder / CameraView proofs, not GPU acceptance.
use super::*;
use engine_core::{CameraView, GpuTransform, RenderFrame};

const KINDS: [MotifKind; 3] = [MotifKind::Galaxy, MotifKind::Nebula, MotifKind::Supernova];
const CORNERS: [Vec2; 4] = [
    Vec2::new(-0.5, -0.5),
    Vec2::new(0.5, -0.5),
    Vec2::new(0.5, 0.5),
    Vec2::new(-0.5, 0.5),
];

fn parts(kind: MotifKind) -> [&'static Pair; 2] {
    match kind {
        MotifKind::Galaxy => [&ART.galaxy_arms, &ART.galaxy_core],
        MotifKind::Nebula => [&ART.nebula, &ART.nebula_thread],
        MotifKind::Supernova => [&ART.nova_shell, &ART.nova_core],
    }
}
fn scene(reduced: bool) -> SceneInput {
    SceneInput {
        seed: 17,
        camera: Vec2::ZERO,
        viewport: Vec2::new(533.0, 300.0),
        time: 18.0,
        reduced,
        reduced_motion: false,
    }
}
fn motif_input(kind: MotifKind) -> Motif {
    Motif {
        kind,
        origin: Vec2::ZERO,
        hash: 93,
        scale: 1.1,
    }
}
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
fn bounds(body: &PixelBody, scale: f32) -> (Vec2, Vec2) {
    body.cells().fold(
        (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
        |(min, max), (p, _)| {
            (
                min.min((p - Vec2::splat(0.5)) * scale),
                max.max((p + Vec2::splat(0.5)) * scale),
            )
        },
    )
}

#[test]
fn cached_half_unit_clusters_have_bounded_whole_motif_cost_and_same_physical_extents() {
    for kind in KINDS {
        let parts = parts(kind);
        let mut colors = std::collections::BTreeSet::new();
        let full = motif_cost(kind, false);
        let low = motif_cost(kind, true);
        assert!(full <= 2_200, "{kind:?} full={full}");
        assert!(low <= 1_100, "{kind:?} reduced={low}");
        assert!(
            full > low * 3,
            "{kind:?}: finer source must contain actual detail"
        );
        for pair in parts {
            eprintln!(
                "{kind:?} part {}x{}: full={}, reduced={}x{} / {}",
                pair.full.width,
                pair.full.height,
                pair.full.occupied_count(),
                pair.low.width,
                pair.low.height,
                pair.low.occupied_count()
            );
            assert_eq!(pair.full.width, pair.low.width * 2);
            assert_eq!(pair.full.height, pair.low.height * 2);
            let (min, max) = bounds(&pair.full, 0.5);
            let (low_min, low_max) = bounds(&pair.low, 1.0);
            assert!(
                (min - low_min).abs().max_element() <= 2.5,
                "{kind:?} bounds {min:?} / reduced {low_min:?}"
            );
            assert!(
                (max - low_max).abs().max_element() <= 2.5,
                "{kind:?} bounds {max:?} / reduced {low_max:?}"
            );
            colors.extend(pair.full.cells().map(|(_, c)| c));
        }
        assert!(
            (12..=24).contains(&colors.len()),
            "{kind:?} colors={}",
            colors.len()
        );
        eprintln!(
            "{kind:?}: full={full}, reduced={low}, cached filament colors={}, {} transforms",
            colors.len(),
            motif_transforms(kind)
        );
    }
    assert_eq!(
        [ART.galaxy_arms.full.width, ART.galaxy_arms.full.height],
        [192, 192]
    );
    assert_eq!([ART.nebula.full.width, ART.nebula.full.height], [160, 96]);
    assert_eq!(
        [ART.nova_shell.full.width, ART.nova_shell.full.height],
        [128, 128]
    );
}

#[test]
fn reduced_sampling_preserves_actual_dark_lanes_and_broken_rim() {
    for reduced in [false, true] {
        let scale = if reduced { 1.0 } else { 0.5 };
        assert!(
            ART.galaxy_arms
                .body(reduced)
                .cells()
                .all(|(p, _)| (p * scale).length() >= 5.0)
        );
        // Fine strands live in the gas field, rather than the superseded
        // uniform empty sinusoidal channel with two matching bright edges.
        assert!(
            ART.nebula
                .body(reduced)
                .cells()
                .all(|(p, _)| gas_density(p * scale) >= 0.14)
        );
        assert!(ART.nova_shell.body(reduced).cells().all(|(p, _)| {
            let p = p * scale;
            !(p.y.atan2(p.x) > -0.18 && p.y.atan2(p.x) < 0.32)
        }));
    }
}

#[test]
fn qa_galaxy_and_remnant_centers_use_actual_seed_selection_and_calm_group_geometry() {
    let seed = 871_238_449;
    for (grid, hash, kind, center) in [
        (
            [3, -4],
            2_591_400_577,
            MotifKind::Galaxy,
            [59725.008, -49541.473],
        ),
        (
            [-3, 1],
            1_888_154_543,
            MotifKind::Supernova,
            [-35332.82, 22642.145],
        ),
    ] {
        let m = motif(seed, grid).unwrap();
        assert_eq!(m.hash, hash);
        assert!(matches!(
            (m.kind, kind),
            (MotifKind::Galaxy, MotifKind::Galaxy) | (MotifKind::Supernova, MotifKind::Supernova)
        ));
        let camera = m.origin / MOTIF_PARALLAX;
        assert!((camera - Vec2::from_array(center)).abs().max_element() < 0.01);
        eprintln!(
            "QA {kind:?} grid{grid:?} h{hash}: WARP {} {}",
            camera.x, camera.y
        );
        for reduced in [false, true] {
            let mut states = Vec::new();
            for time in [0.0, 120.0, 901.0] {
                let mut frame = FrameBuilder::new();
                let scene = SceneInput {
                    seed,
                    camera,
                    time,
                    reduced,
                    viewport: Vec2::new(533.0, 300.0),
                    reduced_motion: false,
                };
                let required = motif_cost(kind, reduced);
                let mut remaining = required;
                assert_eq!(draw_motif(&mut frame, m, &scene, &mut remaining), required);
                assert_eq!(remaining, 0);
                let basis: Vec<_> = frame
                    .frame()
                    .batches
                    .iter()
                    .flat_map(|b| b.transforms.iter())
                    .map(|t| [t.a[2], t.a[3], t.b[0], t.b[1]])
                    .collect();
                assert_eq!(basis.len(), motif_transforms(kind));
                states.push(basis);
            }
            for later in &states[1..] {
                for (a, b) in states[0].iter().zip(later) {
                    for (x, y) in a.iter().zip(b) {
                        assert!(
                            (x - y).abs()
                                < if matches!(kind, MotifKind::Galaxy) {
                                    0.000001
                                } else {
                                    0.025
                                }
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn inclined_galaxy_has_connected_interior_mass_and_winding_dust_without_a_central_slit() {
    for reduced in [false, true] {
        let spacing = GALAXY_DISK_SPACING * if reduced { 2.0 } else { 1.0 };
        let body = ART.galaxy_disk.body(reduced);
        let mut interior = 0;
        let mut central_band = 0;
        let mut outer_disk = 0;
        for (p, c) in body.cells() {
            let p = p * spacing;
            assert!(galactic_disk_density(p) > 0.05);
            assert!((14..=51).contains(&c.to_le_bytes()[3]));
            interior += usize::from(p.length() < 20.0);
            central_band += usize::from(p.y.abs() < 3.5 && p.x.abs() < 16.0);
            outer_disk += usize::from(p.length() > 32.0);
        }
        assert!(interior > if reduced { 25 } else { 100 });
        assert!(central_band > if reduced { 4 } else { 20 });
        assert!(outer_disk > if reduced { 10 } else { 40 });
        assert!(galactic_disk_density(Vec2::ZERO) > 0.3);
        let off_arm_knots = ART
            .galaxy_arms
            .body(reduced)
            .cells()
            .filter(|(p, _)| galactic_field(*p * if reduced { 1.0 } else { 0.5 }).arm < 0.1)
            .count();
        assert!(off_arm_knots > if reduced { 2 } else { 8 });
        eprintln!(
            "Galaxy low{reduced}: soft {} / knots {} / bulge {} / off-arm {off_arm_knots}",
            body.occupied_count(),
            ART.galaxy_arms.body(reduced).occupied_count(),
            ART.galaxy_core.body(reduced).occupied_count()
        );
        assert!(motif_cost(MotifKind::Galaxy, reduced) <= if reduced { 1100 } else { 2200 });
    }
}

#[test]
#[ignore = "explicit bounded native galaxy packet proof"]
fn export_actual_galaxy_revision_packets() {
    let output = std::path::PathBuf::from(
        std::env::var_os("GALAXY_PROOF_DIR").expect("explicit owned output path"),
    );
    assert!(output.is_absolute());
    std::fs::create_dir_all(&output).unwrap();
    let seed = 871_238_449;
    let m = motif(seed, [3, -4]).unwrap();
    assert_eq!(m.kind, MotifKind::Galaxy);
    let camera = m.origin / MOTIF_PARALLAX;
    for reduced in [false, true] {
        for time in [0.0, 120.0] {
            let mut frame = FrameBuilder::new();
            let count = draw_motif(
                &mut frame,
                m,
                &SceneInput {
                    seed,
                    camera,
                    time,
                    ..scene(reduced)
                },
                &mut 8_000,
            );
            let views: Vec<_> = [0.6, 1.0, 1.6].into_iter().map(|zoom| {
                let view = CameraView::new(camera, Vec2::new(256.0, 180.0), zoom).unwrap();
                let cells: Vec<_> = frame.frame().batches.iter().flat_map(|b| b.pixels.iter().map(|p| {
                    let t = b.transforms[p.transform_slot()];
                    let corners: Vec<_> = CORNERS.iter().map(|c| view.world_to_screen(t.point(Vec2::from_array(p.local) + *c)).to_array()).collect();
                    serde_json::json!({"corners":corners,"rgba":p.color.to_le_bytes(),"material":[t.b[2],t.b[3]]})
                })).collect();
                serde_json::json!({"zoom":zoom,"cells":cells})
            }).collect();
            let packet = serde_json::json!({"seed":seed,"grid":[3,-4],"camera":camera.to_array(),"reduced":reduced,"time":time,"quads":count,"transforms":motif_transforms(m.kind),"views":views,"scope":"actual native FrameBuilder/CameraView cells; offline material approximation, not GPU or browser acceptance"});
            std::fs::write(
                output.join(format!("galaxy-low{reduced}-t{time}.json")),
                serde_json::to_vec(&packet).unwrap(),
            )
            .unwrap();
            eprintln!("Actual seeded galaxy low{reduced} t{time}: {count} cells / 3 transforms");
        }
    }
}

#[test]
fn actual_fine_cells_compose_reflection_shear_and_fractional_zoom_without_minimum_geometry() {
    let parent = AffineTransform {
        position: Vec2::new(17.0, -9.0),
        basis_x: Vec2::new(-1.3, 0.2),
        basis_y: Vec2::new(0.4, 0.75),
    };
    for reduced in [false, true] {
        let scale = if reduced { 1.0 } else { 0.5 };
        let child = Transform {
            position: Vec2::new(7.0, -10.0),
            rotation: -0.37,
            scale: Vec2::new(scale, scale * 0.7),
        }
        .affine();
        let body = ART.nebula.body(reduced);
        let style = CellStyle {
            time: 18.0,
            phase: 0.7,
            cell_scale: scale,
            tint: TINTS[1],
            alpha: 0.8,
            curl: 0.14,
        };
        let mut frame = FrameBuilder::new();
        draw_part(&mut frame, body, parent.compose(child), style);
        let batch = &frame.frame().batches[0];
        assert_eq!(batch.transforms.len(), 1);
        assert_eq!(frame.frame().pixel_count, body.occupied_count());
        let t = batch.transforms[0];
        assert_eq!(t.b[2..], [0.0, 0.0]);
        for (actual, (source, _)) in batch.pixels.iter().zip(body.cells()) {
            let local = Vec2::from_array(actual.local);
            assert!((local - source).abs().max_element() * scale <= 0.141);
            for corner in CORNERS {
                let expected = parent.point(child.point(local + corner));
                assert!(t.point(local + corner).distance(expected) < 0.00002);
            }
        }
        for zoom in [0.6, 1.0, 1.6] {
            let view = CameraView::new(Vec2::ZERO, Vec2::new(848.0, 390.0), zoom).unwrap();
            let pixel = &batch.pixels[0];
            let local = Vec2::from_array(pixel.local);
            let width = view
                .world_to_screen(t.point(local + Vec2::X * 0.5))
                .distance(view.world_to_screen(t.point(local - Vec2::X * 0.5)));
            let expected = parent.vector(child.vector(Vec2::X)).length() * zoom;
            assert!((width - expected).abs() < 0.0001);
            if !reduced && zoom == 0.6 {
                assert!(width < 0.5);
            }
        }
    }
}

#[test]
fn actual_motifs_keep_small_continuous_motion_and_freeze_under_reduced_motion() {
    for kind in KINDS {
        for reduced in [false, true] {
            let mut a = FrameBuilder::new();
            let mut b = FrameBuilder::new();
            let s = SceneInput {
                time: 20.0,
                ..scene(reduced)
            };
            draw_motif(&mut a, motif_input(kind), &s, &mut 8_000);
            draw_motif(
                &mut b,
                motif_input(kind),
                &SceneInput { time: 20.001, ..s },
                &mut 8_000,
            );
            assert_eq!(a.frame().pixel_count, b.frame().pixel_count);
            for (x, y) in packets(a.frame()).iter().zip(packets(b.frame())) {
                let delta =
                    x.2.point(Vec2::from_array(x.0))
                        .distance(y.2.point(Vec2::from_array(y.0)));
                assert!(delta < 0.002, "{kind:?} point jump={delta}");
                assert!(
                    x.1.to_le_bytes()
                        .iter()
                        .zip(y.1.to_le_bytes())
                        .all(|(a, b)| a.abs_diff(b) <= 1)
                );
            }
            let mut frozen = FrameBuilder::new();
            let mut later = FrameBuilder::new();
            draw_motif(
                &mut frozen,
                motif_input(kind),
                &SceneInput {
                    reduced_motion: true,
                    ..s
                },
                &mut 8_000,
            );
            draw_motif(
                &mut later,
                motif_input(kind),
                &SceneInput {
                    time: 901.0,
                    reduced_motion: true,
                    ..s
                },
                &mut 8_000,
            );
            assert_eq!(packets(frozen.frame()), packets(later.frame()));
        }
    }
}

#[test]
fn atomic_whole_motif_admission_preserves_prior_packets_at_transform_boundary() {
    for kind in KINDS {
        for reduced in [false, true] {
            let required = motif_cost(kind, reduced);
            let mut frame = FrameBuilder::new();
            for i in 0..127 {
                frame.push_cells_affine_material(
                    Transform {
                        position: Vec2::new(i as f32, 0.0),
                        ..Transform::default()
                    }
                    .affine(),
                    PixelMaterial::default(),
                    [(Vec2::ZERO, 0xffaabbcc)],
                );
            }
            let before = packets(frame.frame());
            let mut short = required - 1;
            assert_eq!(
                draw_motif(&mut frame, motif_input(kind), &scene(reduced), &mut short),
                0
            );
            assert_eq!(packets(frame.frame()), before);
            assert_eq!(short, required - 1);
            let mut exact = required;
            assert_eq!(
                draw_motif(&mut frame, motif_input(kind), &scene(reduced), &mut exact),
                required
            );
            assert_eq!(exact, 0);
            assert_eq!(frame.frame().pixel_count, required + 127);
            assert_eq!(frame.frame().batches.len(), 2);
            assert_eq!(frame.frame().batches[0].transforms.len(), 128);
            assert_eq!(
                frame.frame().batches[1].transforms.len(),
                motif_transforms(kind) - 1
            );
            assert!(frame.frame().batches.iter().all(|b| {
                b.pixels
                    .iter()
                    .all(|p| (p.transform_slot()) < b.transforms.len())
            }));
        }
    }
}

#[test]
fn far_streamed_motifs_keep_local_detail_finite_and_inside_conservative_culling_bound() {
    for camera in [
        Vec2::new(999_000.0, -999_000.0),
        Vec2::new(-999_000.0, 999_000.0),
    ] {
        for kind in KINDS {
            let m = Motif {
                origin: camera * MOTIF_PARALLAX,
                ..motif_input(kind)
            };
            for reduced in [false, true] {
                for zoom in [0.6, 1.6] {
                    let logical = Vec2::new(848.0, 390.0);
                    let view = CameraView::new(camera, logical, zoom).unwrap();
                    let mut f = FrameBuilder::new();
                    let s = SceneInput {
                        camera,
                        viewport: logical / zoom,
                        ..scene(reduced)
                    };
                    assert!(draw_motif(&mut f, m, &s, &mut 8_000) > 0);
                    for batch in &f.frame().batches {
                        for p in &batch.pixels {
                            let t = view.relative_transform(batch.transforms[p.transform_slot()]);
                            for corner in CORNERS {
                                let q = t.point(Vec2::from_array(p.local) + corner);
                                assert!(q.is_finite() && q.length() < MOTIF_RADIUS);
                            }
                            let width = t.point(Vec2::from_array(p.local) + Vec2::X)
                                - t.point(Vec2::from_array(p.local));
                            if t.b[3] < 0.0 {
                                // Actual authored soft volume footprints are larger than
                                // the hard half-unit filaments; neither is a raster clamp.
                                let spacing = if t.b[3] == -1.0 {
                                    GALAXY_DISK_SPACING * 2.4
                                } else if (t.b[3] + 1.04).abs() < 0.00001 {
                                    VEIL_SPACING * 2.7
                                } else {
                                    BILLOW_SPACING * 2.4
                                };
                                let expected = spacing * if reduced { 2.0 } else { 1.0 } * m.scale;
                                assert!((width.length() - expected).abs() < 0.0001);
                            } else {
                                assert!(width.length() > 0.3 && width.length() < 1.3);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
#[ignore = "explicit native software evidence export; requires owned output directory"]
fn export_actual_fine_cosmic_packets_for_software_contact_proof() {
    let output = std::path::PathBuf::from(
        std::env::var_os("COSMIC_DETAIL_PROOF_DIR").expect("explicit owned output path"),
    );
    assert!(output.is_absolute());
    std::fs::create_dir_all(&output).unwrap();
    for (index, kind) in KINDS.into_iter().enumerate() {
        for reduced in [false, true] {
            for time in [0.0, 18.0] {
                let mut frame = FrameBuilder::new();
                let required = draw_motif(
                    &mut frame,
                    motif_input(kind),
                    &SceneInput {
                        time,
                        ..scene(reduced)
                    },
                    &mut 8_000,
                );
                let views: Vec<_> = [0.6, 1.0, 1.6].into_iter().map(|zoom| {
                    let view = CameraView::new(Vec2::ZERO, Vec2::new(192.0, 144.0), zoom).unwrap();
                    let cells: Vec<_> = frame.frame().batches.iter().flat_map(|b| b.pixels.iter().map(|p| {
                        let t = b.transforms[p.transform_slot()];
                        let corners: Vec<_> = CORNERS.iter().map(|c| view.world_to_screen(t.point(Vec2::from_array(p.local) + *c)).to_array()).collect();
                        serde_json::json!({"corners": corners, "rgba": p.color.to_le_bytes(), "material": [t.b[2],t.b[3]]})
                    })).collect();
                    serde_json::json!({"zoom": zoom, "cells": cells})
                }).collect();
                let data = serde_json::json!({"kind": format!("{kind:?}"), "reduced": reduced, "time": time, "quads": required, "transforms": motif_transforms(kind), "scope": "actual native FrameBuilder/CameraView packets including material tags; offline approximation, no GPU or threat contrast acceptance", "views": views});
                std::fs::write(
                    output.join(format!("motif-{index}-reduced-{reduced}-time-{time}.json")),
                    serde_json::to_vec(&data).unwrap(),
                )
                .unwrap();
            }
        }
    }
}
