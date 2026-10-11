use super::*;
use engine_core::{CameraView, GpuTransform, RenderFrame, Transform};

fn input() -> DrawInput {
    DrawInput {
        parent: Transform {
            scale: Vec2::splat(0.7),
            ..Transform::default()
        }
        .affine(),
        material: crate::ACTOR_MATERIAL,
        time: 0.0,
        reduced_motion: true,
    }
}

fn packet(frame: &RenderFrame) -> Vec<(Vec2, u32, GpuTransform)> {
    frame
        .batches
        .iter()
        .flat_map(|b| {
            b.pixels.iter().map(|p| {
                (
                    Vec2::from_array(p.local),
                    p.color,
                    b.transforms[p.transform as usize],
                )
            })
        })
        .collect()
}

fn draw(body: &PixelBody, corporation: Corporation, input: DrawInput) -> (RenderFrame, Stats) {
    let mut frame = FrameBuilder::new();
    let mut budget = Budget::default();
    let stats = append(
        &mut frame,
        body,
        corporation,
        input,
        &mut budget,
        |_, c| c,
        |_, c| c,
    )
    .unwrap();
    assert_eq!(
        budget.pixels,
        PLAYER_CELL_BUDGET - stats.body_cells - stats.overlay_cells
    );
    assert_eq!(budget.transforms, 2 - stats.transforms);
    (frame.finish(), stats)
}

#[test]
fn exact_intact_counts_and_authored_landmarks_keep_borrowed_cache_and_two_transforms() {
    let assets = ASSETS.as_ref().unwrap();
    for (i, (coarse, fine)) in [(662, 1205), (688, 1176), (1040, 1740)]
        .into_iter()
        .enumerate()
    {
        let corp = corporation(i);
        let body = starter_body(corp);
        let (frame, stats) = draw(body, corp, input());
        assert_eq!(
            stats,
            Stats {
                body_cells: coarse,
                overlay_cells: fine,
                transforms: 2
            }
        );
        assert_eq!(frame.pixel_count, coarse + fine);
        assert_eq!(
            frame
                .batches
                .iter()
                .map(|b| b.transforms.len())
                .sum::<usize>(),
            2
        );
        assert_eq!(assets[i].cells.len(), fine);
        assert!(std::ptr::eq(assets, ASSETS.as_ref().unwrap()));
        let cells = packet(&frame);
        for (x, y) in [(47, 30), (47, 54), (35, 58), (35, 59)] {
            let local = Vec2::new(x as f32 + 0.5 - 48.0, y as f32 + 0.5 - 48.0);
            let source = assets[i]
                .cells
                .iter()
                .find(|c| c.local == local)
                .expect("authored landmark in sparse overlay");
            assert!(
                cells[coarse..]
                    .iter()
                    .any(|(p, c, _)| *p == local && *c == source.color)
            );
        }
        assert_eq!(body.occupied_count(), coarse);
    }
}

#[test]
fn actual_projectile_contact_damage_removes_all_mapped_fine_cells_without_mutating_templates() {
    for i in 0..3 {
        let corp = corporation(i);
        let mut live = starter_body(corp).clone();
        let from = Vec2::new(-20.0, -8.0);
        let to = Vec2::new(20.0, 8.0);
        let hit = live
            .hit_segment(from, to)
            .expect("real occupied-cell contact");
        let removed = live.damage(hit, 3.5);
        assert!(!removed.is_empty());
        let (frame, stats) = draw(&live, corp, input());
        assert_eq!(
            stats.body_cells,
            starter_body(corp).occupied_count() - removed.len()
        );
        let assets = ASSETS.as_ref().unwrap();
        let expected = assets[i]
            .cells
            .iter()
            .filter(|c| live.is_occupied(c.coarse_x, c.coarse_y))
            .count();
        assert_eq!(stats.overlay_cells, expected);
        assert!(expected < assets[i].cells.len());
        for (p, _, _) in packet(&frame).into_iter().skip(stats.body_cells) {
            let x = ((p.x + 48.0).floor() as usize) / 2;
            let y = ((p.y + 48.0).floor() as usize) / 2;
            assert!(
                live.is_occupied(x, y),
                "destroyed coarse cell refilled by detail"
            );
        }
        let pristine = starter_body(corp);
        assert!(
            removed
                .iter()
                .all(|(p, _)| pristine.hit_segment(*p, *p).is_some())
        );
    }
}

