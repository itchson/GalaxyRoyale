//! Native position/admission proofs; contact images remain software, not GPU.
use super::*;
use engine_core::{CameraView, GpuTransform, RenderFrame};

const SEED: u32 = 871_238_449;
const CORNERS: [Vec2; 4] = [
    Vec2::new(-0.5, -0.5),
    Vec2::new(0.5, -0.5),
    Vec2::new(0.5, 0.5),
    Vec2::new(-0.5, 0.5),
];

fn examples() -> Vec<([i32; 2], Vec2, [Vec2; 7], u32)> {
    (-16..=16)
        .flat_map(|y| {
            (-16..=16).filter_map(move |x| {
                stellar_system(SEED, [x, y]).map(|(o, n, h)| ([x, y], o, n, h))
            })
        })
        .collect()
}

fn shape(nodes: &[Vec2; 7]) -> (f32, f32) {
    let center = nodes.iter().copied().sum::<Vec2>() / 7.0;
    let (mut xx, mut xy, mut yy, mut diameter) = (0.0, 0.0, 0.0, 0.0_f32);
    for (i, p) in nodes.iter().enumerate() {
        let q = *p - center;
        xx += q.x * q.x;
        xy += q.x * q.y;
        yy += q.y * q.y;
        for other in &nodes[..i] {
            diameter = diameter.max(p.distance(*other));
        }
    }
    let root = ((xx - yy).powi(2) + 4.0 * xy * xy).sqrt();
    ((xx + yy - root) / (xx + yy + root), diameter)
}

