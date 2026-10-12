use super::*;
use engine_core::{CameraView, RenderFrame};
use game_core::{Game, Profile};

fn input() -> DockInput {
    DockInput {
        parent: Transform::default(),
        corporation: None,
        time: 0.0,
        reduced_motion: false,
    }
}
fn draw(input: DockInput) -> (RenderFrame, DockStats) {
    let mut frame = FrameBuilder::new();
    let mut budget = DockBudget::default();
    let stats = append(&mut frame, &input, &mut budget).unwrap();
    assert_eq!(
        budget,
        DockBudget {
            cells: 32000 - CELL_COST,
            transforms: 0
        }
    );
    (frame.finish(), stats)
}
type CellPacket = ([f32; 2], u32, [f32; 4], [f32; 4]);

fn capture_live_hub(game: &Game, enabled: bool) -> (RenderFrame, std::ops::Range<usize>) {
    let mut frame = FrameBuilder::new();
    let mut span = 0..0;
    game.render_into_with_home(1.0, &mut frame, |frame, parent, time| {
        if !enabled {
            frame.clear_planar_shadow();
        }
        span.start = frame.frame().pixel_count;
        let stats = append(
            frame,
            &DockInput {
                parent,
                corporation: Some(Corporation::Pan),
                time,
                reduced_motion: true,
            },
            &mut DockBudget::default(),
        )
        .unwrap();
        assert_eq!((stats.cells, stats.transforms), (28_549, 16));
        span.end = frame.frame().pixel_count;
        true
    });
    (frame.finish(), span)
}

fn live_shadow_at_hub_gap(gap: Vec2) -> (Game, RenderFrame, std::ops::Range<usize>) {
    let profile =
        game_core::identity::create_profile("dora", "pan", "renko", "0123456789abcdef").unwrap();
    let mut game = Game::new(profile);
    game.set_reduced_motion(true);
    assert!(game.diagnostic("warp 0 0"));
    let (initial, _) = capture_live_hub(&game, true);
    let shadows = initial.shadows.as_ref().unwrap();
    let caster = shadows.caster.unwrap();
    let key = shadows.light.unwrap();
    let cell = game_core::identity::starter_body(Corporation::Pan)
        .cells()
        .filter(|(_, color)| color >> 24 >= 128)
        .min_by(|a, b| a.0.length_squared().total_cmp(&b.0.length_squared()))
        .unwrap()
        .0;
    let fraction = caster.height / (key.radius * 0.35);
    let intercept = gap.lerp(Vec2::from_array(key.position), fraction);
    let position = intercept - caster.transform.vector(cell);
    assert!(game.diagnostic(&format!("warp {} {}", position.x, position.y)));
    let (frame, span) = capture_live_hub(&game, true);
    (game, frame, span)
}

fn hub_cells(
    frame: &RenderFrame,
    span: std::ops::Range<usize>,
) -> Vec<(engine_core::GpuTransform, Vec2, engine_core::ObjectMeta)> {
    frame
        .batches
        .iter()
        .flat_map(|batch| {
            batch.pixels.iter().map(|pixel| {
                let slot = pixel.transform_slot();
                (
                    batch.transforms[slot],
                    Vec2::from_array(pixel.local),
                    batch
                        .objects
                        .as_ref()
                        .and_then(|objects| objects.get(slot))
                        .copied()
                        .unwrap_or_default(),
                )
            })
        })
        .skip(span.start)
        .take(span.len())
        .collect()
}

fn quad_covers(t: engine_core::GpuTransform, center: Vec2, point: Vec2) -> bool {
    let affine = AffineTransform {
        position: Vec2::new(t.a[0], t.a[1]),
        basis_x: Vec2::new(t.a[2], t.a[3]),
        basis_y: Vec2::new(t.b[0], t.b[1]),
    };
    affine
        .try_inverse_point(point)
        .is_some_and(|local| (local - center).abs().max_element() <= 0.5)
}