#[test]
fn reflected_sheared_rotated_nested_affines_and_all_zoom_endpoints_preserve_all_cell_corners() {
    for i in 0..3 {
        let corp = corporation(i);
        for rotation in [0.0, 0.7, 2.3] {
            let parent = Transform {
                position: Vec2::new(131.0, -47.0),
                rotation: 0.4,
                scale: Vec2::new(-1.7, 0.6),
            }
            .compose_affine(Transform {
                position: Vec2::new(4.0, 3.0),
                rotation,
                scale: Vec2::new(0.7, 1.1),
            });
            let (frame, stats) = draw(starter_body(corp), corp, DrawInput { parent, ..input() });
            let cells = packet(&frame);
            for zoom in [0.6, 1.0, 1.6] {
                let camera =
                    CameraView::new(Vec2::new(120.0, -40.0), Vec2::new(533.0, 300.0), zoom)
                        .unwrap();
                for (j, (local, _, t)) in cells.iter().enumerate() {
                    for offset in [
                        Vec2::new(-0.5, -0.5),
                        Vec2::new(-0.5, 0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::new(0.5, 0.5),
                    ] {
                        let p = *local + offset;
                        let expected = parent.point(if j < stats.body_cells { p } else { p * 0.5 });
                        let world = t.point(p);
                        assert!((world - expected).length() < 0.00004);
                        assert!(
                            (camera.world_to_screen(world) - camera.world_to_screen(expected))
                                .length()
                                < 0.00008
                        );
                        assert_eq!(t.b[2], crate::ACTOR_MATERIAL.light_response);
                        assert_eq!(t.b[3], crate::ACTOR_MATERIAL.emission);
                    }
                }
            }
        }
    }
}

#[test]
fn exact_pair_budget_and_alpha_zero_callbacks_count_every_submission_atomically() {
    let body = starter_body(Corporation::Hugo);
    for budget in [0, 1040, 2779] {
        let mut frame = FrameBuilder::new();
        frame.push_pixel(Vec2::ZERO, 1.0, 1);
        let before = format!("{:?}", frame.frame());
        let mut remaining = Budget {
            pixels: budget,
            transforms: 2,
        };
        let mut callbacks = 0;
        assert_eq!(
            append(
                &mut frame,
                body,
                Corporation::Hugo,
                input(),
                &mut remaining,
                |_, c| {
                    callbacks += 1;
                    c
                },
                |_, c| c
            ),
            Err(Reject::Budget)
        );
        assert_eq!(callbacks, 0);
        assert_eq!(
            remaining,
            Budget {
                pixels: budget,
                transforms: 2
            }
        );
        assert_eq!(format!("{:?}", frame.frame()), before);
    }
    let mut frame = FrameBuilder::new();
    let mut budget = Budget {
        pixels: 2780,
        transforms: 2,
    };
    let stats = append(
        &mut frame,
        body,
        Corporation::Hugo,
        input(),
        &mut budget,
        |_, c| c & 0xffffff,
        |_, c| c & 0xffffff,
    )
    .unwrap();
    assert_eq!(stats.body_cells + stats.overlay_cells, 2780);
    assert_eq!(frame.frame().pixel_count, 2780);
    assert_eq!(
        budget,
        Budget {
            pixels: 0,
            transforms: 0
        }
    );
    assert!(packet(frame.frame()).iter().all(|(_, c, _)| c >> 24 == 0));
}

#[test]
fn transform_table_boundary_splits_batches_without_partial_overlay_or_false_group_limit() {
    let mut frame = FrameBuilder::new();
    let tiny = PixelBody::from_ascii(&["x"], &[('x', 1)]);
    for i in 0..127 {
        frame.push_body(
            &tiny,
            Transform {
                position: Vec2::new(i as f32, 0.0),
                ..Transform::default()
            },
        );
    }
    let mut budget = Budget::default();
    let stats = append(
        &mut frame,
        starter_body(Corporation::Pan),
        Corporation::Pan,
        input(),
        &mut budget,
        |_, c| c,
        |_, c| c,
    )
    .unwrap();
    assert_eq!(stats.transforms, 2);
    assert_eq!(frame.frame().batches.len(), 2);
    assert_eq!(frame.frame().batches[0].transforms.len(), 128);
    assert_eq!(frame.frame().batches[1].transforms.len(), 1);
    assert_eq!(frame.frame().pixel_count, 127 + 1867);
    assert!(frame.frame().batches.iter().all(|b| {
        b.pixels
            .iter()
            .all(|p| (p.transform as usize) < b.transforms.len())
    }));
}

#[test]
fn reduced_motion_freezes_only_signal_rgb_and_host_damage_tint_keeps_geometry_and_alpha() {
    let corp = Corporation::Renko;
    let body = starter_body(corp);
    let (a, stats) = draw(body, corp, input());
    let (b, _) = draw(
        body,
        corp,
        DrawInput {
            time: 1000.0,
            ..input()
        },
    );
    assert_eq!(packet(&a), packet(&b));
    let (animated, _) = draw(
        body,
        corp,
        DrawInput {
            time: 0.4,
            reduced_motion: false,
            ..input()
        },
    );
    let static_cells = packet(&a);
    let animated_cells = packet(&animated);
    let art = &ASSETS.as_ref().unwrap()[1];
    let mut changes = 0;
    for (i, (original, current)) in static_cells.iter().zip(&animated_cells).enumerate() {
        assert_eq!(original.0, current.0);
        assert_eq!(original.2, current.2);
        assert_eq!(original.1 >> 24, current.1 >> 24);
        if original.1 != current.1 {
            assert!(i >= stats.body_cells && art.cells[i - stats.body_cells].signal);
            changes += 1;
        }
    }
    assert!(changes > 0 && changes <= 27);
    let mut tinted = FrameBuilder::new();
    let mut budget = Budget::default();
    let tint = |_, c| crate::player_presentation::tint(c, 20.0, 100.0, 0.1, 0.4, false);
    append(&mut tinted, body, corp, input(), &mut budget, tint, tint).unwrap();
    for ((p, c, t), (q, d, u)) in static_cells.into_iter().zip(packet(tinted.frame())) {
        assert_eq!((p, t), (q, u));
        assert_eq!(
            d,
            crate::player_presentation::tint(c, 20.0, 100.0, 0.1, 0.4, false)
        );
        assert_eq!(c >> 24, d >> 24);
    }
}

#[test]
fn invalid_grid_live_occupancy_or_nonfinite_corners_rejects_without_any_frame_mutation() {
    let mut wrong = PixelBody::from_ascii(&["x"], &[('x', 1)]);
    for parent in [
        input().parent,
        AffineTransform {
            position: Vec2::splat(f32::NAN),
            ..input().parent
        },
        AffineTransform {
            basis_x: Vec2::ZERO,
            ..input().parent
        },
        AffineTransform {
            position: Vec2::splat(f32::MAX),
            basis_x: Vec2::splat(f32::MAX),
            ..input().parent
        },
    ] {
        let mut frame = FrameBuilder::new();
        let mut budget = Budget::default();
        assert!(
            append(
                &mut frame,
                &wrong,
                Corporation::Pan,
                DrawInput { parent, ..input() },
                &mut budget,
                |_, c| c,
                |_, c| c
            )
            .is_err()
        );
        assert_eq!(frame.frame().pixel_count, 0);
        assert_eq!(budget, Budget::default());
        wrong = starter_body(Corporation::Pan).clone();
    }
    let full = PixelBody::from_ascii(&["x".repeat(48).as_str(); 48], &[('x', 1)]);
    let mut frame = FrameBuilder::new();
    let mut budget = Budget::default();
    assert_eq!(
        append(
            &mut frame,
            &full,
            Corporation::Pan,
            input(),
            &mut budget,
            |_, c| c,
            |_, c| c
        ),
        Err(Reject::IncompatibleBody)
    );
    assert!(frame.frame().batches.is_empty());
    let mut erased = starter_body(Corporation::Pan).clone();
    erased.damage(Vec2::ZERO, 100.0);
    let (empty, stats) = draw(&erased, Corporation::Pan, input());
    assert_eq!(stats, Stats::default());
    assert!(empty.batches.is_empty());
}

#[test]
fn parser_rejects_lossy_or_ambiguous_grid_palette_mask_and_mapping_changes() {
    let base: serde_json::Value = serde_json::from_str(ATLAS).unwrap();
    let mutations: [fn(&mut serde_json::Value); 12] = [
        |v| v["version"] = 2.into(),
        |v| v["raicho_detail"]["relative_cell_scale"] = 1.0.into(),
        |v| v["palette"][0] = 255.into(),
        |v| v["palette"][1] = 1.into(),
        |v| v["palette"][1] = v["palette"][2].clone(),
        |v| v["sprites"]["pan_hikari_fine"]["width"] = 48.into(),
        |v| v["sprites"]["pan_hikari_fine"]["indices"][0] = 255.into(),
        |v| {
            v["sprites"].as_object_mut().unwrap().remove("pan_hikari");
        },
        |v| v["sprites"]["pan_hikari_fine"]["material_cells"]["metal"] = serde_json::json!([0]),
        |v| v["sprites"]["pan_hikari_fine"]["signal_cells"] = serde_json::json!([0]),
        |v| v["sprites"]["pan_hikari_fine"]["fine_cells"] = serde_json::json!([0]),
        |v| v["sprites"]["pan_hikari_fine"]["indices"][47 * 96 + 47] = 31.into(),
    ];
    for mutate in mutations {
        let mut changed = base.clone();
        mutate(&mut changed);
        assert!(parse(&changed.to_string()).is_err());
    }
    assert!(parse(ATLAS).is_ok());
}

#[test]
fn transform_quota_rejects_both_layers_and_live_edge_only_body_uses_one_transform() {
    let corp = Corporation::Pan;
    let live = starter_body(corp);
    for transforms in [0, 1] {
        let mut frame = FrameBuilder::new();
        let mut budget = Budget {
            pixels: 3000,
            transforms,
        };
        assert_eq!(
            append(
                &mut frame,
                live,
                corp,
                input(),
                &mut budget,
                |_, c| c,
                |_, c| c
            ),
            Err(Reject::Budget)
        );
        assert!(frame.frame().batches.is_empty());
        assert_eq!(
            budget,
            Budget {
                pixels: 3000,
                transforms
            }
        );
    }
    let art = &ASSETS.as_ref().unwrap()[0];
    let keep = live
        .cells_indexed()
        .find(|(i, _, _)| !art.cells.iter().any(|c| c.coarse_y * 48 + c.coarse_x == *i))
        .unwrap()
        .1;
    let mut damaged = live.clone();
    for (local, _) in live.cells().filter(|(p, _)| *p != keep) {
        damaged.damage(local, 0.0);
    }
    assert_eq!(damaged.occupied_count(), 1);
    let mut frame = FrameBuilder::new();
    let mut budget = Budget {
        pixels: 1,
        transforms: 1,
    };
    let stats = append(
        &mut frame,
        &damaged,
        corp,
        input(),
        &mut budget,
        |_, c| c,
        |_, c| c,
    )
    .unwrap();
    assert_eq!(
        stats,
        Stats {
            body_cells: 1,
            overlay_cells: 0,
            transforms: 1
        }
    );
    assert_eq!(
        budget,
        Budget {
            pixels: 0,
            transforms: 0
        }
    );
    assert_eq!(frame.frame().pixel_count, 1);
}

#[test]
fn repeated_sprite_and_material_keys_cannot_silently_overwrite_valid_authored_data() {
    let mut value: serde_json::Value = serde_json::from_str(ATLAS).unwrap();
    let sprite = value["sprites"]["pan_hikari_fine"].to_string();
    value["sprites"]
        .as_object_mut()
        .unwrap()
        .remove("pan_hikari_fine");
    let prefix = value.to_string().replace(
        "\"sprites\":{",
        &format!("\"sprites\":{{\"pan_hikari_fine\":{sprite},\"pan_hikari_fine\":{sprite},"),
    );
    assert!(parse(&prefix).is_err());
    let repeated_material = ATLAS.replacen(
        "\"material_cells\":{",
        "\"material_cells\":{\"metal\":[],",
        1,
    );
    assert!(parse(&repeated_material).is_err());
}