fn packet(frame: &RenderFrame) -> Vec<([f32; 2], u32, GpuTransform)> {
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
fn seeded_neighborhoods_include_broad_clusters_and_different_physical_spreads() {
    let systems = examples();
    assert!(systems.len() >= 48);
    let (mut broad, mut smallest, mut largest) = (0, f32::INFINITY, 0.0_f32);
    for (grid, origin, nodes, h) in &systems {
        assert_eq!(stellar_system(SEED, *grid), Some((*origin, *nodes, *h)));
        assert!(origin.is_finite());
        let (ratio, diameter) = shape(nodes);
        assert!(ratio.is_finite() && diameter.is_finite());
        broad += usize::from(ratio > 0.35);
        smallest = smallest.min(diameter);
        largest = largest.max(diameter);
        for (i, p) in nodes.iter().enumerate() {
            assert!(p.is_finite() && p.length() + 1.15 < 90.0);
            assert!(nodes[..i].iter().all(|other| p.distance(*other) > 1.15));
        }
    }
    eprintln!(
        "actual {} systems: {broad} broad, diameter {smallest}..{largest}",
        systems.len()
    );
    // Rotation-independent covariance catches seven stars still confined to
    // a strip, even if someone shuffles their order. Spread catches clones
    // which differ only by angle. These are visual properties, not exact nodes.
    assert!(
        broad * 4 >= systems.len(),
        "neighborhoods remain narrow strips"
    );
    assert!(
        largest > smallest * 1.7,
        "every neighborhood keeps the same spread"
    );
}

#[test]
fn whole_seven_star_packets_keep_quality_motion_parallax_and_atomic_admission() {
    for (_, origin, nodes, h) in examples().into_iter().take(12) {
        let camera = origin / SYSTEM_PARALLAX;
        let mut reference = None;
        for reduced in [false, true] {
            for time in [0.0, 901.0] {
                let scene = SceneInput {
                    seed: SEED,
                    camera,
                    viewport: Vec2::new(533.0, 300.0),
                    time,
                    reduced,
                    reduced_motion: true,
                };
                let mut f = FrameBuilder::new();
                f.push_pixel(Vec2::new(300.0, 200.0), 1.0, 0xffaabbcc);
                let before = packet(f.frame());
                assert_eq!(draw_system(&mut f, &scene, origin, nodes, h, 6), 0);
                assert_eq!(packet(f.frame()), before);
                assert_eq!(draw_system(&mut f, &scene, origin, nodes, h, 7), 7);
                assert_eq!(f.frame().pixel_count, 8);
                let actual = packet(f.frame());
                if let Some(ref expected) = reference {
                    assert_eq!(&actual, expected);
                } else {
                    reference = Some(actual);
                }
                for delta in [Vec2::ZERO, Vec2::new(11.0, -9.0)] {
                    let moved_camera = camera + delta;
                    let view = CameraView::new(moved_camera, Vec2::new(533.0, 300.0), 1.6).unwrap();
                    let mut moved = FrameBuilder::new();
                    assert_eq!(
                        draw_system(
                            &mut moved,
                            &SceneInput {
                                camera: moved_camera,
                                ..scene
                            },
                            origin,
                            nodes,
                            h,
                            7
                        ),
                        7
                    );
                    assert_eq!(
                        moved
                            .frame()
                            .batches
                            .iter()
                            .map(|b| b.transforms.len())
                            .sum::<usize>(),
                        1
                    );
                    for (i, cell) in moved.frame().batches[0].pixels.iter().enumerate() {
                        let native = moved.frame().batches[0].transforms[cell.transform_slot()];
                        let relative = view.relative_transform(native);
                        for corner in CORNERS {
                            let actual =
                                relative.point(Vec2::from_array(cell.local) + corner) * 1.6;
                            let expected =
                                (nodes[i] - delta * SYSTEM_PARALLAX + corner * 1.15) * 1.6;
                            assert!(actual.is_finite() && actual.distance(expected) < 0.025);
                        }
                    }
                }
            }
        }
    }
}

#[test]
#[ignore = "explicit native stellar packet writer; requires owned absolute output"]
fn export_actual_seeded_stellar_neighborhood_packets() {
    let output =
        std::path::PathBuf::from(std::env::var_os("STELLAR_PROOF_DIR").expect("owned output"));
    assert!(output.is_absolute());
    std::fs::create_dir_all(&output).unwrap();
    for (index, (grid, origin, nodes, h)) in examples().into_iter().take(6).enumerate() {
        let camera = origin / SYSTEM_PARALLAX;
        for reduced in [false, true] {
            let scene = SceneInput {
                seed: SEED,
                camera,
                viewport: Vec2::splat(300.0),
                time: 0.0,
                reduced,
                reduced_motion: true,
            };
            let mut f = FrameBuilder::new();
            assert_eq!(draw_system(&mut f, &scene, origin, nodes, h, 7), 7);
            let views: Vec<_> = [0.6, 1.0, 1.6].into_iter().map(|zoom| {
                let view = CameraView::new(camera, Vec2::splat(300.0), zoom).unwrap();
                let cells: Vec<_> = f.frame().batches.iter().flat_map(|b| b.pixels.iter().map(|p| {
                    let t = view.relative_transform(b.transforms[p.transform_slot()]);
                    let corners: Vec<_> = CORNERS.iter().map(|c| (t.point(Vec2::from_array(p.local) + *c) * zoom + Vec2::splat(150.0)).to_array()).collect();
                    serde_json::json!({"corners": corners, "rgba": p.color.to_le_bytes(), "material": [t.b[2], t.b[3]]})
                })).collect();
                serde_json::json!({"zoom": zoom, "cells": cells})
            }).collect();
            let (ratio, diameter) = shape(&nodes);
            let data = serde_json::json!({
                "seed": SEED, "grid": grid, "hash": h, "origin": origin.to_array(), "camera": camera.to_array(),
                "nodes": nodes.map(|p| p.to_array()), "reduced": reduced, "reduced_motion": true,
                "time": 0, "quads": 7, "transforms": 1, "covariance_ratio": ratio, "diameter": diameter,
                "scope": "actual native FrameBuilder/CameraView packet; flat software review only, no GPU/physical behavior acceptance",
                "views": views,
            });
            std::fs::write(
                output.join(format!("system-{index}-reduced-{reduced}.json")),
                serde_json::to_vec(&data).unwrap(),
            )
            .unwrap();
        }
    }
}