#[test]
fn actual_hub_gap_blocks_live_shadow_rays_without_adding_receiver_or_quad() {
    for gap in [Vec2::new(50.0, 50.0), Vec2::new(50.0, -50.0)] {
        let (game, frame, span) = live_shadow_at_hub_gap(gap);
        let shadows = frame.shadows.as_deref().unwrap();
        let key = shadows.light.unwrap();
        let cells = hub_cells(&frame, span.clone());
        assert_eq!(span.len(), 28_549);
        assert!(
            !cells
                .iter()
                .any(|(t, center, _)| quad_covers(*t, *center, gap)),
            "actual native256 opening must have no quad"
        );
        for offset in [
            Vec2::new(-1.5, -1.5),
            Vec2::new(1.5, -1.5),
            Vec2::new(-1.5, 1.5),
            Vec2::new(1.5, 1.5),
        ] {
            assert!(
                shadows.blocks_ray(
                    gap,
                    0.0,
                    Vec2::from_array(key.position) + offset,
                    key.radius * 0.35
                ),
                "the empty-space exclusion must not be a vacuous unblocked ray"
            );
        }
        let occupied_blocked = cells
            .iter()
            .filter(|(t, center, meta)| {
                meta.flags == engine_core::shadow::SHADOW_RECEIVER
                    && shadows.blocks_ray(
                        t.point(*center),
                        meta.height,
                        Vec2::from_array(key.position),
                        key.radius * 0.35,
                    )
            })
            .count();
        assert!(
            occupied_blocked > 0,
            "neighboring real receivers must still admit the same live shadow"
        );
        let (unshadowed, plain_span) = capture_live_hub(&game, false);
        assert_eq!(span, plain_span);
        assert_eq!(frame.pixel_count, unshadowed.pixel_count);
        assert_eq!(
            packet(&frame),
            packet(&unshadowed),
            "shadow presentation must add no offset sprite or quad anywhere in the actual Game frame"
        );
    }
}

#[test]
fn actual_hub_gap_and_live_shadow_projection_survive_fractional_zoom_and_rebase() {
    let gap = Vec2::new(50.0, 50.0);
    let (game, frame, span) = live_shadow_at_hub_gap(gap);
    let shadows = frame.shadows.as_deref().unwrap();
    let key = shadows.light.unwrap();
    let cells = hub_cells(&frame, span);
    let lights = game.light_sources(1.0);
    let home_slot = lights
        .iter()
        .position(|light| {
            light.position == key.position
                && light.radius == key.radius
                && light.color == key.color
                && light.intensity == key.intensity
        })
        .unwrap();
    assert!(
        home_slot > 0 && home_slot < crate::lighting::MAX_LIGHTS,
        "actual exhaust light precedes the home beacon in this live fixture"
    );
    assert!(lights.iter().any(|light| light.position == key.position
        && light.color == key.color
        && light.intensity == key.intensity));
    for camera in [
        Vec2::from_array(game.render_camera(1.0)),
        Vec2::new(51.125, 47.25),
        Vec2::new(1024.125, -512.25),
    ] {
        let descriptors =
            crate::shadow_uniforms::uniforms(Some(shadows), camera.to_array(), &lights);
        assert_eq!(descriptors.header, [1, 1 << home_slot, 4, 1]);
        let caster = descriptors.casters[0];
        for zoom in [0.6, 1.125, 1.6] {
            let view = CameraView::new(camera, Vec2::new(853.0, 480.0), zoom).unwrap();
            let recovered = view.screen_to_world(view.world_to_screen(gap));
            assert!(recovered.distance(gap) < 0.0005);
            assert!(
                !cells
                    .iter()
                    .any(|(t, center, _)| quad_covers(*t, *center, recovered))
            );
            for offset in [
                Vec2::new(-1.5, -1.5),
                Vec2::new(1.5, -1.5),
                Vec2::new(-1.5, 1.5),
                Vec2::new(1.5, 1.5),
            ] {
                let source = Vec2::from_array(key.position) + offset;
                let fraction = caster.x[3] / (key.radius * 0.35);
                let relative_intercept = recovered.lerp(source, fraction) - camera;
                let local = Vec2::new(
                    caster.x[0] * relative_intercept.x
                        + caster.x[1] * relative_intercept.y
                        + caster.x[2],
                    caster.y[0] * relative_intercept.x
                        + caster.y[1] * relative_intercept.y
                        + caster.y[2],
                );
                let expected_local = shadows
                    .caster
                    .unwrap()
                    .transform
                    .try_inverse_point(recovered.lerp(source, fraction))
                    .unwrap();
                assert!(
                    local.distance(expected_local) < 0.0005,
                    "packed inverse must preserve the actual caster coordinates, not merely land on some occupied cell"
                );
                assert!(
                    shadows.occupied(local),
                    "rebased shader descriptor must retain the same occupied occluder across zoom"
                );
                assert!(shadows.blocks_ray(recovered, 0.0, source, key.radius * 0.35));
            }
            let receiver = cells
                .iter()
                .find(|(t, center, meta)| {
                    meta.flags == engine_core::shadow::SHADOW_RECEIVER
                        && shadows.blocks_ray(
                            t.point(*center),
                            0.0,
                            Vec2::from_array(key.position),
                            key.radius * 0.35,
                        )
                })
                .unwrap();
            let relative = view.relative_transform(receiver.0);
            let screen = relative.point(receiver.1) * zoom + Vec2::new(853.0, 480.0) * 0.5;
            assert!(screen.distance(view.world_to_screen(receiver.0.point(receiver.1))) < 0.001);
        }
    }
}

