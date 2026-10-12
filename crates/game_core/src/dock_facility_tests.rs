use super::*;
use crate::{Game, Motion, Profile, SAFE_RADIUS};
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
fn draw(
    id: usize,
    time: f32,
    theme: Option<crate::identity::Corporation>,
    parent: AffineTransform,
) -> RenderFrame {
    let mut frame = FrameBuilder::new();
    assert_eq!(
        append(&mut frame, id, parent, time, theme, &mut Budget::default()),
        Some(ART[id].body.occupied_count())
    );
    frame.finish()
}
#[test]
fn actual_home_hit_centers_and_radius_preserve_all_gameplay_and_profile_gates() {
    let game = Game::new(Profile::default());
    let before = serde_json::to_value(game.profile()).unwrap();
    for (id, center) in CENTERS.iter().enumerate() {
        assert!(Vec2::from_array(*center).length() + HIT_RADIUS < SAFE_RADIUS);
        assert_eq!(game.dock_facility_at(*center), Some(id as u8));
        assert_eq!(
            game.dock_facility_at([center[0] + 24.0, center[1]]),
            Some(id as u8)
        );
        assert_eq!(game.dock_facility_at([center[0] + 24.01, center[1]]), None);
    }
    for point in [[0.0, 0.0], [f32::NAN, 0.0], [0.0, f32::INFINITY]] {
        assert_eq!(game.dock_facility_at(point), None);
    }
    game.world.get::<&mut Motion>(game.player).unwrap().position = Vec2::new(240.01, 0.0);
    for center in CENTERS {
        assert_eq!(game.dock_facility_at(center), None);
    }
    assert_eq!(serde_json::to_value(game.profile()).unwrap(), before);
}
#[test]
fn five_distinct_detailed_fields_partition_cells_once_and_fit_full_hit_disk() {
    let palette: serde_json::Value = serde_json::from_str(include_str!(
        "../../../assets/pipeline/pan-coop-dock/native256/pack.json"
    ))
    .unwrap();
    let colors: std::collections::HashSet<u32> = palette["palette"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u32)
        .collect();
    let mut silhouettes = std::collections::HashSet::new();
    for asset in ART.iter() {
        assert_eq!((asset.body.width, asset.body.height), (96, 96));
        assert!(
            asset.body.occupied_count() > 2000 && asset.body.occupied_count() <= HUB_CELL_BUDGET
        );
        let occupied: Vec<_> = asset.body.cells_indexed().map(|(i, _, _)| i).collect();
        assert!(silhouettes.insert(occupied.clone()));
        let mut grouped: Vec<_> = asset
            .parts
            .iter()
            .flat_map(|p| p.iter().map(|&i| usize::from(i)))
            .collect();
        grouped.sort_unstable();
        assert_eq!(grouped, occupied);
        assert!(asset.parts.iter().all(|p| p.len() >= 40));
        let shades: std::collections::HashSet<_> = asset.body.cells().map(|(_, c)| c).collect();
        assert!(shades.len() >= 16);
        for (p, color) in asset.body.cells() {
            assert!(colors.contains(&color));
            assert_eq!(color >> 24, 255);
            for corner in [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::splat(0.5),
                Vec2::new(-0.5, 0.5),
            ] {
                assert!(((p + corner) * CELL_SCALE).length() < HIT_RADIUS);
            }
        }
    }
}
#[test]
fn corporation_and_animation_touch_only_disjoint_signal_indices() {
    for id in 0..5 {
        let base = packets(&draw(id, 0.0, None, AffineTransform::default()));
        for (time, theme) in [
            (1.2, None),
            (0.0, Some(crate::identity::Corporation::Renko)),
            (0.0, Some(crate::identity::Corporation::Hugo)),
        ] {
            let other = packets(&draw(id, time, theme, AffineTransform::default()));
            assert_eq!(base.len(), other.len());
            let signal_offset = ART[id].parts[..3].iter().map(|p| p.len()).sum::<usize>();
            assert!(
                base[signal_offset..]
                    .iter()
                    .zip(&other[signal_offset..])
                    .any(|(a, b)| a.1 != b.1)
            );
            for (index, (a, b)) in base.iter().zip(other).enumerate() {
                assert_eq!(a.0, b.0);
                assert_eq!(a.2, b.2);
                assert_eq!(a.1 >> 24, b.1 >> 24);
                if index < signal_offset {
                    assert_eq!(a.1, b.1);
                }
            }
        }
        assert_eq!(
            base,
            packets(&draw(id, 0.0, None, AffineTransform::default()))
        );
    }
}
#[test]
fn actual_full_scene_cost_is_bounded_and_far_camera_culls_without_cloning_cache() {
    let addresses: Vec<_> = ART
        .iter()
        .flat_map(|a| a.parts.iter().map(|p| p.as_ptr()))
        .collect();
    let mut f = FrameBuilder::new();
    render(&mut f, Vec2::ZERO, Vec2::new(533.0, 400.0), 0.0, None);
    let frame = f.finish();
    let count: usize = ART.iter().map(|a| a.body.occupied_count()).sum();
    assert_eq!(frame.pixel_count, count);
    assert!(count <= SCENE_CELL_BUDGET);
    assert_eq!(
        frame
            .batches
            .iter()
            .map(|b| b.transforms.len())
            .sum::<usize>(),
        20
    );
    let mut none = FrameBuilder::new();
    render(
        &mut none,
        Vec2::splat(999000.0),
        Vec2::new(533.0, 400.0),
        0.0,
        None,
    );
    assert_eq!(none.frame().pixel_count, 0);
    let mut one = FrameBuilder::new();
    render(
        &mut one,
        Vec2::from_array(CENTERS[0]),
        Vec2::splat(16.0),
        0.0,
        None,
    );
    assert_eq!(one.frame().pixel_count, ART[0].body.occupied_count());
    assert_eq!(
        addresses,
        ART.iter()
            .flat_map(|a| a.parts.iter().map(|p| p.as_ptr()))
            .collect::<Vec<_>>()
    );
    println!(
        "HAVEN cells={:?} total={count} transforms=20 cache_payload_bytes={}",
        ART.each_ref().map(|a| a.body.occupied_count()),
        5 * (SIDE * SIDE * 4 + (SIDE * SIDE).div_ceil(64) * 8) + count * 2
    );
}
#[test]
fn atomic_quota_and_nonfinite_invalid_rejections_leave_frame_and_budget_unchanged() {
    for (parent, time, cells, transforms) in [
        (
            AffineTransform::default(),
            0.0,
            ART[0].body.occupied_count() - 1,
            4,
        ),
        (AffineTransform::default(), 0.0, 5000, 3),
        (AffineTransform::default(), f32::NAN, 5000, 4),
        (
            AffineTransform {
                basis_x: Vec2::ZERO,
                ..Default::default()
            },
            0.0,
            5000,
            4,
        ),
        (
            AffineTransform {
                position: Vec2::splat(f32::INFINITY),
                ..Default::default()
            },
            0.0,
            5000,
            4,
        ),
        (
            AffineTransform {
                position: Vec2::splat(1_100_100.0),
                ..Default::default()
            },
            0.0,
            5000,
            4,
        ),
    ] {
        let mut frame = FrameBuilder::new();
        frame.push_pixel(Vec2::ZERO, 1.0, 0xffffffff);
        let before = packets(frame.frame());
        let mut budget = Budget { cells, transforms };
        assert_eq!(append(&mut frame, 0, parent, time, None, &mut budget), None);
        assert_eq!(packets(frame.frame()), before);
        assert_eq!((budget.cells, budget.transforms), (cells, transforms));
    }
}
#[test]
fn affine_rotation_reflection_nested_shear_and_fractional_camera_keep_all_original_cell_corners() {
    let parent = Transform {
        position: Vec2::new(127.25, -35.625),
        rotation: 0.7,
        scale: Vec2::new(0.6, 0.25),
    };
    let child = Transform {
        rotation: -0.4,
        scale: Vec2::new(-0.8, 1.3),
        ..Default::default()
    };
    let affine = parent.compose_affine(child);
    for id in 0..5 {
        let plain = packets(&draw(id, 0.0, None, AffineTransform::default()));
        let nested = packets(&draw(id, 0.0, None, affine));
        for (a, b) in plain.iter().zip(nested) {
            assert_eq!(a.0, b.0);
            assert_eq!(a.1, b.1);
            for corner in [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::splat(0.5),
                Vec2::new(-0.5, 0.5),
            ] {
                let local = Vec2::from_array(b.0) + corner;
                let expected = parent.point(child.point(local));
                let actual = b.2.point(local);
                assert!(actual.distance(expected) < 0.0001);
                for zoom in [0.6, 1.0, 1.6] {
                    let view =
                        CameraView::new(Vec2::new(120.125, -33.375), Vec2::new(853.0, 480.0), zoom)
                            .unwrap();
                    let projected = view.world_to_screen(actual);
                    assert!(view.screen_to_world(projected).distance(actual) < 0.0001);
                    assert_eq!(view.relative_transform(b.2).a[2..], b.2.a[2..]);
                    assert_eq!(view.relative_transform(b.2).b, b.2.b);
                }
            }
        }
    }
}
#[test]
fn actual_game_reduced_motion_freezes_hub_extraction_without_profile_or_rng_changes() {
    let mut game = Game::new(Profile::default());
    game.set_reduced_motion(true);
    let before = serde_json::to_value(game.profile()).unwrap();
    let rng = game.rng;
    let mut direct = FrameBuilder::new();
    render(&mut direct, Vec2::ZERO, game.viewport, 0.0, None);
    let expected = packets(direct.frame());
    for time in [0.0, 1.2, 50.0] {
        game.time = time;
        assert!(
            packets(&game.render(1.0))
                .windows(expected.len())
                .any(|w| w == expected)
        );
    }
    assert_eq!(game.rng, rng);
    assert_eq!(serde_json::to_value(game.profile()).unwrap(), before);
}
#[test]
#[ignore = "explicit native facility packet exporter requires GALAXY_HAVEN_FACILITY_EVIDENCE"]
fn export_actual_haven_facility_packets() {
    let path = std::path::PathBuf::from(
        std::env::var("GALAXY_HAVEN_FACILITY_EVIDENCE").expect("explicit directory"),
    );
    std::fs::create_dir_all(&path).unwrap();
    let mut views = Vec::new();
    for theme in [
        None,
        Some(crate::identity::Corporation::Renko),
        Some(crate::identity::Corporation::Hugo),
    ] {
        for id in 0..5 {
            for time in if theme.is_none() {
                vec![0.0, 1.2]
            } else {
                vec![0.0]
            } {
                let frame = draw(
                    id,
                    time,
                    theme,
                    Transform {
                        scale: Vec2::splat(CELL_SCALE),
                        ..Default::default()
                    }
                    .affine(),
                );
                views.push(serde_json::json!({"id":id,"theme":theme.map_or("pan", |t| if t == crate::identity::Corporation::Renko {"renko"} else {"hugo"}),"time":time,"cells":packets(&frame).into_iter().map(|(local,rgba,t)|serde_json::json!({"local":local,"rgba":rgba,"a":t.a,"b":t.b})).collect::<Vec<_>>() }));
            }
        }
    }
    std::fs::write(path.join("native-packets.json"), serde_json::to_string(&serde_json::json!({"version":1,"scope":"Actual native FrameBuilder inputs; flat enlarged software contact proof, no GPU/browser/performance claim", "views":views})).unwrap()).unwrap();
}