#[test]
fn dock_receiver_metadata_is_occupied_role_only_and_preserves_all_artwork() {
    let (plain, _) = draw(input());
    let mut frame = FrameBuilder::new();
    let ship = PixelBody::from_ascii(&["#"], &[('#', 0xff_ffffff)]);
    frame.set_planar_shadow(
        &ship,
        Transform::default().affine(),
        6.0,
        engine_core::ShadowLight {
            position: [0.0; 2],
            radius: 120.0,
            color: [0.2, 0.85, 0.92],
            intensity: 0.5,
        },
    );
    let stats = append(&mut frame, &input(), &mut DockBudget::default()).unwrap();
    assert_eq!(stats.cells, 28549);
    assert_eq!(stats.transforms, 16);
    assert_eq!(packet(frame.frame()), packet(&plain));
    let mut receivers = 0;
    for batch in &frame.frame().batches {
        let objects = batch.objects.as_ref().unwrap();
        assert_eq!(objects.len(), batch.transforms.len());
        for p in &batch.pixels {
            let slot = p.transform_slot();
            let meta = objects[slot];
            if meta.flags == engine_core::shadow::SHADOW_RECEIVER {
                receivers += 1;
                assert_eq!(meta.height, 0.0);
                assert!(slot % 4 < 2);
            } else {
                assert_eq!(meta, engine_core::ObjectMeta::default());
                assert!(slot % 4 >= 2);
            }
        }
    }
    assert_eq!(receivers, 25919);
}

#[test]
#[ignore = "explicit actual native256 deck/shadow packet exporter; requires GALAXY_HUB_SHADOW_NATIVE and GALAXY_HUB_SHADOW_DECK"]
fn export_actual_hub_receiver_shadow_packets() {
    let native = std::env::var("GALAXY_HUB_SHADOW_NATIVE").expect("native fixture input");
    let output = std::env::var("GALAXY_HUB_SHADOW_DECK").expect("absolute packet output");
    let source: serde_json::Value =
        serde_json::from_slice(&std::fs::read(native).unwrap()).unwrap();
    let mut cases = Vec::new();
    for item in source["cases"].as_array().unwrap() {
        let width = item["width"].as_u64().unwrap() as usize;
        let height = item["height_cells"].as_u64().unwrap() as usize;
        let cells: Vec<[u32; 2]> = serde_json::from_value(item["cells"].clone()).unwrap();
        let body = PixelBody::from_indexed_rgba(width, height, &cells, 16384).unwrap();
        let vector =
            |key: &str| Vec2::from_array(serde_json::from_value(item[key].clone()).unwrap());
        let transform = AffineTransform {
            position: vector("position"),
            basis_x: vector("basis_x"),
            basis_y: vector("basis_y"),
        };
        let key = game_core::LightSource {
            position: serde_json::from_value(item["light"]["position"].clone()).unwrap(),
            radius: item["light"]["radius"].as_f64().unwrap() as f32,
            color: serde_json::from_value(item["light"]["color"].clone()).unwrap(),
            intensity: item["light"]["intensity"].as_f64().unwrap() as f32,
        };
        let mut frame = FrameBuilder::new();
        assert!(frame.set_planar_shadow(
            &body,
            transform,
            item["height"].as_f64().unwrap() as f32,
            engine_core::ShadowLight {
                position: key.position,
                radius: key.radius,
                color: key.color,
                intensity: key.intensity
            }
        ));
        assert_eq!(
            serde_json::to_value(frame.frame().shadows.as_ref().unwrap().masks().as_slice())
                .unwrap(),
            item["mask"]
        );
        append(
            &mut frame,
            &DockInput {
                parent: Transform {
                    scale: Vec2::splat(1.5),
                    ..Transform::default()
                },
                corporation: Some(Corporation::Pan),
                reduced_motion: true,
                ..input()
            },
            &mut DockBudget::default(),
        )
        .unwrap();
        let s = frame.frame().shadows.as_deref().unwrap();
        let descriptors =
            crate::shadow_uniforms::uniforms(Some(s), [0.0; 2], std::slice::from_ref(&key));
        assert_eq!(descriptors.header, [1, 1, 4, 1]);
        let pixels=frame.frame().batches.iter().flat_map(|batch|batch.pixels.iter().map(|p| {
            let t=batch.transforms[p.transform_slot()];let meta=batch.objects.as_ref().unwrap()[p.transform_slot()];
            let center=t.point(Vec2::from_array(p.local));
            let occluded=meta.flags==engine_core::shadow::SHADOW_RECEIVER && s.blocks_ray(center,meta.height,Vec2::from_array(key.position),42.0);
            serde_json::json!({"center":center.to_array(),"basis_x":&t.a[2..4],"basis_y":&t.b[..2],"rgba":p.color,
                "receiver":meta.flags==engine_core::shadow::SHADOW_RECEIVER,"blocked_center":occluded,"material":&t.b[2..4]})
        })).collect::<Vec<_>>();
        let blocked = pixels
            .iter()
            .filter(|p| p["blocked_center"].as_bool() == Some(true))
            .count();
        assert_eq!(pixels.len(), CELL_COST);
        println!(
            "native256 case {:?} damaged={} caster={} receiver25919 centerBlocked={blocked}",
            item["position"],
            item["damaged"],
            body.occupied_count()
        );
        cases.push(serde_json::json!({"native":item,"pixels":pixels,"center_blocked":blocked,
            "descriptor_header":descriptors.header,"descriptor_x":descriptors.casters[0].x,"descriptor_y":descriptors.casters[0].y}));
    }
    std::fs::write(
        output,
        serde_json::to_vec(&serde_json::json!({"cases":cases})).unwrap(),
    )
    .unwrap();
}
fn packet(frame: &RenderFrame) -> Vec<CellPacket> {
    frame
        .batches
        .iter()
        .flat_map(|batch| {
            batch.pixels.iter().map(|pixel| {
                let t = batch.transforms[pixel.transform_slot()];
                (pixel.local, pixel.color, t.a, t.b)
            })
        })
        .collect()
}
#[test]
fn dock_art_emits_exact_28549_cells_16_transforms_for_all_main_corps_and_poses() {
    let mut minimum = Vec2::splat(f32::INFINITY);
    let mut maximum = Vec2::splat(f32::NEG_INFINITY);
    for corporation in [
        None,
        Some(Corporation::Pan),
        Some(Corporation::Renko),
        Some(Corporation::Hugo),
    ] {
        for (state, time) in [0.0, 0.7, 1.4].into_iter().enumerate() {
            let (frame, stats) = draw(DockInput {
                corporation,
                time,
                ..input()
            });
            assert_eq!(
                stats,
                DockStats {
                    cells: CELL_COST,
                    transforms: 16,
                    state
                }
            );
            assert_eq!(frame.pixel_count, CELL_COST);
            assert_eq!(
                frame
                    .batches
                    .iter()
                    .map(|b| b.transforms.len())
                    .sum::<usize>(),
                TRANSFORM_COST
            );
            let mut unique = std::collections::HashSet::new();
            for batch in &frame.batches {
                for pixel in &batch.pixels {
                    let t = batch.transforms[pixel.transform_slot()];
                    let center = t.point(Vec2::from_array(pixel.local));
                    assert!(
                        unique.insert((
                            (center.x * 4.0).round() as i32,
                            (center.y * 4.0).round() as i32
                        )),
                        "tile/role emitted twice"
                    );
                    assert_eq!(t.a[2], 0.5);
                    assert_eq!(t.b[1], 0.5);
                    assert!(pixel.color >> 24 == 255);
                    let half = Vec2::splat(0.25);
                    minimum = minimum.min(center - half);
                    maximum = maximum.max(center + half);
                }
            }
        }
    }
    println!(
        "Dock source bounds {:?}..{:?}; host1.5 bounds {:?}..{:?}; cost28549/16",
        minimum,
        maximum,
        minimum * 1.5,
        maximum * 1.5
    );
    assert!(minimum.min_element() >= -64.0 && maximum.max_element() <= 64.0);
}

#[test]
fn dock_art_only_signals_recolor_and_only_external_lamps_animate() {
    let pack: Pack = serde_json::from_str(include_str!(
        "../../../assets/pipeline/pan-coop-dock/native256/pack.json"
    ))
    .unwrap();
    let mut changed = [0, 0];
    let mut recolored = [0, 0];
    for (index, tile) in pack.pixel_rig_tiles.tiles.iter().enumerate() {
        for state in 0..3 {
            let sprite = &pack.sprites[&tile.frame_sprites[state]];
            for (cell, _, color) in ART.tiles[index].bodies[0][state].cells_indexed() {
                assert_eq!(color, pack.palette[sprite.indices[cell] as usize]);
                let baseline = pack.sprites[&tile.frame_sprites[0]].indices[cell];
                if sprite.indices[cell] != baseline {
                    let global =
                        (cell / SIDE + (index / 2) * SIDE) * 256 + cell % SIDE + (index % 2) * SIDE;
                    assert!(pack.pixel_rig_tiles.lamp_cells_source.contains(&global));
                    if state > 0 {
                        changed[state - 1] += 1;
                    }
                }
            }
            for corporation in 1..3 {
                for (cell, _, color) in ART.tiles[index].bodies[corporation][state].cells_indexed()
                {
                    let original = pack.palette[sprite.indices[cell] as usize];
                    if tile.roles[cell] != 3 {
                        assert_eq!(
                            color, original,
                            "neutral alloy/deck/cyan instrument recolored"
                        );
                    } else if state == 0 && color != original {
                        recolored[corporation - 1] += 1;
                    }
                }
            }
        }
    }
    assert!(changed.into_iter().all(|count| count > 1000));
    assert_eq!(recolored, [1925, 1925]);
}

#[test]
fn dock_art_reduced_motion_freezes_complete_geometry_color_and_state() {
    for corporation in [
        Some(Corporation::Pan),
        Some(Corporation::Renko),
        Some(Corporation::Hugo),
    ] {
        let baseline = packet(
            &draw(DockInput {
                corporation,
                reduced_motion: true,
                ..input()
            })
            .0,
        );
        for time in [0.7, 1.4, 29.6, f32::NAN, f32::INFINITY] {
            let (frame, stats) = draw(DockInput {
                corporation,
                time,
                reduced_motion: true,
                ..input()
            });
            assert_eq!(stats.state, 0);
            assert_eq!(packet(&frame), baseline);
        }
    }
}

#[test]
fn dock_art_preserves_fractional_reflected_nonuniform_cells_and_zoom_projection() {
    for camera in [Vec2::ZERO, Vec2::new(999000.0, -999000.0)] {
        let parent = Transform {
            position: camera + Vec2::new(3.5, -1.25),
            scale: Vec2::new(-0.31, 0.73),
            rotation: 0.37,
        };
        let (frame, _) = draw(DockInput { parent, ..input() });
        for zoom in [0.6, 1.6] {
            let view = CameraView::new(camera, Vec2::new(533.0, 300.0), zoom).unwrap();
            for batch in &frame.batches {
                for pixel in &batch.pixels {
                    let t = view.relative_transform(batch.transforms[pixel.transform_slot()]);
                    // Relative camera extraction preserves the fine basis even far
                    // from origin, without a hard-cell raster minimum.
                    assert!((Vec2::new(t.a[2], t.a[3]).length() - 0.155).abs() < 0.00001);
                    assert!((Vec2::new(t.b[0], t.b[1]).length() - 0.365).abs() < 0.00001);
                    for corner in [
                        Vec2::splat(-0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::new(-0.5, 0.5),
                        Vec2::splat(0.5),
                    ] {
                        let projected = t.point(Vec2::from_array(pixel.local) + corner) * zoom
                            + Vec2::new(533.0, 300.0) * 0.5;
                        assert!(projected.is_finite());
                    }
                }
            }
        }
    }
}

#[test]
fn dock_art_short_quota_and_invalid_parent_never_append_or_debit() {
    for variant in 0..7 {
        let mut i = input();
        let mut budget = DockBudget::default();
        match variant {
            0 => budget.cells = CELL_COST - 1,
            1 => budget.transforms = 15,
            2 => i.parent.position.x = f32::NAN,
            3 => i.parent.position.x = 1_100_001.0,
            4 => i.parent.rotation = f32::INFINITY,
            5 => i.parent.scale.y = 0.0,
            _ => i.parent.scale.x = 65.0,
        }
        let before = budget;
        let mut frame = FrameBuilder::new();
        frame.push_pixel(Vec2::ZERO, 1.0, 0xffaabbcc);
        let previous = packet(frame.frame());
        assert_eq!(append(&mut frame, &i, &mut budget), None);
        assert_eq!(packet(frame.frame()), previous);
        assert_eq!(budget, before);
    }
}

#[test]
fn dock_art_actual_game_hook_preserves_fallback_order_facilities_and_profile() {
    let mut game = Game::new(Profile::default());
    game.set_viewport(533.0, 300.0);
    let profile = serde_json::to_string(game.profile()).unwrap();
    let mut baseline = FrameBuilder::new();
    game.render_into(1.0, &mut baseline);
    let mut rejected = FrameBuilder::new();
    game.render_into_with_home(1.0, &mut rejected, |frame, parent, time| {
        let mut budget = DockBudget {
            cells: CELL_COST - 1,
            ..Default::default()
        };
        append(
            frame,
            &DockInput {
                parent,
                time,
                ..input()
            },
            &mut budget,
        )
        .is_some()
    });
    assert_eq!(packet(baseline.frame()), packet(rejected.frame()));
    let mut accepted = FrameBuilder::new();
    let mut start = 0;
    let mut expected = Vec::new();
    game.render_into_with_home(1.0, &mut accepted, |frame, parent, time| {
        assert_eq!(parent.position, Vec2::ZERO);
        assert_eq!(parent.scale, Vec2::splat(1.5));
        start = frame.frame().pixel_count;
        expected = packet(
            &draw(DockInput {
                parent,
                time,
                ..input()
            })
            .0,
        );
        append(
            frame,
            &DockInput {
                parent,
                time,
                ..input()
            },
            &mut DockBudget::default(),
        )
        .is_some()
    });
    let actual = packet(accepted.frame());
    assert_eq!(actual[start..start + CELL_COST], expected);
    assert!(
        actual.len() > start + CELL_COST,
        "gameplay objects remain above home artwork"
    );
    for (id, center) in [
        [-120.0, -80.0],
        [120.0, -80.0],
        [-120.0, 95.0],
        [120.0, 95.0],
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(game.dock_facility_at(center), Some(id as u8));
        // Current dock art ends at96 on each axis; the actual facility cells
        // lie within13 of center. All four semantic structures remain clear.
        assert!(center[0].abs() - 13.0 > 96.0);
    }
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), profile);
    game.diagnostic("warp 10000 10000");
    let mut calls = 0;
    game.render_into_with_home(1.0, &mut accepted, |_, _, _| {
        calls += 1;
        true
    });
    assert_eq!(calls, 0, "invisible home artwork should not extract");
}
